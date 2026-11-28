Warning: truncated output (original token count: 593973)
... 1327316 bytes omitted ...

mod community;
mod layout;
mod proximity;

use eframe::egui;
use quick_xml::{
    Reader as XmlReader, escape::resolve_predefined_entity, events::Event as XmlEvent,
};
use rusqlite::{
    Connection, ErrorCode, InterruptHandle, OpenFlags, OptionalExtension, TransactionBehavior,
    params_from_iter,
    types::{Value, ValueRef},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    env, fs,
    io::{BufRead, BufReader, Read, Write},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    sync::{
        Arc, Mutex, OnceLock,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use uuid::Uuid;

const CURRENT_CASE_FORMAT_VERSION: u32 = 19;
const SQL_QUERY_TIMEOUT_MINUTES: u16 = 15;
const MAX_SQL_QUERY_TIMEOUT_MINUTES: u16 = 1440;
const SQL_PLAN_TIMEOUT_SECONDS: u64 = 30;
const SQL_PLAN_TIMEOUT: Duration = Duration::from_secs(SQL_PLAN_TIMEOUT_SECONDS);
const SQL_ENTITY_LOOKUP_TIMEOUT_SECONDS: u64 = 120;
const SQL_ENTITY_LOOKUP_MAX_ROWS: usize = 20;
const SQL_ENTITY_LOOKUP_MAX_COLUMNS: usize = 64;
const SQL_ENTITY_LOOKUP_MAX_CELL_BYTES: usize = 1024;
const SQL_ENTITY_LOOKUP_MAX_TOTAL_BYTES: usize = 64 * 1024;
const SQL_ENTITY_LOOKUP_MAX_SCANNED_ROWS: usize = 1_000_000;
const SQLITE_LOCK_RETRY_ATTEMPTS: usize = 3;
const SQLITE_LOCK_RETRY_DELAY: Duration = Duration::from_secs(2);
const MAX_CHART_DIAGNOSTIC_SAMPLES_PER_METRIC: usize = 64;

static CHART_DIAGNOSTIC_PATH: OnceLock<Option<PathBuf>> = OnceLock::new();
static CHART_DIAGNOSTIC_COUNTS: OnceLock<Mutex<HashMap<&'static str, usize>>> = OnceLock::new();
#[cfg(test)]
static DISABLE_CHART_ROUTE_CULLING_FOR_BENCHMARK: AtomicBool = AtomicBool::new(false);

fn record_chart_diagnostic(
    metric: &'static str,
    elapsed: Duration,
    entity_count: usize,
    relationship_count: usize,
    label_count: usize,
) {
    let path = CHART_DIAGNOSTIC_PATH
        .get_or_init(|| env::var_os("ARCH_NOTEBOOK_CHART_DIAGNOSTICS").map(PathBuf::from));
    let Some(path) = path else { return };
    let counts = CHART_DIAGNOSTIC_COUNTS.get_or_init(|| Mutex::new(HashMap::new()));
    let Ok(mut counts) = counts.lock() else {
        return;
    };
    let count = counts.entry(metric).or_default();
    if *count >= MAX_CHART_DIAGNOSTIC_SAMPLES_PER_METRIC {
        return;
    }
    *count += 1;
    let event = serde_json::json!({
        "schema_version": 1,
        "event": "chart_cpu_timing",
        "metric": metric,
        "sample": *count,
        "elapsed_ns": elapsed.as_nanos(),
        "entities": entity_count,
        "relationship_records": relationship_count,
        "labels": label_count,
        "sample_cap_per_metric": MAX_CHART_DIAGNOSTIC_SAMPLES_PER_METRIC,
        "measurement_scope": "native_app_cpu_work_not_frame_presentation",
    });
    if let Ok(mut output) = fs::OpenOptions::new().create(true).append(true).open(path) {
        let _ = writeln!(output, "{event}");
    }
}

fn sqlite_capture_error(error: rusqlite::Error, retryable_lock: &AtomicBool) -> String {
    if matches!(
        &error,
        rusqlite::Error::SqliteFailure(failure, _)
            if matches!(failure.code, ErrorCode::DatabaseBusy | ErrorCode::DatabaseLocked)
    ) {
        retryable_lock.store(true, Ordering::Release);
    }
    error.to_string()
}

fn enforce_sqlite_query_only(connection: &Connection) -> rusqlite::Result<()> {
    connection.pragma_update(None, "query_only", true)
}

fn default_sql_query_timeout_minutes() -> u16 {
    SQL_QUERY_TIMEOUT_MINUTES
}

fn sql_query_timeout_message(timeout_minutes: u16) -> String {
    format!("SQL query exceeded the {timeout_minutes} minute execution limit")
}

fn sql_schema_adaptation_kind(error: &str) -> Option<&'static str> {
    let error = error.to_ascii_lowercase();
    if error.contains("no such table:") {
        Some("missing_table")
    } else if error.contains("no such column:") || error.contains("has no column named") {
        Some("missing_column")
    } else if error.contains("ambiguous column name:") {
        Some("ambiguous_column")
    } else {
        None
    }
}

fn sql_schema_adaptation_hint(error: &str) -> Option<&'static str> {
    match sql_schema_adaptation_kind(error) {
        Some("ambiguous_column") => Some(
            "the column exists in multiple joined tables; qualify it with the intended table or alias, then validate the query before saving",
        ),
        Some(_) => Some(
            "the source schema may have changed; inspect the SQLite schema, rebuild or edit the query, then validate it before saving",
        ),
        None => None,
    }
}

fn sql_schema_adaptation_reference(error: &str) -> Option<String> {
    let lowercase = error.to_ascii_lowercase();
    [
        "no such table:",
        "no such column:",
        "has no column named",
        "ambiguous column name:",
    ]
    .into_iter()
    .find_map(|marker| {
        let start = lowercase.find(marker)? + marker.len();
        let reference = error.get(start..)?;
        let reference_lowercase = reference.to_ascii_lowercase();
        let context_start = [" in select ", " at offset "]
            .into_iter()
            .filter_map(|context| reference_lowercase.find(context))
            .min()
            .unwrap_or(reference.len());
        let reference = reference.get(..context_start)?.trim();
        (!reference.is_empty()).then(|| reference.to_owned())
    })
}

fn sql_schema_adaptation_details(error: &str) -> Option<SqlSchemaAdaptation> {
    Some(SqlSchemaAdaptation {
        kind: sql_schema_adaptation_kind(error)?.to_owned(),
        reference: sql_schema_adaptation_reference(error),
        hint: sql_schema_adaptation_hint(error).map(str::to_owned),
    })
}

fn install_automation_schedule_command(
    case_path: &Path,
    schedule: &str,
    schedule_time: &str,
    schedule_weekday: u8,
    schedule_month_day: u8,
    chart_export_dir: &str,
    require_usable_connections: bool,
    progress: &Arc<Mutex<String>>,
) -> Result<String, String> {
    let executable =
        env::current_exe().map_err(|error| format!("cannot locate app executable: {error}"))?;
    let mut command: Command;
    #[cfg(target_os = "linux")]
    {
        let packaged_installer = executable
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("arch-notebook-install-automation");
        if packaged_installer.is_file() {
            command = Command::new(packaged_installer);
            command.arg(case_path).arg(executable);
        } else {
            command = Command::new("bash");
            command
                .arg(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("scripts/install-linux-automation.sh"),
                )
                .arg(case_path)
                .arg(executable);
        }
    }
    #[cfg(target_os = "macos")]
    {
        let contents = executable
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "app executable is not inside an application bundle".to_string())?;
        let app_bundle = contents
            .parent()
            .ok_or_else(|| "application bundle path could not be resolved".to_string())?;
        let installer = contents.join("Resources/install-macos-automation.sh");
        command = Command::new("bash");
        let installer = if installer.is_file() {
            installer
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/install-macos-automation.sh")
        };
        command.arg(installer).arg(case_path).arg(app_bundle);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (
            case_path,
            schedule,
            schedule_time,
            schedule_weekday,
            executable,
        );
        return Err("scheduled SQL automation is supported on Linux and macOS".into());
    }
    command.env("ARCH_NOTEBOOK_AUTOMATION_SCHEDULE", schedule);
    command.env("ARCH_NOTEBOOK_AUTOMATION_TIME", schedule_time);
    command.env(
        "ARCH_NOTEBOOK_AUTOMATION_WEEKDAY",
        schedule_weekday.to_string(),
    );
    command.env(
        "ARCH_NOTEBOOK_AUTOMATION_MONTH_DAY",
        schedule_month_day.to_string(),
    );
    command.env("ARCH_NOTEBOOK_CHART_EXPORT_DIR", chart_export_dir);
    command.env(
        "ARCH_NOTEBOOK_REQUIRE_USABLE_CONNECTIONS",
        if require_usable_connections {
            "true"
        } else {
            "false"
        },
    );
    command.stdout(Stdio::piped()).stderr(Stdio::piped());
    let mut child = command
        .spawn()
        .map_err(|error| format!("could not launch schedule installer: {error}"))?;
    let Some(stdout) = child.stdout.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("schedule installer output stream is unavailable".into());
    };
    let Some(stderr) = child.stderr.take() else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("schedule installer diagnostic stream is unavailable".into());
    };
    let stdout_worker = thread::spawn(move || {
        let mut output = String::new();
        BufReader::new(stdout)
            .read_to_string(&mut output)
            .map(|_| output)
            .map_err(|error| error.to_string())
    });
    let progress = Arc::clone(progress);
    let stderr_worker = thread::spawn(move || {
        let mut diagnostics = String::new();
        for line in BufReader::new(stderr).lines() {
            let Ok(line) = line else {
                continue;
            };
            let progress_line = line
                .strip_prefix("ARCH_NOTEBOOK_PREFLIGHT_PROGRESS ")
                .or_else(|| line.strip_prefix("ARCH_NOTEBOOK_SQL_PROGRESS "));
            if let Some(progress_line) = progress_line {
                if let Ok(mut current) = progress.lock() {
                    *current = progress_line.to_owned();
                }
            } else {
                diagnostics.push_str(&line);
                diagnostics.push('\n');
            }
        }
        diagnostics
    });
    let status = match child.wait() {
        Ok(status) => status,
        Err(error) => {
            let _ = child.kill();
            let _ = child.wait();
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            return Err(format!("could not wait for schedule installer: {error}"));
        }
    };
    let stdout = stdout_worker
        .join()
        .map_err(|_| "schedule installer output reader stopped unexpectedly".to_owned())?
        .map_err(|error| format!("could not read schedule installer output: {error}"))?
        .trim()
        .to_owned();
    let stderr = stderr_worker
        .join()
        .unwrap_or_else(|_| "schedule installer diagnostic reader stopped unexpectedly".into())
        .trim()
        .to_owned();
    if status.success() {
        Ok(if stdout.is_empty() {
            "SQL AUTOMATION SCHEDULE INSTALLED".into()
        } else {
            stdout
        })
    } else {
        Err(if stderr.is_empty() {
            format!(
                "schedule installer exited with {}{}",
                status,
                if stdout.is_empty() {
                    String::new()
                } else {
                    format!(": {stdout}")
                }
            )
        } else {
            stderr
        })
    }
}

fn remove_automation_schedule_command(case_path: &Path) -> Result<String, String> {
    let executable =
        env::current_exe().map_err(|error| format!("cannot locate app executable: {error}"))?;
    let mut command: Command;
    #[cfg(target_os = "linux")]
    {
        let packaged_installer = executable
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("arch-notebook-install-automation");
        if packaged_installer.is_file() {
            command = Command::new(packaged_installer);
            command.arg("--remove").arg(case_path);
        } else {
            command = Command::new("bash");
            command
                .arg(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("scripts/install-linux-automation.sh"),
                )
                .arg("--remove")
                .arg(case_path);
        }
    }
    #[cfg(target_os = "macos")]
    {
        let contents = executable
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "app executable is not inside an application bundle".to_string())?;
        let packaged_installer = contents.join("Resources/install-macos-automation.sh");
        let installer = if packaged_installer.is_file() {
            packaged_installer
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/install-macos-automation.sh")
        };
        command = Command::new("bash");
        command.arg(installer).arg("--remove").arg(case_path);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (executable, case_path);
        return Err("scheduled SQL automation is supported on Linux and macOS".into());
    }
    let output = command
        .output()
        .map_err(|error| format!("could not launch schedule remover: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(if stdout.is_empty() {
            "SQL AUTOMATION SCHEDULE REMOVED".into()
        } else {
            stdout
        })
    } else {
        Err(if stderr.is_empty() {
            format!("schedule remover exited with {}", output.status)
        } else {
            stderr
        })
    }
}

fn check_automation_schedule_command(case_path: &Path) -> Result<String, String> {
    let executable =
        env::current_exe().map_err(|error| format!("cannot locate app executable: {error}"))?;
    let mut command: Command;
    #[cfg(target_os = "linux")]
    {
        let packaged_installer = executable
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .join("arch-notebook-install-automation");
        if packaged_installer.is_file() {
            command = Command::new(packaged_installer);
            command.arg("--status").arg(case_path);
        } else {
            command = Command::new("bash");
            command
                .arg(
                    Path::new(env!("CARGO_MANIFEST_DIR"))
                        .join("scripts/install-linux-automation.sh"),
                )
                .arg("--status")
                .arg(case_path);
        }
    }
    #[cfg(target_os = "macos")]
    {
        let contents = executable
            .parent()
            .and_then(Path::parent)
            .ok_or_else(|| "app executable is not inside an application bundle".to_string())?;
        let packaged_installer = contents.join("Resources/install-macos-automation.sh");
        let installer = if packaged_installer.is_file() {
            packaged_installer
        } else {
            Path::new(env!("CARGO_MANIFEST_DIR")).join("scripts/install-macos-automation.sh")
        };
        command = Command::new("bash");
        command.arg(installer).arg("--status").arg(case_path);
    }
    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        let _ = (executable, case_path);
        return Err("scheduled SQL automation is supported on Linux and macOS".into());
    }
    let output = command
        .output()
        .map_err(|error| format!("could not check schedule status: {error}"))?;
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    if output.status.success() {
        Ok(if stdout.is_empty() {
            "SQL AUTOMATION SCHEDULE STATUS UNAVAILABLE".into()
        } else {
            stdout
        })
    } else {
        Err(if stderr.is_empty() {
            format!("schedule status command exited with {}", output.status)
        } else {
            stderr
        })
    }
}

fn schedule_cadence_from_status(message: &str) -> Option<&'static str> {
    let message = message.to_ascii_lowercase();
    if message.contains("every 15 minutes") {
        Some("15m")
    } else if message.contains("hourly") {
        Some("hourly")
    } else if message.contains("daily") {
        Some("daily")
    } else if message.contains("weekly") {
        Some("weekly")
    } else if message.contains("monthly") {
        Some("monthly")
    } else {
        None
    }
}

fn schedule_chart_export_dir_from_status(message: &str) -> Option<String> {
    message.lines().find_map(|line| {
        let value = line.strip_prefix("Chart CSV export directory: ")?;
        Some(if value == "disabled" {
            String::new()
        } else {
            value.to_owned()
        })
    })
}

fn schedule_install_preflight_requires_usable_from_status(message: &str) -> Option<bool> {
    message.lines().find_map(|line| {
        match line.strip_prefix("Schedule install preflight requires usable connections: ")? {
            "true" => Some(true),
            "false" => Some(false),
            _ => None,
        }
    })
}

fn schedule_time_from_status(message: &str) -> Option<String> {
    message
        .lines()
        .find_map(|line| line.strip_prefix("Schedule time: ").map(str::to_owned))
        .filter(|value| valid_schedule_time(value))
}

fn schedule_month_day_from_status(message: &str) -> Option<u8> {
    message
        .lines()
        .find_map(|line| line.strip_prefix("Schedule day: ")?.parse().ok())
        .filter(|day| (1..=28).contains(day))
}

fn schedule_weekday_name(weekday: u8) -> Option<&'static str> {
    [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ]
    .get(weekday as usize)
    .copied()
}

fn schedule_weekday_from_status(message: &str) -> Option<u8> {
    let weekday = message
        .lines()
        .find_map(|line| line.strip_prefix("Schedule weekday: "))?;
    (0..=6).find(|index| {
        schedule_weekday_name(*index).is_some_and(|name| name.eq_ignore_ascii_case(weekday))
    })
}

fn valid_schedule_time(value: &str) -> bool {
    let Some((hour, minute)) = value.split_once(':') else {
        return false;
    };
    hour.len() == 2
        && minute.len() == 2
        && hour.parse::<u8>().is_ok_and(|hour| hour < 24)
        && minute.parse::<u8>().is_ok_and(|minute| minute < 60)
}

fn preflight_automation_jobs_command(
    case_path: &Path,
    job_ids: &[Uuid],
    parameter_overrides: &AutomationParameterOverrides,
    require_usable_connections: bool,
    require_unchanged_schema: bool,
    progress: &Arc<Mutex<String>>,
    cancel: &AtomicBool,
) -> Result<String, String> {
    let executable =
        env::current_exe().map_err(|error| format!("cannot locate app executable: {error}"))?;
    let mut command = Command::new(executable);
    command
        .arg("--check-enabled-job-queries")
        .arg(case_path)
        .arg("--progress")
        .arg("--json")
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    if require_usable_connections {
        command.arg("--require-usable");
    }
    if require_unchanged_schema {
        command.arg("--require-unchanged-schema");
    }
    for job_id in job_ids {
        command.arg("--job").arg(job_id.to_string());
    }
    for (name, value) in &parameter_overrides.global {
        command.arg("--param").arg(format!("{name}={value}"));
    }
    for (selector, name, value) in &parameter_overrides.scoped {
        command
            .arg("--job-param")
            .arg(selector)
            .arg(format!("{name}={value}"));
    }
    let mut child = command
        .spawn()
        .map_err(|error| format!("could not start SQL query preflight: {error}"))?;
    let stdout = child.stdout.take();
    let stderr = child.stderr.take();
    let Some(stdout) = stdout else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("SQL preflight output stream is unavailable".into());
    };
    let Some(stderr) = stderr else {
        let _ = child.kill();
        let _ = child.wait();
        return Err("SQL preflight error stream is unavailable".into());
    };
    let (stdout_sender, stdout_receiver) = mpsc::channel();
    let stdout_worker = thread::spawn(move || {
        let mut stdout_text = String::new();
        let result = BufReader::new(stdout)
            .read_to_string(&mut stdout_text)
            .map(|_| stdout_text)
            .map_err(|error| error.to_string());
        let _ = stdout_sender.send(result);
    });
    let stderr_progress = Arc::clone(progress);
    let stderr_worker = thread::spawn(move || {
        let mut stderr_text = String::new();
        for line in BufReader::new(stderr).lines() {
            let Ok(line) = line else {
                continue;
            };
            let progress_status = line
                .strip_prefix("ARCH_NOTEBOOK_PREFLIGHT_PROGRESS ")
                .or_else(|| line.strip_prefix("ARCH_NOTEBOOK_SQL_PROGRESS "));
            if let Some(status) = progress_status {
                if let Ok(mut current) = stderr_progress.lock() {
                    *current = status.to_owned();
                }
            } else {
                stderr_text.push_str(&line);
                stderr_text.push('\n');
            }
        }
        stderr_text
    });
    let mut cancellation_checked = false;
    let mut was_cancelled = false;
    let status = loop {
        if cancel.load(Ordering::Acquire) && !cancellation_checked {
            was_cancelled = child.kill().is_ok();
            cancellation_checked = true;
        }
        match child.try_wait() {
            Ok(Some(status)) => break status,
            Ok(None) => thread::sleep(Duration::from_millis(50)),
            Err(error) => {
                let _ = child.kill();
                let _ = child.wait();
                let _ = stdout_worker.join();
                let _ = stderr_worker.join();
                return Err(format!("could not wait for SQL query preflight: {error}"));
            }
        }
    };
    let stdout_report = match stdout_receiver.recv() {
        Ok(Ok(report)) => report,
        Ok(Err(error)) => {
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            return Err(format!("SQL preflight output could not be read: {error}"));
        }
        Err(_) => {
            let _ = stdout_worker.join();
            let _ = stderr_worker.join();
            return Err("SQL preflight output stream stopped unexpectedly".into());
        }
    };
    let _ = stdout_worker.join();
    let stderr = stderr_worker
        .join()
        .unwrap_or_else(|_| "SQL preflight error stream stopped unexpectedly".to_owned());
    if was_cancelled {
        return Err("cancelled by analyst".into());
    }
    if let Ok(result) = serde_json::from_str::<serde_json::Value>(&stdout_report) {
        let succeeded = result["ok"].as_bool().unwrap_or_else(|| status.success());
        let summary = serde_json::to_string_pretty(&result).unwrap_or(stdout_report);
        let mut report = format!(
            "SQL PREFLIGHT {}\n{summary}",
            if succeeded { "PASSED" } else { "NEEDS REVIEW" }
        );
        if !stderr.trim().is_empty() {
            report.push_str("\n\nSQL PREFLIGHT DIAGNOSTICS\n");
            report.push_str(stderr.trim());
        }
        return Ok(report);
    }
    let mut report = stdout_report;
    if !stderr.trim().is_empty() {
        if !report.is_empty() {
            report.push('\n');
        }
        report.push_str(stderr.trim());
    }
    if status.success() {
        Ok(if report.is_empty() {
            "ENABLED SQL JOBS PASSED PREFLIGHT".into()
        } else {
            report
        })
    } else {
        Err(if report.is_empty() {
            format!("SQL preflight exited with {status}")
        } else {
            report
        })
    }
}

struct SqlQueryDeadline {
    cancel: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
    expired: Arc<AtomicBool>,
    cancellation_requested: Arc<AtomicBool>,
}

impl SqlQueryDeadline {
    fn new(
        interrupt: InterruptHandle,
        cancellation_requested: Arc<AtomicBool>,
        timeout_minutes: u16,
    ) -> Result<Self, String> {
        Self::with_timeout(
            interrupt,
            cancellation_requested,
            Duration::from_secs(u64::from(timeout_minutes) * 60),
        )
    }

    fn with_timeout(
        interrupt: InterruptHandle,
        cancellation_requested: Arc<AtomicBool>,
        timeout: Duration,
    ) -> Result<Self, String> {
        let (cancel, cancelled) = mpsc::channel();
        let expired = Arc::new(AtomicBool::new(false));
        let worker_expired = Arc::clone(&expired);
        let worker_cancellation = Arc::clone(&cancellation_requested);
        let worker = thread::Builder::new()
            .name("arch-notebook-sql-deadline".into())
            .spawn(move || {
                let deadline = Instant::now() + timeout;
                loop {
                    if worker_cancellation.load(Ordering::Acquire) {
                        interrupt.interrupt();
                        break;
                    }
                    let remaining = deadline.saturating_duration_since(Instant::now());
                    if remaining.is_zero() {
                        worker_expired.store(true, Ordering::Release);
                        interrupt.interrupt();
                        break;
                    }
                    match cancelled.recv_timeout(remaining.min(Duration::from_millis(100))) {
                        Ok(()) | Err(mpsc::RecvTimeoutError::Disconnected) => break,
                        Err(mpsc::RecvTimeoutError::Timeout) => {}
                    }
                }
            })
            .map_err(|error| format!("could not start SQL timeout monitor: {error}"))?;
        Ok(Self {
            cancel,
            worker: Some(worker),
            expired,
            cancellation_requested,
        })
    }

    fn expired(&self) -> bool {
        self.expired.load(Ordering::Acquire)
    }

    fn cancelled(&self) -> bool {
        self.cancellation_requested.load(Ordering::Acquire)
    }
}

