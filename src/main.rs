Warning: truncated output (original token count: 494203)
... 928235 bytes omitted ...

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
    configuration_fingerprint: String,
    #[serde(default)]
    selected_jobs: Vec<AutomationBatchJobSelection>,
    #[serde(default)]
    effective_jobs: Vec<AutomationJob>,
    #[serde(default, alias = "successful_jobs")]
    completed_jobs: Vec<AutomationJobCheckpoint>,
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
}

const MAX_CHART_LEGEND_ENTRIES: usize = 24;
const MAX_CHART_LEGEND_DESCRIPTION_BYTES: usize = 4_096;
const MAX_CHART_LEGEND_CONDITIONS: usize = 8;

#[derive(Clone, Serialize, Deserialize)]
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

#[derive(Clone, Serialize, Deserialize)]
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
    for entry in labels
        .iter()
        .filter(|label| label.is_legend)
        .flat_map(|label| &label.legend_entries)
    {
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
            .find(…232152 tokens truncated…))
                    else {
                        report_cli_usage_error("--name requires a design name", json_output);
                    };
                    name = Some(value.as_str());
                    seen_name = true;
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        let Some(name) = name else {
            report_cli_usage_error(usage, json_output);
        };
        if let Err(error) =
            create_import_design_package_cli(template_path, output_path, name, json_output)
        {
            report_cli_error("import design package creation failed", error, json_output);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--export-import-design")
    {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --export-import-design <case.json> <design-name-or-id> <output.json> [--json]";
        let (Some(case_path), Some(selector), Some(output_path)) =
            (args.get(1), args.get(2), args.get(3))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 5 || args.get(4).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        if let Err(error) = export_import_design_cli(case_path, selector, output_path, json_output)
        {
            report_cli_error("import design export failed", error, json_output);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--import-import-design")
    {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --import-import-design <case.json> <input.json> [--name <design-name>] [--update <design-name-or-id>] [--json]";
        let (Some(case_path), Some(input_path)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut name_override = None;
        let mut replace_selector = None;
        let mut seen_name = false;
        let mut seen_update = false;
        let mut seen_json = false;
        let mut index = 3;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--name" if !seen_name => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error("--name requires a design name", json_output);
                    };
                    name_override = Some(value.as_str());
                    seen_name = true;
                }
                "--update" if !seen_update => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error(
                            "--update requires an existing design name or UUID",
                            json_output,
                        );
                    };
                    replace_selector = Some(value.as_str());
                    seen_update = true;
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        if let Err(error) = import_import_design_cli(
            case_path,
            input_path,
            name_override,
            replace_selector,
            json_output,
        ) {
            report_cli_error("import design import failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--export-sql-series") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --export-sql-series <case.json> <snapshot-id> <output.csv|tsv> [--json]";
        let (Some(case_path), Some(snapshot_id), Some(output_path)) =
            (args.get(1), args.get(2), args.get(3))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 5 || args.get(4).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let snapshot_id = Uuid::parse_str(snapshot_id)
            .unwrap_or_else(|_| report_cli_usage_error("invalid snapshot UUID", json_output));
        if let Err(error) = export_sql_series_cli(case_path, snapshot_id, output_path, json_output)
        {
            report_cli_error("SQL series export failed", error, json_output);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--export-sql-snapshot")
    {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --export-sql-snapshot <case.json> <snapshot-id> <output.csv|tsv> [--json]";
        let (Some(case_path), Some(snapshot_id), Some(output_path)) =
            (args.get(1), args.get(2), args.get(3))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 5 || args.get(4).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let snapshot_id = Uuid::parse_str(snapshot_id)
            .unwrap_or_else(|_| report_cli_usage_error("invalid snapshot UUID", json_output));
        if let Err(error) =
            export_sql_snapshot_cli(case_path, snapshot_id, output_path, json_output)
        {
            report_cli_error("SQL snapshot export failed", error, json_output);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--export-sql-chart-csv")
    {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --export-sql-chart-csv <case.json> <snapshot-id> <output.csv|tsv> [--json]";
        let (Some(case_path), Some(snapshot_id), Some(output_path)) =
            (args.get(1), args.get(2), args.get(3))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 5 || args.get(4).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let snapshot_id = Uuid::parse_str(snapshot_id)
            .unwrap_or_else(|_| report_cli_usage_error("invalid snapshot UUID", json_output));
        if let Err(error) =
            export_sql_snapshot_chart_csv_cli(case_path, snapshot_id, output_path, json_output)
        {
            report_cli_error("SQL chart CSV export failed", error, json_output);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--export-sql-comparison")
    {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --export-sql-comparison <case.json> <baseline-snapshot-id> <current-snapshot-id> <output.csv|tsv> [--json]";
        let (Some(case_path), Some(baseline_id), Some(current_id), Some(output_path)) =
            (args.get(1), args.get(2), args.get(3), args.get(4))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 6 || args.get(5).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let baseline_id = Uuid::parse_str(baseline_id).unwrap_or_else(|_| {
            report_cli_usage_error("invalid baseline snapshot UUID", json_output)
        });
        let current_id = Uuid::parse_str(current_id).unwrap_or_else(|_| {
            report_cli_usage_error("invalid current snapshot UUID", json_output)
        });
        if let Err(error) =
            export_sql_comparison_cli(case_path, baseline_id, current_id, output_path, json_output)
        {
            report_cli_error("SQL comparison export failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--list-jobs") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --list-jobs <case.json> [--enabled-only] [--schema-adaptation-needed] [--json]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut enabled_only = false;
        let mut schema_adaptation_needed = false;
        let mut seen_json = false;
        let mut seen_enabled_only = false;
        let mut seen_schema_adaptation_needed = false;
        for option in args.iter().skip(2) {
            match option.as_str() {
                "--json" if !seen_json => seen_json = true,
                "--enabled-only" if !seen_enabled_only => {
                    seen_enabled_only = true;
                    enabled_only = true;
                }
                "--schema-adaptation-needed" if !seen_schema_adaptation_needed => {
                    seen_schema_adaptation_needed = true;
                    schema_adaptation_needed = true;
                }
                _ => report_cli_usage_error(usage, json_output),
            }
        }
        if let Err(error) = list_automation_jobs_cli(
            case_path,
            json_output,
            enabled_only,
            schema_adaptation_needed,
        ) {
            report_cli_error("job listing failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--list-batches") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --list-batches <case.json> [--json]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 3 || (args.len() == 3 && args[2] != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        if let Err(error) = list_automation_batches_cli(case_path, json_output) {
            report_cli_error("automation batch listing failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--annotate-run") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --annotate-run <case.json> <run-id> <comment> [--json]";
        let (Some(case_path), Some(run_id), Some(note)) = (args.get(1), args.get(2), args.get(3))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 5 || (args.len() == 5 && args[4] != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let run_id = run_id
            .parse::<Uuid>()
            .unwrap_or_else(|_| report_cli_usage_error("run-id must be a UUID", json_output));
        if let Err(error) = annotate_automation_run_cli(case_path, run_id, note, json_output) {
            report_cli_error("automation run comment failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--review-run") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage =
            "usage: arch-notebook --review-run <case.json> <run-id> <reviewed|unreviewed> [--json]";
        let (Some(case_path), Some(run_id), Some(review_state)) =
            (args.get(1), args.get(2), args.get(3))
        else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 5 || (args.len() == 5 && args[4] != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let run_id = run_id
            .parse::<Uuid>()
            .unwrap_or_else(|_| report_cli_usage_error("run-id must be a UUID", json_output));
        let reviewed = match review_state.as_str() {
            "reviewed" => true,
            "unreviewed" => false,
            _ => report_cli_usage_error("review-state must be reviewed or unreviewed", json_output),
        };
        if let Err(error) = set_automation_run_review_cli(case_path, run_id, reviewed, json_output)
        {
            report_cli_error("automation run review update failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--list-runs") {
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(
                "usage: arch-notebook --list-runs <case.json> [--json] [--job <name-or-id>] [--batch <batch-uuid>] [--trigger <desktop|scheduled|cli|replay|legacy>] [--schema-adaptation-needed] [--schema-stability-failed] [--offset <count>] [--limit <count>]",
                args.iter().any(|arg| arg == "--json"),
            );
        };
        let json_output = args.iter().any(|arg| arg == "--json");
        let mut job_filter = None;
        let mut batch_filter = None;
        let mut trigger_filter = None;
        let mut schema_adaptation_needed = false;
        let mut seen_schema_adaptation_needed = false;
        let mut schema_stability_failed = false;
        let mut seen_schema_stability_failed = false;
        let mut offset = 0usize;
        let mut limit = None;
        let mut seen_json = false;
        let mut seen_job = false;
        let mut seen_batch = false;
        let mut seen_trigger = false;
        let mut seen_offset = false;
        let mut seen_limit = false;
        let mut index = 2;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--schema-adaptation-needed" if !seen_schema_adaptation_needed => {
                    schema_adaptation_needed = true;
                    seen_schema_adaptation_needed = true;
                }
                "--schema-stability-failed" if !seen_schema_stability_failed => {
                    schema_stability_failed = true;
                    seen_schema_stability_failed = true;
                }
                "--job" if !seen_job => {
                    index += 1;
                    let Some(value) = args.get(index).filter(|value| !value.trim().is_empty())
                    else {
                        report_cli_usage_error("--job requires a job name or UUID", json_output);
                    };
                    job_filter = Some(value.clone());
                    seen_job = true;
                }
                "--batch" if !seen_batch => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error("--batch requires a batch UUID", json_output);
                    };
                    batch_filter = Some(Uuid::parse_str(value).unwrap_or_else(|_| {
                        report_cli_usage_error("--batch requires a valid batch UUID", json_output)
                    }));
                    seen_batch = true;
                }
                "--trigger" if !seen_trigger => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error(
                            "--trigger requires desktop, scheduled, cli, replay, or legacy",
                            json_output,
                        );
                    };
                    let normalized = value.to_ascii_lowercase();
                    if !matches!(
                        normalized.as_str(),
                        "desktop" | "scheduled" | "cli" | "replay" | "legacy"
                    ) {
                        report_cli_usage_error(
                            "--trigger requires desktop, scheduled, cli, replay, or legacy",
                            json_output,
                        );
                    }
                    trigger_filter = Some(normalized);
                    seen_trigger = true;
                }
                "--offset" if !seen_offset => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error(
                            "--offset requires a nonnegative integer",
                            json_output,
                        );
                    };
                    offset = value.parse::<usize>().unwrap_or_else(|_| {
                        report_cli_usage_error(
                            "--offset requires a nonnegative integer",
                            json_output,
                        )
                    });
                    seen_offset = true;
                }
                "--limit" if !seen_limit => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error("--limit requires a positive integer", json_output);
                    };
                    limit = Some(
                        value
                            .parse::<usize>()
                            .ok()
                            .filter(|value| *value > 0)
                            .unwrap_or_else(|| {
                                report_cli_usage_error(
                                    "--limit requires a positive integer",
                                    json_output,
                                )
                            }),
                    );
                    seen_limit = true;
                }
                _ => report_cli_usage_error(
                    "usage: arch-notebook --list-runs <case.json> [--json] [--job <name-or-id>] [--batch <batch-uuid>] [--trigger <desktop|scheduled|cli|replay|legacy>] [--schema-adaptation-needed] [--schema-stability-failed] [--offset <count>] [--limit <count>]",
                    json_output,
                ),
            }
            index += 1;
        }
        if let Err(error) = list_automation_runs_cli(
            case_path,
            json_output,
            job_filter.as_deref(),
            batch_filter,
            trigger_filter.as_deref(),
            schema_adaptation_needed,
            schema_stability_failed,
            offset,
            limit,
        ) {
            report_cli_error("run history listing failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--export-runs") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --export-runs <case.json> <output.csv|tsv> [--job <name-or-id>] [--batch <batch-uuid>] [--trigger <desktop|scheduled|cli|replay|legacy>] [--schema-adaptation-needed] [--schema-stability-failed] [--offset <count>] [--limit <count>] [--json]";
        let (Some(case_path), Some(output_path)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut job_filter = None;
        let mut batch_filter = None;
        let mut trigger_filter = None;
        let mut schema_adaptation_needed = false;
        let mut seen_schema_adaptation_needed = false;
        let mut schema_stability_failed = false;
        let mut seen_schema_stability_failed = false;
        let mut offset = 0usize;
        let mut limit = None;
        let mut seen_json = false;
        let mut seen_job = false;
        let mut seen_batch = false;
        let mut seen_trigger = false;
        let mut seen_offset = false;
        let mut seen_limit = false;
        let mut index = 3;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--schema-adaptation-needed" if !seen_schema_adaptation_needed => {
                    schema_adaptation_needed = true;
                    seen_schema_adaptation_needed = true;
                }
                "--schema-stability-failed" if !seen_schema_stability_failed => {
                    schema_stability_failed = true;
                    seen_schema_stability_failed = true;
                }
                "--job" if !seen_job => {
                    index += 1;
                    let Some(value) = args.get(index).filter(|value| !value.trim().is_empty())
                    else {
                        report_cli_usage_error("--job requires a job name or UUID", json_output);
                    };
                    job_filter = Some(value.clone());
                    seen_job = true;
                }
                "--batch" if !seen_batch => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error("--batch requires a batch UUID", json_output);
                    };
                    batch_filter = Some(Uuid::parse_str(value).unwrap_or_else(|_| {
                        report_cli_usage_error("--batch requires a valid batch UUID", json_output)
                    }));
                    seen_batch = true;
                }
                "--trigger" if !seen_trigger => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error(
                            "--trigger requires desktop, scheduled, cli, replay, or legacy",
                            json_output,
                        );
                    };
                    let normalized = value.to_ascii_lowercase();
                    if !matches!(
                        normalized.as_str(),
                        "desktop" | "scheduled" | "cli" | "replay" | "legacy"
                    ) {
                        report_cli_usage_error(
                            "--trigger requires desktop, scheduled, cli, replay, or legacy",
                            json_output,
                        );
                    }
                    trigger_filter = Some(normalized);
                    seen_trigger = true;
                }
                "--offset" if !seen_offset => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error(
                            "--offset requires a nonnegative integer",
                            json_output,
                        );
                    };
                    offset = value.parse::<usize>().unwrap_or_else(|_| {
                        report_cli_usage_error(
                            "--offset requires a nonnegative integer",
                            json_output,
                        )
                    });
                    seen_offset = true;
                }
                "--limit" if !seen_limit => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error("--limit requires a positive integer", json_output);
                    };
                    limit = Some(
                        value
                            .parse::<usize>()
                            .ok()
                            .filter(|value| *value > 0)
                            .unwrap_or_else(|| {
                                report_cli_usage_error(
                                    "--limit requires a positive integer",
                                    json_output,
                                )
                            }),
                    );
                    seen_limit = true;
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        if let Err(error) = export_automation_runs_cli(
            case_path,
            output_path,
            job_filter.as_deref(),
            batch_filter,
            trigger_filter.as_deref(),
            schema_adaptation_needed,
            schema_stability_failed,
            offset,
            limit,
            json_output,
        ) {
            report_cli_error("run history export failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--explain-job") {
        let usage = "usage: arch-notebook --explain-job <case.json> <job-name-or-id> [--param name=value ...] [--json]";
        let json_output = args.iter().any(|arg| arg == "--json");
        let (Some(case_path), Some(selector)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut parameter_options = Vec::new();
        let mut seen_json = false;
        let mut index = 3;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--param" => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error("--param requires name=value", json_output);
                    };
                    parameter_options.push(value.clone());
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        let parameter_overrides = parse_sql_parameters(&parameter_options.join("\n"))
            .unwrap_or_else(|error| report_cli_usage_error(&error, json_output));
        if let Err(error) =
            explain_automation_job_cli(case_path, selector, &parameter_overrides, json_output)
        {
            report_cli_error("SQL query plan inspection failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--validate-job") {
        let usage = "usage: arch-notebook --validate-job <case.json> <job-name-or-id> [--query-only] [--param name=value ...] [--json]";
        let json_output = args.iter().any(|arg| arg == "--json");
        let (Some(case_path), Some(selector)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut queries_only = false;
        let mut seen_query_only = false;
        let mut seen_json = false;
        let mut parameter_options = Vec::new();
        let mut index = 3;
        while index < args.len() {
            match args[index].as_str() {
                "--query-only" if !seen_query_only => {
                    queries_only = true;
                    seen_query_only = true;
                }
                "--json" if !seen_json => seen_json = true,
                "--param" => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error("--param requires name=value", json_output);
                    };
                    parameter_options.push(value.clone());
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        let parameter_overrides = parse_sql_parameters(&parameter_options.join("\n"))
            .unwrap_or_else(|error| report_cli_usage_error(&error, json_output));
        match validate_automation_job_cli(
            case_path,
            selector,
            &parameter_overrides,
            queries_only,
            json_output,
        ) {
            Err(error) => report_cli_error("automation job validation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation job validation failed; case file was not changed");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--validate-enabled-jobs")
    {
        let usage = "usage: arch-notebook --validate-enabled-jobs <case.json> [--json] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, args.iter().any(|arg| arg == "--json"));
        };
        let (json_output, selected_jobs, parameter_overrides, scoped_parameter_overrides, _, _) =
            parse_batch_selection_args(&args, usage, false);
        match validate_enabled_automation_jobs_cli(
            case_path,
            selected_jobs.as_deref(),
            &parameter_overrides,
            &scoped_parameter_overrides,
            false,
            false,
            false,
            false,
            json_output,
        ) {
            Err(error) => report_cli_error("automation validation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation validation failed; case file was not changed");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--check-enabled-job-queries")
    {
        let usage = "usage: arch-notebook --check-enabled-job-queries <case.json> [--json] [--progress] [--require-usable] [--require-unchanged-schema] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, args.iter().any(|arg| arg == "--json"));
        };
        if args.iter().filter(|arg| *arg == "--require-usable").count() > 1 {
            report_cli_usage_error(
                "--require-usable may be specified once",
                args.iter().any(|arg| arg == "--json"),
            );
        }
        let require_usable_connections = args.iter().any(|arg| arg == "--require-usable");
        if args
            .iter()
            .filter(|arg| *arg == "--require-unchanged-schema")
            .count()
            > 1
        {
            report_cli_usage_error(
                "--require-unchanged-schema may be specified once",
                args.iter().any(|arg| arg == "--json"),
            );
        }
        let require_unchanged_schema = args.iter().any(|arg| arg == "--require-unchanged-schema");
        if args.iter().filter(|arg| *arg == "--progress").count() > 1 {
            report_cli_usage_error(
                "--progress may be specified once",
                args.iter().any(|arg| arg == "--json"),
            );
        }
        let report_progress = args.iter().any(|arg| arg == "--progress");
        let filtered_args = args
            .iter()
            .filter(|arg| {
                !matches!(
                    arg.as_str(),
                    "--require-usable" | "--require-unchanged-schema" | "--progress"
                )
            })
            .cloned()
            .collect::<Vec<_>>();
        let (json_output, selected_jobs, parameter_overrides, scoped_parameter_overrides, _, _) =
            parse_batch_selection_args(&filtered_args, usage, false);
        match validate_enabled_automation_jobs_cli(
            case_path,
            selected_jobs.as_deref(),
            &parameter_overrides,
            &scoped_parameter_overrides,
            true,
            require_usable_connections,
            require_unchanged_schema,
            report_progress,
            json_output,
        ) {
            Err(error) => report_cli_error("SQL query preflight failed", error, json_output),
            Ok(false) => {
                eprintln!("SQL query preflight failed; case file was not changed");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--run-enabled-jobs") {
        let usage = "usage: arch-notebook --run-enabled-jobs <case.json> [--json] [--progress] [--resume-batch <batch-id>] [--export-chart-csv-dir <absolute-directory>] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, args.iter().any(|arg| arg == "--json"));
        };
        if args.iter().filter(|arg| *arg == "--progress").count() > 1 {
            report_cli_usage_error(
                "--progress may be specified once",
                args.iter().any(|arg| arg == "--json"),
            );
        }
        let report_progress = args.iter().any(|arg| arg == "--progress");
        let filtered_args = args
            .iter()
            .filter(|arg| arg.as_str() != "--progress")
            .cloned()
            .collect::<Vec<_>>();
        let (
            json_output,
            selected_jobs,
            parameter_overrides,
            scoped_parameter_overrides,
            resume_batch_id,
            export_chart_csv_dir,
        ) = parse_batch_selection_args(&filtered_args, usage, true);
        match run_automation_cli(
            case_path,
            None,
            selected_jobs.as_deref(),
            &parameter_overrides,
            &scoped_parameter_overrides,
            resume_batch_id,
            None,
            export_chart_csv_dir.as_deref(),
            report_progress,
            json_output,
        ) {
            Err(error) => report_cli_error("automation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation failed; run history saved");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--run-job") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --run-job <case.json> <job-name-or-id> [--param name=value ...] [--resume-batch <batch-id>] [--export-chart-csv <output.csv|tsv>] [--progress] [--json]";
        let (Some(case_path), Some(job_name)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut parameter_options = Vec::new();
        let mut seen_json = false;
        let mut resume_batch_id = None;
        let mut export_chart_csv_path = None;
        let mut report_progress = false;
        let mut seen_progress = false;
        let mut index = 3;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--progress" if !seen_progress => {
                    report_progress = true;
                    seen_progress = true;
                }
                "--param" => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error("--param requires name=value", json_output);
                    };
                    parameter_options.push(value.clone());
                }
                "--resume-batch" if resume_batch_id.is_none() => {
                    index += 1;
                    let Some(value) = args.get(index) else {
                        report_cli_usage_error("--resume-batch requires a batch UUID", json_output);
                    };
                    resume_batch_id = Some(Uuid::parse_str(value).unwrap_or_else(|_| {
                        report_cli_usage_error(
                            "--resume-batch requires a valid batch UUID",
                            json_output,
                        )
                    }));
                }
                "--export-chart-csv" if export_chart_csv_path.is_none() => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error(
                            "--export-chart-csv requires an output.csv or output.tsv path",
                            json_output,
                        );
                    };
                    export_chart_csv_path = Some(value.clone());
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        let parameter_overrides = parse_sql_parameters(&parameter_options.join("\n"))
            .unwrap_or_else(|error| report_cli_usage_error(&error, json_output));
        match run_automation_cli(
            case_path,
            Some(job_name),
            None,
            &parameter_overrides,
            &[],
            resume_batch_id,
            export_chart_csv_path.as_deref(),
            None,
            report_progress,
            json_output,
        ) {
            Err(error) => report_cli_error("automation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation failed; run history saved");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--validate-run") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --validate-run <case.json> <run-id> [--json]";
        let (Some(case_path), Some(run_id)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 4 || args.get(3).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let run_id = Uuid::parse_str(run_id)
            .unwrap_or_else(|_| report_cli_usage_error("invalid run UUID", json_output));
        match validate_saved_automation_configuration_cli(case_path, run_id, json_output) {
            Err(error) => report_cli_error("automation validation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation validation failed; case file was not changed");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--rerun-run") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --rerun-run <case.json> <run-id> [--export-chart-csv <output.csv|tsv>] [--json]";
        let (Some(case_path), Some(run_id)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut export_chart_csv_path = None;
        let mut seen_json = false;
        let mut index = 3;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--export-chart-csv" if export_chart_csv_path.is_none() => {
                    index += 1;
                    let Some(value) = args
                        .get(index)
                        .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                    else {
                        report_cli_usage_error(
                            "--export-chart-csv requires an output.csv or output.tsv path",
                            json_output,
                        );
                    };
                    export_chart_csv_path = Some(value.clone());
                }
                _ => report_cli_usage_error(usage, json_output),
            }
            index += 1;
        }
        let run_id = Uuid::parse_str(run_id)
            .unwrap_or_else(|_| report_cli_usage_error("invalid run UUID", json_output));
        match rerun_saved_automation_configuration_cli(
            case_path,
            run_id,
            export_chart_csv_path.as_deref(),
            json_output,
        ) {
            Err(error) => report_cli_error("automation rerun failed", error, json_output),
            Ok(false) => {
                eprintln!("automation rerun failed; run history saved");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    let desktop_case_path = match args.as_slice() {
        [] => None,
        [flag, path] if flag == "--open-case" && !path.trim().is_empty() => Some(path.clone()),
        [path] if path.to_ascii_lowercase().ends_with(".case.json") => Some(path.clone()),
        _ => report_cli_usage_error("usage: arch-notebook [--open-case <case.json>]", false),
    };
    if let Some(path) = desktop_case_path.as_deref() {
        let contents = match fs::read_to_string(path) {
            Ok(contents) => contents,
            Err(error) => report_cli_error("case open failed", error.to_string(), false),
        };
        if let Err(error) = serde_json::from_str::<CaseFile>(&contents) {
            report_cli_error("case open failed", error.to_string(), false);
        }
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1000.0, 650.0])
            .with_icon(
                eframe::icon_data::from_png_bytes(include_bytes!(
                    "../assets/icons/arch-notebook.png"
                ))
                .expect("embedded Arch Notebook icon must be valid PNG"),
            ),
        ..Default::default()
    };
    eframe::run_native(
        "Arch Notebook · Link Analysis",
        options,
        Box::new(move |_| {
            let mut app = App::default();
            if let Some(case_path) = desktop_case_path {
                app.case_path_input = case_path;
                app.load();
            }
            Ok(Box::new(app))
        }),
    )
}

#[cfg(test)]
mod tests {
    use super::{
        CaseFile, ConnectionFact, ConnectionSnapshot, Entity, EntityRepresentation, Uuid,
        canonical_connection, compare_snapshots, entity_type_layout_positions,
        grid_layout_dimensions,
    };

    fn layout_entity(name: &str, kind: &str, position_locked: bool) -> Entity {
        Entity {
            id: Uuid::new_v4(),
            name: name.into(),
            kind: kind.into(),
            representation: EntityRepresentation::Icon,
            detail: String::new(),
            attributes: Vec::new(),
            sql_attribute_values: Vec::new(),
            sql_attribute_baselines: Vec::new(),
            aliases: Vec::new(),
            external_ids: Vec::new(),
            merged_records: Vec::new(),
            x: 0.5,
            y: 0.5,
            position_locked,
            color: [55, 128, 162],
        }
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
        let positions = entity_type_layout_positions(&entities);
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
        let center = super::connection_curve_points(start, end, 0.0);
        let upper_lane = super::connection_curve_points(start, end, 12.0);
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
        let loop_points = super::connection_curve_points(loop_start, loop_start, loop_lane);
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
            let mut points = super::connection_curve_points(start, end, lanes[index]);
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
    }

    #[test]
    fn chart_fit_ignores_missing_ids_and_returns_none_for_empty_selection() {
        let mut entities = vec![layout_entity("A", "PERSON", false)];
        entities[0].id = Uuid::from_u128(1);
        entities[0].x = 0.2;
        entities[0].y = 0.8;
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
        let rect = egui::Rect::from_min_size(egui::pos2(0.0, 0.0), egui::vec2(1000.0, 700.0));
        assert!(super::chart_fit_transform(&case, &[], rect).is_none());
        let result =
            super::chart_fit_transform(&case, &[Uuid::from_u128(99), Uuid::from_u128(1)], rect)
                .expect("one selected entity exists");
        assert!(result.0.is_finite() && (0.25..=3.0).contains(&result.0));
        assert!(result.1.x.is_finite() && result.1.y.is_finite());
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
    fn navigator_projection_clamps_entities_and_clips_offscreen_viewports() {
        let entity = layout_entity("edge", "PERSON", false);
        let map = egui::Rect::from_min_size(egui::pos2(10.0, 20.0), egui::vec2(100.0, 60.0));
        let mut edge = entity;
        edge.x = 1.2;
        edge.y = -0.2;
        assert_eq!(
            super::chart_navigator_map_point(&edge, map),
            map.right_top()
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
