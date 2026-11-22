use eframe::egui;
use rusqlite::{Connection, OpenFlags, types::ValueRef};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    env, fs,
    path::{Path, PathBuf},
};
use uuid::Uuid;

#[derive(Clone, Serialize, Deserialize)]
struct Entity {
    id: Uuid,
    name: String,
    kind: String,
    detail: String,
    #[serde(default)]
    attributes: Vec<(String, String)>,
    #[serde(default)]
    aliases: Vec<String>,
    #[serde(default)]
    merged_records: Vec<MergedEntityRecord>,
    x: f32,
    y: f32,
    color: [u8; 3],
}

#[derive(Clone, Serialize, Deserialize)]
struct MergedEntityRecord {
    id: Uuid,
    name: String,
    kind: String,
    detail: String,
    attributes: Vec<(String, String)>,
}

#[derive(Clone, Serialize, Deserialize)]
struct Link {
    from: Uuid,
    to: Uuid,
    label: String,
    date: String,
    confidence: u8,
    directed: bool,
    notes: String,
    #[serde(default)]
    source_reference: String,
    #[serde(default)]
    ingest_source: Option<String>,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConnectionFact {
    key: String,
    source: String,
    source_type: String,
    target: String,
    target_type: String,
    relationship: String,
    date: String,
    confidence: u8,
    directed: bool,
    source_reference: String,
    notes: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct ConnectionSnapshot {
    series: String,
    source_key: String,
    database_path: String,
    query: String,
    captured_at: String,
    connections: Vec<ConnectionFact>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationJob {
    id: Uuid,
    name: String,
    database_path: String,
    series: String,
    query: String,
    enabled: bool,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationRun {
    job_id: Uuid,
    job_name: String,
    started_at: String,
    finished_at: String,
    success: bool,
    message: String,
}

#[derive(Clone, Serialize, Deserialize)]
struct CaseFile {
    title: String,
    #[serde(default)]
    description: String,
    entities: Vec<Entity>,
    links: Vec<Link>,
    #[serde(default)]
    snapshots: Vec<ConnectionSnapshot>,
    #[serde(default)]
    automation_jobs: Vec<AutomationJob>,
    #[serde(default)]
    automation_runs: Vec<AutomationRun>,
}

struct App {
    case: CaseFile,
    selected: Option<Uuid>,
    query: String,
    status: String,
    show_timeline: bool,
    import_path: String,
    export_path: String,
    selected_link: Option<usize>,
    page: WorkspacePage,
    entity_sort: EntitySort,
    link_sort: LinkSort,
    data_query: String,
    path_start: Option<Uuid>,
    path_end: Option<Uuid>,
    path_result: Vec<Uuid>,
    case_path: String,
    data_mode: DataMode,
    merge_candidate: Option<(Uuid, Uuid)>,
    duplicate_candidates: Vec<(Uuid, Uuid)>,
    sql_path: String,
    sql_series: String,
    sql_query: String,
    automation_job_name: String,
    editing_automation_job: Option<Uuid>,
}

#[derive(PartialEq)]
enum WorkspacePage {
    Chart,
    Data,
}

#[derive(PartialEq, Clone, Copy)]
enum DataMode {
    Entities,
    Relationships,
    SqlRegressions,
    Automation,
}

#[derive(PartialEq, Clone, Copy)]
enum EntitySort {
    Name,
    Type,
    Connections,
}

#[derive(PartialEq, Clone, Copy)]
enum LinkSort {
    Date,
    Relationship,
    Confidence,
}

#[derive(Clone, Copy, PartialEq)]
enum DeltaKind {
    Removed,
    ConfidenceDeclined,
    DateChanged,
    Updated,
    Added,
    ConfidenceIncreased,
}

struct ConnectionDelta {
    kind: DeltaKind,
    connection: ConnectionFact,
    previous_confidence: Option<u8>,
    previous_date: Option<String>,
}

impl Default for App {
    fn default() -> Self {
        let data_dir = app_data_dir();
        let status = match initialize_user_data(&data_dir) {
            Ok(()) => "CASE READY".into(),
            Err(error) => format!("USER DATA SETUP FAILED · {error}"),
        };
        Self {
            case: demo_case(),
            selected: None,
            query: String::new(),
            status,
            show_timeline: false,
            import_path: data_dir.join("sample_relationships.csv").to_string_lossy().into_owned(),
            export_path: data_dir.join("chart-export.csv").to_string_lossy().into_owned(),
            selected_link: None,
            page: WorkspacePage::Chart,
            entity_sort: EntitySort::Name,
            link_sort: LinkSort::Date,
            data_query: String::new(),
            path_start: None,
            path_end: None,
            path_result: Vec::new(),
            case_path: data_dir.join("harbor-light.case.json").to_string_lossy().into_owned(),
            data_mode: DataMode::Relationships,
            merge_candidate: None,
            duplicate_candidates: Vec::new(),
            sql_path: data_dir.join("sample_connections.sqlite").to_string_lossy().into_owned(),
            sql_series: "Harbor Light connection review".into(),
            sql_query: "SELECT connection_id, source, source_type, target, target_type, relationship, event_date AS date, confidence, directed, source_reference, notes FROM connection_observations WHERE snapshot_id = 'baseline' ORDER BY connection_id;".into(),
            automation_job_name: String::new(),
            editing_automation_job: None,
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

fn initialize_user_data(data_dir: &Path) -> std::io::Result<()> {
    fs::create_dir_all(data_dir)?;
    let samples: [(&str, &[u8]); 2] = [
        (
            "sample_connections.sqlite",
            include_bytes!("../sample_connections.sqlite"),
        ),
        (
            "sample_relationships.csv",
            include_bytes!("../sample_relationships.csv"),
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
        detail: detail.into(),
        attributes: Vec::new(),
        aliases: Vec::new(),
        merged_records: Vec::new(),
        x,
        y,
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
            source_reference: "REG-1193 · Corporate registry".into(),
            ingest_source: None,
        },
        Link {
            from: id(1),
            to: id(2),
            label: "LEASES".into(),
            date: "2025-01-28".into(),
            confidence: 2,
            directed: false,
            notes: "Property record".into(),
            source_reference: "LOC-81 · Property record".into(),
            ingest_source: None,
        },
        Link {
            from: id(0),
            to: id(3),
            label: "USES NUMBER".into(),
            date: "2025-02-11".into(),
            confidence: 3,
            directed: false,
            notes: String::new(),
            source_reference: "SIG-14 · Communications report".into(),
            ingest_source: None,
        },
        Link {
            from: id(3),
            to: id(4),
            label: "CONTACTED".into(),
            date: "2025-02-14".into(),
            confidence: 1,
            directed: true,
            notes: "Source report; uncorroborated".into(),
            source_reference: "HUM-203 · Uncorroborated report".into(),
            ingest_source: None,
        },
        Link {
            from: id(4),
            to: id(2),
            label: "DELIVERS TO".into(),
            date: "2025-02-14".into(),
            confidence: 2,
            directed: true,
            notes: String::new(),
            source_reference: "LOG-77 · Delivery log".into(),
            ingest_source: None,
        },
    ];
    CaseFile {
        title: "HARBOR LIGHT // SAMPLE CASE".into(),
        description: "Fictional sample relationship investigation".into(),
        entities,
        links,
        snapshots: Vec::new(),
        automation_jobs: Vec::new(),
        automation_runs: Vec::new(),
    }
}

impl App {
    fn save(&mut self) {
        let path = self.case_path.trim();
        let result = if path.is_empty() {
            Err("case path cannot be empty".to_string())
        } else {
            serde_json::to_string_pretty(&self.case)
                .map_err(|e| e.to_string())
                .and_then(|s| fs::write(path, s).map_err(|e| e.to_string()))
        };
        match result {
            Ok(()) => self.status = format!("CASE SAVED · {}", self.case_path),
            Err(e) => self.status = format!("SAVE FAILED · {e}"),
        }
    }
    fn load(&mut self) {
        if self.case_path.trim().is_empty() {
            self.status = "LOAD FAILED · case path cannot be empty".into();
            return;
        }
        match fs::read_to_string(self.case_path.trim())
            .map_err(|e| e.to_string())
            .and_then(|s| serde_json::from_str::<CaseFile>(&s).map_err(|e| e.to_string()))
        {
            Ok(c) => {
                self.case = c;
                self.selected = None;
                self.selected_link = None;
                self.path_result.clear();
                self.duplicate_candidates.clear();
                self.merge_candidate = None;
                self.status = "CASE LOADED".into();
            }
            Err(e) => self.status = format!("LOAD FAILED · {e}"),
        }
    }

    fn import_csv(&mut self) {
        let import_path = self.import_path.clone();
        let result = (|| -> Result<(usize, Vec<Entity>, Vec<Link>), String> {
            let mut reader = csv::Reader::from_path(import_path).map_err(|e| e.to_string())?;
            let headers = reader.headers().map_err(|e| e.to_string())?.clone();
            let col = |names: &[&str]| {
                headers
                    .iter()
                    .position(|h| names.iter().any(|n| h.eq_ignore_ascii_case(n)))
            };
            let source = col(&["source", "from", "entity_a"]).ok_or("CSV needs a source column")?;
            let target = col(&["target", "to", "entity_b"]).ok_or("CSV needs a target column")?;
            let rel = col(&["relationship", "link", "type"]);
            let date = col(&["date", "datetime"]);
            let confidence = col(&["confidence"]);
            let source_type = col(&["source_type", "from_type", "entity_a_type"]);
            let target_type = col(&["target_type", "to_type", "entity_b_type"]);
            let direction = col(&["directed", "direction"]);
            let notes = col(&["notes", "description"]);
            let source_reference = col(&["source_reference", "evidence", "reference", "source_id"]);
            let mut created = 0;
            let mut entities = self.case.entities.clone();
            let mut links = self.case.links.clone();
            for row in reader.records() {
                let row = row.map_err(|e| e.to_string())?;
                let source_name = row.get(source).unwrap_or("").trim();
                let target_name = row.get(target).unwrap_or("").trim();
                if source_name.is_empty() || target_name.is_empty() {
                    continue;
                }
                let find_or_add = |name: &str, kind: &str, entities: &mut Vec<Entity>| {
                    if let Some(e) = entities.iter().find(|e| e.name.eq_ignore_ascii_case(name)) {
                        return e.id;
                    }
                    let id = Uuid::new_v4();
                    let n = entities.len();
                    entities.push(Entity {
                        id,
                        name: name.into(),
                        kind: if kind.is_empty() { "ENTITY" } else { kind }.into(),
                        detail: String::new(),
                        attributes: Vec::new(),
                        aliases: Vec::new(),
                        merged_records: Vec::new(),
                        x: 0.12 + (n % 4) as f32 * 0.23,
                        y: 0.15 + (n / 4) as f32 * 0.2,
                        color: [65, 125, 153],
                    });
                    id
                };
                let a = find_or_add(
                    source_name,
                    source_type.and_then(|i| row.get(i)).unwrap_or(""),
                    &mut entities,
                );
                let b = find_or_add(
                    target_name,
                    target_type.and_then(|i| row.get(i)).unwrap_or(""),
                    &mut entities,
                );
                let confidence = confidence
                    .and_then(|i| row.get(i))
                    .and_then(|v| v.parse::<u8>().ok())
                    .unwrap_or(2)
                    .clamp(1, 3);
                links.push(Link {
                    from: a,
                    to: b,
                    label: rel
                        .and_then(|i| row.get(i))
                        .filter(|s| !s.is_empty())
                        .unwrap_or("ASSOCIATED WITH")
                        .into(),
                    date: date.and_then(|i| row.get(i)).unwrap_or("").into(),
                    confidence,
                    directed: !matches!(
                        direction
                            .and_then(|i| row.get(i))
                            .unwrap_or("true")
                            .to_ascii_lowercase()
                            .as_str(),
                        "false" | "no" | "undirected" | "↔"
                    ),
                    notes: notes.and_then(|i| row.get(i)).unwrap_or("").into(),
                    source_reference: source_reference
                        .and_then(|i| row.get(i))
                        .unwrap_or("")
                        .into(),
                    ingest_source: None,
                });
                created += 1;
            }
            Ok((created, entities, links))
        })();
        self.status = match result {
            Ok((n, entities, links)) => {
                self.case.entities = entities;
                self.case.links = links;
                self.path_result.clear();
                self.duplicate_candidates.clear();
                self.merge_candidate = None;
                format!("IMPORTED {n} RELATIONSHIPS")
            }
            Err(e) => format!("IMPORT FAILED · {e}"),
        };
    }

    fn ingest_sqlite_snapshot(&mut self) {
        let path = self.sql_path.trim().to_string();
        let series = self.sql_series.trim().to_string();
        let query = self.sql_query.trim().to_string();
        let result = (|| -> Result<(Vec<Entity>, Vec<Link>, ConnectionSnapshot), String> {
            if path.is_empty() {
                return Err("database path cannot be empty".into());
            }
            if series.is_empty() {
                return Err("snapshot series cannot be empty".into());
            }
            if query.is_empty() {
                return Err("SQL query cannot be empty".into());
            }
            // Enforce read-only access at the SQLite open boundary for every configured job.
            let connection = Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                .map_err(|e| e.to_string())?;
            let captured_at: String = connection
                .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |row| {
                    row.get(0)
                })
                .map_err(|e| e.to_string())?;
            let mut statement = connection.prepare(&query).map_err(|e| e.to_string())?;
            let column_names: Vec<String> = statement
                .column_names()
                .map(|name| name.to_ascii_lowercase())
                .collect();
            let col = |names: &[&str]| {
                column_names
                    .iter()
                    .position(|column| names.iter().any(|name| column == name))
            };
            let source =
                col(&["source", "from", "entity_a"]).ok_or("SQL result needs a source column")?;
            let target =
                col(&["target", "to", "entity_b"]).ok_or("SQL result needs a target column")?;
            let connection_id = col(&["connection_id", "link_id", "record_id", "id"]);
            let source_type = col(&["source_type", "from_type", "entity_a_type"]);
            let target_type = col(&["target_type", "to_type", "entity_b_type"]);
            let relationship = col(&["relationship", "link_type", "relation"]);
            let date = col(&["date", "event_date", "datetime", "timestamp"]);
            let confidence = col(&["confidence", "strength"]);
            let directed = col(&["directed", "is_directed", "direction"]);
            let source_reference = col(&["source_reference", "evidence", "reference", "source_id"]);
            let notes = col(&["notes", "description"]);
            let mut facts = Vec::new();
            let mut occurrences: HashMap<String, usize> = HashMap::new();
            let mut rows = statement.query([]).map_err(|e| e.to_string())?;
            while let Some(row) = rows.next().map_err(|e| e.to_string())? {
                let source_name = sql_value(row, Some(source))
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
                let target_name = sql_value(row, Some(target))
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
                if source_name.is_empty() || target_name.is_empty() {
                    continue;
                }
                let source_kind = sql_value(row, source_type).map_err(|e| e.to_string())?;
                let target_kind = sql_value(row, target_type).map_err(|e| e.to_string())?;
                let relationship = sql_value(row, relationship).map_err(|e| e.to_string())?;
                let date = sql_value(row, date).map_err(|e| e.to_string())?;
                let confidence_value = sql_value(row, confidence).map_err(|e| e.to_string())?;
                let confidence = confidence_value.parse::<u8>().unwrap_or(2).clamp(1, 3);
                let directed_text = sql_value(row, directed)
                    .map_err(|e| e.to_string())?
                    .to_ascii_lowercase();
                let is_directed = !matches!(
                    directed_text.as_str(),
                    "false" | "0" | "no" | "undirected" | "↔"
                );
                let source_reference =
                    sql_value(row, source_reference).map_err(|e| e.to_string())?;
                let notes = sql_value(row, notes).map_err(|e| e.to_string())?;
                let explicit_id = sql_value(row, connection_id).map_err(|e| e.to_string())?;
                let natural =
                    if !is_directed && source_name.to_lowercase() > target_name.to_lowercase() {
                        format!(
                            "{}|{}|{}|{}|{}|0",
                            target_name, target_kind, relationship, source_name, source_kind
                        )
                    } else {
                        format!(
                            "{}|{}|{}|{}|{}|{}",
                            source_name,
                            source_kind,
                            relationship,
                            target_name,
                            target_kind,
                            is_directed
                        )
                    };
                let identity = if explicit_id.trim().is_empty() {
                    natural
                } else {
                    format!("id:{}", explicit_id.trim())
                };
                facts.push(ConnectionFact {
                    key: identity,
                    source: source_name,
                    source_type: source_kind,
                    target: target_name,
                    target_type: target_kind,
                    relationship: if relationship.is_empty() {
                        "ASSOCIATED WITH".into()
                    } else {
                        relationship
                    },
                    date,
                    confidence,
                    directed: is_directed,
                    source_reference,
                    notes,
                });
            }
            // Stable ordering gives duplicate natural keys repeatable occurrence suffixes.
            facts.sort_by(|a, b| {
                a.key
                    .cmp(&b.key)
                    .then_with(|| a.source.cmp(&b.source))
                    .then_with(|| a.target.cmp(&b.target))
                    .then_with(|| a.date.cmp(&b.date))
                    .then_with(|| a.source_reference.cmp(&b.source_reference))
            });
            for fact in &mut facts {
                if fact.key.starts_with("id:") {
                    continue;
                }
                let occurrence = occurrences.entry(fact.key.clone()).or_insert(0);
                *occurrence += 1;
                fact.key = format!("{}#{}", fact.key, *occurrence);
            }
            // Stage chart updates separately and swap them in only after the entire query succeeds.
            let source_key = format!("{}::{}", path, series);
            let mut entities = self.case.entities.clone();
            let mut links = self.case.links.clone();
            links.retain(|link| link.ingest_source.as_deref() != Some(source_key.as_str()));
            let add_entity = |name: &str, kind: &str, entities: &mut Vec<Entity>| {
                if let Some(entity) = entities
                    .iter_mut()
                    .find(|entity| entity.name.eq_ignore_ascii_case(name))
                {
                    if (entity.kind.is_empty() || entity.kind == "ENTITY") && !kind.is_empty() {
                        entity.kind = kind.into();
                    }
                    return entity.id;
                }
                let id = Uuid::new_v4();
                let n = entities.len();
                entities.push(Entity {
                    id,
                    name: name.into(),
                    kind: if kind.is_empty() { "ENTITY" } else { kind }.into(),
                    detail: String::new(),
                    attributes: Vec::new(),
                    aliases: Vec::new(),
                    merged_records: Vec::new(),
                    x: 0.12 + (n % 4) as f32 * 0.23,
                    y: 0.15 + (n / 4) as f32 * 0.2,
                    color: [65, 125, 153],
                });
                id
            };
            for fact in &facts {
                let from = add_entity(&fact.source, &fact.source_type, &mut entities);
                let to = add_entity(&fact.target, &fact.target_type, &mut entities);
                links.push(Link {
                    from,
                    to,
                    label: fact.relationship.clone(),
                    date: fact.date.clone(),
                    confidence: fact.confidence,
                    directed: fact.directed,
                    notes: fact.notes.clone(),
                    source_reference: fact.source_reference.clone(),
                    ingest_source: Some(source_key.clone()),
                });
            }
            let snapshot = ConnectionSnapshot {
                series,
                source_key,
                database_path: path,
                query,
                captured_at,
                connections: facts,
            };
            Ok((entities, links, snapshot))
        })();
        match result {
            Ok((entities, links, snapshot)) => {
                let count = snapshot.connections.len();
                self.case.entities = entities;
                self.case.links = links;
                self.case.snapshots.push(snapshot);
                self.path_result.clear();
                self.status = format!("SQL SNAPSHOT INGESTED · {count} CONNECTIONS");
            }
            Err(error) => self.status = format!("SQL INGEST FAILED · {error}"),
        }
    }

    fn save_automation_job(&mut self) {
        let name = self.automation_job_name.trim().to_string();
        if name.is_empty()
            || self.sql_path.trim().is_empty()
            || self.sql_series.trim().is_empty()
            || self.sql_query.trim().is_empty()
        {
            self.status = "JOB NOT SAVED · name, database, series, and query are required".into();
            return;
        }
        if self.case.automation_jobs.iter().any(|job| {
            job.name.eq_ignore_ascii_case(&name) && Some(job.id) != self.editing_automation_job
        }) {
            self.status = "JOB NOT SAVED · job names must be unique within the case".into();
            return;
        }
        if let Some(id) = self.editing_automation_job {
            if let Some(job) = self
                .case
                .automation_jobs
                .iter_mut()
                .find(|job| job.id == id)
            {
                job.name = name.clone();
                job.database_path = self.sql_path.trim().into();
                job.series = self.sql_series.trim().into();
                job.query = self.sql_query.trim().into();
                self.status = format!("AUTOMATION JOB UPDATED · {name}");
                return;
            }
        }
        self.case.automation_jobs.push(AutomationJob {
            id: Uuid::new_v4(),
            name: name.clone(),
            database_path: self.sql_path.trim().into(),
            series: self.sql_series.trim().into(),
            query: self.sql_query.trim().into(),
            enabled: true,
        });
        self.editing_automation_job = None;
        self.status = format!("AUTOMATION JOB ADDED · {name}");
    }

    fn edit_automation_job(&mut self, id: Uuid) {
        if let Some(job) = self
            .case
            .automation_jobs
            .iter()
            .find(|job| job.id == id)
            .cloned()
        {
            self.automation_job_name = job.name;
            self.sql_path = job.database_path;
            self.sql_series = job.series;
            self.sql_query = job.query;
            self.editing_automation_job = Some(id);
            self.status = "EDITING AUTOMATION JOB".into();
        }
    }

    fn run_automation_job(&mut self, id: Uuid) {
        let Some(job) = self
            .case
            .automation_jobs
            .iter()
            .find(|job| job.id == id)
            .cloned()
        else {
            self.status = "JOB RUN FAILED · job not found".into();
            return;
        };
        let started_at = epoch_timestamp();
        self.sql_path = job.database_path.clone();
        self.sql_series = job.series.clone();
        self.sql_query = job.query.clone();
        self.ingest_sqlite_snapshot();
        let message = self.status.clone();
        let success = message.starts_with("SQL SNAPSHOT INGESTED");
        self.case.automation_runs.push(AutomationRun {
            job_id: job.id,
            job_name: job.name,
            started_at,
            finished_at: epoch_timestamp(),
            success,
            message: message.clone(),
        });
        if self.case.automation_runs.len() > 200 {
            // Keep case files bounded even when a scheduled job runs for long periods.
            self.case
                .automation_runs
                .drain(..self.case.automation_runs.len() - 200);
        }
    }

    fn run_enabled_automation_jobs(&mut self) {
        let jobs: Vec<Uuid> = self
            .case
            .automation_jobs
            .iter()
            .filter(|job| job.enabled)
            .map(|job| job.id)
            .collect();
        if jobs.is_empty() {
            self.status = "NO ENABLED AUTOMATION JOBS".into();
            return;
        }
        let count = jobs.len();
        for id in jobs {
            self.run_automation_job(id);
        }
        self.status = format!("AUTOMATION RUN COMPLETE · {count} JOBS · SEE RUN HISTORY");
    }

    fn export_csv(&mut self) {
        let result = (|| -> Result<(), String> {
            let mut writer =
                csv::Writer::from_path(&self.export_path).map_err(|e| e.to_string())?;
            writer
                .write_record([
                    "source",
                    "source_type",
                    "relationship",
                    "target",
                    "target_type",
                    "date",
                    "confidence",
                    "directed",
                    "notes",
                    "source_reference",
                ])
                .map_err(|e| e.to_string())?;
            for link in &self.case.links {
                let a = self.case.entities.iter().find(|e| e.id == link.from);
                let b = self.case.entities.iter().find(|e| e.id == link.to);
                if let (Some(a), Some(b)) = (a, b) {
                    let conf = link.confidence.to_string();
                    writer
                        .write_record([
                            a.name.as_str(),
                            a.kind.as_str(),
                            link.label.as_str(),
                            b.name.as_str(),
                            b.kind.as_str(),
                            link.date.as_str(),
                            conf.as_str(),
                            if link.directed { "true" } else { "false" },
                            link.notes.as_str(),
                            link.source_reference.as_str(),
                        ])
                        .map_err(|e| e.to_string())?;
                }
            }
            writer.flush().map_err(|e| e.to_string())
        })();
        self.status = match result {
            Ok(()) => format!("EXPORTED {}", self.export_path),
            Err(e) => format!("EXPORT FAILED · {e}"),
        };
    }

    fn auto_layout(&mut self) {
        let n = self.case.entities.len();
        if n == 0 {
            return;
        }
        let degrees: Vec<usize> = self
            .case
            .entities
            .iter()
            .map(|e| {
                self.case
                    .links
                    .iter()
                    .filter(|l| l.from == e.id || l.to == e.id)
                    .count()
            })
            .collect();
        let mut order: Vec<usize> = (0..n).collect();
        order.sort_by_key(|i| std::cmp::Reverse(degrees[*i]));
        for (rank, index) in order.into_iter().enumerate() {
            let angle =
                std::f32::consts::TAU * rank as f32 / n as f32 - std::f32::consts::FRAC_PI_2;
            self.case.entities[index].x = 0.5 + angle.cos() * 0.36;
            self.case.entities[index].y = 0.5 + angle.sin() * 0.36;
        }
        self.status = "LAYOUT ARRANGED BY CONNECTIVITY".into();
    }

    fn delete_selected(&mut self) {
        if let Some(id) = self.selected.take() {
            self.case.entities.retain(|e| e.id != id);
            self.case.links.retain(|l| l.from != id && l.to != id);
            self.selected_link = None;
            self.path_result.clear();
            self.status = "ENTITY AND ITS RELATIONSHIPS REMOVED".into();
        } else if let Some(i) = self.selected_link.take() {
            if i < self.case.links.len() {
                self.case.links.remove(i);
            }
            self.path_result.clear();
            self.status = "RELATIONSHIP REMOVED".into();
        }
    }

    fn new_case(&mut self) {
        self.case = CaseFile {
            title: "UNTITLED CASE".into(),
            description: String::new(),
            entities: Vec::new(),
            links: Vec::new(),
            snapshots: Vec::new(),
            automation_jobs: Vec::new(),
            automation_runs: Vec::new(),
        };
        self.case_path = app_data_dir()
            .join("untitled.case.json")
            .to_string_lossy()
            .into_owned();
        self.selected = None;
        self.selected_link = None;
        self.path_start = None;
        self.path_end = None;
        self.path_result.clear();
        self.duplicate_candidates.clear();
        self.merge_candidate = None;
        self.status = "NEW CASE CREATED".into();
    }

    fn analyze_path(&mut self) {
        let Some(start) = self.path_start else {
            self.status = "CHOOSE A START ENTITY".into();
            return;
        };
        let Some(end) = self.path_end else {
            self.status = "CHOOSE AN END ENTITY".into();
            return;
        };
        self.path_result = shortest_path(&self.case, start, end).unwrap_or_default();
        if self.path_result.is_empty() {
            self.status = "NO CONNECTION PATH FOUND".into();
        } else {
            self.status = format!("PATH FOUND · {} ENTITIES", self.path_result.len());
        }
    }

    fn find_duplicates(&mut self) {
        self.duplicate_candidates = find_duplicate_pairs(&self.case.entities);
        self.merge_candidate = None;
        self.status = if self.duplicate_candidates.is_empty() {
            "NO LIKELY DUPLICATES FOUND".into()
        } else {
            format!("{} POSSIBLE MATCHES FOUND", self.duplicate_candidates.len())
        };
    }

    fn merge_candidate_into_first(&mut self) {
        let Some((keep_id, remove_id)) = self.merge_candidate else {
            return;
        };
        let Some(remove_index) = self.case.entities.iter().position(|e| e.id == remove_id) else {
            self.merge_candidate = None;
            return;
        };
        let removed = self.case.entities[remove_index].clone();
        let Some(keep_index) = self.case.entities.iter().position(|e| e.id == keep_id) else {
            self.merge_candidate = None;
            return;
        };
        let distance = edit_distance(
            &normalized_name(&self.case.entities[keep_index].name),
            &normalized_name(&removed.name),
        );
        let max_len = normalized_name(&self.case.entities[keep_index].name)
            .len()
            .max(normalized_name(&removed.name).len());
        if distance > if max_len < 8 { 1 } else { 2 } {
            self.merge_candidate = None;
            self.status = "MATCH CHANGED · FIND LIKELY MATCHES AGAIN".into();
            return;
        }
        let keep = &mut self.case.entities[keep_index];
        if !keep
            .aliases
            .iter()
            .any(|a| a.eq_ignore_ascii_case(&removed.name))
            && !keep.name.eq_ignore_ascii_case(&removed.name)
        {
            keep.aliases.push(removed.name.clone());
        }
        for alias in &removed.aliases {
            if !keep.name.eq_ignore_ascii_case(alias)
                && !keep.aliases.iter().any(|a| a.eq_ignore_ascii_case(alias))
            {
                keep.aliases.push(alias.clone());
            }
        }
        keep.merged_records.extend(removed.merged_records.clone());
        keep.merged_records.push(MergedEntityRecord {
            id: removed.id,
            name: removed.name.clone(),
            kind: removed.kind.clone(),
            detail: removed.detail.clone(),
            attributes: removed.attributes.clone(),
        });
        for (key, value) in &removed.attributes {
            if let Some((_, existing)) = keep
                .attributes
                .iter_mut()
                .find(|(k, _)| k.eq_ignore_ascii_case(key))
            {
                if !value.is_empty() && !existing.split("; ").any(|v| v == value) {
                    if !existing.is_empty() {
                        existing.push_str("; ");
                    }
                    existing.push_str(value);
                }
            } else {
                keep.attributes.push((key.clone(), value.clone()));
            }
        }
        if !removed.detail.trim().is_empty() && !keep.detail.contains(removed.detail.trim()) {
            if !keep.detail.is_empty() {
                keep.detail.push('\n');
            }
            keep.detail.push_str(&format!(
                "Merged record ({}): {}",
                removed.name, removed.detail
            ));
        }
        for link in &mut self.case.links {
            if link.from == remove_id {
                link.from = keep_id;
            }
            if link.to == remove_id {
                link.to = keep_id;
            }
        }
        self.case.entities.remove(remove_index);
        self.path_start = if self.path_start == Some(remove_id) {
            Some(keep_id)
        } else {
            self.path_start
        };
        self.path_end = if self.path_end == Some(remove_id) {
            Some(keep_id)
        } else {
            self.path_end
        };
        let mut path = Vec::new();
        for id in &self.path_result {
            let merged = if *id == remove_id { keep_id } else { *id };
            if path.last() != Some(&merged) {
                path.push(merged);
            }
        }
        self.path_result = path;
        self.selected = Some(keep_id);
        self.selected_link = None;
        self.merge_candidate = None;
        self.duplicate_candidates.clear();
        self.status = format!(
            "MERGED {} INTO {} · ALIASES AND ATTRIBUTES PRESERVED",
            removed.name,
            self.case
                .entities
                .iter()
                .find(|e| e.id == keep_id)
                .map(|e| e.name.as_str())
                .unwrap_or("entity")
        );
    }
}

fn normalized_name(name: &str) -> String {
    name.chars()
        .filter(|c| c.is_alphanumeric())
        .flat_map(char::to_lowercase)
        .collect()
}

fn edit_distance(a: &str, b: &str) -> usize {
    let b: Vec<char> = b.chars().collect();
    let mut previous: Vec<usize> = (0..=b.len()).collect();
    for (i, ca) in a.chars().enumerate() {
        let mut current = Vec::with_capacity(b.len() + 1);
        current.push(i + 1);
        for (j, cb) in b.iter().enumerate() {
            current.push(
                (current[j] + 1)
                    .min(previous[j + 1] + 1)
                    .min(previous[j] + usize::from(ca != *cb)),
            );
        }
        previous = current;
    }
    previous[b.len()]
}

fn find_duplicate_pairs(entities: &[Entity]) -> Vec<(Uuid, Uuid)> {
    let mut matches = Vec::new();
    for i in 0..entities.len() {
        let a = normalized_name(&entities[i].name);
        if a.len() < 3 {
            continue;
        }
        for b_entity in entities.iter().skip(i + 1) {
            let b = normalized_name(&b_entity.name);
            if b.len() < 3 || a.chars().next() != b.chars().next() || a.len().abs_diff(b.len()) > 2
            {
                continue;
            }
            let distance = edit_distance(&a, &b);
            let threshold = if a.len().max(b.len()) < 8 { 1 } else { 2 };
            if distance <= threshold {
                matches.push((
                    distance,
                    entities[i].name.clone(),
                    b_entity.name.clone(),
                    entities[i].id,
                    b_entity.id,
                ));
            }
        }
    }
    matches.sort_by(|a, b| {
        a.0.cmp(&b.0)
            .then_with(|| a.1.cmp(&b.1))
            .then_with(|| a.2.cmp(&b.2))
    });
    matches
        .into_iter()
        .take(20)
        .map(|(_, _, _, a, b)| (a, b))
        .collect()
}

fn sql_value(row: &rusqlite::Row<'_>, index: Option<usize>) -> rusqlite::Result<String> {
    let Some(index) = index else {
        return Ok(String::new());
    };
    Ok(match row.get_ref(index)? {
        ValueRef::Null => String::new(),
        ValueRef::Integer(value) => value.to_string(),
        ValueRef::Real(value) => value.to_string(),
        ValueRef::Text(value) => String::from_utf8_lossy(value).into_owned(),
        ValueRef::Blob(value) => value.iter().map(|byte| format!("{byte:02x}")).collect(),
    })
}

fn epoch_timestamp() -> String {
    let seconds = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| duration.as_secs())
        .unwrap_or_default();
    let days = (seconds / 86_400) as i64;
    let day_seconds = seconds % 86_400;
    let shifted = days + 719_468;
    let era = shifted / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let mut year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_prime = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_prime + 2) / 5 + 1;
    let month = month_prime + if month_prime < 10 { 3 } else { -9 };
    year += i64::from(month <= 2);
    format!(
        "{year:04}-{month:02}-{day:02}T{:02}:{:02}:{:02}Z",
        day_seconds / 3_600,
        day_seconds / 60 % 60,
        day_seconds % 60
    )
}

fn compare_snapshots(
    baseline: &ConnectionSnapshot,
    current: &ConnectionSnapshot,
) -> Vec<ConnectionDelta> {
    let old: HashMap<&str, &ConnectionFact> = baseline
        .connections
        .iter()
        .map(|connection| (connection.key.as_str(), connection))
        .collect();
    let new: HashMap<&str, &ConnectionFact> = current
        .connections
        .iter()
        .map(|connection| (connection.key.as_str(), connection))
        .collect();
    let mut deltas = Vec::new();
    for (key, previous) in &old {
        match new.get(key) {
            None => deltas.push(ConnectionDelta {
                kind: DeltaKind::Removed,
                connection: (*previous).clone(),
                previous_confidence: Some(previous.confidence),
                previous_date: Some(previous.date.clone()),
            }),
            Some(now) => {
                let mut changed = false;
                if now.confidence < previous.confidence {
                    deltas.push(ConnectionDelta {
                        kind: DeltaKind::ConfidenceDeclined,
                        connection: (*now).clone(),
                        previous_confidence: Some(previous.confidence),
                        previous_date: Some(previous.date.clone()),
                    });
                    changed = true;
                } else if now.confidence > previous.confidence {
                    deltas.push(ConnectionDelta {
                        kind: DeltaKind::ConfidenceIncreased,
                        connection: (*now).clone(),
                        previous_confidence: Some(previous.confidence),
                        previous_date: Some(previous.date.clone()),
                    });
                    changed = true;
                }
                if now.date != previous.date {
                    deltas.push(ConnectionDelta {
                        kind: DeltaKind::DateChanged,
                        connection: (*now).clone(),
                        previous_confidence: Some(previous.confidence),
                        previous_date: Some(previous.date.clone()),
                    });
                    changed = true;
                }
                if !changed
                    && (now.source != previous.source
                        || now.source_type != previous.source_type
                        || now.target != previous.target
                        || now.target_type != previous.target_type
                        || now.relationship != previous.relationship
                        || now.directed != previous.directed
                        || now.source_reference != previous.source_reference
                        || now.notes != previous.notes)
                {
                    deltas.push(ConnectionDelta {
                        kind: DeltaKind::Updated,
                        connection: (*now).clone(),
                        previous_confidence: Some(previous.confidence),
                        previous_date: Some(previous.date.clone()),
                    });
                }
            }
        }
    }
    for (key, connection) in &new {
        if !old.contains_key(key) {
            deltas.push(ConnectionDelta {
                kind: DeltaKind::Added,
                connection: (*connection).clone(),
                previous_confidence: None,
                previous_date: None,
            });
        }
    }
    deltas.sort_by(|a, b| {
        delta_order(a.kind)
            .cmp(&delta_order(b.kind))
            .then_with(|| a.connection.source.cmp(&b.connection.source))
            .then_with(|| a.connection.target.cmp(&b.connection.target))
    });
    deltas
}

fn delta_order(kind: DeltaKind) -> u8 {
    match kind {
        DeltaKind::Removed => 0,
        DeltaKind::ConfidenceDeclined => 1,
        DeltaKind::DateChanged => 2,
        DeltaKind::Updated => 3,
        DeltaKind::Added => 4,
        DeltaKind::ConfidenceIncreased => 5,
    }
}

fn shortest_path(case: &CaseFile, start: Uuid, goal: Uuid) -> Option<Vec<Uuid>> {
    if !case.entities.iter().any(|entity| entity.id == start)
        || !case.entities.iter().any(|entity| entity.id == goal)
    {
        return None;
    }
    let mut queue = VecDeque::from([start]);
    let mut previous: HashMap<Uuid, Uuid> = HashMap::new();
    let mut seen = std::collections::HashSet::from([start]);
    while let Some(current) = queue.pop_front() {
        if current == goal {
            break;
        }
        for link in &case.links {
            let next = if link.from == current {
                Some(link.to)
            } else if !link.directed && link.to == current {
                Some(link.from)
            } else {
                None
            };
            if let Some(next) = next {
                if seen.insert(next) {
                    previous.insert(next, current);
                    queue.push_back(next);
                }
            }
        }
    }
    if !seen.contains(&goal) {
        return None;
    }
    let mut path = vec![goal];
    let mut current = goal;
    while current != start {
        current = *previous.get(&current)?;
        path.push(current);
    }
    path.reverse();
    Some(path)
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _: &mut eframe::Frame) {
        let bg = egui::Color32::from_rgb(12, 19, 27);
        let panel = egui::Color32::from_rgb(18, 28, 39);
        let muted = egui::Color32::from_rgb(125, 148, 166);
        let cyan = egui::Color32::from_rgb(73, 211, 207);
        egui::CentralPanel::default().frame(egui::Frame::new().fill(bg).inner_margin(0)).show(ctx, |ui| {
            ui.vertical(|ui| {
                ui.set_min_size(ui.available_size());
                egui::Frame::new().fill(panel).inner_margin(egui::Margin::symmetric(22, 13)).show(ui, |ui| {
                    ui.horizontal(|ui| {
                        ui.label(egui::RichText::new("◈  ARCH NOTEBOOK").size(17.0).strong().color(cyan));
                        ui.separator(); ui.add(egui::TextEdit::singleline(&mut self.case.title).desired_width(220.0));
                        ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                            if ui.button("Save case").clicked() { self.save(); }
                            if ui.button("Open case").clicked() { self.load(); }
                            if ui.button("New case").clicked() { self.new_case(); }
                            ui.label(egui::RichText::new("LOCAL WORKSPACE  ·  UNCLASSIFIED").small().color(muted));
                        });
                    });
                });
                ui.add_space(1.0);
                ui.horizontal(|ui| {
                    for (label, active) in [("CHART",self.page==WorkspacePage::Chart && !self.show_timeline),("TIMELINE",self.show_timeline),("DATA",self.page==WorkspacePage::Data)] {
                        let response=ui.selectable_label(active,label);
                        if response.clicked() && label=="TIMELINE" { self.page=WorkspacePage::Chart;self.show_timeline=!self.show_timeline; }
                        if response.clicked() && label=="CHART" { self.page=WorkspacePage::Chart;self.show_timeline=false; }
                        if response.clicked() && label=="DATA" { self.page=WorkspacePage::Data;self.show_timeline=false; }
                    }
                    ui.separator();
                    if ui.button("＋ Entity").clicked() {
                        let item=Entity { id:Uuid::new_v4(),name:"New entity".into(),kind:"PERSON".into(),detail:"Add an analyst note".into(),attributes:Vec::new(),aliases:Vec::new(),merged_records:Vec::new(),x:0.5,y:0.5,color:[55,128,162] };
                        self.selected=Some(item.id); self.selected_link=None; self.case.entities.push(item);self.duplicate_candidates.clear();
                    }
                    if ui.button("＋ Link").clicked() && self.case.entities.len()>1 {
                        let a=self.selected.unwrap_or(self.case.entities[0].id);
                        let b=self.case.entities.iter().find(|e|e.id!=a).map(|e|e.id).unwrap();
                        self.case.links.push(Link{from:a,to:b,label:"ASSOCIATED WITH".into(),date:"2025-02-15".into(),confidence:2,directed:true,notes:String::new(),source_reference:String::new(),ingest_source:None});
                        self.selected=None; self.selected_link=Some(self.case.links.len()-1);
                        self.duplicate_candidates.clear();
                    }
                    if ui.button("Import CSV").clicked(){self.import_csv();}
                    if ui.button("Export CSV").clicked(){self.export_csv();}
                    if ui.button("Auto layout").clicked(){self.auto_layout();}
                    if self.selected.is_some() || self.selected_link.is_some() {if ui.button("Delete selected").clicked(){self.delete_selected();}}
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| { ui.label(egui::RichText::new(&self.status).small().color(muted)); });
                }).response;
                ui.separator();
                let avail=ui.available_size();
                ui.horizontal(|ui| {
                    let left=190.0; let right=260.0;
                    ui.allocate_ui_with_layout(egui::vec2(left,avail.y),egui::Layout::top_down(egui::Align::Min),|ui| {
                    egui::Frame::new().fill(panel).inner_margin(12).show(ui,|ui| {
                        ui.set_min_width(left); ui.set_max_width(left);
                        ui.label(egui::RichText::new("CASE FILE").size(10.0).strong().color(muted));
                        ui.add_space(8.0);
                        ui.label(egui::RichText::new(&self.case.title).strong().color(egui::Color32::WHITE));
                        ui.label(egui::RichText::new(&self.case.description).small().color(muted));
                        ui.add_space(18.0);
                        ui.label(egui::RichText::new("CHART ITEMS").size(10.0).strong().color(muted));
                        ui.label(format!("{} entities",self.case.entities.len())); ui.label(format!("{} relationships",self.case.links.len()));
                        ui.add_space(18.0);
                        ui.label(egui::RichText::new("FILTER ITEMS").size(10.0).strong().color(muted));
                        ui.add(egui::TextEdit::singleline(&mut self.query).hint_text("Search chart…").desired_width(160.0));
                        egui::ComboBox::from_id_salt("entity_sort").selected_text(match self.entity_sort {EntitySort::Name=>"Sort: name",EntitySort::Type=>"Sort: type",EntitySort::Connections=>"Sort: connections"}).show_ui(ui,|ui|{
                            ui.selectable_value(&mut self.entity_sort,EntitySort::Name,"Name");ui.selectable_value(&mut self.entity_sort,EntitySort::Type,"Type");ui.selectable_value(&mut self.entity_sort,EntitySort::Connections,"Connections");
                        });
                        ui.add_space(10.0);
                        let mut sorted:Vec<_>=self.case.entities.iter().collect();
                        sorted.sort_by(|a,b|match self.entity_sort {
                            EntitySort::Name=>a.name.to_lowercase().cmp(&b.name.to_lowercase()),
                            EntitySort::Type=>a.kind.to_lowercase().cmp(&b.kind.to_lowercase()).then_with(||a.name.cmp(&b.name)),
                            EntitySort::Connections=>self.case.links.iter().filter(|l|l.from==b.id||l.to==b.id).count().cmp(&self.case.links.iter().filter(|l|l.from==a.id||l.to==a.id).count()),
                        });
                        egui::ScrollArea::vertical().show(ui,|ui| for ent in sorted {
                            if !self.query.is_empty() && !format!("{} {} {}",ent.name,ent.kind,ent.detail).to_lowercase().contains(&self.query.to_lowercase()) {continue;}
                            let chosen=self.selected==Some(ent.id);
                            if ui.selectable_label(chosen,format!("● {}",ent.name)).clicked(){self.selected=Some(ent.id);self.selected_link=None;}
                        });
                    });
                    });
                    let canvas_w=(ui.available_width()-right-8.0).max(320.0);
                    ui.allocate_ui_with_layout(egui::vec2(canvas_w,avail.y),egui::Layout::top_down(egui::Align::Min),|ui|{
                        ui.set_min_size(egui::vec2(canvas_w,avail.y));
                        if self.page==WorkspacePage::Data {
                            ui.horizontal(|ui|{ui.label(egui::RichText::new("CASE RECORDS").strong().color(cyan));ui.selectable_value(&mut self.data_mode,DataMode::Entities,"Entities");ui.selectable_value(&mut self.data_mode,DataMode::Relationships,"Relationships");ui.selectable_value(&mut self.data_mode,DataMode::SqlRegressions,"SQL regressions");ui.selectable_value(&mut self.data_mode,DataMode::Automation,"Automation");if self.data_mode==DataMode::Entities||self.data_mode==DataMode::Relationships{ui.label(egui::RichText::new("CSV headers: source,target,relationship,date,confidence").small().color(muted));}});
                            ui.horizontal(|ui|{ui.label("Case file");ui.add(egui::TextEdit::singleline(&mut self.case_path).desired_width(260.0));ui.label("Description");ui.add(egui::TextEdit::singleline(&mut self.case.description).desired_width(260.0));});
                            ui.horizontal(|ui|{ui.label("Import path");ui.add(egui::TextEdit::singleline(&mut self.import_path).desired_width(180.0));ui.label("Export path");ui.add(egui::TextEdit::singleline(&mut self.export_path).desired_width(180.0));});
                            if self.data_mode==DataMode::Automation {
                                ui.label(egui::RichText::new("REPEATABLE SQL INGESTION JOBS").strong().color(cyan));
                                ui.label(egui::RichText::new("Jobs are saved in this case file. Run them here or from a script with the command-line runner.").small().color(muted));
                                ui.horizontal(|ui|{ui.label("Job name");ui.add(egui::TextEdit::singleline(&mut self.automation_job_name).desired_width(220.0));ui.label("Database");ui.add(egui::TextEdit::singleline(&mut self.sql_path).desired_width(220.0));});
                                ui.horizontal(|ui|{ui.label("Snapshot series");ui.add(egui::TextEdit::singleline(&mut self.sql_series).desired_width(220.0));if ui.button(if self.editing_automation_job.is_some(){"Save job changes"}else{"Add job"}).clicked(){self.save_automation_job();}if self.editing_automation_job.is_some()&&ui.button("Cancel edit").clicked(){self.editing_automation_job=None;self.automation_job_name.clear();}});
                                ui.add(egui::TextEdit::multiline(&mut self.sql_query).desired_width(canvas_w-20.0).desired_rows(5).code_editor());
                                if ui.button("Run all enabled jobs").clicked(){self.run_enabled_automation_jobs();}
                                ui.add_space(8.0);ui.label(egui::RichText::new("JOBS").small().strong().color(cyan));
                                let mut action: Option<(String,Uuid)> = None;
                                for job in &mut self.case.automation_jobs {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut job.enabled, "");
                                        ui.label(format!("{} · {} · {}",job.name,job.series,job.database_path));
                                        if ui.small_button("Edit").clicked(){action=Some(("edit".into(),job.id));}
                                        if ui.small_button("Run").clicked(){action=Some(("run".into(),job.id));}
                                        if ui.small_button("Delete").clicked(){action=Some(("delete".into(),job.id));}
                                    });
                                }
                                if let Some((verb,id))=action {match verb.as_str(){"edit"=>self.edit_automation_job(id),"run"=>self.run_automation_job(id),"delete"=>{self.case.automation_jobs.retain(|job|job.id!=id);self.status="AUTOMATION JOB DELETED".into();},_=>{}}}
                                ui.add_space(8.0);ui.label(egui::RichText::new("RUN HISTORY · newest first").small().strong().color(cyan));
                                egui::ScrollArea::vertical().max_height(210.0).show(ui,|ui|for run in self.case.automation_runs.iter().rev(){ui.label(egui::RichText::new(format!("{} · {} · {} → {} · {}",if run.success{"PASS"}else{"FAIL"},run.job_name,run.started_at,run.finished_at,run.message)).color(if run.success{muted}else{egui::Color32::from_rgb(238,139,115)}));});
                                return;
                            }
                            if self.data_mode==DataMode::SqlRegressions {
                                let latest=self.case.snapshots.last().cloned();
                                let baseline=latest.as_ref().and_then(|current|self.case.snapshots.iter().rev().skip(1).find(|snapshot|snapshot.source_key==current.source_key).cloned());
                                ui.horizontal(|ui|{ui.label("SQLite file");ui.add(egui::TextEdit::singleline(&mut self.sql_path).desired_width(260.0));ui.label("Snapshot series");ui.add(egui::TextEdit::singleline(&mut self.sql_series).desired_width(260.0));});
                                ui.label(egui::RichText::new("Read-only SQL query · required columns: source, target").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_query).desired_width(canvas_w-20.0).desired_rows(4).code_editor());
                                if ui.button("Ingest SQL snapshot").clicked(){self.ingest_sqlite_snapshot();}
                                if let Some(current)=latest {
                                    ui.label(format!("Latest: {} · {} · {} connections",current.captured_at,current.series,current.connections.len()));
                                    if let Some(base)=baseline {
                                        let deltas=compare_snapshots(&base,&current);
                                        let removed=deltas.iter().filter(|delta|delta.kind==DeltaKind::Removed).count();
                                        let declined=deltas.iter().filter(|delta|delta.kind==DeltaKind::ConfidenceDeclined).count();
                                        ui.label(egui::RichText::new(format!("Baseline {} → current {} · {removed} removed · {declined} confidence declines · {} other changes",base.captured_at,current.captured_at,deltas.len()-removed-declined)).strong().color(if removed+declined>0{egui::Color32::from_rgb(238,139,115)}else{cyan}));
                                        egui::ScrollArea::both().show(ui,|ui|egui::Grid::new("sql_regression_table").striped(true).show(ui,|ui|{
                                            for heading in ["CHANGE","CONNECTION","CONFIDENCE","DATE"]{ui.label(egui::RichText::new(heading).small().strong().color(cyan));}ui.end_row();
                                            for delta in deltas {let change=match delta.kind{DeltaKind::Removed=>"REMOVED",DeltaKind::ConfidenceDeclined=>"CONFIDENCE DOWN",DeltaKind::DateChanged=>"DATE CHANGED",DeltaKind::Updated=>"RECORD UPDATED",DeltaKind::Added=>"ADDED",DeltaKind::ConfidenceIncreased=>"CONFIDENCE UP"};ui.label(change);ui.label(format!("{} — {} — {}",delta.connection.source,delta.connection.relationship,delta.connection.target));ui.label(format!("{} → {}",delta.previous_confidence.map(|value|value.to_string()).unwrap_or_else(||"—".into()),if delta.kind==DeltaKind::Removed{"—".into()}else{delta.connection.confidence.to_string()}));ui.label(format!("{} → {}",delta.previous_date.unwrap_or_else(||"—".into()),if delta.kind==DeltaKind::Removed{"—".into()}else{delta.connection.date.clone()}));ui.end_row();}
                                        }));
                                    } else {ui.label(egui::RichText::new("This is the baseline snapshot for its database and series. Ingest the next period with the same series to compare connections.").color(muted));}
                                } else {ui.label(egui::RichText::new("No SQL snapshots yet. Ingest a baseline result, then ingest a later result using the same database and series label.").color(muted));}
                                ui.add_space(8.0);ui.label(egui::RichText::new("SNAPSHOT HISTORY").small().strong().color(cyan));
                                for snapshot in self.case.snapshots.iter().rev(){ui.label(format!("{} · {} · {} connections · {}",snapshot.captured_at,snapshot.series,snapshot.connections.len(),snapshot.database_path));}
                                return;
                            }
                            ui.horizontal(|ui|{ui.add(egui::TextEdit::singleline(&mut self.data_query).hint_text(if self.data_mode==DataMode::Entities{"Filter entities…"}else{"Filter relationships…"}).desired_width(240.0));if self.data_mode==DataMode::Relationships{egui::ComboBox::from_id_salt("link_sort").selected_text(match self.link_sort {LinkSort::Date=>"Sort: date",LinkSort::Relationship=>"Sort: relationship",LinkSort::Confidence=>"Sort: confidence"}).show_ui(ui,|ui|{ui.selectable_value(&mut self.link_sort,LinkSort::Date,"Date");ui.selectable_value(&mut self.link_sort,LinkSort::Relationship,"Relationship");ui.selectable_value(&mut self.link_sort,LinkSort::Confidence,"Confidence");});}});
                            if self.data_mode==DataMode::Entities {
                                egui::ScrollArea::both().show(ui,|ui|egui::Grid::new("entities_table").striped(true).show(ui,|ui|{
                                    for h in ["ENTITY","TYPE","NOTES","CONNECTIONS","ALIASES","MERGED RECORDS","ATTRIBUTES"]{ui.label(egui::RichText::new(h).small().strong().color(cyan));}ui.end_row();
                                    let mut entities:Vec<&Entity>=self.case.entities.iter().collect();
                                    entities.sort_by(|a,b|match self.entity_sort{EntitySort::Name=>a.name.to_lowercase().cmp(&b.name.to_lowercase()),EntitySort::Type=>a.kind.to_lowercase().cmp(&b.kind.to_lowercase()),EntitySort::Connections=>self.case.links.iter().filter(|l|l.from==b.id||l.to==b.id).count().cmp(&self.case.links.iter().filter(|l|l.from==a.id||l.to==a.id).count())});
                                    for entity in entities{if !format!("{} {} {} {:?} {:?}",entity.name,entity.kind,entity.detail,entity.attributes,entity.aliases).to_lowercase().contains(&self.data_query.to_lowercase()){continue;}let selected=self.selected==Some(entity.id);if ui.selectable_label(selected,&entity.name).clicked(){self.selected=Some(entity.id);self.selected_link=None;}ui.label(&entity.kind);ui.label(&entity.detail);ui.label(self.case.links.iter().filter(|l|l.from==entity.id||l.to==entity.id).count().to_string());ui.label(entity.aliases.join(" · "));ui.label(entity.merged_records.len().to_string());ui.label(entity.attributes.iter().map(|(k,v)|format!("{k}: {v}")).collect::<Vec<_>>().join(" · "));ui.end_row();}
                                }));
                                return;
                            }
                            egui::ScrollArea::both().show(ui,|ui|{
                                egui::Grid::new("links_table").striped(true).show(ui,|ui|{
                                    for h in ["FROM","RELATIONSHIP","TO","DATE","CONFIDENCE","DIR","SOURCE REFERENCE"] {ui.label(egui::RichText::new(h).small().strong().color(cyan));}ui.end_row();
                                    let mut indices:Vec<usize>=(0..self.case.links.len()).collect();
                                    indices.retain(|i|{let l=&self.case.links[*i];let from=self.case.entities.iter().find(|e|e.id==l.from).map(|e|e.name.as_str()).unwrap_or("");let to=self.case.entities.iter().find(|e|e.id==l.to).map(|e|e.name.as_str()).unwrap_or("");format!("{} {} {} {} {} {}",from,l.label,to,l.date,l.notes,l.source_reference).to_lowercase().contains(&self.data_query.to_lowercase())});
                                    indices.sort_by(|a,b|{let a=&self.case.links[*a];let b=&self.case.links[*b];match self.link_sort {LinkSort::Date=>a.date.cmp(&b.date),LinkSort::Relationship=>a.label.to_lowercase().cmp(&b.label.to_lowercase()),LinkSort::Confidence=>b.confidence.cmp(&a.confidence)}});
                                    for i in indices {let l=&self.case.links[i];
                                        let from=self.case.entities.iter().find(|e|e.id==l.from).map(|e|e.name.as_str()).unwrap_or("(missing)");
                                        let to=self.case.entities.iter().find(|e|e.id==l.to).map(|e|e.name.as_str()).unwrap_or("(missing)");
                                        let selected=self.selected_link==Some(i);
                                        if ui.selectable_label(selected,from).clicked(){self.selected_link=Some(i);self.selected=None;}
                                        if ui.selectable_label(selected,&l.label).clicked(){self.selected_link=Some(i);self.selected=None;}
                                        if ui.selectable_label(selected,to).clicked(){self.selected_link=Some(i);self.selected=None;}
                                        ui.label(&l.date);ui.label(l.confidence.to_string());ui.label(if l.directed{"→"}else{"↔"});ui.label(&l.source_reference);ui.end_row();
                                    }
                                });
                            });
                            return;
                        }
                        let (rect,response)=ui.allocate_exact_size(egui::vec2(canvas_w,avail.y),egui::Sense::click_and_drag());
                        let p=ui.painter_at(rect);
                        p.rect_filled(rect,4.0,egui::Color32::from_rgb(14,23,32));
                        for i in 0..20 { let x=rect.left()+i as f32*48.0; p.line_segment([egui::pos2(x,rect.top()),egui::pos2(x,rect.bottom())],(1.0,egui::Color32::from_rgb(24,38,49))); }
                        for i in 0..14 { let y=rect.top()+i as f32*48.0; p.line_segment([egui::pos2(rect.left(),y),egui::pos2(rect.right(),y)],(1.0,egui::Color32::from_rgb(24,38,49))); }
                        let center=|e:&Entity|egui::pos2(rect.left()+e.x*rect.width(),rect.top()+e.y*rect.height());
                        for (link_index, link) in self.case.links.iter().enumerate() {
                            if let (Some(a),Some(b))=(self.case.entities.iter().find(|e|e.id==link.from),self.case.entities.iter().find(|e|e.id==link.to)) {
                                let ca=center(a);let cb=center(b);let in_path=self.path_result.windows(2).any(|pair|(link.from==pair[0]&&link.to==pair[1])||(!link.directed&&link.from==pair[1]&&link.to==pair[0]));let col=if self.selected_link==Some(link_index){cyan}else if in_path{egui::Color32::from_rgb(236,174,83)}else{egui::Color32::from_rgb(72,104,120)};
                                let segments=if link.confidence==1 {8}else{1};
                                for s in 0..segments {let start=ca.lerp(cb,s as f32/segments as f32);let end=ca.lerp(cb,(s as f32+if segments==1{1.0}else{0.55})/segments as f32);p.line_segment([start,end],(if link.confidence==1{1.0}else{2.0},col));}
                                if link.directed {let d=(cb-ca).normalized();let tip=cb-d*75.0;let wing=egui::vec2(-d.y,d.x)*5.0;p.line_segment([tip,tip-d*10.0+wing],(1.5,col));p.line_segment([tip,tip-d*10.0-wing],(1.5,col));}
                                let mid=ca.lerp(cb,0.5); let label_rect=egui::Rect::from_center_size(mid,egui::vec2(110.0,18.0));
                                p.rect_filled(label_rect,3.0,if self.selected_link==Some(link_index){egui::Color32::from_rgb(34,61,70)}else{egui::Color32::from_rgb(19,31,41)});
                                p.text(mid,egui::Align2::CENTER_CENTER,&link.label,egui::FontId::proportional(9.0),muted);
                            }
                        }
                        for e in &self.case.entities {
                            let c=center(e); let fill=egui::Color32::from_rgb(e.color[0],e.color[1],e.color[2]);
                            let r=egui::Rect::from_center_size(c,egui::vec2(142.0,56.0));
                            let picked=self.selected==Some(e.id);
                            let on_path=self.path_result.contains(&e.id);p.rect_filled(r,7.0,if on_path{egui::Color32::from_rgb(55,48,38)}else{egui::Color32::from_rgb(25,39,51)});p.rect_stroke(r,7.0,egui::Stroke::new(if picked{2.0_f32}else{1.0_f32},if picked{cyan}else if on_path{egui::Color32::from_rgb(236,174,83)}else{fill}),egui::StrokeKind::Inside);
                            p.circle_filled(c+egui::vec2(-58.0,0.0),6.0,fill);
                            p.text(c+egui::vec2(-44.0,-8.0),egui::Align2::LEFT_CENTER,&e.name,egui::FontId::proportional(12.0),egui::Color32::WHITE);
                            p.text(c+egui::vec2(-44.0,11.0),egui::Align2::LEFT_CENTER,&e.kind,egui::FontId::proportional(8.0),muted);
                        }
                        if response.clicked() { if let Some(pos)=response.interact_pointer_pos(){
                            if self.show_timeline && pos.y >= rect.bottom()-105.0 {
                                let mut events:Vec<usize>=(0..self.case.links.len()).collect();events.sort_by(|a,b|self.case.links[*a].date.cmp(&self.case.links[*b].date));
                                let nearest=events.iter().enumerate().map(|(rank,index)|{let x=rect.left()+40.0+rank as f32*(rect.width()-80.0)/(events.len().max(2)-1) as f32;(*index,egui::pos2(x,rect.bottom()-52.5).distance(pos))}).min_by(|a,b|a.1.total_cmp(&b.1)).filter(|(_,distance)|*distance<20.0).map(|(index,_)|index);
                                self.selected=None;self.selected_link=nearest;
                            } else if let Some(entity)=self.case.entities.iter().find(|e|center(e).distance(pos)<75.0){self.selected=Some(entity.id);self.selected_link=None;}
                            else {let nearest=self.case.links.iter().enumerate().filter_map(|(i,l)|{let a=self.case.entities.iter().find(|e|e.id==l.from)?;let b=self.case.entities.iter().find(|e|e.id==l.to)?;Some((i,segment_distance(pos,center(a),center(b))))}).filter(|(_,d)|*d<10.0).min_by(|a,b|a.1.total_cmp(&b.1)).map(|(i,_)|i);self.selected=None;self.selected_link=nearest;}
                        } }
                        if response.drag_started() { if let Some(pos)=response.interact_pointer_pos(){if let Some(entity)=self.case.entities.iter().find(|e|center(e).distance(pos)<75.0){self.selected=Some(entity.id);self.selected_link=None;}else{self.selected=None;}} }
                        if response.dragged() { if let Some(pos)=response.interact_pointer_pos(){if let Some(id)=self.selected{if let Some(e)=self.case.entities.iter_mut().find(|e|e.id==id){e.x=((pos.x-rect.left())/rect.width()).clamp(0.05,0.95);e.y=((pos.y-rect.top())/rect.height()).clamp(0.08,0.92);}}} }
                        if self.show_timeline {
                            let tr=egui::Rect::from_min_size(rect.left_bottom()-egui::vec2(0.0,105.0),egui::vec2(rect.width(),105.0));
                            p.rect_filled(tr,4.0,egui::Color32::from_rgb(20,32,43));p.text(tr.left_top()+egui::vec2(14.0,14.0),egui::Align2::LEFT_TOP,"EVENT TIMELINE",egui::FontId::proportional(10.0),cyan);
                            let y=tr.center().y+12.0;p.line_segment([egui::pos2(tr.left()+25.0,y),egui::pos2(tr.right()-25.0,y)],(1.0,muted));
                            let mut events:Vec<usize>=(0..self.case.links.len()).collect();events.sort_by(|a,b|self.case.links[*a].date.cmp(&self.case.links[*b].date));
                            for (rank,index) in events.iter().enumerate(){let l=&self.case.links[*index];let x=tr.left()+40.0+rank as f32*(tr.width()-80.0)/(events.len().max(2)-1) as f32;let event_color=if self.selected_link==Some(*index){cyan}else if l.confidence==1{egui::Color32::from_rgb(223,137,105)}else{egui::Color32::from_rgb(236,174,83)};p.circle_filled(egui::pos2(x,y),5.0,event_color);p.text(egui::pos2(x,y-8.0),egui::Align2::CENTER_BOTTOM,&l.label,egui::FontId::proportional(7.0),muted);p.text(egui::pos2(x,y+8.0),egui::Align2::CENTER_TOP,&l.date,egui::FontId::proportional(8.0),muted);}
                        }
                    });
                    ui.allocate_ui_with_layout(egui::vec2(right,avail.y),egui::Layout::top_down(egui::Align::Min),|ui|{
                    egui::Frame::new().fill(panel).inner_margin(12).show(ui,|ui|{
                        ui.set_min_width(right);ui.set_max_width(right);
                        ui.label(egui::RichText::new("ANALYSIS").size(10.0).strong().color(muted));ui.add_space(10.0);
                        let mut degrees:Vec<_>=self.case.entities.iter().map(|e|(e,self.case.links.iter().filter(|l|l.from==e.id||l.to==e.id).count())).collect();degrees.sort_by_key(|(_,n)|std::cmp::Reverse(*n));
                        ui.label(egui::RichText::new("MOST CONNECTED").size(9.0).strong().color(cyan));
                        for (rank,(e,n)) in degrees.iter().take(5).enumerate(){ui.horizontal(|ui|{ui.label(egui::RichText::new(format!("{:02}",rank+1)).color(muted).monospace());ui.label(&e.name);ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(n.to_string());});});}
                        ui.add_space(16.0);ui.label(egui::RichText::new("SHORTEST PATH").size(9.0).strong().color(cyan));
                        egui::ComboBox::from_id_salt("path_start").selected_text(self.path_start.and_then(|id|self.case.entities.iter().find(|e|e.id==id).map(|e|e.name.clone())).unwrap_or_else(||"Start entity".into())).show_ui(ui,|ui|{for e in &self.case.entities{ui.selectable_value(&mut self.path_start,Some(e.id),&e.name);}});
                        egui::ComboBox::from_id_salt("path_end").selected_text(self.path_end.and_then(|id|self.case.entities.iter().find(|e|e.id==id).map(|e|e.name.clone())).unwrap_or_else(||"End entity".into())).show_ui(ui,|ui|{for e in &self.case.entities{ui.selectable_value(&mut self.path_end,Some(e.id),&e.name);}});
                        if ui.button("Analyze path").clicked(){self.analyze_path();}
                        if !self.path_result.is_empty(){let labels:Vec<_>=self.path_result.iter().filter_map(|id|self.case.entities.iter().find(|e|e.id==*id).map(|e|e.name.as_str())).collect();ui.label(egui::RichText::new(labels.join(" → ")).small().color(egui::Color32::from_rgb(236,174,83)));if ui.small_button("Clear path").clicked(){self.path_result.clear();}}
                        ui.add_space(16.0);ui.label(egui::RichText::new("POSSIBLE DUPLICATES").size(9.0).strong().color(cyan));
                        if ui.button("Find likely matches").clicked(){self.find_duplicates();}
                        for (a,b) in self.duplicate_candidates.clone().iter().take(5){let an=self.case.entities.iter().find(|e|e.id==*a).map(|e|e.name.as_str()).unwrap_or("(missing)");let bn=self.case.entities.iter().find(|e|e.id==*b).map(|e|e.name.as_str()).unwrap_or("(missing)");if ui.selectable_label(self.merge_candidate==Some((*a,*b)),format!("{an}  ↔  {bn}")).clicked(){self.merge_candidate=Some((*a,*b));}}
                        if let Some((keep,remove))=self.merge_candidate{let keep_name=self.case.entities.iter().find(|e|e.id==keep).map(|e|e.name.clone()).unwrap_or_else(||"(missing)".into());let remove_name=self.case.entities.iter().find(|e|e.id==remove).map(|e|e.name.clone()).unwrap_or_else(||"(missing)".into());ui.label(egui::RichText::new(format!("Keep {keep_name}; merge {remove_name} into it. Aliases, notes, attributes, and links will be retained.")).small().color(muted));if ui.button(format!("Merge {remove_name} into {keep_name}")).clicked(){self.merge_candidate_into_first();}}
                        ui.add_space(20.0);ui.label(egui::RichText::new("SELECTED ITEM").size(9.0).strong().color(cyan));ui.add_space(5.0);
                        if let Some(id)=self.selected{if let Some(entity)=self.case.entities.iter().find(|e|e.id==id){if !entity.aliases.is_empty(){ui.label(egui::RichText::new("KNOWN ALIASES").size(9.0).strong().color(cyan));for alias in &entity.aliases{ui.label(alias);}}if !entity.merged_records.is_empty(){egui::CollapsingHeader::new(format!("Merged record history ({})",entity.merged_records.len())).show(ui,|ui|for record in &entity.merged_records{ui.label(format!("{} · {} · {}",record.name,record.kind,record.id));if !record.detail.is_empty(){ui.label(&record.detail);}for (key,value) in &record.attributes{ui.label(format!("{key}: {value}"));}});}}}
                        if let Some(id)=self.selected { if let Some(e)=self.case.entities.iter_mut().find(|e|e.id==id){ui.label(egui::RichText::new("TYPE").size(9.0).color(muted));ui.add(egui::TextEdit::singleline(&mut e.kind).desired_width(230.0));ui.label(egui::RichText::new("LABEL").size(9.0).color(muted));ui.add(egui::TextEdit::singleline(&mut e.name).desired_width(230.0));ui.label(egui::RichText::new("NOTES").size(9.0).color(muted));ui.add(egui::TextEdit::multiline(&mut e.detail).desired_width(230.0).desired_rows(3));ui.label(egui::RichText::new("ATTRIBUTES").size(9.0).strong().color(cyan));let mut remove=None;for (i,(key,value)) in e.attributes.iter_mut().enumerate(){ui.horizontal(|ui|{ui.add(egui::TextEdit::singleline(key).hint_text("Attribute").desired_width(90.0));ui.add(egui::TextEdit::singleline(value).hint_text("Value").desired_width(100.0));if ui.small_button("×").clicked(){remove=Some(i);}});}if let Some(i)=remove{e.attributes.remove(i);}if ui.small_button("＋ Attribute").clicked(){e.attributes.push((String::new(),String::new()));}ui.label(egui::RichText::new("LINKS").size(9.0).strong().color(cyan));for l in self.case.links.iter().filter(|l|l.from==id||l.to==id){ui.label(format!("{}  ·  {}",l.label,l.date));}}}else if let Some(i)=self.selected_link {if let Some(link)=self.case.links.get_mut(i){ui.label(egui::RichText::new("RELATIONSHIP").size(9.0).color(muted));ui.add(egui::TextEdit::singleline(&mut link.label));ui.horizontal(|ui|{ui.label("Date");ui.add(egui::TextEdit::singleline(&mut link.date).desired_width(100.0));});ui.add(egui::Slider::new(&mut link.confidence,1..=3).text("Confidence"));ui.checkbox(&mut link.directed,"Directed relationship");ui.label("Source reference");ui.add(egui::TextEdit::singleline(&mut link.source_reference).desired_width(230.0));ui.label("Analyst notes");ui.add(egui::TextEdit::multiline(&mut link.notes).desired_rows(4));}}else{ui.label(egui::RichText::new("Select an entity on the chart to inspect its attributes and connections. Select a relationship in Data to edit its properties.").small().color(muted));}
                        ui.add_space(20.0);ui.label(egui::RichText::new("LEGEND").size(9.0).strong().color(cyan));ui.label(egui::RichText::new("Solid link  ·  corroborated").small().color(muted));ui.label(egui::RichText::new("Dashed link  ·  unverified").small().color(muted));
                    });
                    });
                });
            });
        });
    }
}