impl Drop for SqlQueryDeadline {
    fn drop(&mut self) {
        let _ = self.cancel.send(());
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct Entity {
    id: Uuid,
    name: String,
    kind: String,
    #[serde(default)]
    representation: EntityRepresentation,
    detail: String,
    #[serde(default)]
    attributes: Vec<(String, String)>,
    #[serde(default)]
    sql_attribute_values: Vec<(String, String, String)>,
    #[serde(default)]
    sql_attribute_baselines: Vec<(String, Option<String>)>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    external_ids: Vec<(String, String)>,
    #[serde(default)]
    merged_records: Vec<MergedEntityRecord>,
    x: f32,
    y: f32,
    #[serde(default)]
    position_locked: bool,
    color: [u8; 3],
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum EntityRepresentation {
    #[default]
    Icon,
    ThemeLine,
    EventFrame,
    Box,
    Circle,
    TextBlock,
}

impl EntityRepresentation {
    fn label(self) -> &'static str {
        match self {
            Self::Icon => "Icon",
            Self::ThemeLine => "Theme line",
            Self::EventFrame => "Event frame",
            Self::Box => "Box",
            Self::Circle => "Circle",
            Self::TextBlock => "Text block",
        }
    }

    fn parse(value: &str) -> Option<Self> {
        match value.trim().to_ascii_lowercase().as_str() {
            "icon" => Some(Self::Icon),
            "theimeline" | "theme_line" | "theme line" => Some(Self::ThemeLine),
            "eventframe" | "event_frame" | "event frame" => Some(Self::EventFrame),
            "box" => Some(Self::Box),
            "circle" => Some(Self::Circle),
            "textblock" | "text_block" | "text block" => Some(Self::TextBlock),
            _ => None,
        }
    }
}

#[derive(Clone, Serialize, Deserialize)]
struct MergedEntityRecord {
    id: Uuid,
    name: String,
    kind: String,
    detail: String,
    attributes: Vec<(String, String)>,
    #[serde(default)]
    sql_attribute_values: Vec<(String, String, String)>,
    #[serde(default)]
    sql_attribute_baselines: Vec<(String, Option<String>)>,
}

#[derive(Clone, Serialize, Deserialize, PartialEq)]
struct Link {
    from: Uuid,
    to: Uuid,
    #[serde(default)]
    connection_id: String,
    label: String,
    date: String,
    confidence: u8,
    directed: bool,
    notes: String,
    #[serde(default)]
    source_notes: String,
    #[serde(default)]
    source_reference: String,
    #[serde(default)]
    ingest_source: Option<String>,
    #[serde(default)]
    attributes: Vec<(String, String)>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct NetworkAnalysisMethodCard {
    entity_count: usize,
    isolate_count: usize,
    relationship_record_count: usize,
    directed_record_count: usize,
    undirected_record_count: usize,
    unique_endpoint_pair_count: usize,
    repeated_endpoint_pair_record_count: usize,
    missing_endpoint_record_count: usize,
    timeline_recognized_date_count: usize,
    blank_date_count: usize,
    unrecognized_date_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CommunityAnalysis {
    groups: Vec<Vec<Uuid>>,
    interactions: Vec<CommunityInteraction>,
    proxies: Vec<CommunityProxy>,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CommunityProxy {
    entity_id: Uuid,
    distinct_neighbor_entities: usize,
    distinct_community_contacts: usize,
    relationship_record_count: usize,
}

#[derive(Clone, Debug, Default, PartialEq, Eq)]
struct CommunityInteraction {
    first_community: usize,
    second_community: usize,
    relationship_record_count: usize,
    first_to_second_record_count: usize,
    second_to_first_record_count: usize,
    undirected_record_count: usize,
    distinct_endpoint_pair_count: usize,
    link_indices: Vec<usize>,
}

struct ChartVectorGeometry {
    link_lanes: Vec<f32>,
    entity_positions: HashMap<Uuid, egui::Pos2>,
    route_lanes: Vec<f32>,
}

#[derive(Clone)]
struct CommunityLayoutWorkerResult {
    chart_id: Uuid,
    entity_ids: Vec<Uuid>,
    original_positions: Vec<(f32, f32, bool)>,
    link_endpoints: Vec<(Uuid, Uuid)>,
    positions: Vec<(f32, f32)>,
}

struct CommunityLayoutRestoreSnapshot {
    chart_id: Uuid,
    entities: Vec<(Uuid, f32, f32, bool)>,
}

fn community_layout_snapshot_matches(
    active_chart_id: Uuid,
    entities: &[Entity],
    links: &[Link],
    result: &CommunityLayoutWorkerResult,
) -> bool {
    active_chart_id == result.chart_id
        && entities.len() == result.entity_ids.len()
        && result.positions.len() == entities.len()
        && entities
            .iter()
            .zip(&result.entity_ids)
            .all(|(entity, id)| entity.id == *id)
        && entities
            .iter()
            .zip(&result.original_positions)
            .all(|(entity, (x, y, pinned))| {
                entity.x == *x && entity.y == *y && entity.position_locked == *pinned
            })
        && links.len() == result.link_endpoints.len()
        && links
            .iter()
            .zip(&result.link_endpoints)
            .all(|(link, (from, to))| link.from == *from && link.to == *to)
}

fn community_layout_restore_matches(
    active_chart_id: Uuid,
    entities: &[Entity],
    snapshot: &CommunityLayoutRestoreSnapshot,
) -> bool {
    active_chart_id == snapshot.chart_id
        && entities.len() == snapshot.entities.len()
        && entities
            .iter()
            .zip(&snapshot.entities)
            .all(|(entity, (id, _, _, pinned))| {
                entity.id == *id && entity.position_locked == *pinned
            })
}

struct ConnectionGroup {
    first: Uuid,
    second: Uuid,
    link_indices: Vec<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConnectionFact {
    key: String,
    #[serde(default)]
    connection_id: String,
    #[serde(default)]
    source_entity_id: String,
    #[serde(default)]
    target_entity_id: String,
    source: String,
    source_type: String,
    #[serde(default)]
    source_representation: String,
    target: String,
    target_type: String,
    #[serde(default)]
    target_representation: String,
    relationship: String,
    date: String,
    confidence: u8,
    directed: bool,
    source_reference: String,
    notes: String,
    #[serde(default)]
    source_attributes: Vec<(String, String)>,
    #[serde(default)]
    target_attributes: Vec<(String, String)>,
    #[serde(default)]
    attributes: Vec<(String, String)>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConnectionSnapshot {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    #[serde(default)]
    chart_id: Option<Uuid>,
    series: String,
    source_key: String,
    database_path: String,
    #[serde(default)]
    configured_database_path: String,
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    captured_at: String,
    #[serde(default)]
    sqlite_schema_version: Option<i64>,
    #[serde(default)]
    sqlite_schema_fingerprint: Option<String>,
    #[serde(default)]
    sqlite_schema_objects: Option<Vec<SqliteSchemaObjectFingerprint>>,
    connections: Vec<ConnectionFact>,
    #[serde(default)]
    skipped_rows: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct SqliteSchemaObjectFingerprint {
    object_type: String,
    name: String,
    definition_fingerprint: String,
    #[serde(default)]
    rename_fingerprint: String,
    #[serde(default)]
    columns: Vec<SqliteSchemaColumnFingerprint>,
    #[serde(default)]
    columns_available: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct SqliteSchemaColumnFingerprint {
    name: String,
    #[serde(default)]
    declared_type: String,
    #[serde(default)]
    not_null: bool,
    #[serde(default)]
    primary_key_order: i64,
    #[serde(default)]
    hidden: i64,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationJob {
    id: Uuid,
    #[serde(default)]
    chart_id: Option<Uuid>,
    name: String,
    database_path: String,
    series: String,
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    #[serde(default = "default_sql_query_timeout_minutes")]
    query_timeout_minutes: u16,
    enabled: bool,
    #[serde(default)]
    minimum_connections: usize,
    #[serde(default)]
    minimum_confidence: Option<u8>,
    #[serde(default)]
    maximum_connections: Option<usize>,
    #[serde(default)]
    maximum_regressions: Option<usize>,
    #[serde(default)]
    maximum_added_connections: Option<usize>,
    #[serde(default)]
    maximum_connection_property_changes: Option<usize>,
    #[serde(default)]
    maximum_entity_property_changes: Option<usize>,
    #[serde(default)]
    maximum_skipped_rows: Option<usize>,
    #[serde(default)]
    require_unchanged_schema: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationJobPackage {
    format: String,
    schema_version: u32,
    #[serde(default)]
    database_base_path: Option<String>,
    job: AutomationJobPackageDefinition,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationJobPackageDefinition {
    name: String,
    #[serde(default)]
    chart_title: Option<String>,
    database_path: String,
    series: String,
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    #[serde(default = "default_sql_query_timeout_minutes")]
    query_timeout_minutes: u16,
    #[serde(default)]
    minimum_connections: usize,
    #[serde(default)]
    minimum_confidence: Option<u8>,
    #[serde(default)]
    maximum_connections: Option<usize>,
    #[serde(default)]
    maximum_regressions: Option<usize>,
    #[serde(default)]
    maximum_added_connections: Option<usize>,
    #[serde(default)]
    maximum_connection_property_changes: Option<usize>,
    #[serde(default)]
    maximum_entity_property_changes: Option<usize>,
    #[serde(default)]
    maximum_skipped_rows: Option<usize>,
    #[serde(default)]
    require_unchanged_schema: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct ImportDesignPackage {
    format: String,
    schema_version: u32,
    design: ImportDesignPackageDefinition,
}

#[derive(Clone, Serialize, Deserialize)]
struct ImportDesignPackageDefinition {
    name: String,
    #[serde(default)]
    mapping: ImportColumnMapping,
    #[serde(default)]
    header_signature: Vec<String>,
}

#[derive(Deserialize)]
struct TranscriptionCandidateManifest {
    schema: String,
    source_file: String,
    source_sha256: String,
    source_bytes: u64,
    page_count: usize,
    normalizer: String,
    trans_head: String,
    ocr_engine: String,
    ocr_render_dpi: u16,
    output_file: String,
    output_sha256: String,
    human_review_required: bool,
    #[serde(default)]
    qwen_analysis_status: String,
    pages: Vec<TranscriptionCandidatePage>,
}

#[derive(Deserialize)]
struct TranscriptionCandidatePage {
    page: usize,
    render_sha256: String,
    normalized_text_sha256: String,
    requires_review: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationRun {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    #[serde(default)]
    batch_id: Option<Uuid>,
    #[serde(default)]
    batch_configuration_fingerprint: Option<String>,
    job_id: Uuid,
    #[serde(default)]
    chart_id: Option<Uuid>,
    job_name: String,
    #[serde(default)]
    database_path: String,
    #[serde(default)]
    series: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    #[serde(default = "default_sql_query_timeout_minutes")]
    query_timeout_minutes: u16,
    #[serde(default)]
    parameter_overrides: Option<Vec<(String, String)>>,
    #[serde(default)]
    minimum_connections: usize,
    #[serde(default)]
    minimum_confidence: Option<u8>,
    #[serde(default)]
    maximum_connections: Option<usize>,
    #[serde(default)]
    maximum_regressions: Option<usize>,
    #[serde(default)]
    maximum_added_connections: Option<usize>,
    #[serde(default)]
    maximum_connection_property_changes: Option<usize>,
    #[serde(default)]
    maximum_entity_property_changes: Option<usize>,
    #[serde(default)]
    maximum_skipped_rows: Option<usize>,
    #[serde(default)]
    require_unchanged_schema: bool,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    configuration_recorded: bool,
    #[serde(default = "default_automation_run_trigger")]
    trigger: String,
    started_at: String,
    finished_at: String,
    #[serde(default)]
    duration_ms: Option<u64>,
    #[serde(default)]
    sqlite_lock_retries: usize,
    success: bool,
    message: String,
    #[serde(default)]
    schema_diagnosis: Option<SqlSchemaDiagnosis>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    schema_adaptation: Option<SqlSchemaAdaptation>,
    #[serde(default)]
    analyst_note: String,
    #[serde(default)]
    regressions_reviewed: Option<bool>,
    #[serde(default)]
    snapshot_index: Option<usize>,
    #[serde(default)]
    snapshot_id: Option<Uuid>,
    #[serde(default)]
    rerun_of: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SqlSchemaDiagnosis {
    baseline_available: bool,
    schema_changed: Option<bool>,
    baseline_fingerprint: Option<String>,
    current_fingerprint: Option<String>,
    query_changed: Option<bool>,
    parameters_changed: Option<bool>,
    summary: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct SqlSchemaAdaptation {
    kind: String,
    #[serde(default)]
    reference: Option<String>,
    #[serde(default)]
    hint: Option<String>,
}

fn automation_run_schema_adaptation_kind(run: &AutomationRun) -> Option<&str> {
    run.schema_adaptation
        .as_ref()
        .map(|adaptation| adaptation.kind.as_str())
        .or_else(|| sql_schema_adaptation_kind(&run.message))
}

fn automation_run_schema_adaptation_reference(run: &AutomationRun) -> Option<String> {
    run.schema_adaptation
        .as_ref()
        .and_then(|adaptation| adaptation.reference.clone())
        .or_else(|| sql_schema_adaptation_reference(&run.message))
}

fn automation_run_schema_adaptation_hint(run: &AutomationRun) -> Option<&str> {
    run.schema_adaptation
        .as_ref()
        .and_then(|adaptation| adaptation.hint.as_deref())
        .or_else(|| sql_schema_adaptation_hint(&run.message))
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationBatchCheckpoint {
    batch_id: Uuid,
    #[serde(default = "default_automation_batch_trigger")]
    created_trigger: String,
    #[serde(default)]
    has_failed_attempt: Option<bool>,
    configuration_fingerprint: String,
    #[serde(default)]
    selected_jobs: Vec<AutomationBatchJobSelection>,
    #[serde(default)]
    effective_jobs: Vec<AutomationJob>,
    #[serde(default, alias = "successful_jobs")]
    completed_jobs: Vec<AutomationJobCheckpoint>,
}

fn default_automation_batch_trigger() -> String {
    "legacy".into()
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationBatchJobSelection {
    job_id: Uuid,
    job_name: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationJobCheckpoint {
    job_id: Uuid,
    run_id: Uuid,
    finished_at: String,
    #[serde(default = "default_checkpoint_success")]
    success: bool,
    #[serde(default)]
    message: String,
    #[serde(default)]
    duration_ms: Option<u64>,
    #[serde(default)]
    sqlite_lock_retries: usize,
    #[serde(default)]
    snapshot_id: Option<Uuid>,
}

fn default_checkpoint_success() -> bool {
    true
}

fn default_automation_run_trigger() -> String {
    "legacy".into()
}

fn automation_run_trigger_label(trigger: &str) -> &str {
    match trigger {
        "desktop" => "Desktop",
        "scheduled" => "Scheduled",
        "cli" => "Command line",
        "replay" => "Replay",
        "legacy" => "Older run",
        other => other,
    }
}

struct SqlValidationPreview {
    configured_database_path: String,
    series: String,
    query: String,
    parameters: Vec<(String, String)>,
    captured_at: String,
    connection_count: usize,
    skipped_rows: usize,
    has_baseline: bool,
    regression_count: usize,
    added_connection_count: usize,
    connection_property_change_count: usize,
    entity_property_change_count: usize,
    below_minimum_confidence_count: usize,
    change_count: usize,
    changes: Vec<ConnectionDelta>,
    regression_changes: Vec<ConnectionDelta>,
    result_quality: String,
    minimum_connections: usize,
    minimum_confidence: Option<u8>,
    maximum_connections: Option<usize>,
    maximum_regressions: Option<usize>,
    maximum_added_connections: Option<usize>,
    maximum_connection_property_changes: Option<usize>,
    maximum_entity_property_changes: Option<usize>,
    maximum_skipped_rows: Option<usize>,
    connections: Vec<ConnectionFact>,
    skipped_endpoints: Vec<SkippedSqlEndpoint>,
}

#[derive(Clone)]
struct SkippedSqlEndpoint {
    connection_id: String,
    source: String,
    source_entity_id: String,
    target: String,
    target_entity_id: String,
}

#[derive(Clone)]
struct SqlPlanStep {
    id: i64,
    parent: i64,
    detail: String,
}

struct SqlPlanResult {
    job_id: Option<Uuid>,
    draft: bool,
    job_name: String,
    database_path: String,
    case_path: String,
    query: String,
    parameters: Vec<(String, String)>,
    result: Result<Vec<SqlPlanStep>, String>,
}

struct SqlEntitySourceRows {
    snapshot_id: Uuid,
    source_entity_id: String,
    database_path: String,
    queried_at: String,
    columns: Vec<String>,
    rows: Vec<Vec<String>>,
    rows_scanned: usize,
    scan_limit_reached: bool,
    truncated: bool,
}

struct SqlEntitySourceLookupRequest {
    request_id: Uuid,
    snapshot_id: Uuid,
    chart_entity_id: Uuid,
    case_path: String,
}

struct SqlEntitySourceLookupMessage {
    request: SqlEntitySourceLookupRequest,
    result: Result<SqlEntitySourceRows, String>,
}

#[derive(Default)]
struct SqlPlanSummary {
    scan_steps: usize,
    unindexed_scan_steps: usize,
    indexed_search_steps: usize,
    indexed_scan_steps: usize,
}

fn sql_plan_summary(steps: &[SqlPlanStep]) -> SqlPlanSummary {
    let mut summary = SqlPlanSummary::default();
    for step in steps {
        let detail = step.detail.to_ascii_uppercase();
        let is_scan = detail.contains("SCAN ");
        let uses_index = detail.contains("USING INDEX")
            || detail.contains("USING COVERING INDEX")
            || detail.contains("AUTOMATIC INDEX")
            || detail.contains("AUTOMATIC COVERING INDEX")
            || detail.contains("USING INTEGER PRIMARY KEY")
            || detail.contains("USING PRIMARY KEY");
        if is_scan {
            summary.scan_steps += 1;
            if uses_index {
                summary.indexed_scan_steps += 1;
            } else if !detail.contains("VIRTUAL TABLE INDEX") && !detail.contains("CONSTANT ROW") {
                summary.unindexed_scan_steps += 1;
            }
        } else if detail.starts_with("SEARCH ") && uses_index {
            summary.indexed_search_steps += 1;
        }
    }
    summary
}

fn sql_plan_step_depths(steps: &[SqlPlanStep]) -> Vec<usize> {
    let parents: HashMap<i64, i64> = steps.iter().map(|step| (step.id, step.parent)).collect();
    steps
        .iter()
        .map(|step| {
            let mut depth = 0;
            let mut current = step.id;
            let mut seen = HashSet::new();
            seen.insert(current);
            while let Some(parent) = parents.get(&current).copied() {
                if parent == current || !seen.insert(parent) {
                    break;
                }
                depth += 1;
                current = parent;
            }
            depth
        })
        .collect()
}

#[derive(Clone)]
struct SqlQueryTemplate {
    object_name: String,
    query: String,
    endpoint_identity_count: usize,
    has_stable_connection_id: bool,
    endpoint_join_count: usize,
    recognized_field_count: usize,
}
type SqlSchemaTemplates = Vec<SqlQueryTemplate>;
struct SqliteSchemaInventory {
    objects: Vec<serde_json::Value>,
    schema_version: i64,
    schema_fingerprint: String,
    schema_object_fingerprints: Vec<SqliteSchemaObjectFingerprint>,
}
type SqlSchemaInspection = Result<(SqlSchemaTemplates, Vec<String>, SqliteSchemaInventory), String>;

struct AutomationParameterOverrides {
    global: Vec<(String, String)>,
    scoped: Vec<(String, String, String)>,
}

#[derive(Clone, Serialize, Deserialize)]
struct CaseFile {
    #[serde(default)]
    format_version: u32,
    title: String,
    #[serde(default)]
    description: String,
    entities: Vec<Entity>,
    links: Vec<Link>,
    #[serde(default)]
    labels: Vec<ChartLabel>,
    #[serde(default = "Uuid::new_v4")]
    active_chart_id: Uuid,
    #[serde(default = "default_chart_title")]
    active_chart_title: String,
    #[serde(default = "default_chart_zoom")]
    active_chart_zoom: f32,
    #[serde(default)]
    active_chart_pan: [f32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    chart_coordinate_bounds: Option<[f32; 4]>,
    #[serde(default)]
    chart_tabs: Vec<ChartDocument>,
    #[serde(default)]
    chart_templates: Vec<ChartTemplate>,
    #[serde(default)]
    active_chart_template: Option<Uuid>,
    #[serde(default)]
    snapshots: Vec<ConnectionSnapshot>,
    #[serde(default)]
    automation_jobs: Vec<AutomationJob>,
    #[serde(default)]
    automation_runs: Vec<AutomationRun>,
    #[serde(default)]
    automation_batches: Vec<AutomationBatchCheckpoint>,
    #[serde(default)]
    import_specifications: Vec<ImportSpecification>,
    #[serde(default)]
    graphml_source_catalogs: Vec<serde_json::Value>,
    #[serde(default)]
    saved_paths: Vec<SavedPath>,
    #[serde(default)]
    selection_sets: Vec<SavedSelectionSet>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChartDocument {
    id: Uuid,
    title: String,
    #[serde(default = "default_chart_zoom")]
    zoom: f32,
    #[serde(default)]
    pan: [f32; 2],
    #[serde(default, skip_serializing_if = "Option::is_none")]
    coordinate_bounds: Option<[f32; 4]>,
    entities: Vec<Entity>,
    links: Vec<Link>,
    #[serde(default)]
    labels: Vec<ChartLabel>,
    #[serde(default)]
    template_id: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChartTemplate {
    id: Uuid,
    name: String,
    entity_styles: Vec<ChartTemplateEntityStyle>,
    relationship_types: Vec<String>,
    #[serde(default)]
    conditional_rules: Vec<ChartLegendEntry>,
    #[serde(default)]
    conditional_rules_truncated: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct ChartTemplateEntityStyle {
    kind: String,
    representation: EntityRepresentation,
    color: [u8; 3],
}

#[derive(Clone, Serialize, Deserialize)]
struct ChartLabel {
    id: Uuid,
    text: String,
    x: f32,
    y: f32,
    #[serde(default = "default_chart_label_font_size")]
    font_size: f32,
    #[serde(default = "default_chart_label_color")]
    color: [u8; 3],
    #[serde(default)]
    is_legend: bool,
    #[serde(default)]
    legend_entries: Vec<ChartLegendEntry>,
    #[serde(default)]
    legend_entries_truncated: bool,
    #[serde(default)]
    template_rules_for: Option<Uuid>,
}

const MAX_CHART_LEGEND_ENTRIES: usize = 24;
const MAX_CHART_LEGEND_DESCRIPTION_BYTES: usize = 4_096;
const MAX_CHART_LEGEND_CONDITIONS: usize = 8;
const MAX_CHART_TEMPLATE_CONDITIONAL_RULES: usize = MAX_CHART_LEGEND_ENTRIES * 4;
const PROXIMITY_MAP_SELECTED_EDGE_COLOR: egui::Color32 = egui::Color32::from_rgb(244, 224, 166);
const PROXIMITY_MAP_SELECTED_NODE_COLOR: egui::Color32 = egui::Color32::from_rgb(244, 224, 166);
const PROXIMITY_MAP_DIRECTION_COLOR: egui::Color32 = egui::Color32::from_rgb(84, 194, 205);
const PROXIMITY_MAP_ROUTE_SEGMENTS: usize = 12;

fn proximity_map_node_radius(member_count: usize) -> f32 {
    (15.0 + (member_count as f32).max(1.0).log2() * 2.0).clamp(15.0, 23.0)
}

fn proximity_map_node_hit_radius(member_count: usize, is_proxy: bool) -> f32 {
    proximity_map_node_radius(member_count) + if is_proxy { 7.0 } else { 4.0 }
}

fn proximity_map_node_position(column: usize, row: usize) -> egui::Pos2 {
    egui::pos2(74.0 + column as f32 * 148.0, 50.0 + row as f32 * 64.0)
}

fn proximity_map_edge_route(
    first: egui::Pos2,
    second: egui::Pos2,
    interaction_index: usize,
) -> [egui::Pos2; PROXIMITY_MAP_ROUTE_SEGMENTS + 1] {
    let control = proximity_map_edge_control_point(first, second, interaction_index);
    std::array::from_fn(|step| {
        let t = step as f32 / PROXIMITY_MAP_ROUTE_SEGMENTS as f32;
        let inverse = 1.0 - t;
        let point = first.to_vec2() * (inverse * inverse)
            + control.to_vec2() * (2.0 * inverse * t)
            + second.to_vec2() * (t * t);
        egui::pos2(point.x, point.y)
    })
}

fn proximity_map_edge_control_point(
    first: egui::Pos2,
    second: egui::Pos2,
    interaction_index: usize,
) -> egui::Pos2 {
    let delta = second - first;
    let length = delta.length();
    let normal = if length > f32::EPSILON {
        egui::vec2(-delta.y / length, delta.x / length)
    } else {
        egui::Vec2::ZERO
    };
    let lane = (interaction_index % 9) as f32 - 4.0;
    first + delta * 0.5 + normal * lane * 5.0
}

fn proximity_map_route_length(route: &[egui::Pos2]) -> f32 {
    route
        .windows(2)
        .map(|segment| segment[0].distance(segment[1]))
        .sum()
}

fn proximity_map_edge_may_intersect_viewport(
    first: egui::Pos2,
    second: egui::Pos2,
    interaction_index: usize,
    viewport: egui::Rect,
) -> bool {
    let control = proximity_map_edge_control_point(first, second, interaction_index);
    if !first.x.is_finite()
        || !first.y.is_finite()
        || !second.x.is_finite()
        || !second.y.is_finite()
        || !control.x.is_finite()
        || !control.y.is_finite()
    {
        return true;
    }
    // A quadratic Bézier stays within the convex hull of its endpoints and
    // control point, so this coarse test is conservative before route sampling.
    let minimum = egui::pos2(
        first.x.min(second.x).min(control.x),
        first.y.min(second.y).min(control.y),
    );
    let maximum = egui::pos2(
        first.x.max(second.x).max(control.x),
        first.y.max(second.y).max(control.y),
    );
    // Keep route strokes, arrows and the 8-point picking tolerance available
    // when their centerline sits just beyond the scroll viewport edge.
    egui::Rect::from_min_max(minimum, maximum)
        .expand(10.0)
        .intersects(viewport)
}

fn proximity_map_route_point_with_offset(
    route: &[egui::Pos2],
    distance: f32,
    offset: f32,
) -> egui::Pos2 {
    let mut remaining = distance.max(0.0);
    for segment in route.windows(2) {
        let length = segment[0].distance(segment[1]);
        if remaining <= length {
            let t = if length > f32::EPSILON {
                remaining / length
            } else {
                0.0
            };
            let direction = (segment[1] - segment[0]).normalized();
            let normal = egui::vec2(-direction.y, direction.x);
            return segment[0].lerp(segment[1], t) + normal * offset;
        }
        remaining -= length;
    }
    route.last().copied().unwrap_or(egui::Pos2::ZERO)
}

fn proximity_map_route_tangent(route: &[egui::Pos2], distance: f32) -> egui::Vec2 {
    let mut remaining = distance.max(0.0);
    for segment in route.windows(2) {
        let delta = segment[1] - segment[0];
        let length = delta.length();
        if remaining <= length && length > f32::EPSILON {
            return delta / length;
        }
        remaining -= length;
    }
    route
        .windows(2)
        .rev()
        .find_map(|segment| {
            let delta = segment[1] - segment[0];
            (delta.length_sq() > f32::EPSILON).then(|| delta.normalized())
        })
        .unwrap_or(egui::Vec2::ZERO)
}

fn proximity_map_paint_direction_markers(
    painter: &egui::Painter,
    route: &[egui::Pos2],
    forward_count: usize,
    reverse_count: usize,
) {
    let route_length = proximity_map_route_length(route);
    if route_length <= f32::EPSILON {
        return;
    }

    let reciprocal = forward_count > 0 && reverse_count > 0;
    let lane = if reciprocal { 2.0 } else { 0.0 };
    let midpoint = route_length * 0.5;
    let arrow_half_length = (route_length * 0.08).clamp(4.0, 6.0);
    let arrow_center = proximity_map_route_point_with_offset(route, midpoint, lane);
    let tangent = proximity_map_route_tangent(route, midpoint);
    let stroke = egui::Stroke::new(1.2_f32, PROXIMITY_MAP_DIRECTION_COLOR);

    if forward_count > 0 {
        let start = arrow_center - tangent * arrow_half_length;
        let end = arrow_center + tangent * arrow_half_length;
        painter.arrow(start, end - start, stroke);
    }
    if reverse_count > 0 {
        let reverse_center = proximity_map_route_point_with_offset(route, midpoint, -lane);
        let reverse_tangent = -tangent;
        let reverse_start = reverse_center - reverse_tangent * arrow_half_length;
        let reverse_end = reverse_center + reverse_tangent * arrow_half_length;
        painter.arrow(reverse_start, reverse_end - reverse_start, stroke);
    }
}

fn proximity_map_distance_to_segment(
    point: egui::Pos2,
    first: egui::Pos2,
    second: egui::Pos2,
) -> f32 {
    let segment = second - first;
    let length_sq = segment.length_sq();
    if length_sq <= f32::EPSILON {
        return point.distance(first);
    }
    let t = ((point - first).dot(segment) / length_sq).clamp(0.0, 1.0);
    point.distance(first + segment * t)
}

fn proximity_map_height(row_count: usize) -> f32 {
    50.0 + row_count as f32 * 64.0
}

fn count_with_noun(count: usize, singular: &str, plural: &str) -> String {
    format!("{count} {}", if count == 1 { singular } else { plural })
}

fn chart_template_saved_status(
    action: &str,
    name: &str,
    conditional_rule_count: usize,
    conditional_rules_truncated: bool,
) -> String {
    let mut status = format!("CHART TEMPLATE {action} · {name}");
    if conditional_rule_count > 0 {
        status.push_str(&format!(" · {conditional_rule_count} CONDITIONAL RULES"));
    }
    if conditional_rules_truncated {
        status.push_str(" · RULE LIMIT REACHED");
    }
    status
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct ChartLegendEntry {
    description: String,
    #[serde(default)]
    symbol: ChartLegendSymbol,
    #[serde(default = "default_chart_legend_entry_color")]
    color: [u8; 3],
    #[serde(default)]
    attribute_class: Option<String>,
    #[serde(default)]
    show_on_chart: bool,
    #[serde(default)]
    conditional_color: bool,
    #[serde(default)]
    match_exact_value: bool,
    #[serde(default)]
    condition: ChartLegendCondition,
    #[serde(default)]
    conditional_value: String,
    #[serde(default)]
    additional_conditions: Vec<ChartLegendConditionRule>,
    #[serde(default)]
    additional_conditions_match_any: bool,
    #[serde(default)]
    conditional_line_width: Option<f32>,
}

#[derive(Clone, PartialEq, Serialize, Deserialize)]
struct ChartLegendConditionRule {
    #[serde(default)]
    attribute_class: String,
    #[serde(default)]
    condition: ChartLegendCondition,
    #[serde(default)]
    value: String,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChartLegendCondition {
    #[default]
    AnyValue,
    IsSet,
    IsAbsent,
    Equal,
    NotEqual,
    GreaterThan,
    GreaterThanOrEqual,
    LessThan,
    LessThanOrEqual,
    BeforeDate,
    OnOrBeforeDate,
    AfterDate,
    OnOrAfterDate,
    Contains,
    DoesNotContain,
    StartsWith,
    EndsWith,
}

impl ChartLegendCondition {
    const fn options() -> &'static [Self] {
        &[
            Self::AnyValue,
            Self::IsSet,
            Self::IsAbsent,
            Self::Equal,
            Self::NotEqual,
            Self::GreaterThan,
            Self::GreaterThanOrEqual,
            Self::LessThan,
            Self::LessThanOrEqual,
            Self::BeforeDate,
            Self::OnOrBeforeDate,
            Self::AfterDate,
            Self::OnOrAfterDate,
            Self::Contains,
            Self::DoesNotContain,
            Self::StartsWith,
            Self::EndsWith,
        ]
    }

    fn label(self) -> &'static str {
        match self {
            Self::AnyValue => "Any value",
            Self::IsSet => "Is set",
            Self::IsAbsent => "Is absent",
            Self::Equal => "Equal to",
            Self::NotEqual => "Not equal to",
            Self::GreaterThan => "Greater than",
            Self::GreaterThanOrEqual => "At least",
            Self::LessThan => "Less than",
            Self::LessThanOrEqual => "At most",
            Self::BeforeDate => "Before date",
            Self::OnOrBeforeDate => "On or before date",
            Self::AfterDate => "After date",
            Self::OnOrAfterDate => "On or after date",
            Self::Contains => "Contains",
            Self::DoesNotContain => "Does not contain",
            Self::StartsWith => "Starts with",
            Self::EndsWith => "Ends with",
        }
    }
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum ChartLegendSymbol {
    #[default]
    Marker,
    Entity,
    Relationship,
    AttributeCircle,
    AttributeSquare,
    AttributeTriangle,
    AttributeDiamond,
}

impl ChartLegendSymbol {
    fn is_attribute(self) -> bool {
        matches!(
            self,
            Self::AttributeCircle
                | Self::AttributeSquare
                | Self::AttributeTriangle
                | Self::AttributeDiamond
        )
    }

    fn label(self) -> &'static str {
        match self {
            Self::Marker => "Marker",
            Self::Entity => "Entity",
            Self::Relationship => "Link",
            Self::AttributeCircle => "Attribute · circle",
            Self::AttributeSquare => "Attribute · square",
            Self::AttributeTriangle => "Attribute · triangle",
            Self::AttributeDiamond => "Attribute · diamond",
        }
    }
}

fn paint_attribute_symbol(
    painter: &egui::Painter,
    center: egui::Pos2,
    zoom: f32,
    symbol: ChartLegendSymbol,
    color: egui::Color32,
) {
    let radius = 4.0 * zoom.clamp(0.55, 3.0);
    match symbol {
        ChartLegendSymbol::AttributeCircle => {
            painter.circle_filled(center, radius, color);
        }
        ChartLegendSymbol::AttributeSquare => {
            painter.rect_filled(
                egui::Rect::from_center_size(center, egui::vec2(radius * 2.0, radius * 2.0)),
                1.0,
                color,
            );
        }
        ChartLegendSymbol::AttributeTriangle => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    center + egui::vec2(0.0, -radius),
                    center + egui::vec2(radius, radius),
                    center + egui::vec2(-radius, radius),
                ],
                color,
                egui::Stroke::NONE,
            ));
        }
        ChartLegendSymbol::AttributeDiamond => {
            painter.add(egui::Shape::convex_polygon(
                vec![
                    center + egui::vec2(0.0, -radius),
                    center + egui::vec2(radius, 0.0),
                    center + egui::vec2(0.0, radius),
                    center + egui::vec2(-radius, 0.0),
                ],
                color,
                egui::Stroke::NONE,
            ));
        }
        _ => {}
    }
}

fn default_chart_legend_entry_color() -> [u8; 3] {
    [73, 211, 207]
}

fn default_chart_label_font_size() -> f32 {
    13.0
}

fn default_chart_label_color() -> [u8; 3] {
    [225, 232, 237]
}

fn default_chart_zoom() -> f32 {
    1.0
}

fn chart_view_needs_initial_fit(case: &CaseFile) -> bool {
    case.chart_coordinate_bounds.is_none()
        && case.active_chart_zoom == default_chart_zoom()
        && case.active_chart_pan == [0.0, 0.0]
}

fn clamp_chart_zoom(zoom: f32) -> f32 {
    if zoom.is_finite() {
        zoom.clamp(0.25, 3.0)
    } else {
        default_chart_zoom()
    }
}

fn chart_pan_from_saved(pan: [f32; 2]) -> egui::Vec2 {
    egui::Vec2::new(
        if pan[0].is_finite() { pan[0] } else { 0.0 },
        if pan[1].is_finite() { pan[1] } else { 0.0 },
    )
}

fn chart_label_bounds(text: &str, center: egui::Pos2, zoom: f32, font_size: f32) -> egui::Rect {
    let font_size = font_size.clamp(8.0, 48.0);
    let lines = text.lines().collect::<Vec<_>>();
    let width = lines
        .iter()
        .map(|line| line.chars().count())
        .max()
        .unwrap_or(1)
        .max(1) as f32
        * font_size
        * 0.62
        * zoom;
    let height = lines.len().max(1) as f32 * font_size * 1.35 * zoom;
    egui::Rect::from_center_size(
        center,
        egui::vec2(width + 20.0 * zoom, height + 12.0 * zoom),
    )
}

fn chart_canvas_item_bounds(label: &ChartLabel, center: egui::Pos2, zoom: f32) -> egui::Rect {
    if !label.is_legend {
        return chart_label_bounds(&label.text, center, zoom, label.font_size);
    }
    let font_size = label.font_size.clamp(8.0, 16.0);
    let description_width = label
        .legend_entries
        .iter()
        .map(|entry| entry.description.chars().count())
        .max()
        .unwrap_or(12)
        .max(label.text.chars().count())
        .max(12) as f32
        * font_size
        * 0.58;
    let minimum_width = if label.legend_entries_truncated {
        260.0
    } else {
        128.0
    };
    let width = (description_width + 44.0).clamp(minimum_width, 860.0) * zoom;
    let footer_rows = if label.legend_entries_truncated {
        1.0
    } else {
        0.0
    };
    let height =
        (font_size * 1.5 * (label.legend_entries.len() as f32 + 1.0 + footer_rows) + 20.0) * zoom;
    egui::Rect::from_center_size(center, egui::vec2(width, height))
}

fn chart_legend_entries(case: &CaseFile) -> (Vec<ChartLegendEntry>, bool) {
    let mut entries = Vec::new();
    let mut seen_entity_types = HashSet::new();
    let mut seen_relationship_types = HashSet::new();
    let mut seen_attribute_classes = HashSet::new();
    let mut truncated = false;
    for entity in &case.entities {
        let description = entity.kind.trim();
        if !description.is_empty() && seen_entity_types.insert(description.to_ascii_lowercase()) {
            if entries.len() >= MAX_CHART_LEGEND_ENTRIES {
                truncated = true;
                continue;
            }
            entries.push(ChartLegendEntry {
                description: truncate_chart_label(description, 1_024),
                symbol: ChartLegendSymbol::Entity,
                color: entity.color,
                attribute_class: None,
                show_on_chart: false,
                conditional_color: false,
                match_exact_value: false,
                condition: ChartLegendCondition::AnyValue,
                conditional_value: String::new(),
                additional_conditions: Vec::new(),
                additional_conditions_match_any: false,
                conditional_line_width: None,
            });
        }
    }
    for link in &case.links {
        let description = link.label.trim();
        if !description.is_empty()
            && seen_relationship_types.insert(description.to_ascii_lowercase())
        {
            if entries.len() >= MAX_CHART_LEGEND_ENTRIES {
                truncated = true;
                continue;
            }
            entries.push(ChartLegendEntry {
                description: truncate_chart_label(description, 1_024),
                symbol: ChartLegendSymbol::Relationship,
                color: default_chart_legend_entry_color(),
                attribute_class: None,
                show_on_chart: false,
                conditional_color: false,
                match_exact_value: false,
                condition: ChartLegendCondition::AnyValue,
                conditional_value: String::new(),
                additional_conditions: Vec::new(),
                additional_conditions_match_any: false,
                conditional_line_width: None,
            });
        }
    }
    for entity in &case.entities {
        for (attribute_name, _) in &entity.attributes {
            let attribute_name = attribute_name.trim();
            if attribute_name.is_empty()
                || !seen_attribute_classes.insert(attribute_name.to_ascii_lowercase())
            {
                continue;
            }
            if attribute_name.len() > MAX_CHART_LEGEND_DESCRIPTION_BYTES {
                truncated = true;
                continue;
            }
            if entries.len() >= MAX_CHART_LEGEND_ENTRIES {
                truncated = true;
                continue;
            }
            entries.push(ChartLegendEntry {
                description: truncate_chart_label(attribute_name, 1_024),
                symbol: ChartLegendSymbol::AttributeCircle,
                color: default_chart_legend_entry_color(),
                attribute_class: Some(attribute_name.to_owned()),
                show_on_chart: false,
                conditional_color: false,
                match_exact_value: false,
                condition: ChartLegendCondition::AnyValue,
                conditional_value: String::new(),
                additional_conditions: Vec::new(),
                additional_conditions_match_any: false,
                conditional_line_width: None,
            });
        }
    }
    for link in &case.links {
        for (attribute_name, _) in &link.attributes {
            let attribute_name = attribute_name.trim();
            if attribute_name.is_empty()
                || !seen_attribute_classes.insert(attribute_name.to_ascii_lowercase())
            {
                continue;
            }
            if attribute_name.len() > MAX_CHART_LEGEND_DESCRIPTION_BYTES
                || entries.len() >= MAX_CHART_LEGEND_ENTRIES
            {
                truncated = true;
                continue;
            }
            entries.push(ChartLegendEntry {
                description: truncate_chart_label(attribute_name, 1_024),
                symbol: ChartLegendSymbol::AttributeCircle,
                color: default_chart_legend_entry_color(),
                attribute_class: Some(attribute_name.to_owned()),
                show_on_chart: false,
                conditional_color: false,
                match_exact_value: false,
                condition: ChartLegendCondition::AnyValue,
                conditional_value: String::new(),
                additional_conditions: Vec::new(),
                additional_conditions_match_any: false,
                conditional_line_width: None,
            });
        }
    }
    (entries, truncated)
}

fn chart_conditional_attribute_color(
    attributes: &[(String, String)],
    labels: &[ChartLabel],
) -> Option<[u8; 3]> {
    chart_matching_conditional_entry(attributes, labels).map(|entry| entry.color)
}

fn chart_conditional_attribute_line_width(
    attributes: &[(String, String)],
    labels: &[ChartLabel],
) -> Option<f32> {
    chart_matching_conditional_entry(attributes, labels)
        .and_then(|entry| entry.conditional_line_width)
}

fn chart_matching_conditional_entry<'a>(
    attributes: &[(String, String)],
    labels: &'a [ChartLabel],
) -> Option<&'a ChartLegendEntry> {
    let template_entries = labels
        .iter()
        .filter(|label| label.is_legend && label.template_rules_for.is_some())
        .flat_map(|label| &label.legend_entries);
    let chart_entries = labels
        .iter()
        .filter(|label| label.is_legend && label.template_rules_for.is_none())
        .flat_map(|label| &label.legend_entries);
    for entry in template_entries.chain(chart_entries) {
        if !entry.conditional_color || !entry.symbol.is_attribute() {
            continue;
        }
        let Some(attribute_class) = entry
            .attribute_class
            .as_deref()
            .map(str::trim)
            .filter(|name| !name.is_empty())
        else {
            continue;
        };
        let value = attributes
            .iter()
            .find(|(name, _)| name.trim().eq_ignore_ascii_case(attribute_class))
            .map(|(_, value)| value.as_str());
        let primary_matches = if entry.match_exact_value {
            value.is_some_and(|value| value.eq_ignore_ascii_case(&entry.conditional_value))
        } else {
            chart_legend_condition_matches(entry.condition, value, &entry.conditional_value)
        };
        let additional_match = if entry.additional_conditions.is_empty() {
            true
        } else if entry.additional_conditions_match_any {
            entry
                .additional_conditions
                .iter()
                .any(|rule| chart_legend_additional_condition_matches(attributes, rule))
        } else {
            entry
                .additional_conditions
                .iter()
                .all(|rule| chart_legend_additional_condition_matches(attributes, rule))
        };
        if primary_matches && additional_match {
            return Some(entry);
        }
    }
    None
}

fn chart_legend_additional_condition_matches(
    attributes: &[(String, String)],
    rule: &ChartLegendConditionRule,
) -> bool {
    if rule.attribute_class.trim().is_empty() {
        return false;
    }
    let value = attributes
        .iter()
        .find(|(name, _)| {
            name.trim()
                .eq_ignore_ascii_case(rule.attribute_class.trim())
        })
        .map(|(_, value)| value.as_str());
    chart_legend_condition_matches(rule.condition, value, &rule.value)
}

fn chart_legend_condition_matches(
    condition: ChartLegendCondition,
    value: Option<&str>,
    expected: &str,
) -> bool {
    match (condition, value) {
        (ChartLegendCondition::AnyValue | ChartLegendCondition::IsSet, Some(_)) => true,
        (ChartLegendCondition::IsAbsent, None) => true,
        (ChartLegendCondition::Equal, Some(value)) => value == expected,
        (ChartLegendCondition::NotEqual, Some(value)) => value != expected,
        (ChartLegendCondition::GreaterThan, Some(value)) => {
            chart_numeric_comparison(value, expected)
                .is_some_and(|(value, threshold)| value > threshold)
        }
        (ChartLegendCondition::GreaterThanOrEqual, Some(value)) => {
            chart_numeric_comparison(value, expected)
                .is_some_and(|(value, threshold)| value >= threshold)
        }
        (ChartLegendCondition::LessThan, Some(value)) => chart_numeric_comparison(value, expected)
            .is_some_and(|(value, threshold)| value < threshold),
        (ChartLegendCondition::LessThanOrEqual, Some(value)) => {
            chart_numeric_comparison(value, expected)
                .is_some_and(|(value, threshold)| value <= threshold)
        }
        (ChartLegendCondition::BeforeDate, Some(value)) => chart_date_comparison(value, expected)
            .is_some_and(|(value, threshold)| value < threshold),
        (ChartLegendCondition::OnOrBeforeDate, Some(value)) => {
            chart_date_comparison(value, expected)
                .is_some_and(|(value, threshold)| value <= threshold)
        }
        (ChartLegendCondition::AfterDate, Some(value)) => chart_date_comparison(value, expected)
            .is_some_and(|(value, threshold)| value > threshold),
        (ChartLegendCondition::OnOrAfterDate, Some(value)) => {
            chart_date_comparison(value, expected)
                .is_some_and(|(value, threshold)| value >= threshold)
        }
        (ChartLegendCondition::Contains, Some(value)) => value.contains(expected),
        (ChartLegendCondition::DoesNotContain, Some(value)) => !value.contains(expected),
        (ChartLegendCondition::StartsWith, Some(value)) => value.starts_with(expected),
        (ChartLegendCondition::EndsWith, Some(value)) => value.ends_with(expected),
        _ => false,
    }
}

fn chart_numeric_comparison(value: &str, threshold: &str) -> Option<(f64, f64)> {
    let value = value.trim().parse::<f64>().ok()?;
    let threshold = threshold.trim().parse::<f64>().ok()?;
    (value.is_finite() && threshold.is_finite()).then_some((value, threshold))
}

fn chart_date_comparison(value: &str, threshold: &str) -> Option<(i64, i64)> {
    let ordinal = |value: &str| {
        let value = value.trim();
        (value.len() == 10)
            .then(|| parse_iso_date_ordinal(value))
            .flatten()
    };
    Some((ordinal(value)?, ordinal(threshold)?))
}

fn chart_legend_initial_position(
    case: &CaseFile,
    entries: &[ChartLegendEntry],
    truncated: bool,
) -> [f32; 2] {
    let font_size = 12.0_f32;
    let longest_description = entries
        .iter()
        .map(|entry| entry.description.chars().count())
        .max()
        .unwrap_or(12)
        .max("Chart legend".chars().count()) as f32;
    let width = (longest_description * font_size * 0.58 + 44.0).clamp(128.0, 860.0);
    let footer_rows = if truncated { 1.0 } else { 0.0 };
    let height = font_size * 1.5 * (entries.len() as f32 + 1.0 + footer_rows) + 20.0;
    let half_width = width / 2.0 / 900.0;
    let half_height = height / 2.0 / 800.0;
    let minimum_x = (half_width + 0.025).min(0.5);
    let maximum_x = 1.0 - minimum_x;
    let minimum_y = (half_height + 0.025).min(0.5);
    let maximum_y = 1.0 - minimum_y;
    let mut best_position = [minimum_x, minimum_y];
    let mut best_score = usize::MAX;
    for row in 0..5 {
        let y = minimum_y + (maximum_y - minimum_y) * row as f32 / 4.0;
        for column in 0..5 {
            let x = minimum_x + (maximum_x - minimum_x) * column as f32 / 4.0;
            let mut score = case
                .entities
                .iter()
                .filter(|entity| {
                    (entity.x - x).abs() < half_width + 0.035
                        && (entity.y - y).abs() < half_height + 0.035
                })
                .count()
                + case
                    .labels
                    .iter()
                    .filter(|label| {
                        (label.x - x).abs() < half_width + 0.035
                            && (label.y - y).abs() < half_height + 0.035
                    })
                    .count();
            let overlaps_navigator = x + half_width > 0.82
                && x - half_width < 1.0
                && y - half_height < 0.2
                && y + half_height > 0.0;
            if overlaps_navigator {
                score += 10_000;
            }
            if score < best_score {
                best_score = score;
                best_position = [x, y];
            }
        }
    }
    best_position
}

fn normalize_chart_viewports(case: &mut CaseFile) {
    case.active_chart_zoom = clamp_chart_zoom(case.active_chart_zoom);
    case.active_chart_pan = [
        if case.active_chart_pan[0].is_finite() {
            case.active_chart_pan[0]
        } else {
            0.0
        },
        if case.active_chart_pan[1].is_finite() {
            case.active_chart_pan[1]
        } else {
            0.0
        },
    ];
    for chart in &mut case.chart_tabs {
        chart.zoom = clamp_chart_zoom(chart.zoom);
        chart.pan = [
            if chart.pan[0].is_finite() {
                chart.pan[0]
            } else {
                0.0
            },
            if chart.pan[1].is_finite() {
                chart.pan[1]
            } else {
                0.0
            },
        ];
    }
}

fn default_chart_title() -> String {
    "Chart 1".into()
}

fn is_generated_chart_title(title: &str) -> bool {
    title.strip_prefix("Chart ").is_some_and(|number| {
        !number.is_empty() && number.bytes().all(|byte| byte.is_ascii_digit())
    })
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedPath {
    id: Uuid,
    name: String,
    #[serde(default)]
    chart_id: Option<Uuid>,
    start_entity_id: Uuid,
    end_entity_id: Uuid,
    follow_directions: bool,
    #[serde(default)]
    time_flow: PathTimeFlow,
    use_timeline_range: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedSelectionSet {
    id: Uuid,
    name: String,
    #[serde(default)]
    chart_id: Option<Uuid>,
    entity_ids: Vec<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedPathPackage {
    format: String,
    schema_version: u32,
    path: SavedPathPackageDefinition,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedPathPackageDefinition {
    name: String,
    start: SavedPathEndpoint,
    end: SavedPathEndpoint,
    follow_directions: bool,
    #[serde(default)]
    time_flow: PathTimeFlow,
    #[serde(default)]
    use_timeline_range: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct SavedPathEndpoint {
    id: Uuid,
    name: String,
    kind: String,
}

struct DelimitedImportInspection {
    headers: Vec<String>,
    preview: Vec<Vec<String>>,
    record_count: usize,
}

struct App {
    case: CaseFile,
    selected: Option<Uuid>,
    selected_entities: HashSet<Uuid>,
    drag_entity_origins: HashMap<Uuid, (f32, f32)>,
    selected_chart_label: Option<Uuid>,
    chart_label_creation_armed: bool,
    chart_label_drag_origin: Option<(Uuid, f32, f32)>,
    query: String,
    chart_search_scope: ChartSearchScope,
    chart_search_all_charts: bool,
    status: String,
    show_timeline: bool,
    timeline_range: Option<(i64, i64)>,
    timeline_brush_anchor: Option<i64>,
    chart_zoom: f32,
    chart_canvas_aspect: f32,
    analysis_pane_width: f32,
    chart_pan: egui::Vec2,
    chart_pan_origin: egui::Vec2,
    chart_focus: Option<Uuid>,
    chart_focus_link: Option<usize>,
    chart_fit_entities: Option<Vec<Uuid>>,
    chart_fit_path: Option<Vec<Uuid>>,
    chart_focus_group: Option<HashSet<Uuid>>,
    network_groups: Option<std::sync::Arc<Vec<Vec<Uuid>>>>,
    community_analysis: Option<std::sync::Arc<CommunityAnalysis>>,
    selected_community_interaction: Option<(usize, usize)>,
    network_bridges: Option<std::sync::Arc<Vec<(Uuid, usize)>>>,
    network_proximity_analysis: Option<(Uuid, std::sync::Arc<proximity::Analysis>)>,
    selected_proximity_group: Option<usize>,
    selected_proximity_interaction: Option<(usize, usize)>,
    analysis_method_card: Option<(Uuid, NetworkAnalysisMethodCard)>,
    chart_navigator_dragging: bool,
    new_link_from: Option<Uuid>,
    new_link_to: Option<Uuid>,
    new_link_label: String,
    link_creation_armed: bool,
    link_creation_from: Option<Uuid>,
    new_entity_type_text: String,
    new_link_type_text: String,
    import_path: String,
    import_into_new_chart: bool,
    import_ingest_source_override: Option<String>,
    delimited_import_counts: Option<(usize, usize, usize)>,
    import_headers: Vec<String>,
    import_preview: Vec<Vec<String>>,
    import_record_count: usize,
    import_inspection_worker: Option<Receiver<Result<DelimitedImportInspection, String>>>,
    import_inspection_cancel: Option<Arc<AtomicBool>>,
    import_inspection_progress: Option<Arc<std::sync::atomic::AtomicUsize>>,
    background_imports_enabled: bool,
    anx_import_worker: Option<Receiver<(Uuid, Result<InterchangeImportPayload, String>)>>,
    import_column_mapping: ImportColumnMapping,
    import_spec_name: String,
    import_design_package_path: String,
    import_design_package_name: String,
    import_design_replace_selected: bool,
    selected_import_spec: Option<Uuid>,
    show_import_wizard: bool,
    export_path: String,
    graphml_export_path: String,
    graphml_include_source_catalog: bool,
    kml_export_path: String,
    anx_export_path: String,
    selected_link: Option<usize>,
    show_all_relationship_labels: bool,
    chart_link_overlap_point: Option<egui::Pos2>,
    chart_link_overlap_candidates: Vec<usize>,
    chart_link_overlap_cycle: usize,
    page: WorkspacePage,
    entity_sort: EntitySort,
    most_connected_measure: MostConnectedMeasure,
    most_connected_relationship: Option<String>,
    linked_search_depth: u8,
    linked_search_follow_directions: bool,
    layout_mode: LayoutMode,
    layout_iterations: usize,
    layout_worker: Option<Receiver<CommunityLayoutWorkerResult>>,
    layout_restore: Option<CommunityLayoutRestoreSnapshot>,
    link_sort: LinkSort,
    link_table_view: LinkTableView,
    data_query: String,
    data_source_filter: Option<String>,
    connections_for_selected: bool,
    connections_within_timeline_range: bool,
    path_start: Option<Uuid>,
    path_end: Option<Uuid>,
    path_entity_filter: String,
    saved_path_name: String,
    selection_set_name: String,
    saved_path_package_path: String,
    saved_path_import_name: String,
    path_follow_directions: bool,
    path_time_flow: PathTimeFlow,
    path_use_timeline_range: bool,
    path_timeline_range: Option<(i64, i64)>,
    path_result: Vec<Uuid>,
    path_link_result: Vec<usize>,
    case_path: String,
    case_path_input: String,
    saved_case_json: Option<String>,
    disk_case_json: Option<String>,
    pending_case_action: Option<PendingCaseAction>,
    pending_chart_delete: Option<Uuid>,
    command_palette_open: bool,
    command_palette_query: String,
    data_mode: DataMode,
    merge_candidate: Option<(Uuid, Uuid)>,
    duplicate_candidates: Vec<(Uuid, Uuid)>,
    sql_path: String,
    sql_series: String,
    sql_query: String,
    sql_parameter_text: String,
    sql_cleared_parameter_draft: Option<(String, String)>,
    sql_validation_preview: Option<SqlValidationPreview>,
    sql_skipped_endpoints: Vec<SkippedSqlEndpoint>,
    sql_validation_preview_regressions_only: bool,
    sql_schema_templates: SqlSchemaTemplates,
    sql_schema_objects: Vec<serde_json::Value>,
    sql_schema_object_fingerprints: Vec<SqliteSchemaObjectFingerprint>,
    sql_schema_filter: String,
    sql_schema_template_index: usize,
    sql_schema_template_source: Option<(String, String)>,
    automation_repair_context: Option<(Option<Uuid>, String, Option<String>)>,
    automation_job_filter: String,
    automation_run_filter: String,
    automation_run_trigger_filter: String,
    automation_run_note_drafts: HashMap<Uuid, String>,
    automation_run_parameter_text: String,
    automation_preflight_require_usable_connections: bool,
    automation_preflight_require_unchanged_schema: bool,
    automation_job_import_path: String,
    automation_job_export_path: String,
    automation_job_import_name: String,
    automation_job_import_chart: String,
    automation_job_import_database_path: String,
    automation_job_import_series: String,
    automation_job_import_parameter_text: String,
    automation_failed_runs_only: bool,
    automation_schema_adaptation_only: bool,
    automation_schema_stability_only: bool,
    automation_runs_needing_review: bool,
    automation_runs_with_unreviewed_regressions: bool,
    sql_snapshot_source: Option<String>,
    sql_snapshot_index: Option<usize>,
    sql_snapshot_filter: String,
    sql_comparison_baseline_id: Option<Uuid>,
    sql_delta_filter: SnapshotDeltaFilter,
    sql_delta_search: String,
    sql_source_focus: Option<String>,
    automation_job_name: String,
    automation_job_chart_id: Uuid,
    automation_query_timeout_minutes: u16,
    automation_minimum_connections: usize,
    automation_minimum_confidence: Option<u8>,
    automation_maximum_connections: Option<usize>,
    automation_maximum_regressions: Option<usize>,
    automation_maximum_added_connections: Option<usize>,
    automation_maximum_connection_property_changes: Option<usize>,
    automation_maximum_entity_property_changes: Option<usize>,
    automation_maximum_skipped_rows: Option<usize>,
    automation_require_unchanged_schema: bool,
    editing_automation_job: Option<Uuid>,
    automation_duplicate_draft: Option<Uuid>,
    automation_editor_before_edit: Option<(
        String,
        String,
        String,
        String,
        String,
        usize,
        u16,
        Option<u8>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        Option<usize>,
        bool,
        Uuid,
    )>,
    automation_worker: Option<Receiver<Result<AutomationWorkerResult, String>>>,
    automation_progress: Option<Arc<Mutex<String>>>,
    sql_plan_worker: Option<Receiver<SqlPlanResult>>,
    sql_plan_result: Option<SqlPlanResult>,
    sql_entity_source_worker: Option<Receiver<SqlEntitySourceLookupMessage>>,
    sql_entity_source_request: Option<SqlEntitySourceLookupRequest>,
    sql_entity_source_cancel: Option<Arc<AtomicBool>>,
    sql_entity_source_progress: Option<Arc<Mutex<String>>>,
    sql_entity_source_result: Option<(Uuid, Result<SqlEntitySourceRows, String>)>,
    sql_schema_worker: Option<Receiver<(Uuid, String, String, SqlSchemaInspection)>>,
    sql_schema_request: Option<(Uuid, String, String)>,
    sql_lock_retry_count: usize,
    sql_query_timeout_minutes: u16,
    automation_schedule: String,
    automation_schedule_time: String,
    automation_schedule_weekday: u8,
    automation_schedule_month_day: u8,
    automation_schedule_require_usable_connections: bool,
    automation_chart_export_dir: String,
    automation_schedule_message: Option<(String, bool, String)>,
    automation_schedule_target_path: Option<String>,
    automation_schedule_progress: Option<Arc<Mutex<String>>>,
    automation_schedule_worker: Option<Receiver<(String, Result<String, String>)>>,
    automation_preflight_cancel: Option<Arc<AtomicBool>>,
    automation_cancel: Arc<AtomicBool>,
    close_after_automation: bool,
}

enum AutomationWorkerResult {
    SavedCase {
        case: CaseFile,
        disk_case_json: String,
        status: String,
        selected_snapshot: Option<(String, usize)>,
        selected_link: Option<usize>,
        chart_fit_entities: Option<Vec<Uuid>>,
    },
    FailedCase {
        case: CaseFile,
        disk_case_json: String,
        error: String,
        selected_snapshot: Option<(String, usize)>,
        selected_link: Option<usize>,
        chart_fit_entities: Option<Vec<Uuid>>,
    },
    UnsavedCase {
        case: CaseFile,
        status: String,
        selected_snapshot: Option<(String, usize)>,
        selected_link: Option<usize>,
        chart_fit_entities: Option<Vec<Uuid>>,
    },
    Validation {
        preview: Option<SqlValidationPreview>,
        status: String,
    },
}

struct SqlUiRequest {
    database_path: String,
    series: String,
    query: String,
    parameter_text: String,
    minimum_connections: usize,
    minimum_confidence: Option<u8>,
    maximum_connections: Option<usize>,
    maximum_regressions: Option<usize>,
    maximum_added_connections: Option<usize>,
    maximum_connection_property_changes: Option<usize>,
    maximum_entity_property_changes: Option<usize>,
    maximum_skipped_rows: Option<usize>,
    query_timeout_minutes: u16,
    data_mode: DataMode,
    editing_job: Option<Uuid>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ImportSpecification {
    id: Uuid,
    name: String,
    #[serde(default)]
    mapping: ImportColumnMapping,
    #[serde(default)]
    header_signature: Vec<String>,
}

#[derive(Default)]
struct GraphMlRecord {
    id: String,
    source: String,
    target: String,
    directed: Option<bool>,
    data: HashMap<String, String>,
}

#[derive(Default)]
struct AnxItemBuilder {
    id: String,
    label: String,
    x_position: Option<String>,
    end_x: Option<String>,
    end_y: Option<String>,
    date: Option<String>,
    entity: Option<AnxEntityBuilder>,
    link: Option<AnxLinkBuilder>,
    attributes: Vec<(String, String)>,
}

#[derive(Default)]
struct AnxEntityBuilder {
    entity_id: String,
    identity: String,
    label_is_identity: bool,
    representation: String,
    kind: String,
    color: Option<String>,
}

#[derive(Default)]
struct AnxLinkBuilder {
    end1_id: String,
    end2_id: String,
    kind: String,
    strength_id: String,
    directed: Option<bool>,
    date: String,
}

struct AnxPendingLabel {
    label: ChartLabel,
    x: Option<f32>,
    y: Option<f32>,
}

struct GraphMlGraphContext {
    default_directed: bool,
    parent_node_id: Option<String>,
}

#[derive(Default)]
struct GraphMlDocument {
    default_directed: bool,
    graph_data: HashMap<String, String>,
    source_catalogs: Vec<serde_json::Value>,
    nodes: Vec<GraphMlRecord>,
    edges: Vec<GraphMlRecord>,
}

struct InterchangeImportPayload {
    csv_bytes: Vec<u8>,
    source_catalogs: Vec<serde_json::Value>,
    chart_labels: Vec<ChartLabel>,
    has_chart_items: bool,
    chart_title: Option<String>,
}

#[derive(Clone, Default)]
struct KmlPlacemark {
    id: String,
    name: String,
    description: String,
    folder_path: String,
    data: HashMap<String, String>,
    geometries: Vec<KmlGeometryRecord>,
}

#[derive(Clone)]
struct KmlGeometryRecord {
    kind: KmlGeometry,
    coordinates: String,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum KmlGeometry {
    Point,
    LineString,
}

#[derive(Clone, Copy)]
enum KmlTextTarget {
    DocumentName,
    FolderName,
    Name,
    Description,
    Coordinates,
    DataValue,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ImportColumnMapping {
    trim_whitespace: bool,
    row_filters: Vec<ImportRowFilter>,
    value_transforms: Vec<ImportValueTransform>,
    source: String,
    target: String,
    source_type: String,
    target_type: String,
    relationship: String,
    date: String,
    confidence: String,
    direction: String,
    connection_id: String,
    notes: String,
    source_notes: String,
    source_reference: String,
    ingest_source: String,
    record_type: String,
    source_entity_id: String,
    target_entity_id: String,
    source_external_ids: String,
    target_external_ids: String,
    source_detail: String,
    target_detail: String,
    source_aliases: String,
    target_aliases: String,
    source_merged_records: String,
    target_merged_records: String,
    source_representation: String,
    target_representation: String,
    source_color: String,
    target_color: String,
    source_x: String,
    source_y: String,
    target_x: String,
    target_y: String,
    source_position_locked: String,
    target_position_locked: String,
    entity_attributes: Vec<ImportAttributeMapping>,
    connection_attributes: Vec<ImportConnectionAttributeMapping>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ImportValueTransform {
    column: String,
    operation: ImportTransformOperation,
    find: String,
    replace: String,
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
enum ImportTransformOperation {
    #[default]
    Replace,
    ReplaceExact,
    Lowercase,
    Uppercase,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ImportRowFilter {
    column: String,
    operator: ImportFilterOperator,
    value: String,
}

#[derive(Clone, Copy, Default, PartialEq, Serialize, Deserialize)]
enum ImportFilterOperator {
    #[default]
    Equals,
    Contains,
    IsEmpty,
    IsNotEmpty,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ImportAttributeMapping {
    column: String,
    key: String,
    source: bool,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ImportConnectionAttributeMapping {
    column: String,
    key: String,
}

#[derive(PartialEq, Clone, Copy)]
enum WorkspacePage {
    Chart,
    Data,
}

#[derive(Clone, Copy)]
enum PendingCaseAction {
    Open,
    New,
}

#[derive(PartialEq, Clone, Copy)]
enum DataMode {
    Entities,
    Relationships,
    Sources,
    SqlRegressions,
    Automation,
}

#[derive(Clone, Copy)]
enum QuickCommand {
    Chart,
    FitChart,
    Timeline,
    Connections,
    SqlReview,
    Automation,
    Entities,
    Sources,
    SearchChart,
    NewChart,
}

fn quick_command_matches(query: &str, label: &str, keywords: &str) -> bool {
    let filter = query.trim().to_ascii_lowercase();
    filter.is_empty()
        || label.to_ascii_lowercase().contains(&filter)
        || keywords.to_ascii_lowercase().contains(&filter)
}

#[derive(PartialEq, Clone, Copy)]
enum EntitySort {
    Name,
    Type,
    Connections,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum ChartSearchScope {
    #[default]
    All,
    Entities,
    Relationships,
}

#[derive(Default, PartialEq, Clone, Copy)]
enum MostConnectedMeasure {
    #[default]
    ConnectedEntities,
    LinkRecords,
    IncomingLinks,
    OutgoingLinks,
}

#[derive(PartialEq, Clone, Copy)]
enum LayoutMode {
    Connectivity,
    CentralEntity,
    EntityType,
    Hierarchy,
    Grid,
    ThemeLineEventDates,
    CommunityForce,
}

#[derive(PartialEq, Clone, Copy)]
enum LinkSort {
    RecordId,
    Source,
    Date,
    Relationship,
    Target,
    Confidence,
    SourceReference,
}

#[derive(PartialEq, Clone, Copy)]
enum LinkTableView {
    Connections,
    LinkRecords,
}

#[derive(Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PathTimeFlow {
    #[default]
    Any,
    Forward,
    Backward,
}

#[derive(Clone, Copy, PartialEq)]
enum DeltaKind {
    Removed,
    ConfidenceDeclined,
    DateChanged,
    EntityIdentityChanged,
    EntityAttributesChanged,
    ConnectionAttributesChanged,
    Updated,
    DirectionChanged,
    EvidenceChanged,
    SourceNotesChanged,
    Added,
    ConfidenceIncreased,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum SnapshotDeltaFilter {
    All,
    Regressions,
    EntityIdentity,
    EntityProperties,
    Direction,
    Evidence,
    SourceNotes,
    ConnectionProperties,
    Other,
}

#[derive(Clone)]
struct ConnectionDelta {
    kinds: Vec<DeltaKind>,
    connection: ConnectionFact,
    previous_connection: Option<ConnectionFact>,
    previous_confidence: Option<u8>,
    previous_date: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let data_dir = app_data_dir();
        let default_case_path = data_dir.join("harbor-light.case.json");
        let mut case = demo_case();
        let mut case_path = default_case_path.to_string_lossy().into_owned();
        let mut disk_case_json = None;
        let mut fit_initial_chart = true;
        let status = match initialize_user_data(&data_dir) {
            Ok(()) => {
                if default_case_path.exists() {
                    match fs::read_to_string(&default_case_path)
                        .map_err(|error| error.to_string())
                        .and_then(|contents| {
                            let saved_case = serde_json::from_str::<CaseFile>(&contents)
                                .map_err(|error| error.to_string())?;
                            Ok((saved_case, contents))
                        }) {
                        Ok((mut saved_case, contents)) => {
                            normalize_snapshot_database_paths(&mut saved_case, &case_path);
                            fit_initial_chart = chart_view_needs_initial_fit(&saved_case);
                            case = saved_case;
                            disk_case_json = Some(contents);
                            "SAVED CASE RESTORED".into()
                        }
                        Err(error) => {
                            case_path = data_dir
                                .join("untitled.case.json")
                                .to_string_lossy()
                                .into_owned();
                            format!("DEFAULT CASE LOAD FAILED · {error} · ORIGINAL FILE PRESERVED")
                        }
                    }
                } else {
                    "SAMPLE CASE READY · SAVE TO KEEP YOUR CHANGES".into()
                }
            }
            Err(error) => format!("USER DATA SETUP FAILED · {error}"),
        };
        let saved_case_json = serde_json::to_string(&case).ok();
        let initial_chart_zoom = clamp_chart_zoom(case.active_chart_zoom);
        let analysis_pane_width = load_analysis_pane_width(&data_dir);
        let initial_chart_pan = chart_pan_from_saved(case.active_chart_pan);
        let initial_editor_chart_id = case.active_chart_id;
        let initial_chart_fit_entities =
            fit_initial_chart.then(|| case.entities.iter().map(|entity| entity.id).collect());
        Self {
            case,
            selected: None,
            selected_entities: HashSet::new(),
            drag_entity_origins: HashMap::new(),
            selected_chart_label: None,
            chart_label_creation_armed: false,
            chart_label_drag_origin: None,
            query: String::new(),
            chart_search_scope: ChartSearchScope::All,
            chart_search_all_charts: false,
            status,
            show_timeline: false,
            timeline_range: None,
            timeline_brush_anchor: None,
            chart_zoom: initial_chart_zoom,
            chart_canvas_aspect: 1.6,
            analysis_pane_width,
            chart_pan: initial_chart_pan,
            chart_pan_origin: egui::Vec2::ZERO,
            chart_focus: None,
            chart_focus_link: None,
            chart_fit_entities: initial_chart_fit_entities,
            chart_fit_path: None,
            chart_focus_group: None,
            network_groups: None,
            community_analysis: None,
            selected_community_interaction: None,
            network_bridges: None,
            network_proximity_analysis: None,
            selected_proximity_group: None,
            selected_proximity_interaction: None,
            analysis_method_card: None,
            chart_navigator_dragging: false,
            new_link_from: None,
            new_link_to: None,
            new_link_label: "ASSOCIATED WITH".into(),
            link_creation_armed: false,
            link_creation_from: None,
            new_entity_type_text: String::new(),
            new_link_type_text: String::new(),
            import_path: data_dir.join("sample_relationships.csv").to_string_lossy().into_owned(),
            import_into_new_chart: false,
            import_ingest_source_override: None,
            delimited_import_counts: None,
            import_headers: Vec::new(),
            import_preview: Vec::new(),
            import_record_count: 0,
            import_inspection_worker: None,
            import_inspection_cancel: None,
            import_inspection_progress: None,
            background_imports_enabled: true,
            anx_import_worker: None,
            import_column_mapping: ImportColumnMapping::default(),
            import_spec_name: String::new(),
            import_design_package_path: String::new(),
            import_design_package_name: String::new(),
            import_design_replace_selected: false,
            selected_import_spec: None,
            show_import_wizard: false,
            export_path: data_dir.join("chart-export.csv").to_string_lossy().into_owned(),
            graphml_export_path: data_dir
                .join("chart-export.graphml")
                .to_string_lossy()
                .into_owned(),
            graphml_include_source_catalog: false,
            kml_export_path: data_dir
                .join("chart-export.kml")
                .to_string_lossy()
                .into_owned(),
            anx_export_path: data_dir
                .join("chart-export.anx")
                .to_string_lossy()
                .into_owned(),
            selected_link: None,
            show_all_relationship_labels: false,
            chart_link_overlap_point: None,
            chart_link_overlap_candidates: Vec::new(),
            chart_link_overlap_cycle: 0,
            page: WorkspacePage::Chart,
            entity_sort: EntitySort::Name,
            most_connected_measure: MostConnectedMeasure::default(),
            most_connected_relationship: None,
            linked_search_depth: 2,
            linked_search_follow_directions: false,
            layout_mode: LayoutMode::Connectivity,
            layout_iterations: 50,
            layout_worker: None,
            layout_restore: None,
            link_sort: LinkSort::Date,
            link_table_view: LinkTableView::Connections,
            data_query: String::new(),
            data_source_filter: None,
            connections_for_selected: false,
            connections_within_timeline_range: false,
            path_start: None,
            path_end: None,
            path_entity_filter: String::new(),
            saved_path_name: String::new(),
            selection_set_name: String::new(),
            saved_path_package_path: data_dir.join("investigation-path.json").to_string_lossy().into_owned(),
            saved_path_import_name: String::new(),
            path_follow_directions: true,
            path_time_flow: PathTimeFlow::Any,
            path_use_timeline_range: true,
            path_timeline_range: None,
            path_result: Vec::new(),
            path_link_result: Vec::new(),
            case_path: case_path.clone(),
            case_path_input: case_path,
            saved_case_json,
            disk_case_json,
            pending_case_action: None,
            pending_chart_delete: None,
            command_palette_open: false,
            command_palette_query: String::new(),
            data_mode: DataMode::Relationships,
            merge_candidate: None,
            duplicate_candidates: Vec::new(),
            sql_path: data_dir.join("sample_connections.sqlite").to_string_lossy().into_owned(),
            sql_series: "Harbor Light connection review".into(),
            sql_query: "SELECT connection_id, source, source_type, target, target_type, relationship, event_date AS date, confidence, directed, source_reference, notes FROM connection_observations WHERE snapshot_id = :snapshot_id ORDER BY connection_id;".into(),
            sql_parameter_text: "snapshot_id=baseline".into(),
            sql_cleared_parameter_draft: None,
            sql_validation_preview: None,
            sql_skipped_endpoints: Vec::new(),
            sql_validation_preview_regressions_only: false,
            sql_schema_templates: Vec::new(),
            sql_schema_objects: Vec::new(),
            sql_schema_object_fingerprints: Vec::new(),
            sql_schema_filter: String::new(),
            sql_schema_template_index: 0,
            sql_schema_template_source: None,
            automation_repair_context: None,
            automation_job_filter: String::new(),
            automation_run_filter: String::new(),
            automation_run_trigger_filter: "all".into(),
            automation_run_note_drafts: HashMap::new(),
            automation_run_parameter_text: String::new(),
            automation_preflight_require_usable_connections: true,
            automation_preflight_require_unchanged_schema: false,
            automation_job_import_path: String::new(),
            automation_job_export_path: data_dir
                .join("automation-job.json")
                .to_string_lossy()
                .into_owned(),
            automation_job_import_name: String::new(),
            automation_job_import_chart: String::new(),
            automation_job_import_database_path: String::new(),
            automation_job_import_series: String::new(),
            automation_job_import_parameter_text: String::new(),
            automation_failed_runs_only: false,
            automation_schema_adaptation_only: false,
            automation_schema_stability_only: false,
            automation_runs_needing_review: false,
            automation_runs_with_unreviewed_regressions: false,
            sql_snapshot_source: None,
            sql_snapshot_index: None,
            sql_snapshot_filter: String::new(),
            sql_comparison_baseline_id: None,
            sql_delta_filter: SnapshotDeltaFilter::All,
            sql_delta_search: String::new(),
            sql_source_focus: None,
            automation_job_name: String::new(),
            automation_job_chart_id: initial_editor_chart_id,
            automation_query_timeout_minutes: SQL_QUERY_TIMEOUT_MINUTES,
            automation_minimum_connections: 0,
            automation_minimum_confidence: None,
            automation_maximum_connections: None,
            automation_maximum_regressions: None,
            automation_maximum_added_connections: None,
            automation_maximum_connection_property_changes: None,
            automation_maximum_entity_property_changes: None,
            automation_maximum_skipped_rows: None,
            automation_require_unchanged_schema: false,
            editing_automation_job: None,
            automation_duplicate_draft: None,
            automation_editor_before_edit: None,
            automation_worker: None,
            automation_progress: None,
            sql_plan_worker: None,
            sql_plan_result: None,
            sql_entity_source_worker: None,
            sql_entity_source_request: None,
            sql_entity_source_cancel: None,
            sql_entity_source_progress: None,
            sql_entity_source_result: None,
            sql_schema_worker: None,
            sql_schema_request: None,
            sql_lock_retry_count: 0,
            sql_query_timeout_minutes: SQL_QUERY_TIMEOUT_MINUTES,
            automation_schedule: "hourly".into(),
            automation_schedule_time: "00:00".into(),
            automation_schedule_weekday: 1,
            automation_schedule_month_day: 1,
            automation_schedule_require_usable_connections: true,
            automation_chart_export_dir: env::var("ARCH_NOTEBOOK_CHART_EXPORT_DIR")
                .unwrap_or_default(),
            automation_schedule_message: None,
            automation_schedule_target_path: None,
            automation_schedule_progress: None,
            automation_schedule_worker: None,
            automation_preflight_cancel: None,
            automation_cancel: Arc::new(AtomicBool::new(false)),
            close_after_automation: false,
        }
    }
}

// Keep first-run files out of the working directory so installed app launches behave consistently.
fn app_data_dir() -> PathBuf {
    if cfg!(target_os = "macos") {
        return env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Library/Application Support/Arch Notebook");
    }
    if cfg!(target_os = "windows") {
        return env::var_os("APPDATA")
            .map(PathBuf::from)
            .unwrap_or_else(|| PathBuf::from("."))
            .join("Arch Notebook");
    }
    if let Some(xdg_data_home) = env::var_os("XDG_DATA_HOME").map(PathBuf::from) {
        if xdg_data_home.is_absolute() {
            return xdg_data_home.join("arch-notebook");
        }
    }
    env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".local/share/arch-notebook")
}

fn analysis_pane_width_path(data_dir: &Path) -> PathBuf {
    data_dir.join("analysis-pane-width.txt")
}

fn load_analysis_pane_width(data_dir: &Path) -> f32 {
    fs::read_to_string(analysis_pane_width_path(data_dir))
        .ok()
        .and_then(|value| value.trim().parse::<f32>().ok())
        .filter(|value| value.is_finite())
        .unwrap_or(320.0)
        .clamp(240.0, 420.0)
}

fn save_analysis_pane_width(data_dir: &Path, width: f32) -> Result<(), String> {
    let width = width.clamp(240.0, 420.0);
    write_file_atomically(
        &analysis_pane_width_path(data_dir),
        format!("{width:.1}\n").as_bytes(),
    )
}

fn initialize_user_data(data_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    let samples: [(&str, &[u8]); 2] = [
        (
            "sample_connections.sqlite",
            include_bytes!("../archive/research.databases/sample_connections.sqlite"),
        ),
        (
            "sample_relationships.csv",
            include_bytes!("../archive/research.databases/sample_relationships.csv"),
        ),
    ];
    for (name, bytes) in samples {
        let path = data_dir.join(name);
        // Seed examples once; never replace analyst-edited files on a later launch.
        if !path.exists() {
            fs::write(path, bytes)?;
        }
    }
    Ok(())
}

fn demo_case() -> CaseFile {
    let e = |name: &str, kind: &str, detail: &str, x, y, color| Entity {
        id: Uuid::new_v4(),
        name: name.into(),
        kind: kind.into(),
        representation: EntityRepresentation::Icon,
        detail: detail.into(),
        attributes: Vec::new(),
        sql_attribute_values: Vec::new(),
        sql_attribute_baselines: Vec::new(),
        aliases: Vec::new(),
        external_ids: Vec::new(),
        merged_records: Vec::new(),
        x,
        y,
        position_locked: false,
        color,
    };
    let entities = vec![
        e(
            "Avery Rowan",
            "PERSON",
            "Known associate · DOB 1987-04-12",
            0.19,
            0.30,
            [55, 128, 162],
        ),
        e(
            "Cedarline Trading",
            "ORGANIZATION",
            "Regional supplier · Delaware",
            0.49,
            0.19,
            [126, 108, 180],
        ),
        e(
            "Harbor Point Depot",
            "LOCATION",
            "Last verified 2025-02-14",
            0.79,
            0.32,
            [69, 143, 118],
        ),
        e(
            "Phone 7",
            "PHONE",
            "+1 (617) 555-0197",
            0.29,
            0.69,
            [189, 130, 66],
        ),
        e(
            "Westline Transport LLC",
            "ORGANIZATION",
            "Registered 2024-08-03",
            0.61,
            0.72,
            [126, 108, 180],
        ),
    ];
    let id = |i: usize| entities[i].id;
    let links = vec![
        Link {
            from: id(0),
            to: id(1),
            label: "DIRECTOR OF".into(),
            date: "2025-01-06".into(),
            confidence: 3,
            directed: true,
            notes: "Corporate registry filing".into(),
            source_notes: String::new(),
            source_reference: "REG-1193 · Corporate registry".into(),
            connection_id: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        },
        Link {
            from: id(1),
            to: id(2),
            label: "LEASES".into(),
            date: "2025-01-28".into(),
            confidence: 2,
            directed: false,
            notes: "Property record".into(),
            source_notes: String::new(),
            source_reference: "LOC-81 · Property record".into(),
            connection_id: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        },
        Link {
            from: id(0),
            to: id(3),
            label: "USES NUMBER".into(),
            date: "2025-02-11".into(),
            confidence: 3,
            directed: false,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: "SIG-14 · Communications report".into(),
            connection_id: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        },
        Link {
            from: id(3),
            to: id(4),
            label: "CONTACTED".into(),
            date: "2025-02-14".into(),
            confidence: 1,
            directed: true,
            notes: "Source report; uncorroborated".into(),
            source_notes: String::new(),
            source_reference: "HUM-203 · Uncorroborated report".into(),
            connection_id: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        },
        Link {
            from: id(4),
            to: id(2),
            label: "DELIVERS TO".into(),
            date: "2025-02-14".into(),
            confidence: 2,
            directed: true,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: "LOG-77 · Delivery log".into(),
            connection_id: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        },
    ];
    CaseFile {
        format_version: CURRENT_CASE_FORMAT_VERSION,
        title: "HARBOR LIGHT // SAMPLE CASE".into(),
        description: "Fictional sample relationship investigation".into(),
        entities,
        links,
        labels: Vec::new(),
        active_chart_id: Uuid::new_v4(),
        active_chart_title: default_chart_title(),
        active_chart_zoom: default_chart_zoom(),
        active_chart_pan: [0.0, 0.0],
        chart_coordinate_bounds: None,
        chart_tabs: Vec::new(),
        chart_templates: Vec::new(),
        active_chart_template: None,
        snapshots: Vec::new(),
        automation_jobs: Vec::new(),
        automation_runs: Vec::new(),
        automation_batches: Vec::new(),
        import_specifications: Vec::new(),
        graphml_source_catalogs: Vec::new(),
        saved_paths: Vec::new(),
        selection_sets: Vec::new(),
    }
}

impl App {
    fn map_entity_contacts(&mut self, origin: Uuid) {
        let analysis = proximity::analyze(
            self.case
                .entities
                .iter()
                .map(|entity| (entity.id, entity.name.as_str())),
            self.case
                .links
                .iter()
                .map(|link| (link.from, link.to, link.directed)),
            origin,
        );
        self.network_proximity_analysis = Some((origin, std::sync::Arc::new(analysis)));
        self.selected_proximity_group = None;
        self.selected_proximity_interaction = None;
        self.selected_entities.clear();
        self.selected_entities.insert(origin);
        self.selected = Some(origin);
        self.selected_link = None;
        self.chart_focus = Some(origin);
        self.chart_focus_group = None;
        self.chart_fit_entities = None;
        self.chart_fit_path = None;
        self.chart_focus_link = None;
        self.clear_path_result();
        self.sql_source_focus = None;
        self.page = WorkspacePage::Chart;
        self.show_timeline = false;
        self.status = "GROUPED CONTACT TOPOLOGY BUILT FROM SELECTED ENTITY".into();
    }

    fn show_sql_result_mapping_help(ui: &mut egui::Ui, id: &'static str) {
        egui::CollapsingHeader::new("SQL result column mapping")
            .id_salt(id)
            .show(ui, |ui| {
                ui.label("Required: source and target. Include connection_id when the source has a stable relationship ID.");
                ui.label("Optional endpoint columns: source_type, target_type, source_entity_id, target_entity_id, source_representation, target_representation.");
                ui.label("Other optional columns: relationship, date, confidence (1–3), directed, source_reference, notes.");
                ui.label("Endpoint properties use source_attribute:<name> and target_attribute:<name>; relationship properties use connection_attribute:<name>.");
                ui.label("Representations: Icon, Theme line, Event frame, Box, Circle, Text block. Rows for the same entity must use one representation.");
                ui.label("Schema templates project recognized representation columns from the relationship table or joined entity tables.");
                ui.monospace("SELECT source_name AS source, target_name AS target, 'Event frame' AS source_representation, amount AS \"connection_attribute:amount\"");
            });
    }

    fn sync_active_chart_viewport(&mut self) {
        self.chart_zoom = clamp_chart_zoom(self.chart_zoom);
        self.case.active_chart_zoom = self.chart_zoom;
        self.case.active_chart_pan = [self.chart_pan.x, self.chart_pan.y];
    }

    fn reset_chart_creation_drafts(&mut self) {
        self.new_link_from = None;
        self.new_link_to = None;
        self.new_link_label = "ASSOCIATED WITH".into();
        self.new_entity_type_text.clear();
        self.new_link_type_text.clear();
        self.chart_label_creation_armed = false;
        self.chart_label_drag_origin = None;
    }

    fn select_chart_entity(&mut self, entity_id: Uuid, extend_selection: bool) {
        self.selected_chart_label = None;
        self.selected_link = None;
        if extend_selection {
            if !self.selected_entities.insert(entity_id) {
                self.selected_entities.remove(&entity_id);
                if self.selected == Some(entity_id) {
                    self.selected = self.selected_entities.iter().min().copied();
                }
            } else {
                self.selected = Some(entity_id);
            }
            if self.selected_entities.is_empty() {
                self.selected = None;
            }
        } else {
            self…202152 tokens truncated…       ..ImportValueTransform::default()
        }];
        assert!(compile_import_value_transforms(["status"], &unknown_column).is_err());

        let replacement_without_search = vec![ImportValueTransform {
            column: "status".into(),
            operation: ImportTransformOperation::Replace,
            replace: "ready".into(),
            ..ImportValueTransform::default()
        }];
        assert!(compile_import_value_transforms(["status"], &replacement_without_search).is_err());
    }

    fn sql_lookup_snapshot(
        database_path: &std::path::Path,
        query: &str,
        parameters: Vec<(String, String)>,
        series: &str,
    ) -> ConnectionSnapshot {
        ConnectionSnapshot {
            id: Uuid::new_v4(),
            chart_id: None,
            series: series.into(),
            source_key: format!("sqlite:{series}"),
            database_path: database_path.to_string_lossy().into_owned(),
            configured_database_path: database_path.to_string_lossy().into_owned(),
            query: query.into(),
            parameters,
            captured_at: "2026-11-26T00:00:00Z".into(),
            sqlite_schema_version: Some(1),
            sqlite_schema_fingerprint: None,
            sqlite_schema_objects: None,
            connections: Vec::new(),
            skipped_rows: 0,
        }
    }

    fn sql_lookup_fixture_database(path: &std::path::Path, sql: &str) {
        let connection = rusqlite::Connection::open(path).expect("create SQL lookup fixture");
        connection
            .execute_batch(sql)
            .expect("populate SQL lookup fixture");
    }

    fn run_sql_source_lookup(
        snapshot: &ConnectionSnapshot,
        source_entity_id: &str,
        case_path: &std::path::Path,
    ) -> Result<super::SqlEntitySourceRows, String> {
        super::sql_entity_source_rows(
            snapshot,
            source_entity_id,
            &case_path.to_string_lossy(),
            std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false)),
            std::sync::Arc::new(std::sync::Mutex::new(String::new())),
        )
    }

    #[test]
    fn sql_source_record_lookup_keeps_source_scope_and_is_read_only() {
        let root =
            std::env::temp_dir().join(format!("arch-notebook-source-record-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).expect("create isolated source-record fixture directory");
        let alpha_path = root.join("alpha.sqlite");
        let beta_path = root.join("beta.sqlite");
        let fixture_sql = "CREATE TABLE observations (id INTEGER PRIMARY KEY, tenant TEXT NOT NULL, source_id INTEGER, target_id INTEGER, detail TEXT, optional TEXT, opaque BLOB);
            CREATE TABLE source_context (tenant TEXT NOT NULL, entity_id INTEGER NOT NULL, analyst_context TEXT, PRIMARY KEY (tenant, entity_id));
            INSERT INTO observations VALUES (1, 'alpha', 7, 9, 'current alpha row', NULL, X'01ff');
            INSERT INTO observations VALUES (2, 'alpha', 11, 7, 'reverse alpha row', 'present', X'02');
            INSERT INTO source_context VALUES ('alpha', 7, 'alpha joined context');
            INSERT INTO source_context VALUES ('beta', 7, 'beta joined context');";
        sql_lookup_fixture_database(&alpha_path, fixture_sql);
        sql_lookup_fixture_database(
            &beta_path,
            "CREATE TABLE observations (id INTEGER PRIMARY KEY, tenant TEXT NOT NULL, source_id INTEGER, target_id INTEGER, detail TEXT, optional TEXT, opaque BLOB);
             CREATE TABLE source_context (tenant TEXT NOT NULL, entity_id INTEGER NOT NULL, analyst_context TEXT, PRIMARY KEY (tenant, entity_id));
             INSERT INTO observations VALUES (1, 'beta', 7, 8, 'beta source row', NULL, X'03');
             INSERT INTO source_context VALUES ('beta', 7, 'beta joined context');",
        );
        let alpha_before = std::fs::read(&alpha_path).expect("read alpha source before lookup");
        let beta_before = std::fs::read(&beta_path).expect("read beta source before lookup");
        let query = "SELECT o.tenant || ':' || CAST(o.source_id AS TEXT) AS source_entity_id,
                o.tenant || ':' || CAST(o.target_id AS TEXT) AS target_entity_id,
                o.detail AS observation_detail, o.optional AS optional_value,
                o.opaque AS opaque_value, :capture_tag AS capture_tag,
                context.analyst_context AS joined_context
            FROM observations AS o
            LEFT JOIN source_context AS context
                ON context.tenant = o.tenant AND context.entity_id = o.source_id
            WHERE o.tenant = :tenant ORDER BY o.id";
        let alpha = sql_lookup_snapshot(
            &alpha_path,
            query,
            vec![
                ("tenant".into(), "text:alpha".into()),
                ("capture_tag".into(), "text:baseline".into()),
            ],
            "alpha-series",
        );
        let beta = sql_lookup_snapshot(
            &beta_path,
            query,
            vec![
                ("tenant".into(), "text:beta".into()),
                ("capture_tag".into(), "text:current".into()),
            ],
            "beta-series",
        );
        let alpha_snapshot_before = serde_json::to_vec(&alpha).expect("serialize saved snapshot");
        let beta_snapshot_before = serde_json::to_vec(&beta).expect("serialize saved snapshot");

        let alpha_rows = run_sql_source_lookup(&alpha, "ALPHA:7", &root.join("case.json"))
            .expect("lookup composite identity in alpha source");
        let beta_rows = run_sql_source_lookup(&beta, "beta:7", &root.join("case.json"))
            .expect("lookup the same local ID in beta source");

        assert_eq!(alpha_rows.snapshot_id, alpha.id);
        assert_eq!(alpha_rows.source_entity_id, "ALPHA:7");
        assert_eq!(
            alpha_rows.columns.join("|"),
            "source_entity_id|target_entity_id|observation_detail|optional_value|opaque_value|capture_tag|joined_context"
        );
        assert_eq!(alpha_rows.rows_scanned, 2);
        assert_eq!(alpha_rows.rows.len(), 2);
        assert_eq!(alpha_rows.rows[0][0], "alpha:7");
        assert_eq!(alpha_rows.rows[0][2], "current alpha row");
        assert_eq!(alpha_rows.rows[0][3], "");
        assert_eq!(alpha_rows.rows[0][4], "BLOB · 2 bytes · 01ff");
        assert_eq!(alpha_rows.rows[0][5], "baseline");
        assert_eq!(alpha_rows.rows[0][6], "alpha joined context");
        assert_eq!(alpha_rows.rows[1][1], "alpha:7");
        assert_eq!(alpha_rows.rows[1][6], "");
        assert_eq!(beta_rows.snapshot_id, beta.id);
        assert_eq!(
            beta_rows.database_path,
            beta_path
                .canonicalize()
                .unwrap()
                .to_string_lossy()
                .into_owned()
        );
        assert_eq!(beta_rows.rows.len(), 1);
        assert_eq!(beta_rows.rows[0][0], "beta:7");
        assert_eq!(beta_rows.rows[0][5], "current");
        assert_eq!(beta_rows.rows[0][6], "beta joined context");
        assert_eq!(std::fs::read(&alpha_path).unwrap(), alpha_before);
        assert_eq!(std::fs::read(&beta_path).unwrap(), beta_before);
        assert_eq!(serde_json::to_vec(&alpha).unwrap(), alpha_snapshot_before);
        assert_eq!(serde_json::to_vec(&beta).unwrap(), beta_snapshot_before);
        assert!(!alpha_rows.truncated && !beta_rows.truncated);

        std::fs::remove_dir_all(root).expect("remove isolated source-record fixture");
    }

    #[test]
    fn sql_source_record_lookup_rejects_writes_and_ambiguous_identity_columns() {
        let root = std::env::temp_dir().join(format!(
            "arch-notebook-source-record-invalid-{}",
            Uuid::new_v4()
        ));
        std::fs::create_dir(&root).expect("create isolated invalid-query fixture directory");
        let database_path = root.join("source.sqlite");
        sql_lookup_fixture_database(
            &database_path,
            "CREATE TABLE observations (tenant TEXT, actor_id INTEGER, detail TEXT);
             INSERT INTO observations VALUES ('alpha', 7, 'unchanged');",
        );
        let database_before =
            std::fs::read(&database_path).expect("read source before invalid queries");
        let write_query = sql_lookup_snapshot(
            &database_path,
            "UPDATE observations SET detail = 'changed' WHERE tenant = :tenant",
            vec![("tenant".into(), "text:alpha".into())],
            "write-attempt",
        );
        let write_error =
            match run_sql_source_lookup(&write_query, "alpha:7", &root.join("case.json")) {
                Ok(_) => panic!("live source lookup must reject a saved write query"),
                Err(error) => error,
            };
        assert!(write_error.contains("no longer read-only"), "{write_error}");

        let ambiguous_query = sql_lookup_snapshot(
            &database_path,
            "SELECT 'alpha:7' AS source_entity_id, 'alpha:8' AS SOURCE_ENTITY_ID",
            Vec::new(),
            "ambiguous-identity",
        );
        let ambiguous_error =
            match run_sql_source_lookup(&ambiguous_query, "alpha:7", &root.join("case.json")) {
                Ok(_) => panic!("ambiguous identity columns must be rejected"),
                Err(error) => error,
            };
        assert!(
            ambiguous_error.contains("ambiguous source entity ID columns"),
            "{ambiguous_error}"
        );
        assert_eq!(std::fs::read(&database_path).unwrap(), database_before);
        std::fs::remove_dir_all(root).expect("remove isolated invalid-query fixture");
    }

    #[test]
    fn sql_source_record_lookup_bounds_rows_and_cell_values() {
        let root = std::env::temp_dir().join(format!(
            "arch-notebook-source-record-bounds-{}",
            Uuid::new_v4()
        ));
        std::fs::create_dir(&root).expect("create isolated bounds fixture directory");
        let database_path = root.join("source.sqlite");
        sql_lookup_fixture_database(&database_path, "CREATE TABLE placeholder (value TEXT);");
        let many_rows = sql_lookup_snapshot(
            &database_path,
            "WITH RECURSIVE generated(n) AS (
                SELECT 1 UNION ALL SELECT n + 1 FROM generated WHERE n < 25
            ) SELECT 'alpha:7' AS source_entity_id, CAST(n AS TEXT) AS value FROM generated",
            Vec::new(),
            "row-limit",
        );
        let many = run_sql_source_lookup(&many_rows, "alpha:7", &root.join("case.json"))
            .expect("run bounded many-row lookup");
        assert_eq!(many.rows.len(), super::SQL_ENTITY_LOOKUP_MAX_ROWS);
        assert_eq!(many.rows_scanned, super::SQL_ENTITY_LOOKUP_MAX_ROWS + 1);
        assert!(many.truncated);
        assert!(!many.scan_limit_reached);

        let long_cell = sql_lookup_snapshot(
            &database_path,
            "SELECT 'alpha:7' AS source_entity_id, hex(zeroblob(2048)) AS value",
            Vec::new(),
            "cell-limit",
        );
        let bounded = run_sql_source_lookup(&long_cell, "alpha:7", &root.join("case.json"))
            .expect("run bounded cell lookup");
        assert_eq!(bounded.rows.len(), 1);
        assert!(bounded.truncated);
        assert!(bounded.rows[0][1].ends_with("[truncated]"));
        assert!(bounded.rows[0][1].len() < 1100);
        std::fs::remove_dir_all(root).expect("remove isolated bounds fixture");
    }

    #[test]
    fn analysis_method_card_counts_records_pairs_isolates_and_date_quality() {
        let mut case = super::demo_case();
        case.title = "PRIVATE CASE TITLE".into();
        case.entities = vec![
            layout_entity("PRIVATE ALPHA", "Actor", false),
            layout_entity("PRIVATE BRAVO", "Actor", false),
            layout_entity("PRIVATE CHARLIE", "Actor", false),
            layout_entity("PRIVATE ISOLATE", "Actor", false),
        ];
        case.links.clear();
        let alpha = case.entities[0].id;
        let bravo = case.entities[1].id;
        let charlie = case.entities[2].id;
        let absent = Uuid::new_v4();
        let link = |from, to, directed, date: &str| super::Link {
            from,
            to,
            connection_id: "PRIVATE CONNECTION ID".into(),
            label: "PRIVATE RELATIONSHIP LABEL".into(),
            date: date.into(),
            confidence: 3,
            directed,
            notes: "PRIVATE ANALYST NOTE".into(),
            source_notes: "PRIVATE SOURCE NOTE".into(),
            source_reference: "PRIVATE EVIDENCE REFERENCE".into(),
            ingest_source: Some("PRIVATE SOURCE KEY".into()),
            attributes: Vec::new(),
        };
        case.links = vec![
            link(alpha, bravo, true, "2024-02-29"),
            link(bravo, alpha, true, ""),
            link(alpha, bravo, false, "2024-02-30"),
            link(bravo, bravo, true, "2025-01-01"),
            link(charlie, absent, true, "not-a-date"),
        ];

        let case_before_copy = serde_json::to_value(&case).expect("serialize fictional chart");
        let card = super::network_analysis_method_card(&case);
        assert_eq!(card.entity_count, 4);
        assert_eq!(card.isolate_count, 2);
        assert_eq!(card.relationship_record_count, 5);
        assert_eq!(card.directed_record_count, 4);
        assert_eq!(card.undirected_record_count, 1);
        assert_eq!(card.unique_endpoint_pair_count, 2);
        assert_eq!(card.repeated_endpoint_pair_record_count, 2);
        assert_eq!(card.missing_endpoint_record_count, 1);
        assert_eq!(card.timeline_recognized_date_count, 2);
        assert_eq!(card.blank_date_count, 1);
        assert_eq!(card.unrecognized_date_count, 2);

        let copied = super::network_analysis_method_card_text(
            &card,
            true,
            super::PathTimeFlow::Forward,
            true,
        );
        assert!(copied.contains("direction is respected for directed records"));
        assert!(copied.contains("date order is forward date order"));
        assert!(copied.contains("active timeline range is applied"));
        assert!(copied.contains("records with a missing endpoint are ignored"));
        assert!(
            copied.contains("2 recognized by the chart timeline parser; 1 blank; 2 not recognized")
        );
        for private_value in [
            "PRIVATE CASE TITLE",
            "PRIVATE ALPHA",
            "PRIVATE CONNECTION ID",
            "PRIVATE RELATIONSHIP LABEL",
            "PRIVATE ANALYST NOTE",
            "PRIVATE SOURCE NOTE",
            "PRIVATE EVIDENCE REFERENCE",
            "PRIVATE SOURCE KEY",
        ] {
            assert!(!copied.contains(private_value));
        }
        assert_eq!(
            serde_json::to_value(&case).expect("serialize unchanged fictional chart"),
            case_before_copy,
            "building and formatting the card must not mutate case data"
        );
    }

    #[test]
    fn detected_community_interactions_separate_record_pair_and_direction_counts() {
        let mut case = super::demo_case();
        case.entities = (0..8)
            .map(|index| {
                let name = if index < 4 {
                    format!("A{index}")
                } else {
                    format!("B{index}")
                };
                layout_entity(&name, "PERSON", false)
            })
            .collect();
        let ids: Vec<Uuid> = case.entities.iter().map(|entity| entity.id).collect();
        case.links.clear();
        let make_link = |from, to, directed| super::Link {
            from,
            to,
            connection_id: Uuid::new_v4().to_string(),
            label: "CONTACTS".into(),
            date: String::new(),
            confidence: 2,
            directed,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        };
        for (start, end) in [(0, 4), (4, 8)] {
            for left in start..end {
                for right in left + 1..end {
                    case.links.push(make_link(ids[left], ids[right], false));
                }
            }
        }
        case.links.extend([
            make_link(ids[3], ids[4], true),
            make_link(ids[4], ids[3], true),
            make_link(ids[3], ids[4], true),
            make_link(ids[2], ids[5], false),
            make_link(ids[1], ids[6], true),
            make_link(ids[2], Uuid::new_v4(), true),
            make_link(ids[1], ids[1], true),
        ]);

        let analysis = super::detected_community_analysis(&case);
        assert_eq!(
            analysis.groups.iter().map(Vec::len).collect::<Vec<_>>(),
            [4, 4]
        );
        assert_eq!(analysis.interactions.len(), 1);
        let interaction = &analysis.interactions[0];
        assert_eq!(interaction.relationship_record_count, 5);
        assert_eq!(interaction.distinct_endpoint_pair_count, 3);
        if analysis.groups[interaction.first_community].contains(&ids[0]) {
            assert_eq!(interaction.first_to_second_record_count, 3);
            assert_eq!(interaction.second_to_first_record_count, 1);
        } else {
            assert_eq!(interaction.first_to_second_record_count, 1);
            assert_eq!(interaction.second_to_first_record_count, 3);
        }
        assert_eq!(interaction.undirected_record_count, 1);
        assert_eq!(interaction.link_indices, [12, 13, 14, 15, 16]);
        assert_eq!(analysis.groups[interaction.first_community].len(), 4);
        assert_eq!(analysis.groups[interaction.second_community].len(), 4);
        assert_ne!(interaction.first_community, interaction.second_community);
        assert!(
            analysis.proxies.is_empty(),
            "internal ties plus contact with one external community do not make an inter-community proxy"
        );
    }

    #[test]
    fn detected_community_proxy_counts_only_other_communities() {
        let mut case = super::demo_case();
        case.entities = (0..12)
            .map(|index| layout_entity(&format!("Group member {index}"), "PERSON", false))
            .collect();
        let ids: Vec<_> = case.entities.iter().map(|entity| entity.id).collect();
        let make_link = |from, to| super::Link {
            from,
            to,
            connection_id: Uuid::new_v4().to_string(),
            label: "CONTACTS".into(),
            date: String::new(),
            confidence: 2,
            directed: false,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        };
        case.links.clear();
        for start in [0, 4, 8] {
            for left in start..start + 4 {
                for right in left + 1..start + 4 {
                    case.links.push(make_link(ids[left], ids[right]));
                }
            }
        }
        case.links.push(make_link(ids[3], ids[4]));
        case.links.push(make_link(ids[3], ids[8]));

        let analysis = super::detected_community_analysis(&case);
        assert_eq!(
            analysis.groups.iter().map(Vec::len).collect::<Vec<_>>(),
            [4, 4, 4]
        );
        assert_eq!(analysis.proxies.len(), 1);
        let proxy = &analysis.proxies[0];
        assert_eq!(proxy.entity_id, ids[3]);
        assert_eq!(proxy.distinct_community_contacts, 2);
        assert_eq!(proxy.distinct_neighbor_entities, 5);
        assert_eq!(proxy.relationship_record_count, 5);
    }

    #[test]
    fn shortest_path_ignores_relationships_with_missing_endpoints() {
        let mut case = super::demo_case();
        let entities = vec![
            layout_entity("Start", "PERSON", false),
            layout_entity("Goal", "ORGANIZATION", false),
        ];
        let start = entities[0].id;
        let goal = entities[1].id;
        let missing = Uuid::new_v4();
        case.entities = entities;
        let link = |from, to| super::Link {
            from,
            to,
            connection_id: Uuid::new_v4().to_string(),
            label: "CONTACTS".into(),
            date: String::new(),
            confidence: 2,
            directed: true,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        };
        case.links = vec![link(start, missing), link(missing, goal)];

        assert!(
            super::shortest_path(&case, start, goal, false, super::PathTimeFlow::Any, None,)
                .is_none()
        );

        case.links.push(link(start, goal));
        assert_eq!(
            super::shortest_path(&case, start, goal, false, super::PathTimeFlow::Any, None,),
            Some((vec![start, goal], vec![2])),
        );
    }

    #[test]
    fn analysis_packet_redacts_by_default_and_exports_selected_chart_without_overwrite() {
        let root =
            std::env::temp_dir().join(format!("arch-notebook-packet-test-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).expect("create isolated packet fixture directory");
        let case_path = root.join("case.json");
        let mut case = super::demo_case();
        case.title = "PRIVATE CASE TITLE".into();
        case.active_chart_title = "PRIVATE CHART TITLE".into();
        case.chart_coordinate_bounds = Some([-80.0, -40.0, 120.0, 80.0]);
        case.links[0].connection_id = "PRIVATE CONNECTION ID".into();
        case.links[0].source_reference = "PRIVATE EVIDENCE <reference>".into();
        case.links[0].notes = "PRIVATE ANALYST NOTE".into();

        let alternate_chart_id = Uuid::new_v4();
        case.chart_tabs.push(ChartDocument {
            id: alternate_chart_id,
            title: "Alternate **Chart**\n# injected heading".into(),
            zoom: 1.0,
            pan: [0.0, 0.0],
            coordinate_bounds: Some([1_000.0, 2_000.0, 3_000.0, 4_000.0]),
            entities: vec![{
                let mut entity = layout_entity("ALTERNATE ENTITY", "Organization", false);
                entity.x = 2_000.0;
                entity.y = 3_000.0;
                entity
            }],
            links: Vec::new(),
            labels: Vec::new(),
            template_id: None,
        });
        let case_before = serde_json::to_vec_pretty(&case).expect("serialize packet case");
        std::fs::write(&case_path, &case_before).expect("write packet source case");

        let redacted_dir = root.join("redacted");
        export_analysis_packet_cli(
            &case_path.to_string_lossy(),
            &redacted_dir.to_string_lossy(),
            None,
            AnalysisPacketDisclosure::default(),
            true,
        )
        .expect("export privacy-default packet");
        let redacted_report = std::fs::read_to_string(redacted_dir.join("finding.md"))
            .expect("read privacy-default report");
        let redacted_svg = std::fs::read_to_string(redacted_dir.join("network.svg"))
            .expect("read privacy-default network");
        for private_value in [
            "PRIVATE CASE TITLE",
            "PRIVATE CHART TITLE",
            "Avery Rowan",
            "DIRECTOR OF",
            "PRIVATE CONNECTION ID",
            "PRIVATE EVIDENCE",
            "PRIVATE ANALYST NOTE",
        ] {
            assert!(
                !redacted_report.contains(private_value),
                "report disclosed {private_value}"
            );
            assert!(
                !redacted_svg.contains(private_value),
                "SVG disclosed {private_value}"
            );
        }
        assert!(redacted_report.contains("Chart: Redacted"));
        assert!(redacted_svg.contains("Relationship 001"));
        assert!(!redacted_svg.contains("<metadata"));

        let selected_dir = root.join("selected");
        let disclosure = AnalysisPacketDisclosure {
            names: true,
            relationship_labels: true,
            ids: true,
            evidence_references: true,
        };
        export_analysis_packet_cli(
            &case_path.to_string_lossy(),
            &selected_dir.to_string_lossy(),
            Some(&alternate_chart_id.to_string()),
            disclosure,
            true,
        )
        .expect("export selected chart with explicit disclosures");
        let selected_report = std::fs::read_to_string(selected_dir.join("finding.md"))
            .expect("read selected chart report");
        let selected_svg = std::fs::read_to_string(selected_dir.join("network.svg"))
            .expect("read selected chart network");
        assert!(selected_report.contains("Chart: Alternate"));
        assert!(!selected_report.contains("!["));
        assert!(!selected_report.contains("<script"));
        assert!(selected_report.contains("Entities: 1"));
        assert!(selected_report.contains("Relationship records: 0 total"));
        assert!(selected_svg.contains("ALTERNATE ENTITY"));
        assert!(selected_svg.contains("cx=\"600.00\" cy=\"400.00\""));
        assert!(!selected_svg.contains("Avery Rowan"));
        assert!(!selected_svg.contains("PRIVATE EVIDENCE"));

        let same_chart_dir = root.join("same-chart");
        export_analysis_packet_cli(
            &case_path.to_string_lossy(),
            &same_chart_dir.to_string_lossy(),
            None,
            disclosure,
            false,
        )
        .expect("export active chart with disclosures");
        let same_chart_svg = std::fs::read_to_string(same_chart_dir.join("network.svg")).unwrap();
        let repeated_dir = root.join("repeated");
        export_analysis_packet_cli(
            &case_path.to_string_lossy(),
            &repeated_dir.to_string_lossy(),
            None,
            disclosure,
            false,
        )
        .expect("repeat deterministic export");
        assert_eq!(
            same_chart_svg,
            std::fs::read_to_string(repeated_dir.join("network.svg")).unwrap()
        );

        #[cfg(unix)]
        {
            std::os::unix::fs::symlink(root.join("missing-target"), root.join("broken-symlink"))
                .unwrap();
            let error = export_analysis_packet_cli(
                &case_path.to_string_lossy(),
                &root.join("broken-symlink").to_string_lossy(),
                None,
                AnalysisPacketDisclosure::default(),
                false,
            )
            .expect_err("broken symlink destination rejected");
            assert!(error.contains("already exists"));
        }

        let occupied_dir = root.join("occupied");
        std::fs::create_dir(&occupied_dir).expect("create occupied destination");
        std::fs::write(occupied_dir.join("sentinel"), "keep").expect("write sentinel");
        let error = export_analysis_packet_cli(
            &case_path.to_string_lossy(),
            &occupied_dir.to_string_lossy(),
            None,
            AnalysisPacketDisclosure::default(),
            false,
        )
        .expect_err("existing output directory must be rejected");
        assert!(error.contains("already exists"));
        assert_eq!(
            std::fs::read(occupied_dir.join("sentinel")).unwrap(),
            b"keep"
        );
        assert_eq!(std::fs::read(&case_path).unwrap(), case_before);
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            assert_eq!(
                std::fs::metadata(&redacted_dir)
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o700
            );
            assert_eq!(
                std::fs::metadata(redacted_dir.join("finding.md"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
            assert_eq!(
                std::fs::metadata(redacted_dir.join("network.svg"))
                    .unwrap()
                    .permissions()
                    .mode()
                    & 0o777,
                0o600
            );
        }
        let ambiguous_id = Uuid::new_v4();
        case.chart_tabs.push(ChartDocument {
            id: ambiguous_id,
            title: "Alternate **Chart**\n# injected heading".into(),
            zoom: 1.0,
            pan: [0.0, 0.0],
            coordinate_bounds: None,
            entities: Vec::new(),
            links: Vec::new(),
            labels: Vec::new(),
            template_id: None,
        });
        let ambiguous_path = root.join("ambiguous.json");
        std::fs::write(&ambiguous_path, serde_json::to_vec(&case).unwrap()).unwrap();
        let error = export_analysis_packet_cli(
            &ambiguous_path.to_string_lossy(),
            &root.join("ambiguous-output").to_string_lossy(),
            Some("Alternate **Chart**\n# injected heading"),
            AnalysisPacketDisclosure::default(),
            false,
        )
        .expect_err("ambiguous title rejected");
        assert!(error.contains("ambiguous"));
        let mut malformed = case.clone();
        malformed.links.push(super::Link {
            from: Uuid::new_v4(),
            to: malformed.entities[0].id,
            connection_id: String::new(),
            label: "missing endpoint".into(),
            date: String::new(),
            confidence: 1,
            directed: false,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        });
        let malformed_path = root.join("malformed.json");
        std::fs::write(&malformed_path, serde_json::to_vec(&malformed).unwrap()).unwrap();
        let missing_output = root.join("missing-output");
        let error = export_analysis_packet_cli(
            &malformed_path.to_string_lossy(),
            &missing_output.to_string_lossy(),
            None,
            AnalysisPacketDisclosure::default(),
            false,
        )
        .expect_err("missing endpoints rejected");
        assert!(error.contains("missing chart endpoints"));
        assert!(std::fs::symlink_metadata(&missing_output).is_err());
        std::fs::remove_dir_all(root).expect("remove isolated packet fixture directory");
    }

    #[test]
    fn analysis_packet_redacts_default_output_and_escapes_opt_in_text() {
        let mut case = super::demo_case();
        case.title = "SECRET TITLE".into();
        case.entities = vec![
            layout_entity("SECRET <ALPHA>", "Actor", false),
            layout_entity("SECRET BRAVO", "Actor", false),
        ];
        case.links.clear();
        let a = case.entities[0].id;
        let b = case.entities[1].id;
        case.links.push(super::Link {
            from: a,
            to: b,
            connection_id: "SECRET-ID".into(),
            label: "LABEL & SECRET".into(),
            date: "".into(),
            confidence: 3,
            directed: true,
            notes: "SECRET NOTE".into(),
            source_notes: "SECRET SOURCE NOTE".into(),
            source_reference: "SECRET REF".into(),
            ingest_source: None,
            attributes: vec![],
        });
        let default =
            super::analysis_packet_svg(&case, super::AnalysisPacketDisclosure::default()).unwrap();
        for secret in [
            "SECRET TITLE",
            "SECRET <ALPHA>",
            "SECRET BRAVO",
            "SECRET-ID",
            "LABEL & SECRET",
            "SECRET REF",
            "SECRET NOTE",
        ] {
            assert!(!default.contains(secret));
        }
        assert!(default.contains("Entity 001"));
        let opted = super::analysis_packet_svg(
            &case,
            super::AnalysisPacketDisclosure {
                names: true,
                relationship_labels: true,
                ids: true,
                evidence_references: true,
            },
        )
        .unwrap();
        assert!(opted.contains("SECRET &lt;ALPHA&gt;"));
        assert!(opted.contains("LABEL &amp; SECRET"));
        assert!(opted.contains("SECRET-ID"));
        assert!(opted.contains("SECRET REF"));
        let report = super::analysis_packet_report(
            &case,
            super::AnalysisPacketDisclosure::default(),
            &case.active_chart_title,
        );
        assert!(report.contains("Findings (software generated no findings)"));
        assert!(report.contains("topology and aggregate counts may still identify people"));
    }

    #[test]
    fn analysis_packet_reports_detected_communities_and_cross_group_records() {
        let mut case = super::demo_case();
        case.entities = (1..=12)
            .map(|index| {
                let name = if index == 3 {
                    "PRIVATE|ACTOR *<script>".to_owned()
                } else {
                    format!("PRIVATE ACTOR {index}")
                };
                let mut entity = layout_entity(&name, "ACTOR", false);
                entity.id = Uuid::from_u128(index);
                entity
            })
            .collect();
        let mut links = Vec::new();
        for (start, end) in [(1_u128, 5_u128), (5_u128, 9_u128), (9_u128, 13_u128)] {
            for from in start..end {
                for to in from + 1..end {
                    links.push(super::Link {
                        from: Uuid::from_u128(from),
                        to: Uuid::from_u128(to),
                        connection_id: format!("INTERNAL-{from}-{to}"),
                        label: "INTERNAL".into(),
                        date: String::new(),
                        confidence: 2,
                        directed: false,
                        notes: String::new(),
                        source_notes: String::new(),
                        source_reference: String::new(),
                        ingest_source: None,
                        attributes: Vec::new(),
                    });
                }
            }
        }
        for (from, to, directed) in [
            (3_u128, 6_u128, true),
            (4, 5, false),
            (6, 3, true),
            (3, 10, true),
        ] {
            links.push(super::Link {
                from: Uuid::from_u128(from),
                to: Uuid::from_u128(to),
                connection_id: format!("CROSS-{from}-{to}"),
                label: "CONTACTS".into(),
                date: String::new(),
                confidence: 2,
                directed,
                notes: String::new(),
                source_notes: String::new(),
                source_reference: String::new(),
                ingest_source: None,
                attributes: Vec::new(),
            });
        }
        case.links = links;

        let default = super::analysis_packet_report(
            &case,
            AnalysisPacketDisclosure::default(),
            &case.active_chart_title,
        );
        assert!(
            default.contains(
                "**Community 1** · 4 entities: Entity 001, Entity 002, Entity 003, Entity 004"
            ),
            "unexpected community membership summary:\n{default}"
        );
        assert!(default.contains(
            "**Community 2** · 4 entities: Entity 005, Entity 006, Entity 007, Entity 008"
        ));
        assert!(default.contains(
            "**Community 3** · 4 entities: Entity 009, Entity 010, Entity 011, Entity 012"
        ));
        assert!(default.contains("| Community 1 ↔ Community 2 | 3 | 2 | 1 | 1 | 1 |"));
        assert!(default.contains("| 1 | Entity 003 | 2 | 5 | 6 |"));
        assert!(default.contains("single-level greedy modularity local-move heuristic"));
        assert!(default.contains("not asserted real-world groups or affiliations"));
        assert!(!default.contains("PRIVATE ACTOR"));
        assert!(!default.contains("PRIVATE|ACTOR"));
        assert!(!default.contains(&Uuid::from_u128(3).to_string()));

        let disclosed = super::analysis_packet_report(
            &case,
            AnalysisPacketDisclosure {
                names: true,
                ids: true,
                ..Default::default()
            },
            &case.active_chart_title,
        );
        assert!(disclosed.contains("PRIVATE\\|ACTOR \\*\\<script\\> · ACTOR"));
        assert!(!disclosed.contains("<script>"));
        assert!(disclosed.contains(&Uuid::from_u128(3).to_string()));
    }

    #[test]
    fn analysis_packet_routes_reversed_and_high_multiplicity_records_distinctly() {
        let mut case = super::demo_case();
        let mut first = layout_entity("First", "PERSON", false);
        let mut second = layout_entity("Second", "ORGANIZATION", false);
        first.x = 0.32;
        first.y = 0.48;
        second.x = 0.68;
        second.y = 0.48;
        let first_id = first.id;
        let second_id = second.id;
        case.entities = vec![first, second];
        case.links = (0..13)
            .map(|index| {
                let reversed = index % 3 != 2;
                super::Link {
                    from: if reversed { second_id } else { first_id },
                    to: if reversed { first_id } else { second_id },
                    connection_id: format!("ROUTE-{index:02}"),
                    label: "CONTACTS".into(),
                    date: String::new(),
                    confidence: 2,
                    directed: index % 3 != 1,
                    notes: String::new(),
                    source_notes: String::new(),
                    source_reference: String::new(),
                    ingest_source: None,
                    attributes: Vec::new(),
                }
            })
            .collect();

        let svg = super::analysis_packet_svg(
            &case,
            super::AnalysisPacketDisclosure {
                ids: true,
                ..Default::default()
            },
        )
        .expect("render all parallel relationship records");
        let edge_groups = svg
            .lines()
            .filter(|line| line.contains("data-record=") && line.contains("<line x1="))
            .collect::<Vec<_>>();
        assert_eq!(edge_groups.len(), 13);
        for index in 0..13 {
            assert!(svg.contains(&format!("data-record=\"ROUTE-{index:02}\"")));
        }

        let coordinate = |line: &str, key: &str| -> f32 {
            let attribute = format!("{key}=\"");
            let start = line.find(&attribute).expect("line coordinate attribute") + attribute.len();
            let end = line[start..].find('"').expect("line coordinate terminator") + start;
            line[start..end].parse().expect("numeric SVG coordinate")
        };
        let mut geometries = std::collections::HashSet::new();
        for line in edge_groups {
            let first = (coordinate(line, "x1"), coordinate(line, "y1"));
            let second = (coordinate(line, "x2"), coordinate(line, "y2"));
            let (start, end) = if first.0 < second.0 || (first.0 == second.0 && first.1 <= second.1)
            {
                (first, second)
            } else {
                (second, first)
            };
            geometries.insert(format!(
                "{:.2},{:.2}|{:.2},{:.2}",
                start.0, start.1, end.0, end.1
            ));
        }
        assert_eq!(geometries.len(), 13, "each record must have its own lane");
    }

    #[test]
    fn quick_command_search_matches_labels_and_keywords_case_insensitively() {
        assert!(super::quick_command_matches(
            "  sql ",
            "SQL Review",
            "database query"
        ));
        assert!(super::quick_command_matches(
            "CAPTURE",
            "Automation",
            "SQL jobs scheduled captures"
        ));
        assert!(super::quick_command_matches(
            "fit",
            "Fit chart",
            "fit all entities reset zoom frame chart viewport"
        ));
        assert!(super::quick_command_matches(
            "viewport",
            "Fit chart",
            "fit all entities reset zoom frame chart viewport"
        ));
        assert!(super::quick_command_matches("", "Chart", "canvas"));
        assert!(!super::quick_command_matches(
            "nonexistent",
            "Automation",
            "sql jobs scheduled captures"
        ));
    }

    #[test]
    fn automation_workspace_hides_duplicate_navigation_and_file_exchange_fields() {
        let mut case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("correctness fixture is a valid case");
        case.description = "Retained case description".into();
        let mut app = App::automation_context(case, "/tmp/automation-workspace-ui.case.json");
        app.page = super::WorkspacePage::Data;
        app.data_mode = super::DataMode::Automation;
        app.import_path = "sample.csv".into();
        app.export_path = "chart.csv".into();

        let context = egui::Context::default();
        let mut frame = eframe::Frame::_new_kittest();
        let automation_ui =
            render_headless_app_frame(&context, &mut app, &mut frame, 0.0, Vec::new());
        for visible in [
            "AUTOMATION",
            "Case file",
            "SQL AUTOMATION",
            "REPEATABLE SQL INGESTION JOBS",
        ] {
            assert!(
                find_headless_text_point(&automation_ui, visible).is_some(),
                "expected visible Automation workspace control: {visible}"
            );
        }
        for hidden in ["CASE RECORDS", "Description", "Import path", "Export path"] {
            assert!(
                find_headless_text_point(&automation_ui, hidden).is_none(),
                "irrelevant control should be hidden in Automation: {hidden}"
            );
        }

        app.data_mode = super::DataMode::Relationships;
        let connections_ui =
            render_headless_app_frame(&context, &mut app, &mut frame, 1.0 / 60.0, Vec::new());
        for visible in ["CASE RECORDS", "Description", "Import path", "Export path"] {
            assert!(
                find_headless_text_point(&connections_ui, visible).is_some(),
                "Connections workspace should retain its case and interchange control: {visible}"
            );
        }
        assert!(find_headless_text_point(&connections_ui, "Retained case description").is_some());
        assert!(find_headless_text_point(&connections_ui, "sample.csv").is_some());
        assert!(find_headless_text_point(&connections_ui, "chart.csv").is_some());
        assert_eq!(app.import_path, "sample.csv");
        assert_eq!(app.export_path, "chart.csv");
    }

    #[test]
    fn headless_ui_cancels_desktop_sql_automation_and_keeps_prior_checkpoint() {
        let root =
            std::env::temp_dir().join(format!("arch-notebook-cancel-test-{}", Uuid::new_v4()));
        std::fs::create_dir(&root).expect("create isolated cancellation fixture directory");
        let database_path = root.join("source.sqlite");
        let database = rusqlite::Connection::open(&database_path)
            .expect("create disposable read-only SQL source");
        drop(database);
        let source_hash_before = std::fs::read(&database_path).expect("read source bytes");

        let mut case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("correctness fixture is a valid case");
        let make_job = |name: &str, series: &str, query: &str| AutomationJob {
            id: Uuid::new_v4(),
            chart_id: None,
            name: name.into(),
            database_path: database_path.to_string_lossy().into_owned(),
            series: series.into(),
            query: query.into(),
            parameters: Vec::new(),
            query_timeout_minutes: 15,
            enabled: true,
            minimum_connections: 1,
            minimum_confidence: None,
            maximum_connections: None,
            maximum_regressions: None,
            maximum_added_connections: None,
            maximum_connection_property_changes: None,
            maximum_entity_property_changes: None,
            maximum_skipped_rows: Some(0),
            require_unchanged_schema: false,
        };
        let successful_job = make_job(
            "Synthetic checkpoint",
            "cancellation-checkpoint",
            "SELECT 'Synthetic A' AS source, 'Synthetic B' AS target, 'TEST LINK' AS relationship",
        );
        let cancellable_job = make_job(
            "Synthetic cancellable query",
            "cancellation-active-query",
            "WITH RECURSIVE counter(value) AS (SELECT 1 UNION ALL SELECT value + 1 FROM counter WHERE value < 500000000) SELECT 'Synthetic Source' AS source, 'Synthetic Target' AS target, 'TEST LINK' AS relationship FROM counter WHERE value = (SELECT MAX(value) FROM counter)",
        );
        let job_ids = vec![successful_job.id, cancellable_job.id];
        case.automation_jobs = vec![successful_job, cancellable_job];

        let case_path = root.join("cancellation.case.json");
        let mut app = App::automation_context(case, &case_path.to_string_lossy());
        let saved_json = app
            .current_case_json()
            .expect("serialize the cancellation case");
        std::fs::write(&case_path, &saved_json).expect("save the fixture case before automation");
        app.saved_case_json = Some(saved_json.clone());
        app.disk_case_json = Some(saved_json);
        let context = egui::Context::default();
        let mut frame = eframe::Frame::_new_kittest();
        let mut time = 0.0;
        app.start_automation_work(
            job_ids,
            None,
            true,
            AutomationParameterOverrides {
                global: Vec::new(),
                scoped: Vec::new(),
            },
            None,
        );

        let progress = app
            .automation_progress
            .as_ref()
            .cloned()
            .expect("automation worker publishes progress");
        let started = std::time::Instant::now();
        loop {
            let message = progress.lock().expect("read automation progress").clone();
            if message.starts_with("Running job 2 of 2") {
                break;
            }
            assert!(
                started.elapsed() < std::time::Duration::from_secs(10),
                "second SQL job did not start; last progress: {message}"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
        }
        std::thread::sleep(std::time::Duration::from_millis(100));
        let running_ui =
            render_headless_app_frame(&context, &mut app, &mut frame, time, Vec::new());
        assert!(find_headless_text_point(&running_ui, "SQL operation is running").is_some());
        assert!(
            collect_headless_texts(&running_ui)
                .iter()
                .any(|text| text.contains("Running job 2 of 2"))
        );
        assert!(find_headless_text_point(&running_ui, "Cancel automation").is_some());
        let _after_cancel = click_headless_visible_text(
            &context,
            &mut app,
            &mut frame,
            &mut time,
            &running_ui,
            "Cancel automation",
        );
        assert!(
            app.automation_cancel
                .load(std::sync::atomic::Ordering::Acquire)
        );
        assert_eq!(app.status, "CANCELLING AUTOMATION");

        let cancellation_started = std::time::Instant::now();
        while app.automation_worker.is_some() {
            assert!(
                cancellation_started.elapsed() < std::time::Duration::from_secs(15),
                "cancelled SQL worker should finish promptly"
            );
            std::thread::sleep(std::time::Duration::from_millis(5));
            time += 1.0 / 60.0;
            render_headless_app_frame(&context, &mut app, &mut frame, time, Vec::new());
        }
        let completed_case = &app.case;
        let status = &app.status;
        assert!(status.contains("AUTOMATION CANCELLED"), "{status}");
        assert_eq!(completed_case.automation_runs.len(), 2);
        assert!(completed_case.automation_runs[0].success);
        assert!(!completed_case.automation_runs[1].success);
        assert!(
            completed_case.automation_runs[1]
                .message
                .to_ascii_lowercase()
                .contains("sql query cancelled by analyst")
        );
        assert_eq!(completed_case.snapshots.len(), 1);
        assert_eq!(completed_case.snapshots[0].connections.len(), 1);
        assert_eq!(completed_case.automation_batches.len(), 1);
        assert_eq!(completed_case.automation_batches[0].completed_jobs.len(), 1);
        assert_eq!(
            completed_case.automation_batches[0].completed_jobs[0].job_id,
            completed_case.automation_runs[0].job_id
        );
        let persisted_case: CaseFile = serde_json::from_str(
            &std::fs::read_to_string(&case_path).expect("cancelled case checkpoint exists"),
        )
        .expect("cancelled case checkpoint is valid JSON");
        assert_eq!(persisted_case.automation_runs.len(), 2);
        assert_eq!(
            std::fs::read(&database_path).expect("read source after capture"),
            source_hash_before
        );
        std::fs::remove_dir_all(root).expect("remove only this disposable fixture directory");
    }

    #[test]
    fn undirected_connection_identity_is_endpoint_order_independent() {
        let forward =
            canonical_connection("Harbor", "LOCATION", "Mara", "PERSON", "VISITED", false)
                .expect("valid connection");
        let reverse =
            canonical_connection("Mara", "PERSON", "Harbor", "LOCATION", "VISITED", false)
                .expect("valid connection");
        assert_eq!(forward, reverse);
    }

    #[test]
    fn directed_connection_identity_preserves_endpoint_order() {
        let forward = canonical_connection("Harbor", "LOCATION", "Mara", "PERSON", "VISITED", true)
            .expect("valid connection");
        let reverse = canonical_connection("Mara", "PERSON", "Harbor", "LOCATION", "VISITED", true)
            .expect("valid connection");
        assert_ne!(forward.4, reverse.4);
    }

    #[test]
    fn natural_key_delimiters_cannot_collide() {
        let first = canonical_connection("A|B", "PERSON", "C", "PLACE", "LINK", true)
            .expect("valid connection");
        let second = canonical_connection("A", "B|PERSON", "C", "PLACE", "LINK", true)
            .expect("valid connection");
        assert_ne!(first.4, second.4);
    }

    #[test]
    fn legacy_snapshot_key_matches_the_new_natural_identity() {
        let (_, _, _, _, new_key) = canonical_connection(
            "Northstar Imports",
            "ORGANIZATION",
            "Pier 19 Warehouse",
            "LOCATION",
            "LEASES",
            false,
        )
        .expect("valid connection");
        let fact = |key: String| ConnectionFact {
            key,
            connection_id: String::new(),
            source: "Northstar Imports".into(),
            source_type: "ORGANIZATION".into(),
            source_representation: String::new(),
            source_entity_id: String::new(),
            target: "Pier 19 Warehouse".into(),
            target_type: "LOCATION".into(),
            target_representation: String::new(),
            target_entity_id: String::new(),
            source_attributes: Vec::new(),
            target_attributes: Vec::new(),
            attributes: Vec::new(),
            relationship: "LEASES".into(),
            date: "2025-01-28".into(),
            confidence: 2,
            directed: false,
            source_reference: "LOC-81".into(),
            notes: String::new(),
        };
        let make_snapshot = |key: String| ConnectionSnapshot {
            id: super::Uuid::new_v4(),
            chart_id: None,
            series: "test".into(),
            source_key: "db::test".into(),
            database_path: "db".into(),
            configured_database_path: "db".into(),
            query: String::new(),
            parameters: Vec::new(),
            captured_at: String::new(),
            sqlite_schema_version: None,
            sqlite_schema_fingerprint: None,
            sqlite_schema_objects: None,
            connections: vec![fact(key)],
            skipped_rows: 0,
        };
        let old = make_snapshot(
            "Northstar Imports|ORGANIZATION|LEASES|Pier 19 Warehouse|LOCATION|false#1".into(),
        );
        let current = make_snapshot(format!("natural-v2:{new_key}"));
        assert!(compare_snapshots(&old, &current).is_empty());
    }

    #[test]
    fn entity_type_layout_clusters_sorted_names_and_preserves_pins() {
        let entities = vec![
            layout_entity("Zulu", "PERSON", false),
            layout_entity("Alpha", "PERSON", false),
            layout_entity("Pinned", "PERSON", true),
            layout_entity("Harbor", "LOCATION", false),
        ];
        let positions = entity_type_layout_positions(&entities, 1.6);
        assert_eq!(positions.len(), 3);
        assert!(!positions.contains_key(&entities[2].id));
        let alpha = positions[&entities[1].id];
        let zulu = positions[&entities[0].id];
        let harbor = positions[&entities[3].id];
        assert!(alpha.0 > harbor.0);
        assert!(alpha.0 < zulu.0);
        assert_eq!(alpha.1, zulu.1);
    }

    #[test]
    fn community_layout_results_are_rejected_after_chart_or_graph_edits() {
        let chart_id = Uuid::from_u128(900);
        let entities = vec![
            layout_entity("First", "PERSON", false),
            layout_entity("Second", "PERSON", true),
        ];
        let links = vec![super::Link {
            from: entities[0].id,
            to: entities[1].id,
            connection_id: "link-1".into(),
            label: "ASSOCIATED WITH".into(),
            date: String::new(),
            confidence: 2,
            directed: false,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        }];
        let result = super::CommunityLayoutWorkerResult {
            chart_id,
            entity_ids: entities.iter().map(|entity| entity.id).collect(),
            original_positions: entities
                .iter()
                .map(|entity| (entity.x, entity.y, entity.position_locked))
                .collect(),
            link_endpoints: links.iter().map(|link| (link.from, link.to)).collect(),
            positions: vec![(0.3, 0.4), (0.6, 0.7)],
        };
        assert!(super::community_layout_snapshot_matches(
            chart_id, &entities, &links, &result
        ));
        assert!(!super::community_layout_snapshot_matches(
            Uuid::from_u128(901),
            &entities,
            &links,
            &result
        ));

        let mut moved_entities = entities.clone();
        moved_entities[0].x += 0.1;
        assert!(!super::community_layout_snapshot_matches(
            chart_id,
            &moved_entities,
            &links,
            &result
        ));

        let changed_links = vec![super::Link {
            from: entities[1].id,
            ..links[0].clone()
        }];
        assert!(!super::community_layout_snapshot_matches(
            chart_id,
            &entities,
            &changed_links,
            &result
        ));

        let restore = super::CommunityLayoutRestoreSnapshot {
            chart_id,
            entities: entities
                .iter()
                .map(|entity| (entity.id, entity.x, entity.y, entity.position_locked))
                .collect(),
        };
        assert!(super::community_layout_restore_matches(
            chart_id, &entities, &restore
        ));
        assert!(!super::community_layout_restore_matches(
            Uuid::from_u128(901),
            &entities,
            &restore
        ));
        let mut changed_pin = entities.clone();
        changed_pin[1].position_locked = false;
        assert!(!super::community_layout_restore_matches(
            chart_id,
            &changed_pin,
            &restore
        ));
    }

    #[test]
    fn headless_view_menu_arranges_and_restores_community_force_layout() {
        let mut case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("checked-in chart fixture is valid");
        let mut entities = vec![
            layout_entity("North", "PERSON", false),
            layout_entity("Pinned", "PERSON", true),
            layout_entity("South", "PERSON", false),
        ];
        for (index, entity) in entities.iter_mut().enumerate() {
            entity.id = Uuid::from_u128(index as u128 + 1);
        }
        entities[0].x = 0.12;
        entities[0].y = 0.18;
        entities[1].x = 0.31;
        entities[1].y = 0.47;
        entities[2].x = 0.84;
        entities[2].y = 0.78;
        case.entities = entities;
        let ids: Vec<_> = case.entities.iter().map(|entity| entity.id).collect();
        case.links = [(ids[0], ids[1]), (ids[1], ids[2]), (ids[0], ids[2])]
            .into_iter()
            .map(|(from, to)| super::Link {
                from,
                to,
                connection_id: format!("headless-layout-{from}-{to}"),
                label: "CONTACTS".into(),
                date: String::new(),
                confidence: 2,
                directed: false,
                notes: String::new(),
                source_notes: String::new(),
                source_reference: String::new(),
                ingest_source: None,
                attributes: Vec::new(),
            })
            .collect();
        let original: Vec<_> = case
            .entities
            .iter()
            .map(|entity| (entity.x, entity.y, entity.position_locked))
            .collect();
        let mut app = App::automation_context(case, "/tmp/headless-community-layout.case.json");
        app.layout_mode = super::LayoutMode::CommunityForce;
        let context = egui::Context::default();
        let mut frame = eframe::Frame::_new_kittest();
        let mut time = 0.0;

        let initial = render_headless_app_frame(&context, &mut app, &mut frame, time, Vec::new());
        assert!(find_headless_text_point(&initial, "View…").is_some());
        click_headless_text(&context, &mut app, &mut frame, &mut time, "View…");
        let arrange_menu =
            render_headless_app_frame(&context, &mut app, &mut frame, time, Vec::new());
        assert!(find_headless_text_point(&arrange_menu, "Arrange chart").is_some());
        click_headless_visible_text(
            &context,
            &mut app,
            &mut frame,
            &mut time,
            &arrange_menu,
            "Arrange chart",
        );

        for _ in 0..100 {
            if app.layout_worker.is_none() && app.layout_restore.is_some() {
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(1));
            time += 1.0 / 60.0;
            render_headless_app_frame(&context, &mut app, &mut frame, time, Vec::new());
        }
        assert!(app.layout_worker.is_none(), "layout worker should finish");
        let snapshot = app
            .layout_restore
            .as_ref()
            .expect("Arrange chart stores a restore snapshot");
        assert!(super::community_layout_restore_matches(
            app.case.active_chart_id,
            &app.case.entities,
            snapshot,
        ));
        assert_eq!(
            (app.case.entities[1].x, app.case.entities[1].y),
            (original[1].0, original[1].1),
            "pinned node remains in place during arrangement",
        );
        assert_ne!(
            (app.case.entities[0].x, app.case.entities[0].y),
            (original[0].0, original[0].1),
            "arrangement changes a movable node",
        );

        click_headless_text(&context, &mut app, &mut frame, &mut time, "View…");
        let restore_menu =
            render_headless_app_frame(&context, &mut app, &mut frame, time, Vec::new());
        assert!(
            find_headless_text_point(&restore_menu, "Restore positions from before force layout")
                .is_some()
        );
        click_headless_visible_text(
            &context,
            &mut app,
            &mut frame,
            &mut time,
            &restore_menu,
            "Restore positions from before force layout",
        );
        for (entity, (x, y, pinned)) in app.case.entities.iter().zip(original) {
            assert_eq!((entity.x, entity.y, entity.position_locked), (x, y, pinned));
        }
        assert!(app.layout_restore.is_none());
    }

    #[test]
    fn chart_topology_frame_follows_switch_duplicate_and_new_chart_lifecycle() {
        let mut case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("checked-in chart fixture is valid");
        let original_id = case.active_chart_id;
        let original_bounds = [-24.0, -12.0, 48.0, 36.0];
        let alternate_id = Uuid::new_v4();
        let alternate_bounds = [100.0, 200.0, 300.0, 400.0];
        case.chart_coordinate_bounds = Some(original_bounds);
        case.chart_tabs.push(ChartDocument {
            id: alternate_id,
            title: "Alternate topology".into(),
            zoom: 1.0,
            pan: [0.0, 0.0],
            coordinate_bounds: Some(alternate_bounds),
            entities: vec![layout_entity("Alternate", "PERSON", false)],
            links: Vec::new(),
            labels: Vec::new(),
            template_id: None,
        });

        let mut app = App::automation_context(case, "/tmp/topology-lifecycle.case.json");
        app.switch_chart(alternate_id);
        assert_eq!(app.case.chart_coordinate_bounds, Some(alternate_bounds));
        assert_eq!(
            app.case
                .chart_tabs
                .iter()
                .find(|chart| chart.id == original_id)
                .and_then(|chart| chart.coordinate_bounds),
            Some(original_bounds),
            "switching saves the outgoing chart's topology frame",
        );

        app.switch_chart(original_id);
        assert_eq!(app.case.chart_coordinate_bounds, Some(original_bounds));
        app.duplicate_active_chart();
        assert_eq!(app.case.chart_coordinate_bounds, Some(original_bounds));
        let duplicate_id = app.case.active_chart_id;
        app.switch_chart(original_id);
        assert_eq!(app.case.chart_coordinate_bounds, Some(original_bounds));
        app.switch_chart(duplicate_id);
        assert_eq!(app.case.chart_coordinate_bounds, Some(original_bounds));

        app.create_chart(false);
        assert_eq!(app.case.chart_coordinate_bounds, None);
        assert_eq!(
            app.case
                .chart_tabs
                .iter()
                .find(|chart| chart.id == duplicate_id)
                .and_then(|chart| chart.coordinate_bounds),
            Some(original_bounds),
            "creating a blank chart preserves the duplicate's saved frame",
        );
    }

    #[test]
    fn entity_type_layout_adapts_cluster_grid_to_canvas_aspect() {
        let entities = (0..6)
            .map(|index| layout_entity(&format!("Entity {index}"), &format!("TYPE-{index}"), false))
            .collect::<Vec<_>>();
        let narrow = entity_type_layout_positions(&entities, 0.75);
        let wide = entity_type_layout_positions(&entities, 2.5);
        let distinct_columns = |positions: &std::collections::HashMap<Uuid, (f32, f32)>| {
            let mut columns = positions
                .values()
                .map(|(x, _)| (x * 10_000.0).round() as i32)
                .collect::<Vec<_>>();
            columns.sort_unstable();
            columns.dedup();
            columns.len()
        };
        assert_eq!(distinct_columns(&narrow), 3);
        assert_eq!(distinct_columns(&wide), 4);
        assert!(wide[&entities[0].id].1 < wide[&entities[4].id].1);
    }

    #[test]
    fn entity_type_layout_centers_a_partial_cluster_row() {
        let entities = (0..4)
            .map(|index| layout_entity(&format!("Entity {index}"), "PERSON", false))
            .collect::<Vec<_>>();
        let positions = entity_type_layout_positions(&entities, 1.6);
        let first_row_center = (positions[&entities[0].id].0 + positions[&entities[2].id].0) / 2.0;
        assert!((positions[&entities[3].id].0 - first_row_center).abs() < f32::EPSILON);
    }

    #[test]
    fn geodesic_layers_are_undirected_and_ignore_parallel_and_disconnected_links() {
        let entities = vec![
            layout_entity("Center", "PERSON", false),
            layout_entity("First hop", "PERSON", false),
            layout_entity("Second hop", "LOCATION", false),
            layout_entity("Island", "ORGANIZATION", false),
            layout_entity("Island neighbor", "PERSON", false),
        ];
        let link = |from, to, directed| super::Link {
            from,
            to,
            connection_id: Uuid::new_v4().to_string(),
            label: "ASSOCIATED WITH".into(),
            date: String::new(),
            confidence: 2,
            directed,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        };
        let mut case = super::demo_case();
        case.entities = entities.clone();
        case.links = vec![
            link(entities[0].id, entities[1].id, true),
            link(entities[0].id, entities[1].id, false),
            link(entities[1].id, entities[2].id, true),
            link(entities[0].id, entities[0].id, true),
            link(entities[3].id, entities[4].id, true),
        ];
        let analyze = |origin| {
            super::proximity::analyze(
                case.entities
                    .iter()
                    .map(|entity| (entity.id, entity.name.as_str())),
                case.links
                    .iter()
                    .map(|link| (link.from, link.to, link.directed)),
                origin,
            )
        };
        let analysis = analyze(entities[0].id);
        assert_eq!(
            analysis.layers,
            vec![vec![entities[1].id], vec![entities[2].id]]
        );
        assert!(analyze(Uuid::new_v4()).layers.is_empty());
    }

    #[test]
    fn network_analysis_invalidation_discards_a_cached_method_card() {
        let mut app = super::App::default();
        app.analysis_method_card = Some((
            app.case.active_chart_id,
            super::NetworkAnalysisMethodCard {
                entity_count: 1,
                ..Default::default()
            },
        ));
        app.community_analysis = Some(std::sync::Arc::new(super::CommunityAnalysis {
            groups: vec![vec![Uuid::new_v4()]],
            interactions: Vec::new(),
            proxies: Vec::new(),
        }));
        app.network_proximity_analysis = Some((
            app.case.active_chart_id,
            std::sync::Arc::new(super::proximity::Analysis::default()),
        ));
        app.selected_proximity_interaction = Some((0, 1));

        app.invalidate_network_groups();

        assert!(app.analysis_method_card.is_none());
        assert!(app.community_analysis.is_none());
        assert!(app.network_proximity_analysis.is_none());
        assert!(app.selected_proximity_interaction.is_none());
    }

    #[test]
    fn alphabetical_grid_tracks_the_chart_canvas_aspect() {
        let (narrow_columns, narrow_rows) = grid_layout_dimensions(20, 0.75);
        let (wide_columns, wide_rows) = grid_layout_dimensions(20, 2.5);
        assert!(wide_columns > narrow_columns);
        assert!(wide_rows < narrow_rows);
        assert_eq!(grid_layout_dimensions(0, 2.5), (0, 0));
        assert_eq!(
            grid_layout_dimensions(5, f32::NAN),
            grid_layout_dimensions(5, 1.6)
        );
    }

    #[test]
    fn connection_curve_has_finite_endpoints_and_bends_for_parallel_lanes() {
        let start = egui::pos2(40.0, 70.0);
        let end = egui::pos2(240.0, 170.0);
        let center = super::connection_curve_points(start, end, 0.0, false);
        let upper_lane = super::connection_curve_points(start, end, 12.0, false);
        assert_eq!(center.first(), Some(&start));
        assert_eq!(center.last(), Some(&end));
        assert_eq!(upper_lane.first(), Some(&start));
        assert_eq!(upper_lane.last(), Some(&end));
        assert!(
            center
                .iter()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        assert_ne!(center[8], upper_lane[8]);
    }

    #[test]
    fn schema_adaptation_reference_excludes_wrapped_query_context() {
        let error = "SQL INGEST FAILED · no such column: event_date in SELECT event_date FROM connection_observations at offset 589";
        assert_eq!(
            super::sql_schema_adaptation_kind(error),
            Some("missing_column")
        );
        assert_eq!(
            super::sql_schema_adaptation_reference(error).as_deref(),
            Some("event_date")
        );
        assert_eq!(
            super::sql_schema_adaptation_reference("no such table: connection_observations"),
            Some("connection_observations".to_owned())
        );
        let details = super::sql_schema_adaptation_details(error).unwrap();
        assert_eq!(details.kind, "missing_column");
        assert_eq!(details.reference.as_deref(), Some("event_date"));
        assert_eq!(
            details.hint.as_deref(),
            Some(
                "the source schema may have changed; inspect the SQLite schema, rebuild or edit the query, then validate it before saving"
            )
        );
    }

    #[test]
    fn legacy_automation_runs_derive_schema_adaptation_from_message() {
        let run: super::AutomationRun = serde_json::from_value(serde_json::json!({
            "job_id": Uuid::nil(),
            "job_name": "Legacy SQL job",
            "started_at": "2026-01-01T00:00:00Z",
            "finished_at": "2026-01-01T00:00:01Z",
            "success": false,
            "message": "SQL INGEST FAILED · no such column: old_event_date in SELECT old_event_date FROM observations at offset 12"
        }))
        .unwrap();
        assert!(run.schema_adaptation.is_none());
        assert_eq!(
            super::automation_run_schema_adaptation_kind(&run),
            Some("missing_column")
        );
        assert_eq!(
            super::automation_run_schema_adaptation_reference(&run).as_deref(),
            Some("old_event_date")
        );
    }

    #[test]
    fn correctness_fixture_routes_a_visible_self_loop() {
        let case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("checked-in chart correctness fixture is a valid case");
        let lanes = super::link_lane_offsets(&case);

        let loop_index = case
            .links
            .iter()
            .position(|link| link.from == link.to)
            .expect("fixture contains a self-loop");
        let loop_lane = lanes[loop_index];
        assert!(loop_lane.is_finite() && loop_lane > 0.0);
        let loop_start = egui::pos2(100.0, 120.0);
        let loop_points = super::connection_curve_points(loop_start, loop_start, loop_lane, true);
        assert_eq!(loop_points.first(), Some(&loop_start));
        assert_eq!(loop_points.last(), Some(&loop_start));
        assert!(
            loop_points
                .iter()
                .all(|point| point.x.is_finite() && point.y.is_finite())
        );
        assert!(
            loop_points
                .iter()
                .any(|point| point.distance(loop_start) > 1.0)
        );
        let highest_loop_point = loop_points
            .iter()
            .map(|point| point.y)
            .fold(f32::INFINITY, f32::min);
        assert!(highest_loop_point < loop_start.y - 50.0);

        let coincident_distinct_route =
            super::connection_curve_points(loop_start, loop_start, loop_lane, false);
        assert_ne!(loop_points[8], coincident_distinct_route[8]);
        assert!(coincident_distinct_route[8].x > loop_start.x);

        for (route, is_self_loop) in [(&loop_points, true), (&coincident_distinct_route, false)] {
            let label_center =
                super::connection_label_position(route, is_self_loop, loop_lane, 1.0);
            let label_bounds = egui::Rect::from_center_size(label_center, egui::vec2(110.0, 18.0));
            assert!(route.iter().all(|point| !label_bounds.contains(*point)));
        }

        let mut coincident_link = case.links[loop_index].clone();
        coincident_link.from = case.entities[0].id;
        coincident_link.to = case.entities[1].id;
        let coincident_position = egui::pos2(600.0, 600.0);
        let coincident_positions = std::collections::HashMap::from([
            (coincident_link.from, coincident_position),
            (coincident_link.to, coincident_position),
        ]);
        let coincident_lane = [18.0];
        let coincident_points = super::connection_curve_points(
            coincident_position,
            coincident_position,
            coincident_lane[0],
            false,
        );
        assert_eq!(
            super::nearest_link_index_at_position(
                &[coincident_link],
                &coincident_positions,
                &coincident_lane,
                coincident_points[8],
                1.0,
                22.0,
            ),
            Some(0),
            "distinct coincident endpoints remain a visible, selectable non-self relationship"
        );

        let (arrow_tip, arrow_tangent) =
            super::connection_arrowhead(&loop_points, true, 1.0).unwrap();
        assert!(loop_points.contains(&arrow_tip));
        assert!(arrow_tangent.x.is_finite() && arrow_tangent.y.is_finite());
        assert!((arrow_tangent.length() - 1.0).abs() < 0.001);

        let mut entity_positions: std::collections::HashMap<Uuid, egui::Pos2> = case
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| (entity.id, egui::pos2(1000.0 + index as f32 * 300.0, 1000.0)))
            .collect();
        entity_positions.insert(case.links[loop_index].from, loop_start);
        let route_lanes = super::geometric_link_lane_offsets(&case, &entity_positions, &lanes);
        let pointer = loop_points[12];
        let entity_indices: std::collections::HashMap<Uuid, usize> = case
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| (entity.id, index))
            .collect();
        assert!(
            super::chart_link_cycle_candidates(
                &case,
                &entity_positions,
                &entity_indices,
                &route_lanes,
                pointer,
                1.0,
            )
            .contains(&loop_index)
        );
        assert_eq!(
            super::nearest_link_index_at_position(
                &case.links,
                &entity_positions,
                &route_lanes,
                pointer,
                1.0,
                22.0,
            ),
            Some(loop_index),
            "pointer on the self-loop route selects its exact relationship record"
        );
    }

    #[test]
    fn prepared_chart_vector_geometry_matches_direct_lane_and_position_calculation() {
        let case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("checked-in chart fixture is valid");
        let rect = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(900.0, 600.0));
        let zoom = 1.35;
        let pan = egui::vec2(17.0, -9.0);
        let prepared = super::chart_vector_geometry(&case, rect, zoom, pan);
        let expected_positions: std::collections::HashMap<Uuid, egui::Pos2> = case
            .entities
            .iter()
            .map(|entity| {
                let base = egui::pos2(
                    rect.left() + entity.x * rect.width(),
                    rect.top() + entity.y * rect.height(),
                );
                (
                    entity.id,
                    rect.center() + (base - rect.center()) * zoom + pan,
                )
            })
            .collect();
        let expected_lanes = super::link_lane_offsets(&case);
        let expected_routes =
            super::geometric_link_lane_offsets(&case, &expected_positions, &expected_lanes);
        assert_eq!(prepared.link_lanes, expected_lanes);
        assert_eq!(prepared.entity_positions, expected_positions);
        assert_eq!(prepared.route_lanes, expected_routes);
    }

    #[test]
    fn chart_link_route_culling_preserves_crossings_and_nearby_decorations() {
        let viewport = egui::Rect::from_min_max(egui::pos2(0.0, 0.0), egui::pos2(900.0, 600.0));
        let offscreen_label =
            egui::Rect::from_center_size(egui::pos2(-10_000.0, -10_000.0), egui::vec2(110.0, 18.0));
        assert!(super::chart_link_route_may_intersect_viewport(
            &[egui::pos2(-500.0, 300.0), egui::pos2(1_400.0, 300.0)],
            offscreen_label,
            0,
            viewport,
            1.0,
        ));
        let curved_crossing = super::connection_curve_points(
            egui::pos2(-300.0, 300.0),
            egui::pos2(1_200.0, 300.0),
            150.0,
            false,
        );
        assert!(super::chart_link_route_may_intersect_viewport(
            &curved_crossing,
            offscreen_label,
            0,
            viewport,
            1.0,
        ));
        assert!(super::chart_link_route_may_intersect_viewport(
            &[
                egui::pos2(-300.0, -300.0),
                egui::pos2(450.0, 300.0),
                egui::pos2(1_300.0, 900.0),
            ],
            offscreen_label,
            0,
            viewport,
            1.0,
        ));
        let outside_loop_center = egui::pos2(1_000.0, 300.0);
        let self_loop =
            super::connection_curve_points(outside_loop_center, outside_loop_center, 100.0, true);
        let loop_viewport =
            egui::Rect::from_min_max(egui::pos2(950.0, -160.0), egui::pos2(1_050.0, -120.0));
        assert!(super::chart_link_route_may_intersect_viewport(
            &self_loop,
            egui::Rect::from_center_size(
                super::connection_label_position(&self_loop, true, 100.0, 1.0),
                egui::vec2(110.0, 18.0),
            ),
            0,
            loop_viewport,
            1.0,
        ));
        let high_lane_loop = super::connection_curve_points(
            egui::pos2(0.0, 300.0),
            egui::pos2(0.0, 300.0),
            54.0,
            true,
        );
        let high_lane_label = super::connection_label_position(&high_lane_loop, true, 54.0, 1.0);
        let high_lane_viewport = egui::Rect::from_min_max(
            egui::pos2(high_lane_label.x - 2.0, high_lane_label.y - 2.0),
            egui::pos2(high_lane_label.x + 2.0, high_lane_label.y + 2.0),
        );
        assert!(
            super::chart_link_route_may_intersect_viewport(
                &high_lane_loop,
                egui::Rect::from_center_size(high_lane_label, egui::vec2(110.0, 18.0)),
                0,
                high_lane_viewport,
                1.0,
            ),
            "a high lane cannot cull its visible self-loop label"
        );
        assert!(
            super::chart_link_route_may_intersect_viewport(
                &[egui::pos2(1_020.0, 300.0), egui::pos2(1_040.0, 310.0)],
                egui::Rect::from_center_size(egui::pos2(940.0, 300.0), egui::vec2(110.0, 18.0),),
                4,
                viewport,
                1.0,
            ),
            "the conservative margin retains a nearby label or attribute row"
        );
        assert!(!super::chart_link_route_may_intersect_viewport(
            &[egui::pos2(1_300.0, 300.0), egui::pos2(1_400.0, 310.0)],
            egui::Rect::from_center_size(egui::pos2(1_350.0, 300.0), egui::vec2(110.0, 18.0)),
            0,
            viewport,
            1.0,
        ));
        assert!(super::chart_link_route_may_intersect_viewport(
            &[],
            offscreen_label,
            0,
            viewport,
            1.0,
        ));
    }

    #[test]
    fn coincident_position_links_get_distinct_routes_and_exact_picking() {
        let mut entities = (0..8)
            .map(|index| layout_entity(&format!("Co-located {index}"), "PERSON", false))
            .collect::<Vec<_>>();
        for (index, entity) in entities.iter_mut().enumerate() {
            entity.id = Uuid::from_u128(index as u128 + 1);
        }
        let link = |from, to, connection_id: &str| super::Link {
            from,
            to,
            connection_id: connection_id.into(),
            label: "CONTACT".into(),
            date: String::new(),
            confidence: 2,
            directed: true,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        };
        let mut case = super::demo_case();
        case.entities = entities;
        case.links = vec![
            link(Uuid::from_u128(1), Uuid::from_u128(2), "pair-a-forward"),
            link(Uuid::from_u128(2), Uuid::from_u128(1), "pair-a-reverse"),
            link(Uuid::from_u128(3), Uuid::from_u128(4), "pair-b-forward"),
            link(Uuid::from_u128(4), Uuid::from_u128(3), "pair-b-reverse"),
            link(Uuid::from_u128(5), Uuid::from_u128(5), "self"),
            link(
                Uuid::from_u128(5),
                Uuid::from_u128(6),
                "coincident-a-forward",
            ),
            link(
                Uuid::from_u128(6),
                Uuid::from_u128(5),
                "coincident-a-reverse",
            ),
            link(
                Uuid::from_u128(7),
                Uuid::from_u128(8),
                "coincident-b-forward",
            ),
            link(
                Uuid::from_u128(8),
                Uuid::from_u128(7),
                "coincident-b-reverse",
            ),
        ];
        let positions = std::collections::HashMap::from([
            (Uuid::from_u128(1), egui::pos2(300.0, 300.0)),
            (Uuid::from_u128(2), egui::pos2(600.0, 400.0)),
            (Uuid::from_u128(3), egui::pos2(300.0, 300.0)),
            (Uuid::from_u128(4), egui::pos2(600.0, 400.0)),
            (Uuid::from_u128(5), egui::pos2(900.0, 500.0)),
            (Uuid::from_u128(6), egui::pos2(900.0, 500.0)),
            (Uuid::from_u128(7), egui::pos2(900.0, 500.0)),
            (Uuid::from_u128(8), egui::pos2(900.0, 500.0)),
        ]);
        let lanes = super::link_lane_offsets(&case);
        let route_lanes = super::geometric_link_lane_offsets(&case, &positions, &lanes);
        let routes = case
            .links
            .iter()
            .enumerate()
            .map(|(index, link)| {
                let start = positions[&link.from];
                let end = positions[&link.to];
                super::connection_curve_points(start, end, route_lanes[index], link.from == link.to)
            })
            .collect::<Vec<_>>();

        for right in 0..routes.len() {
            for left in 0..right {
                assert_ne!(
                    routes[left], routes[right],
                    "co-located link routes {left} and {right} must not overlap"
                );
            }
        }
        for (index, route) in routes.iter().enumerate() {
            assert_eq!(
                super::nearest_link_index_at_position(
                    &case.links,
                    &positions,
                    &route_lanes,
                    route[8],
                    1.0,
                    0.5,
                ),
                Some(index),
                "pointer on co-located route {index} selects that relationship record"
            );
        }
    }

    #[test]
    fn correctness_fixture_assigns_distinct_lanes_to_parallel_records() {
        let case: CaseFile =
            serde_json::from_str(include_str!("../fixtures/chart/correctness.case.json"))
                .expect("checked-in chart correctness fixture is a valid case");
        let lanes = super::link_lane_offsets(&case);
        let first_link = &case.links[0];
        let parallel_indices: Vec<_> = case
            .links
            .iter()
            .enumerate()
            .filter(|(_, link)| {
                (link.from == first_link.from && link.to == first_link.to)
                    || (link.from == first_link.to && link.to == first_link.from)
            })
            .map(|(index, _)| index)
            .collect();
        assert!(
            parallel_indices.len() >= 4,
            "fixture keeps parallel and reversed records"
        );
        let forward_start = egui::pos2(100.0, 100.0);
        let forward_end = egui::pos2(300.0, 200.0);
        let mut curves = Vec::new();
        for index in parallel_indices {
            let link = &case.links[index];
            let is_reversed = link.from != first_link.from;
            let (start, end) = if is_reversed {
                (forward_end, forward_start)
            } else {
                (forward_start, forward_end)
            };
            let mut points =
                super::connection_curve_points(start, end, lanes[index], link.from == link.to);
            if is_reversed {
                points.reverse();
            }
            curves.push((index, points));
        }
        for right in 0..curves.len() {
            for left in 0..right {
                let (left_index, left_curve) = &curves[left];
                let (right_index, right_curve) = &curves[right];
                assert_eq!(left_curve.len(), right_curve.len());
                let overlaps = left_curve
                    .iter()
                    .zip(right_curve)
                    .all(|(left, right)| left.distance(*right) <= f32::EPSILON);
                assert!(
                    !overlaps,
                    "fixture link indices {left_index} and {right_index} render the same route"
                );
            }
        }

        let mut entity_positions: std::collections::HashMap<Uuid, egui::Pos2> = case
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| (entity.id, egui::pos2(-500.0 - index as f32 * 50.0, -500.0)))
            .collect();
        entity_positions.insert(first_link.from, egui::pos2(100.0, 100.0));
        entity_positions.insert(first_link.to, egui::pos2(300.0, 200.0));
        let route_lanes = super::geometric_link_lane_offsets(&case, &entity_positions, &lanes);
        for (index, link) in case.links.iter().enumerate().filter(|(_, link)| {
            (link.from == first_link.from && link.to == first_link.to)
                || (link.from == first_link.to && link.to == first_link.from)
        }) {
            let points = super::connection_curve_points(
                entity_positions[&link.from],
                entity_positions[&link.to],
                route_lanes[index],
                link.from == link.to,
            );
            let pointer = points[points.len() / 2];
            assert_eq!(
                super::nearest_link_index_at_position(
                    &case.links,
                    &entity_positions,
                    &route_lanes,
                    pointer,
                    1.0,
                    22.0,
                ),
                Some(index),
                "pointer on fixture link {index} selects its exact relationship record"
            );
        }
    }

    #[test]
    fn chart_fit_ignores_missing_ids_and_contains_selected_entity_cards() {
        let mut entities = vec![
            layout_entity("A", "PERSON", false),
            layout_entity("B", "ORGANIZATION", false),
            layout_entity("C", "LOCATION", false),
        ];
        entities[0].id = Uuid::from_u128(1);
        entities[0].x = 0.0;
        entities[0].y = 0.0;
        entities[1].id = Uuid::from_u128(2);
        entities[1].x = 1.0;
        entities[1].y = 0.0;
        entities[2].id = Uuid::from_u128(3);
        entities[2].x = 0.5;
        entities[2].y = 1.0;
        let case = CaseFile {
            format_version: 19,
            title: "fixture".into(),
            description: String::new(),
            entities,
            links: Vec::new(),
            labels: Vec::new(),
            active_chart_id: Uuid::from_u128(2),
            active_chart_title: "fixture".into(),
            active_chart_zoom: 1.0,
            active_chart_pan: [0.0, 0.0],
            chart_coordinate_bounds: None,
            chart_tabs: Vec::new(),
            chart_templates: Vec::new(),
            active_chart_template: None,
            snapshots: Vec::new(),
            automation_jobs: Vec::new(),
            automation_runs: Vec::new(),
            automation_batches: Vec::new(),
            import_specifications: Vec::new(),
            graphml_source_catalogs: Vec::new(),
            saved_paths: Vec::new(),
            selection_sets: Vec::new(),
        };
        assert!(super::chart_fit_transform(&case, &[], egui::Rect::NOTHING).is_none());
        for size in [egui::vec2(1000.0, 700.0), egui::vec2(560.0, 420.0)] {
            let rect = egui::Rect::from_min_size(egui::pos2(37.0, 51.0), size);
            let result = super::chart_fit_transform(
                &case,
                &[
                    Uuid::from_u128(99),
                    Uuid::from_u128(1),
                    Uuid::from_u128(2),
                    Uuid::from_u128(3),
                ],
                rect,
            )
            .expect("valid selected entities exist");
            assert!(result.0.is_finite() && (0.25..=3.0).contains(&result.0));
            assert!(result.1.x.is_finite() && result.1.y.is_finite());

            let transform = super::ChartTransform::new(rect, result.0, result.1);
            for entity in &case.entities {
                let center = transform.normalized_to_screen(entity.x, entity.y);
                assert!(
                    rect.contains_rect(super::chart_entity_card_bounds(center, result.0)),
                    "fitted card for {} must stay inside the chart viewport at {size:?}",
                    entity.name
                );
            }
        }
    }

    #[test]
    fn default_and_unframed_saved_chart_views_request_fit_all_on_open() {
        let mut case = super::demo_case();
        assert!(super::chart_view_needs_initial_fit(&case));
        case.active_chart_pan = [12.0, -4.0];
        assert!(!super::chart_view_needs_initial_fit(&case));
        case.active_chart_pan = [0.0, 0.0];
        case.active_chart_zoom = 1.5;
        assert!(!super::chart_view_needs_initial_fit(&case));
        case.active_chart_zoom = 1.0;
        case.chart_coordinate_bounds = Some([0.0, 0.0, 1.0, 1.0]);
        assert!(!super::chart_view_needs_initial_fit(&case));
    }

    #[test]
    fn anomalous_chart_coordinates_map_consistently_for_fit_selection_routes_and_packet() {
        let mut case = super::demo_case();
        case.entities = vec![
            layout_entity("A", "PERSON", false),
            layout_entity("B", "PERSON", false),
            layout_entity("C", "PERSON", false),
        ];
        case.entities[0].id = Uuid::from_u128(101);
        case.entities[0].x = -10_000.0;
        case.entities[0].y = -80.0;
        case.entities[1].id = Uuid::from_u128(102);
        case.entities[1].x = 5.0;
        case.entities[1].y = 140.0;
        case.entities[2].id = Uuid::from_u128(103);
        case.entities[2].x = 10_000.0;
        case.entities[2].y = 3.0;
        case.links.truncate(1);
        case.links[0].from = case.entities[0].id;
        case.links[0].to = case.entities[1].id;
        case.labels.clear();
        let rect = egui::Rect::from_min_size(egui::pos2(23.0, 17.0), egui::vec2(1200.0, 800.0));
        let map = super::ChartCoordinateMap::for_case(&case);
        let fit = super::chart_fit_transform(
            &case,
            &case.entities.iter().map(|e| e.id).collect::<Vec<_>>(),
            rect,
        )
        .unwrap();
        let transform = super::ChartTransform::with_coordinates(rect, fit.0, fit.1, map);
        let geometry = super::chart_vector_geometry(&case, rect, fit.0, fit.1);
        let mut centers: Vec<_> = case
            .entities
            .iter()
            .map(|e| geometry.entity_positions[&e.id])
            .collect();
        centers.sort_by(|a, b| a.x.total_cmp(&b.x).then(a.y.total_cmp(&b.y)));
        assert!(
            centers
                .windows(2)
                .all(|pair| pair[0].distance(pair[1]) > 1.0),
            "distinct extreme positions must not collapse"
        );
        for entity in &case.entities {
            let center = transform.normalized_to_screen(entity.x, entity.y);
            assert!(center.x.is_finite() && center.y.is_finite());
            assert_eq!(
                super::chart_entity_at_position(
                    &case.entities,
                    &geometry.entity_positions,
                    center,
                    fit.0
                ),
                Some(entity.id)
            );
            let recovered = transform.screen_to_normalized(center).unwrap();
            assert!((recovered.x - entity.x).abs() < 0.05 && (recovered.y - entity.y).abs() < 0.05);
        }
        let link = &case.links[0];
        let from = geometry.entity_positions[&link.from];
        let to = geometry.entity_positions[&link.to];
        let curve =
            super::connection_curve_points(from, to, geometry.route_lanes[0] * fit.0, false);
        let pointer = curve[curve.len() / 2];
        assert!(
            super::polyline_distance(
                pointer,
                &super::connection_curve_points(from, to, geometry.route_lanes[0] * fit.0, false)
            ) < 0.01
        );
        assert_eq!(
            super::nearest_link_index_at_position(
                &case.links,
                &geometry.entity_positions,
                &geometry.route_lanes,
                pointer,
                fit.0,
                22.0
            ),
            Some(0)
        );
        let svg =
            super::analysis_packet_svg(&case, super::AnalysisPacketDisclosure::default()).unwrap();
        assert_eq!(svg.matches("<g data-entity=").count(), case.entities.len());
        assert_eq!(svg.matches("<g data-record=").count(), case.links.len());
        assert!(
            svg.contains("cx=\"152.00\""),
            "SVG uses the same bounded affine mapping, not per-node clamping"
        );

        let mut normal = case.clone();
        for (entity, (x, y)) in
            normal
                .entities
                .iter_mut()
                .zip([(0.0, 0.0), (1.0, 1.0), (0.5, 0.25)])
        {
            entity.x = x;
            entity.y = y;
        }
        assert!(
            super::ChartCoordinateMap::for_case(&normal).identity,
            "ordinary normalized charts retain identity mapping"
        );

        let mut extremes = normal.clone();
        extremes.entities[0].x = -f32::MAX;
        extremes.entities[1].x = f32::MAX;
        extremes.entities[2].y = f32::NAN;
        let extreme_map = super::ChartCoordinateMap::for_case(&extremes);
        assert!(extreme_map.to_viewport(-f32::MAX, 0.0).x.is_finite());
        assert!(extreme_map.to_viewport(f32::MAX, 0.0).x.is_finite());
        assert_eq!(
            extreme_map.to_viewport(f32::NAN, f32::INFINITY),
            egui::pos2(0.5, 0.5)
        );
        let svg = super::analysis_packet_svg(&extremes, super::AnalysisPacketDisclosure::default())
            .unwrap();
        assert!(!svg.contains("NaN") && !svg.contains("inf"));
    }

    #[test]
    fn standalone_chart_labels_do_not_change_entity_coordinate_mapping() {
        let mut case = super::demo_case();
        case.entities.truncate(2);
        case.entities[0].x = -25.0;
        case.entities[0].y = 12.0;
        case.entities[1].x = 80.0;
        case.entities[1].y = 99.0;
        case.labels.push(super::ChartLabel {
            id: Uuid::from_u128(999),
            text: "Outer annotation".into(),
            x: 0.5,
            y: 0.5,
            font_size: 16.0,
            color: [0, 0, 0],
            is_legend: false,
            legend_entries: Vec::new(),
            legend_entries_truncated: false,
            template_rules_for: None,
        });
        let before = super::ChartCoordinateMap::for_case(&case);
        let entity_before = before.to_viewport(case.entities[0].x, case.entities[0].y);
        case.labels[0].x = 1_000_000.0;
        case.labels[0].y = -1_000_000.0;
        let after = super::ChartCoordinateMap::for_case(&case);
        assert_eq!(after.min, before.min);
        assert_eq!(after.max, before.max);
        assert_eq!(
            after.to_viewport(case.entities[0].x, case.entities[0].y),
            entity_before
        );
    }

    #[test]
    fn anomalous_entity_drag_keeps_graph_geometry_on_the_frozen_coordinate_map() {
        let mut case = super::demo_case();
        case.entities.truncate(2);
        case.entities[0].id = Uuid::from_u128(201);
        case.entities[0].x = -10.0;
        case.entities[0].y = 5.0;
        case.entities[1].id = Uuid::from_u128(202);
        case.entities[1].x = 20.0;
        case.entities[1].y = 80.0;
        let frozen = super::ChartCoordinateMap::for_case(&case);
        case.chart_coordinate_bounds = Some(frozen.bounds());
        case.entities[0].x = -10_000.0;
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
        let geometry = super::chart_vector_geometry_with_coordinates(
            &case,
            rect,
            1.0,
            egui::Vec2::ZERO,
            frozen,
        );
        let expected = super::ChartTransform::with_coordinates(rect, 1.0, egui::Vec2::ZERO, frozen)
            .normalized_to_screen(case.entities[0].x, case.entities[0].y);
        assert_eq!(geometry.entity_positions[&case.entities[0].id], expected);
        let live = super::chart_vector_geometry_with_coordinates(
            &case,
            rect,
            1.0,
            egui::Vec2::ZERO,
            super::ChartCoordinateMap::from_entity_positions(&case),
        );
        assert_ne!(
            geometry.entity_positions[&case.entities[0].id],
            live.entity_positions[&case.entities[0].id]
        );
        let reloaded: super::CaseFile = serde_json::from_str(
            &serde_json::to_string(&case).expect("case with coordinate bounds serializes"),
        )
        .expect("saved coordinate bounds deserialize");
        assert_eq!(
            super::ChartCoordinateMap::for_case(&reloaded).min,
            frozen.min
        );
        assert_eq!(
            super::ChartCoordinateMap::for_case(&reloaded).max,
            frozen.max
        );
    }

    #[test]
    fn degenerate_chart_axes_keep_a_reversible_drag_range() {
        let rect = egui::Rect::from_min_size(egui::Pos2::ZERO, egui::vec2(1200.0, 800.0));
        for (x, y) in [(7.0, -3.0), (f32::MAX, -f32::MAX)] {
            let mut case = super::demo_case();
            case.entities.truncate(1);
            case.entities[0].x = x;
            case.entities[0].y = y;
            let coordinates = super::ChartCoordinateMap::for_case(&case);
            let transform =
                super::ChartTransform::with_coordinates(rect, 1.0, egui::Vec2::ZERO, coordinates);
            let center = transform.normalized_to_screen(x, y);
            assert_eq!(center, rect.center());
            let pointer = if x.is_sign_positive() && x > 1.0e30 {
                center + egui::vec2(-80.0, 40.0)
            } else {
                center + egui::vec2(80.0, -40.0)
            };
            let moved = transform.screen_to_normalized(pointer).unwrap();
            assert!(moved.x.is_finite() && moved.y.is_finite());
            assert_ne!(moved.x, x);
            assert_ne!(moved.y, y);
            let round_trip = transform.normalized_to_screen(moved.x, moved.y);
            assert!(round_trip.distance(pointer) < 0.01);
        }
    }

    #[test]
    fn chart_templates_save_apply_and_replace_conditional_attribute_rules() {
        let mut case = super::demo_case();
        case.active_chart_title = "Conditional rules".into();
        case.entities.clear();
        case.links.clear();
        let rule = super::ChartLegendEntry {
            description: "Active priority contacts".into(),
            symbol: super::ChartLegendSymbol::AttributeCircle,
            color: [220, 45, 60],
            attribute_class: Some("Status".into()),
            show_on_chart: true,
            conditional_color: true,
            match_exact_value: false,
            condition: super::ChartLegendCondition::Equal,
            conditional_value: "ACTIVE".into(),
            additional_conditions: vec![super::ChartLegendConditionRule {
                attribute_class: "Priority".into(),
                condition: super::ChartLegendCondition::GreaterThan,
                value: "5".into(),
            }],
            additional_conditions_match_any: false,
            conditional_line_width: Some(3.5),
        };
        case.labels.push(super::ChartLabel {
            id: Uuid::from_u128(10),
            text: "Chart legend".into(),
            x: 0.5,
            y: 0.5,
            font_size: 12.0,
            color: [240, 240, 240],
            is_legend: true,
            legend_entries: vec![rule.clone()],
            legend_entries_truncated: false,
            template_rules_for: None,
        });

        let mut app = super::App::automation_context(case, "/tmp/chart-template-rules.case.json");
        app.save_chart_template();
        let template_id = app.case.chart_templates[0].id;
        assert!(app.case.chart_templates[0].conditional_rules == [rule.clone()]);
        assert!(!app.case.chart_templates[0].conditional_rules_truncated);
        assert!(app.status.contains("1 CONDITIONAL RULES"));

        let legacy_template: super::ChartTemplate = serde_json::from_value(serde_json::json!({
            "id": Uuid::from_u128(11),
            "name": "Legacy template",
            "entity_styles": [],
            "relationship_types": []
        }))
        .expect("older templates without conditional rule fields remain readable");
        assert!(legacy_template.conditional_rules.is_empty());
        assert!(!legacy_template.conditional_rules_truncated);
        let legacy_label: super::ChartLabel = serde_json::from_value(serde_json::json!({
            "id": Uuid::from_u128(13),
            "text": "Legacy legend",
            "x": 0.5,
            "y": 0.5,
            "font_size": 12.0,
            "color": [240, 240, 240],
            "is_legend": true,
            "legend_entries": [],
            "legend_entries_truncated": false
        }))
        .expect("older chart labels without generated-template ownership remain readable");
        assert_eq!(legacy_label.template_rules_for, None);

        let mut entity = layout_entity("Analyst target", "PERSON", false);
        entity.attributes = vec![
            ("Status".into(), "ACTIVE".into()),
            ("Priority".into(), "9".into()),
        ];
        app.case.entities = vec![entity];
        let mut local_rule = rule.clone();
        local_rule.color = [20, 80, 200];
        local_rule.conditional_line_width = Some(1.0);
        app.case.labels = vec![super::ChartLabel {
            id: Uuid::from_u128(12),
            text: "Local rules".into(),
            x: 0.25,
            y: 0.25,
            font_size: 12.0,
            color: [240, 240, 240],
            is_legend: true,
            legend_entries: vec![rule.clone(), local_rule],
            legend_entries_truncated: false,
            template_rules_for: None,
        }];
        app.apply_chart_template(template_id);

        assert_eq!(
            super::chart_conditional_attribute_color(
                &app.case.entities[0].attributes,
                &app.case.labels
            ),
            Some([220, 45, 60]),
            "template rules take precedence over conflicting chart rules"
        );
        assert_eq!(
            super::chart_conditional_attribute_line_width(
                &app.case.entities[0].attributes,
                &app.case.labels
            ),
            Some(3.5)
        );
        assert_eq!(
            app.case
                .labels
                .iter()
                .filter(|label| label.template_rules_for == Some(template_id))
                .count(),
            1
        );

        let saved_case = serde_json::to_string(&app.case).expect("case serialization succeeds");
        let restored_case: super::CaseFile =
            serde_json::from_str(&saved_case).expect("template rules survive case persistence");
        assert!(
            restored_case
                .labels
                .iter()
                .any(|label| label.template_rules_for == Some(template_id))
        );
        assert!(restored_case.chart_templates[0].conditional_rules == [rule]);

        app.apply_chart_template(template_id);
        assert_eq!(
            app.case
                .labels
                .iter()
                .filter(|label| label.template_rules_for == Some(template_id))
                .count(),
            1,
            "reapplying a template replaces its generated rules instead of duplicating them"
        );
        app.delete_chart_template(template_id);
        assert!(
            !app.case
                .labels
                .iter()
                .any(|label| label.template_rules_for == Some(template_id))
        );
        assert!(app.case.chart_templates.is_empty());
    }

    #[test]
    fn chart_template_conditional_rule_limit_is_visible_and_bounded() {
        let mut case = super::demo_case();
        case.active_chart_title = "Bounded rules".into();
        case.entities.clear();
        case.links.clear();
        let entries = (0..=super::MAX_CHART_TEMPLATE_CONDITIONAL_RULES)
            .map(|index| super::ChartLegendEntry {
                description: format!("Rule {index}"),
                symbol: super::ChartLegendSymbol::AttributeSquare,
                color: [index as u8, 30, 60],
                attribute_class: Some(format!("Attribute {index}")),
                show_on_chart: true,
                conditional_color: true,
                match_exact_value: true,
                condition: super::ChartLegendCondition::Equal,
                conditional_value: "active".into(),
                additional_conditions: Vec::new(),
                additional_conditions_match_any: false,
                conditional_line_width: None,
            })
            .collect::<Vec<_>>();
        case.labels = entries
            .chunks(super::MAX_CHART_LEGEND_ENTRIES)
            .enumerate()
            .map(|(index, legend_entries)| super::ChartLabel {
                id: Uuid::from_u128(100 + index as u128),
                text: format!("Legend {index}"),
                x: 0.5,
                y: 0.5,
                font_size: 12.0,
                color: [240, 240, 240],
                is_legend: true,
                legend_entries: legend_entries.to_vec(),
                legend_entries_truncated: false,
                template_rules_for: None,
            })
            .collect();

        let mut app =
            super::App::automation_context(case, "/tmp/chart-template-rule-limit.case.json");
        app.save_chart_template();
        let template_id = app.case.chart_templates[0].id;
        assert_eq!(
            app.case.chart_templates[0].conditional_rules.len(),
            super::MAX_CHART_TEMPLATE_CONDITIONAL_RULES
        );
        assert!(app.case.chart_templates[0].conditional_rules_truncated);
        assert!(app.status.contains("RULE LIMIT REACHED"));

        app.case.labels.clear();
        app.apply_chart_template(template_id);
        let generated = app
            .case
            .labels
            .iter()
            .filter(|label| label.template_rules_for == Some(template_id))
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), 4);
        assert_eq!(
            generated
                .iter()
                .map(|label| label.legend_entries.len())
                .sum::<usize>(),
            super::MAX_CHART_TEMPLATE_CONDITIONAL_RULES
        );
        assert!(generated.last().unwrap().legend_entries_truncated);
        assert!(app.status.contains("SAVED RULE LIMIT REACHED"));

        app.apply_chart_template(template_id);
        assert_eq!(
            app.case
                .labels
                .iter()
                .filter(|label| label.template_rules_for == Some(template_id))
                .count(),
            4,
            "reapplying does not multiply generated rule legends"
        );
    }

    #[test]
    fn chart_transform_round_trips_and_keeps_the_zoom_anchor_stationary() {
        let rect = egui::Rect::from_min_size(egui::pos2(37.0, 51.0), egui::vec2(1200.0, 750.0));
        let transform = super::ChartTransform::new(rect, 1.7, egui::vec2(-41.0, 23.0));
        for (x, y) in [(0.0, 0.0), (0.05, 0.92), (0.5, 0.5), (1.0, 1.0)] {
            let screen = transform.normalized_to_screen(x, y);
            let recovered = transform
                .screen_to_normalized(screen)
                .expect("valid viewport maps back to normalized chart coordinates");
            let reconstructed = transform.normalized_to_screen(recovered.x, recovered.y);
            assert!(reconstructed.distance(screen) <= 0.1);
        }

        let pointer = egui::pos2(713.0, 389.0);
        let before = transform
            .screen_to_normalized(pointer)
            .expect("pointer is inside a non-degenerate chart viewport");
        let zoomed = transform.zoomed_at(pointer, 2.6);
        let after = zoomed
            .screen_to_normalized(pointer)
            .expect("zoom retains a valid chart viewport");
        assert!((before.x - after.x).abs() < 0.00001);
        assert!((before.y - after.y).abs() < 0.00001);

        let degenerate = egui::Rect::from_min_max(egui::pos2(5.0, 5.0), egui::pos2(5.0, 25.0));
        assert!(
            super::ChartTransform::new(degenerate, 1.0, egui::Vec2::ZERO)
                .screen_to_normalized(egui::pos2(5.0, 10.0))
                .is_none()
        );
    }

    #[test]
    fn chart_entity_hit_uses_visible_card_bounds_and_topmost_paint_order() {
        let mut first = layout_entity("FIRST", "PERSON", false);
        let mut second = layout_entity("SECOND", "PERSON", false);
        let first_id = first.id;
        let second_id = second.id;
        first.x = 0.5;
        first.y = 0.5;
        second.x = 0.5;
        second.y = 0.5;
        let entities = vec![first, second];
        let center = egui::pos2(400.0, 300.0);
        let positions = std::collections::HashMap::from([(first_id, center), (second_id, center)]);

        assert_eq!(
            super::chart_entity_at_position(&entities, &positions, center, 1.0),
            Some(second_id),
            "the last painted overlapping card receives the hit"
        );
        assert_eq!(
            super::chart_entity_at_position(
                &entities,
                &positions,
                center + egui::vec2(70.0, 27.0),
                1.0,
            ),
            Some(second_id),
            "the actual card edge remains selectable"
        );
        assert_eq!(
            super::chart_entity_at_position(
                &entities,
                &positions,
                center + egui::vec2(0.0, 29.0),
                1.0,
            ),
            None,
            "points outside the visible card do not use a larger radial hit target"
        );
    }

    #[test]
    fn anonymous_duplicate_selection_tracks_shifts_and_clears_when_ambiguous() {
        let entity_a = Uuid::from_u128(10);
        let entity_b = Uuid::from_u128(11);
        let make_link = || super::Link {
            from: entity_a,
            to: entity_b,
            connection_id: String::new(),
            label: "DUPLICATE FIXTURE".into(),
            date: "2025-01-01".into(),
            confidence: 2,
            directed: false,
            notes: "same anonymous record".into(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: None,
            attributes: Vec::new(),
        };
        let mut prefix = make_link();
        prefix.label = "UNRELATED PREFIX".into();
        let mut links = vec![prefix, make_link(), make_link(), make_link()];
        let selected = super::selected_link_identity(&links, 3).expect("selected duplicate");
        links.remove(0);
        assert_eq!(super::refreshed_link_index(&links, &selected), Some(2));
        links.remove(0);
        links.remove(0);
        assert_eq!(super::refreshed_link_index(&links, &selected), None);
    }

    #[test]
    fn anx_duplicate_connection_ids_receive_stable_record_scopes() {
        let edge = |id: &str, connection_id: &str| {
            let mut edge = super::GraphMlRecord {
                id: id.into(),
                source: "ENTITY-A".into(),
                target: "ENTITY-B".into(),
                ..super::GraphMlRecord::default()
            };
            edge.data
                .insert("connection_id".into(), connection_id.into());
            edge
        };
        let mut edges = vec![
            edge("ITEM-1", "shared"),
            edge("ITEM-2", "shared"),
            edge("ITEM-3", "unique"),
        ];
        super::scope_duplicate_anx_connection_ids(&mut edges);
        assert_eq!(edges[0].data["connection_id"], "shared");
        assert_eq!(edges[1].data["connection_id"], "shared");
        assert_ne!(
            edges[0].data["ingest_source"],
            edges[1].data["ingest_source"]
        );
        assert!(edges[0].data["ingest_source"].starts_with("anx:duplicate-connection:"));
        assert!(edges[1].data["ingest_source"].starts_with("anx:duplicate-connection:"));
        assert!(!edges[2].data.contains_key("ingest_source"));

        let mut repeated = vec![
            edge("ITEM-1", "shared"),
            edge("ITEM-2", "shared"),
            edge("ITEM-3", "unique"),
        ];
        super::scope_duplicate_anx_connection_ids(&mut repeated);
        assert_eq!(
            edges[0].data["ingest_source"],
            repeated[0].data["ingest_source"]
        );
        assert_eq!(
            edges[1].data["ingest_source"],
            repeated[1].data["ingest_source"]
        );
    }

    #[test]
    fn anx_import_worker_reports_parse_errors_with_its_target_chart() {
        let target_chart_id = Uuid::new_v4();
        let missing_path = std::env::temp_dir()
            .join(format!("arch-notebook-missing-{}.anx", Uuid::new_v4()))
            .to_string_lossy()
            .into_owned();
        let receiver = super::spawn_anx_import_worker(missing_path, target_chart_id)
            .expect("background ANX worker starts");
        let (received_chart_id, result) = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("background worker returns a result");
        assert_eq!(received_chart_id, target_chart_id);
        assert!(result.is_err());
    }

    #[test]
    fn anx_import_worker_returns_a_validated_chart_payload() {
        let case = super::demo_case();
        let work_dir =
            std::env::temp_dir().join(format!("arch-notebook-anx-worker-{}", Uuid::new_v4()));
        std::fs::create_dir(&work_dir).expect("create isolated ANX worker directory");
        let case_path = work_dir.join("source.case.json");
        let anx_path = work_dir.join("source.anx");
        super::write_case_anx(
            &case,
            &case_path.to_string_lossy(),
            &anx_path.to_string_lossy(),
        )
        .expect("write fictional ANX worker input");
        let target_chart_id = case.active_chart_id;
        let receiver = super::spawn_anx_import_worker(
            anx_path.to_string_lossy().into_owned(),
            target_chart_id,
        )
        .expect("background ANX worker starts");
        let (received_chart_id, result) = receiver
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("background worker returns a result");
        let payload = result.expect("valid ANX XML parses successfully");
        assert_eq!(received_chart_id, target_chart_id);
        assert!(payload.has_chart_items);
        assert!(!payload.csv_bytes.is_empty());
        assert!(String::from_utf8_lossy(&payload.csv_bytes).contains("Avery Rowan"));
        std::fs::remove_dir_all(&work_dir).expect("remove isolated ANX worker directory");
    }

    #[test]
    fn source_scoped_connection_ids_remain_distinct() {
        let make_link = |source: &str| super::Link {
            from: Uuid::from_u128(20),
            to: Uuid::from_u128(21),
            connection_id: "shared-id".into(),
            label: "SHARED".into(),
            date: "2025-01-01".into(),
            confidence: 2,
            directed: true,
            notes: String::new(),
            source_notes: String::new(),
            source_reference: String::new(),
            ingest_source: Some(source.into()),
            attributes: Vec::new(),
        };
        let links = vec![make_link("source-a"), make_link("source-b")];
        let selected = super::selected_link_identity(&links, 1).expect("selected record");
        assert_eq!(super::refreshed_link_index(&links, &selected), Some(1));
    }

    #[test]
    fn navigator_projection_maps_anomalous_entities_and_clips_offscreen_viewports() {
        let entity = layout_entity("edge", "PERSON", false);
        let map = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(100.0, 60.0));
        let mut edge = entity;
        edge.x = 1.2;
        edge.y = -0.2;
        let mut other = edge.clone();
        other.x = -4.0;
        other.y = 12.0;
        let coordinates = super::ChartCoordinateMap::for_case(&super::CaseFile {
            entities: vec![edge.clone(), other],
            ..super::demo_case()
        });
        let point = super::chart_navigator_map_point(&edge, map, coordinates);
        assert_eq!(
            point,
            egui::pos2(
                map.left() + map.width() * 0.9,
                map.top() + map.height() * 0.1
            )
        );
        let canvas = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 500.0));
        let viewport =
            egui::Rect::from_min_size(egui::pos2(-200.0, 100.0), egui::vec2(400.0, 300.0));
        let projected = super::chart_navigator_viewport(viewport, canvas, map);
        assert_eq!(projected.left(), map.left());
        assert!(projected.right() <= map.right());
        assert!(projected.top() >= map.top() && projected.bottom() <= map.bottom());
    }
}