fn segment_distance(point: egui::Pos2, start: egui::Pos2, end: egui::Pos2) -> f32 {
    let segment = end - start;
    let length_squared = segment.length_sq();
    if length_squared == 0.0 {
        return point.distance(start);
    }
    let t = ((point - start).dot(segment) / length_squared).clamp(0.0, 1.0);
    point.distance(start + segment * t)
}

fn run_automation_cli(case_path: &str, job_name: Option<&str>) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let jobs: Vec<AutomationJob> = if let Some(name) = job_name {
        let matches: Vec<AutomationJob> = case
            .automation_jobs
            .iter()
            .filter(|job| job.name == name)
            .cloned()
            .collect();
        if matches.len() > 1 {
            return Err(format!("job name is ambiguous: {name}"));
        }
        vec![
            matches
                .into_iter()
                .next()
                .ok_or_else(|| format!("automation job not found: {name}"))?,
        ]
    } else {
        case.automation_jobs
            .iter()
            .filter(|job| job.enabled)
            .cloned()
            .collect()
    };
    if jobs.is_empty() {
        return Err("case contains no enabled automation jobs".into());
    }
    let mut app = App::default();
    app.case = case;
    let mut failures = 0;
    for job in jobs {
        let started_at = epoch_timestamp();
        app.sql_path = job.database_path.clone();
        app.sql_series = job.series.clone();
        app.sql_query = job.query.clone();
        app.ingest_sqlite_snapshot();
        let message = app.status.clone();
        let success = message.starts_with("SQL SNAPSHOT INGESTED");
        if !success {
            failures += 1;
        }
        println!(
            "{} · {} · {}",
            if success { "PASS" } else { "FAIL" },
            job.name,
            message
        );
        app.case.automation_runs.push(AutomationRun {
            job_id: job.id,
            job_name: job.name,
            started_at,
            finished_at: epoch_timestamp(),
            success,
            message,
        });
        if app.case.automation_runs.len() > 200 {
            app.case
                .automation_runs
                .drain(..app.case.automation_runs.len() - 200);
        }
    }
    let serialized = serde_json::to_string_pretty(&app.case).map_err(|error| error.to_string())?;
    // Replace by rename so interruption cannot leave a truncated case file behind.
    let temporary_path = format!("{case_path}.tmp-{}", std::process::id());
    fs::write(&temporary_path, serialized)
        .map_err(|error| format!("cannot write temporary case file: {error}"))?;
    if let Err(error) = fs::rename(&temporary_path, case_path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(format!(
            "cannot replace case with updated run history: {error}"
        ));
    }
    if failures > 0 {
        Err(format!(
            "{failures} automation job(s) failed; run history saved"
        ))
    } else {
        Ok(())
    }
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!(
            "Arch Notebook\n\nUsage:\n  arch-notebook\n  arch-notebook --run-enabled-jobs <case.json>\n  arch-notebook --run-job <case.json> <job-name>\n\nBatch runs save snapshots and run history to the case file."
        );
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--run-enabled-jobs") {
        let Some(case_path) = args.get(1) else {
            eprintln!("usage: arch-notebook --run-enabled-jobs <case.json>");
            std::process::exit(2);
        };
        if let Err(error) = run_automation_cli(case_path, None) {
            eprintln!("automation failed: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--run-job") {
        let (Some(case_path), Some(job_name)) = (args.get(1), args.get(2)) else {
            eprintln!("usage: arch-notebook --run-job <case.json> <job-name>");
            std::process::exit(2);
        };
        if let Err(error) = run_automation_cli(case_path, Some(job_name)) {
            eprintln!("automation failed: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1440.0, 900.0])
            .with_min_inner_size([1000.0, 650.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Arch Notebook · Connection Regression Workbench",
        options,
        Box::new(|_| Ok(Box::new(App::default()))),
    )
}
