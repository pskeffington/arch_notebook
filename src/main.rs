use eframe::egui;
use quick_xml::{
    Reader as XmlReader, escape::resolve_predefined_entity, events::Event as XmlEvent,
};
use rusqlite::{
    Connection, InterruptHandle, OpenFlags, TransactionBehavior, params_from_iter,
    types::{Value, ValueRef},
};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, Ordering},
        mpsc::{self, Receiver},
    },
    thread,
    time::{Duration, Instant},
};
use uuid::Uuid;

const CURRENT_CASE_FORMAT_VERSION: u32 = 10;
const SQL_QUERY_TIMEOUT_MINUTES: u64 = 15;
const SQL_QUERY_TIMEOUT: Duration = Duration::from_secs(SQL_QUERY_TIMEOUT_MINUTES * 60);

fn sql_query_timeout_message() -> String {
    format!("SQL query exceeded the {SQL_QUERY_TIMEOUT_MINUTES} minute execution limit")
}

struct SqlQueryDeadline {
    cancel: mpsc::Sender<()>,
    worker: Option<thread::JoinHandle<()>>,
    expired: Arc<AtomicBool>,
}

impl SqlQueryDeadline {
    fn new(interrupt: InterruptHandle) -> Self {
        let (cancel, cancelled) = mpsc::channel();
        let expired = Arc::new(AtomicBool::new(false));
        let worker_expired = Arc::clone(&expired);
        let worker = thread::spawn(move || {
            if cancelled.recv_timeout(SQL_QUERY_TIMEOUT).is_err() {
                worker_expired.store(true, Ordering::Release);
                interrupt.interrupt();
            }
        });
        Self {
            cancel,
            worker: Some(worker),
            expired,
        }
    }

    fn expired(&self) -> bool {
        self.expired.load(Ordering::Acquire)
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
    detail: String,
    #[serde(default)]
    attributes: Vec<(String, String)>,
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

#[derive(Clone, Serialize, Deserialize)]
struct MergedEntityRecord {
    id: Uuid,
    name: String,
    kind: String,
    detail: String,
    attributes: Vec<(String, String)>,
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
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    series: String,
    source_key: String,
    database_path: String,
    #[serde(default)]
    configured_database_path: String,
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    captured_at: String,
    connections: Vec<ConnectionFact>,
    #[serde(default)]
    skipped_rows: usize,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationJob {
    id: Uuid,
    name: String,
    database_path: String,
    series: String,
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    enabled: bool,
    #[serde(default)]
    minimum_connections: usize,
    #[serde(default)]
    maximum_connections: Option<usize>,
    #[serde(default)]
    maximum_regressions: Option<usize>,
    #[serde(default)]
    maximum_skipped_rows: Option<usize>,
}

#[derive(Clone, Serialize, Deserialize)]
struct AutomationRun {
    #[serde(default = "Uuid::new_v4")]
    id: Uuid,
    job_id: Uuid,
    job_name: String,
    #[serde(default)]
    database_path: String,
    #[serde(default)]
    series: String,
    #[serde(default)]
    query: String,
    #[serde(default)]
    parameters: Vec<(String, String)>,
    #[serde(default)]
    minimum_connections: usize,
    #[serde(default)]
    maximum_connections: Option<usize>,
    #[serde(default)]
    maximum_regressions: Option<usize>,
    #[serde(default)]
    maximum_skipped_rows: Option<usize>,
    #[serde(default)]
    enabled: bool,
    #[serde(default)]
    configuration_recorded: bool,
    started_at: String,
    finished_at: String,
    #[serde(default)]
    duration_ms: Option<u64>,
    success: bool,
    message: String,
    #[serde(default)]
    snapshot_index: Option<usize>,
    #[serde(default)]
    snapshot_id: Option<Uuid>,
    #[serde(default)]
    rerun_of: Option<Uuid>,
}

struct SqlValidationPreview {
    configured_database_path: String,
    series: String,
    query: String,
    parameters: Vec<(String, String)>,
    captured_at: String,
    connection_count: usize,
    skipped_rows: usize,
    connections: Vec<ConnectionFact>,
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
    snapshots: Vec<ConnectionSnapshot>,
    #[serde(default)]
    automation_jobs: Vec<AutomationJob>,
    #[serde(default)]
    automation_runs: Vec<AutomationRun>,
    #[serde(default)]
    import_specifications: Vec<ImportSpecification>,
}

struct App {
    case: CaseFile,
    selected: Option<Uuid>,
    query: String,
    status: String,
    show_timeline: bool,
    chart_zoom: f32,
    chart_pan: egui::Vec2,
    chart_pan_origin: egui::Vec2,
    chart_focus: Option<Uuid>,
    chart_focus_link: Option<usize>,
    chart_fit_entities: Option<Vec<Uuid>>,
    chart_fit_path: Option<Vec<Uuid>>,
    chart_focus_group: Option<HashSet<Uuid>>,
    network_groups: Option<std::sync::Arc<Vec<Vec<Uuid>>>>,
    chart_navigator_dragging: bool,
    new_link_from: Option<Uuid>,
    new_link_to: Option<Uuid>,
    import_path: String,
    import_headers: Vec<String>,
    import_preview: Vec<Vec<String>>,
    import_column_mapping: ImportColumnMapping,
    import_spec_name: String,
    selected_import_spec: Option<Uuid>,
    show_import_wizard: bool,
    export_path: String,
    graphml_export_path: String,
    selected_link: Option<usize>,
    page: WorkspacePage,
    entity_sort: EntitySort,
    most_connected_measure: MostConnectedMeasure,
    most_connected_relationship: Option<String>,
    layout_mode: LayoutMode,
    link_sort: LinkSort,
    link_table_view: LinkTableView,
    data_query: String,
    data_source_filter: Option<String>,
    connections_for_selected: bool,
    path_start: Option<Uuid>,
    path_end: Option<Uuid>,
    path_result: Vec<Uuid>,
    case_path: String,
    case_path_input: String,
    saved_case_json: Option<String>,
    disk_case_json: Option<String>,
    pending_case_action: Option<PendingCaseAction>,
    data_mode: DataMode,
    merge_candidate: Option<(Uuid, Uuid)>,
    duplicate_candidates: Vec<(Uuid, Uuid)>,
    sql_path: String,
    sql_series: String,
    sql_query: String,
    sql_parameter_text: String,
    sql_validation_preview: Option<SqlValidationPreview>,
    sql_schema_templates: Vec<(String, String)>,
    sql_schema_template_index: usize,
    sql_schema_template_source: Option<(String, String)>,
    automation_job_filter: String,
    automation_run_filter: String,
    automation_failed_runs_only: bool,
    sql_snapshot_source: Option<String>,
    sql_snapshot_index: Option<usize>,
    sql_delta_filter: SnapshotDeltaFilter,
    sql_delta_search: String,
    sql_source_focus: Option<String>,
    automation_job_name: String,
    automation_minimum_connections: usize,
    automation_maximum_connections: Option<usize>,
    automation_maximum_regressions: Option<usize>,
    automation_maximum_skipped_rows: Option<usize>,
    editing_automation_job: Option<Uuid>,
    automation_editor_before_edit: Option<(
        String,
        String,
        String,
        String,
        String,
        usize,
        Option<usize>,
        Option<usize>,
        Option<usize>,
    )>,
    automation_worker: Option<Receiver<Result<AutomationWorkerResult, String>>>,
    automation_progress: Option<Arc<Mutex<String>>>,
}

enum AutomationWorkerResult {
    SavedCase {
        case: CaseFile,
        disk_case_json: String,
        status: String,
        selected_snapshot: Option<(String, usize)>,
        selected_link: Option<usize>,
    },
    FailedCase {
        case: CaseFile,
        disk_case_json: String,
        error: String,
        selected_snapshot: Option<(String, usize)>,
        selected_link: Option<usize>,
    },
    UnsavedCase {
        case: CaseFile,
        status: String,
        selected_snapshot: Option<(String, usize)>,
        selected_link: Option<usize>,
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
    maximum_connections: Option<usize>,
    maximum_regressions: Option<usize>,
    maximum_skipped_rows: Option<usize>,
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

struct GraphMlGraphContext {
    default_directed: bool,
    parent_node_id: Option<String>,
}

#[derive(Default)]
struct GraphMlDocument {
    default_directed: bool,
    graph_data: HashMap<String, String>,
    nodes: Vec<GraphMlRecord>,
    edges: Vec<GraphMlRecord>,
}

#[derive(Clone, Default, Serialize, Deserialize)]
#[serde(default)]
struct ImportColumnMapping {
    trim_whitespace: bool,
    row_filters: Vec<ImportRowFilter>,
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
    source_color: String,
    target_color: String,
    source_x: String,
    source_y: String,
    target_x: String,
    target_y: String,
    source_position_locked: String,
    target_position_locked: String,
    entity_attributes: Vec<ImportAttributeMapping>,
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
    SqlRegressions,
    Automation,
}

#[derive(PartialEq, Clone, Copy)]
enum EntitySort {
    Name,
    Type,
    Connections,
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
    Hierarchy,
    Grid,
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

#[derive(Clone, Copy, PartialEq)]
enum DeltaKind {
    Removed,
    ConfidenceDeclined,
    DateChanged,
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
    Direction,
    Evidence,
    SourceNotes,
    Other,
}

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
        Self {
            case,
            selected: None,
            query: String::new(),
            status,
            show_timeline: false,
            chart_zoom: 1.0,
            chart_pan: egui::Vec2::ZERO,
            chart_pan_origin: egui::Vec2::ZERO,
            chart_focus: None,
            chart_focus_link: None,
            chart_fit_entities: None,
            chart_fit_path: None,
            chart_focus_group: None,
            network_groups: None,
            chart_navigator_dragging: false,
            new_link_from: None,
            new_link_to: None,
            import_path: data_dir.join("sample_relationships.csv").to_string_lossy().into_owned(),
            import_headers: Vec::new(),
            import_preview: Vec::new(),
            import_column_mapping: ImportColumnMapping::default(),
            import_spec_name: String::new(),
            selected_import_spec: None,
            show_import_wizard: false,
            export_path: data_dir.join("chart-export.csv").to_string_lossy().into_owned(),
            graphml_export_path: data_dir
                .join("chart-export.graphml")
                .to_string_lossy()
                .into_owned(),
            selected_link: None,
            page: WorkspacePage::Chart,
            entity_sort: EntitySort::Name,
            most_connected_measure: MostConnectedMeasure::default(),
            most_connected_relationship: None,
            layout_mode: LayoutMode::Connectivity,
            link_sort: LinkSort::Date,
            link_table_view: LinkTableView::Connections,
            data_query: String::new(),
            data_source_filter: None,
            connections_for_selected: false,
            path_start: None,
            path_end: None,
            path_result: Vec::new(),
            case_path: case_path.clone(),
            case_path_input: case_path,
            saved_case_json,
            disk_case_json,
            pending_case_action: None,
            data_mode: DataMode::Relationships,
            merge_candidate: None,
            duplicate_candidates: Vec::new(),
            sql_path: data_dir.join("sample_connections.sqlite").to_string_lossy().into_owned(),
            sql_series: "Harbor Light connection review".into(),
            sql_query: "SELECT connection_id, source, source_type, target, target_type, relationship, event_date AS date, confidence, directed, source_reference, notes FROM connection_observations WHERE snapshot_id = :snapshot_id ORDER BY connection_id;".into(),
            sql_parameter_text: "snapshot_id=baseline".into(),
            sql_validation_preview: None,
            sql_schema_templates: Vec::new(),
            sql_schema_template_index: 0,
            sql_schema_template_source: None,
            automation_job_filter: String::new(),
            automation_run_filter: String::new(),
            automation_failed_runs_only: false,
            sql_snapshot_source: None,
            sql_snapshot_index: None,
            sql_delta_filter: SnapshotDeltaFilter::All,
            sql_delta_search: String::new(),
            sql_source_focus: None,
            automation_job_name: String::new(),
            automation_minimum_connections: 0,
            automation_maximum_connections: None,
            automation_maximum_regressions: None,
            automation_maximum_skipped_rows: None,
            editing_automation_job: None,
            automation_editor_before_edit: None,
            automation_worker: None,
            automation_progress: None,
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
        detail: detail.into(),
        attributes: Vec::new(),
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
        },
    ];
    CaseFile {
        format_version: CURRENT_CASE_FORMAT_VERSION,
        title: "HARBOR LIGHT // SAMPLE CASE".into(),
        description: "Fictional sample relationship investigation".into(),
        entities,
        links,
        snapshots: Vec::new(),
        automation_jobs: Vec::new(),
        automation_runs: Vec::new(),
        import_specifications: Vec::new(),
    }
}

impl App {
    fn invalidate_network_groups(&mut self) {
        self.network_groups = None;
        self.chart_focus_group = None;
    }

    fn automation_context(case: CaseFile, case_path: &str) -> Self {
        let saved_case_json = serde_json::to_string(&case).ok();
        Self {
            case,
            selected: None,
            query: String::new(),
            status: String::new(),
            show_timeline: false,
            chart_zoom: 1.0,
            chart_pan: egui::Vec2::ZERO,
            chart_pan_origin: egui::Vec2::ZERO,
            chart_focus: None,
            chart_focus_link: None,
            chart_fit_entities: None,
            chart_fit_path: None,
            chart_focus_group: None,
            network_groups: None,
            chart_navigator_dragging: false,
            new_link_from: None,
            new_link_to: None,
            import_path: String::new(),
            import_headers: Vec::new(),
            import_preview: Vec::new(),
            import_column_mapping: ImportColumnMapping::default(),
            import_spec_name: String::new(),
            selected_import_spec: None,
            show_import_wizard: false,
            export_path: String::new(),
            graphml_export_path: String::new(),
            selected_link: None,
            page: WorkspacePage::Chart,
            entity_sort: EntitySort::Name,
            most_connected_measure: MostConnectedMeasure::default(),
            most_connected_relationship: None,
            layout_mode: LayoutMode::Connectivity,
            link_sort: LinkSort::Date,
            link_table_view: LinkTableView::Connections,
            data_query: String::new(),
            data_source_filter: None,
            connections_for_selected: false,
            path_start: None,
            path_end: None,
            path_result: Vec::new(),
            case_path: case_path.to_owned(),
            case_path_input: case_path.to_owned(),
            saved_case_json,
            disk_case_json: None,
            pending_case_action: None,
            data_mode: DataMode::Relationships,
            merge_candidate: None,
            duplicate_candidates: Vec::new(),
            sql_path: String::new(),
            sql_series: String::new(),
            sql_query: String::new(),
            sql_parameter_text: String::new(),
            sql_validation_preview: None,
            sql_schema_templates: Vec::new(),
            sql_schema_template_index: 0,
            sql_schema_template_source: None,
            automation_job_filter: String::new(),
            automation_run_filter: String::new(),
            automation_failed_runs_only: false,
            sql_snapshot_source: None,
            sql_snapshot_index: None,
            sql_delta_filter: SnapshotDeltaFilter::All,
            sql_delta_search: String::new(),
            sql_source_focus: None,
            automation_job_name: String::new(),
            automation_minimum_connections: 0,
            automation_maximum_connections: None,
            automation_maximum_regressions: None,
            automation_maximum_skipped_rows: None,
            editing_automation_job: None,
            automation_editor_before_edit: None,
            automation_worker: None,
            automation_progress: None,
        }
    }

    fn current_case_json(&self) -> Option<String> {
        serde_json::to_string(&self.case).ok()
    }

    fn has_unsaved_changes(&self) -> bool {
        self.current_case_json() != self.saved_case_json
            || self.editing_automation_job.is_some()
            || (self.editing_automation_job.is_none()
                && !self.automation_job_name.trim().is_empty())
    }

    fn request_case_action(&mut self, action: PendingCaseAction) {
        if self.has_unsaved_changes() {
            self.pending_case_action = Some(action);
        } else {
            self.perform_case_action(action);
        }
    }

    fn perform_case_action(&mut self, action: PendingCaseAction) {
        self.pending_case_action = None;
        match action {
            PendingCaseAction::Open => self.load(),
            PendingCaseAction::New => self.new_case(),
        }
    }

    fn save(&mut self) {
        let path = Path::new(self.case_path.trim());
        let result = if path.as_os_str().is_empty() {
            Err("case path cannot be empty".to_string())
        } else {
            save_case_atomically_if_unchanged(path, &self.case, self.disk_case_json.as_deref())
        };
        match result {
            Ok(disk_case_json) => {
                self.disk_case_json = Some(disk_case_json);
                self.saved_case_json = self.current_case_json();
                self.case_path_input = self.case_path.clone();
                self.status = format!("CASE SAVED · {}", self.case_path);
            }
            Err(e) => self.status = format!("SAVE FAILED · {e}"),
        }
    }

    fn save_as(&mut self) {
        let path = Path::new(self.case_path_input.trim());
        let result = if path.as_os_str().is_empty() {
            Err("case path cannot be empty".to_string())
        } else if path == Path::new(self.case_path.trim()) {
            save_case_atomically_if_unchanged(path, &self.case, self.disk_case_json.as_deref())
        } else {
            save_case_atomically_with_lock(path, &self.case)
        };
        match result {
            Ok(disk_case_json) => {
                self.case_path = self.case_path_input.trim().to_string();
                self.disk_case_json = Some(disk_case_json);
                self.saved_case_json = self.current_case_json();
                self.status = format!("CASE SAVED · {}", self.case_path);
            }
            Err(error) => self.status = format!("SAVE AS FAILED · {error}"),
        }
    }

    fn load(&mut self) {
        let requested_path = self.case_path_input.trim().to_string();
        if requested_path.is_empty() {
            self.status = "LOAD FAILED · case path cannot be empty".into();
            return;
        }
        match fs::read_to_string(&requested_path)
            .map_err(|e| e.to_string())
            .and_then(|contents| {
                serde_json::from_str::<CaseFile>(&contents)
                    .map(|case| (case, contents))
                    .map_err(|error| error.to_string())
            }) {
            Ok((mut c, disk_case_json)) => {
                normalize_snapshot_database_paths(&mut c, &requested_path);
                self.case = c;
                self.network_groups = None;
                self.case_path = requested_path.clone();
                self.case_path_input = requested_path;
                self.disk_case_json = Some(disk_case_json);
                self.saved_case_json = self.current_case_json();
                self.sql_snapshot_source = None;
                self.sql_snapshot_index = None;
                self.sql_source_focus = None;
                self.data_source_filter = None;
                self.editing_automation_job = None;
                self.automation_editor_before_edit = None;
                self.automation_job_name.clear();
                self.selected_import_spec = None;
                self.import_spec_name.clear();
                self.show_import_wizard = false;
                self.chart_zoom = 1.0;
                self.chart_pan = egui::Vec2::ZERO;
                self.chart_pan_origin = egui::Vec2::ZERO;
                self.chart_focus = None;
                self.chart_focus_link = None;
                self.chart_fit_entities = None;
                self.chart_fit_path = None;
                self.chart_focus_group = None;
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

    fn import_graphml(&mut self) {
        let source_path = self.import_path.trim().to_owned();
        let result = graphml_csv_bytes(&source_path).and_then(write_graphml_csv_temp);
        let temporary_path = match result {
            Ok(path) => path,
            Err(error) => {
                self.status = format!("GRAPHML IMPORT FAILED · {error}");
                return;
            }
        };
        let original_path = self.import_path.clone();
        let original_mapping = self.import_column_mapping.clone();
        self.import_path = temporary_path.to_string_lossy().into_owned();
        self.import_column_mapping = ImportColumnMapping::default();
        self.import_delimited();
        self.import_path = original_path;
        self.import_column_mapping = original_mapping;
        let import_status = std::mem::take(&mut self.status);
        self.status = if let Some(summary) = import_status.strip_prefix("CSV IMPORT · ") {
            format!("GRAPHML IMPORT · {summary}")
        } else if let Some(error) = import_status.strip_prefix("IMPORT FAILED · ") {
            format!("GRAPHML IMPORT FAILED · {error}")
        } else {
            format!("GRAPHML IMPORT FAILED · {import_status}")
        };
        if let Err(error) = fs::remove_file(&temporary_path) {
            self.status = format!("{} · TEMP FILE CLEANUP FAILED · {error}", self.status);
        }
    }

    fn inspect_delimited_import(&mut self) {
        let result = (|| -> Result<(Vec<String>, Vec<Vec<String>>), String> {
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(delimited_text_delimiter(&self.import_path))
                .from_path(&self.import_path)
                .map_err(|error| error.to_string())?;
            let headers = reader
                .headers()
                .map(|headers| headers.iter().map(str::to_owned).collect::<Vec<String>>())
                .map_err(|error| error.to_string())?;
            validate_delimited_headers(headers.iter())?;
            let preview = reader
                .records()
                .take(4)
                .map(|record| record.map(|record| record.iter().map(str::to_owned).collect()))
                .collect::<Result<Vec<Vec<String>>, csv::Error>>()
                .map_err(|error| error.to_string())?;
            Ok((headers, preview))
        })();
        match result {
            Ok((headers, preview)) => {
                let signature = delimited_header_signature(&headers);
                let matching_specs: Vec<ImportSpecification> = self
                    .case
                    .import_specifications
                    .iter()
                    .filter(|spec| spec.header_signature == signature)
                    .cloned()
                    .collect();
                let matching_spec = (matching_specs.len() == 1).then(|| matching_specs[0].clone());
                self.selected_import_spec = matching_spec.as_ref().map(|spec| spec.id);
                self.import_spec_name = matching_spec
                    .as_ref()
                    .map(|spec| spec.name.clone())
                    .unwrap_or_default();
                let infer = |aliases: &[&str]| {
                    headers
                        .iter()
                        .find(|header| {
                            aliases
                                .iter()
                                .any(|alias| header.eq_ignore_ascii_case(alias))
                        })
                        .cloned()
                        .unwrap_or_default()
                };
                let inferred_mapping = ImportColumnMapping {
                    trim_whitespace: false,
                    row_filters: Vec::new(),
                    source: infer(&["source", "from", "entity_a"]),
                    target: infer(&["target", "to", "entity_b"]),
                    source_type: infer(&["source_type", "from_type", "entity_a_type"]),
                    target_type: infer(&["target_type", "to_type", "entity_b_type"]),
                    relationship: infer(&["relationship", "link", "type"]),
                    date: infer(&["date", "datetime"]),
                    confidence: infer(&["confidence"]),
                    direction: infer(&["directed", "direction"]),
                    connection_id: infer(&[
                        "connection_id",
                        "link_id",
                        "relationship_id",
                        "record_id",
                        "id",
                    ]),
                    notes: infer(&["notes", "description"]),
                    source_notes: infer(&["source_notes"]),
                    source_reference: infer(&[
                        "source_reference",
                        "evidence",
                        "reference",
                        "source_id",
                    ]),
                    ingest_source: infer(&["ingest_source"]),
                    record_type: infer(&["record_type"]),
                    source_entity_id: infer(&["source_entity_id"]),
                    target_entity_id: infer(&["target_entity_id"]),
                    source_external_ids: infer(&["source_external_ids_json"]),
                    target_external_ids: infer(&["target_external_ids_json"]),
                    source_detail: infer(&["source_detail_json"]),
                    target_detail: infer(&["target_detail_json"]),
                    source_aliases: infer(&["source_aliases_json"]),
                    target_aliases: infer(&["target_aliases_json"]),
                    source_color: infer(&["source_color"]),
                    target_color: infer(&["target_color"]),
                    source_x: infer(&["source_x"]),
                    source_y: infer(&["source_y"]),
                    target_x: infer(&["target_x"]),
                    target_y: infer(&["target_y"]),
                    source_position_locked: infer(&["source_position_locked"]),
                    target_position_locked: infer(&["target_position_locked"]),
                    entity_attributes: Vec::new(),
                };
                self.import_column_mapping = matching_spec
                    .map(|spec| {
                        let mut mapping = spec.mapping;
                        remap_import_mapping_headers(&mut mapping, &headers);
                        mapping
                    })
                    .unwrap_or(inferred_mapping);
                self.import_headers = headers;
                self.import_preview = preview;
                self.show_import_wizard = true;
                self.status = if matching_specs.len() > 1 {
                    format!(
                        "IMPORT DESIGN · {} COLUMNS DETECTED · SELECT A SAVED DESIGN",
                        self.import_headers.len()
                    )
                } else if self.selected_import_spec.is_some() {
                    format!(
                        "SAVED IMPORT DESIGN APPLIED · {} COLUMNS",
                        self.import_headers.len()
                    )
                } else {
                    format!(
                        "IMPORT DESIGN · {} COLUMNS DETECTED",
                        self.import_headers.len()
                    )
                };
            }
            Err(error) => self.status = format!("IMPORT INSPECTION FAILED · {error}"),
        }
    }

    fn save_import_specification(&mut self) {
        let name = self.import_spec_name.trim();
        if name.is_empty() {
            self.status = "IMPORT DESIGN NOT SAVED · enter a design name".into();
            return;
        }
        if self.case.import_specifications.iter().any(|spec| {
            spec.name.eq_ignore_ascii_case(name) && Some(spec.id) != self.selected_import_spec
        }) {
            self.status = "IMPORT DESIGN NOT SAVED · names must be unique within the case".into();
            return;
        }
        if let Some(spec) = self.selected_import_spec.and_then(|id| {
            self.case
                .import_specifications
                .iter_mut()
                .find(|spec| spec.id == id)
        }) {
            spec.name = name.to_owned();
            spec.mapping = self.import_column_mapping.clone();
            spec.header_signature = delimited_header_signature(&self.import_headers);
            self.status = format!("IMPORT DESIGN UPDATED · {name}");
        } else {
            let spec = ImportSpecification {
                id: Uuid::new_v4(),
                name: name.to_owned(),
                mapping: self.import_column_mapping.clone(),
                header_signature: delimited_header_signature(&self.import_headers),
            };
            self.selected_import_spec = Some(spec.id);
            self.case.import_specifications.push(spec);
            self.status = format!("IMPORT DESIGN SAVED · {name}");
        }
    }

    fn import_delimited(&mut self) {
        let import_path = self.import_path.clone();
        let mapping = self.import_column_mapping.clone();
        let result = (|| -> Result<(usize, usize, usize, Vec<Entity>, Vec<Link>), String> {
            let delimiter = delimited_text_delimiter(&import_path);
            let mut reader = csv::ReaderBuilder::new()
                .delimiter(delimiter)
                .from_path(&import_path)
                .map_err(|e| e.to_string())?;
            let headers = reader.headers().map_err(|e| e.to_string())?.clone();
            validate_delimited_headers(headers.iter())?;
            let col = |field: &str, names: &[&str]| {
                let selected = match field {
                    "source" => &mapping.source,
                    "target" => &mapping.target,
                    "source_type" => &mapping.source_type,
                    "target_type" => &mapping.target_type,
                    "relationship" => &mapping.relationship,
                    "date" => &mapping.date,
                    "confidence" => &mapping.confidence,
                    "direction" => &mapping.direction,
                    "connection_id" => &mapping.connection_id,
                    "notes" => &mapping.notes,
                    "source_notes" => &mapping.source_notes,
                    "source_reference" => &mapping.source_reference,
                    "ingest_source" => &mapping.ingest_source,
                    "record_type" => &mapping.record_type,
                    "source_entity_id" => &mapping.source_entity_id,
                    "target_entity_id" => &mapping.target_entity_id,
                    "source_external_ids" => &mapping.source_external_ids,
                    "target_external_ids" => &mapping.target_external_ids,
                    "source_detail" => &mapping.source_detail,
                    "target_detail" => &mapping.target_detail,
                    "source_aliases" => &mapping.source_aliases,
                    "target_aliases" => &mapping.target_aliases,
                    "source_color" => &mapping.source_color,
                    "target_color" => &mapping.target_color,
                    "source_x" => &mapping.source_x,
                    "source_y" => &mapping.source_y,
                    "target_x" => &mapping.target_x,
                    "target_y" => &mapping.target_y,
                    "source_position_locked" => &mapping.source_position_locked,
                    "target_position_locked" => &mapping.target_position_locked,
                    _ => unreachable!("unknown import mapping field"),
                };
                if !selected.is_empty() {
                    headers.iter().position(|header| header == selected)
                } else {
                    headers.iter().position(|header| {
                        names.iter().any(|name| header.eq_ignore_ascii_case(name))
                    })
                }
            };
            let record_type = col("record_type", &["record_type"]);
            let source_entity_id = col("source_entity_id", &["source_entity_id"]);
            let target_entity_id = col("target_entity_id", &["target_entity_id"]);
            let source_external_ids = col("source_external_ids", &["source_external_ids_json"]);
            let target_external_ids = col("target_external_ids", &["target_external_ids_json"]);
            let source = col("source", &["source", "from", "entity_a"])
                .ok_or("CSV needs a source column")?;
            let target = col("target", &["target", "to", "entity_b"]);
            if target.is_none() && record_type.is_none() {
                return Err("CSV needs a target column unless it contains record_type".into());
            }
            let rel = col("relationship", &["relationship", "link", "type"]);
            let date = col("date", &["date", "datetime"]);
            let confidence = col("confidence", &["confidence"]);
            let source_type = col(
                "source_type",
                &["source_type", "from_type", "entity_a_type"],
            );
            let target_type = col("target_type", &["target_type", "to_type", "entity_b_type"]);
            let source_detail = col("source_detail", &["source_detail_json"]);
            let target_detail = col("target_detail", &["target_detail_json"]);
            let source_aliases = col("source_aliases", &["source_aliases_json"]);
            let target_aliases = col("target_aliases", &["target_aliases_json"]);
            let source_color = col("source_color", &["source_color"]);
            let target_color = col("target_color", &["target_color"]);
            let source_x = col("source_x", &["source_x"]);
            let source_y = col("source_y", &["source_y"]);
            let target_x = col("target_x", &["target_x"]);
            let target_y = col("target_y", &["target_y"]);
            let source_position_locked = col("source_position_locked", &["source_position_locked"]);
            let target_position_locked = col("target_position_locked", &["target_position_locked"]);
            let direction = col("direction", &["directed", "direction"]);
            let notes = col("notes", &["notes", "description"]);
            let source_notes = col("source_notes", &["source_notes"]);
            let source_reference = col(
                "source_reference",
                &["source_reference", "evidence", "reference", "source_id"],
            );
            let connection_id = col(
                "connection_id",
                &[
                    "connection_id",
                    "link_id",
                    "relationship_id",
                    "record_id",
                    "id",
                ],
            );
            let ingest_source = col("ingest_source", &["ingest_source"]);
            let mut attribute_columns: Vec<(bool, String, usize)> = headers
                .iter()
                .enumerate()
                .filter_map(|(index, header)| {
                    let lower = header.to_ascii_lowercase();
                    [("source_attribute:", true), ("target_attribute:", false)]
                        .into_iter()
                        .find_map(|(prefix, is_source)| {
                            lower
                                .strip_prefix(prefix)
                                .filter(|name| !name.trim().is_empty())
                                .map(|_| {
                                    (is_source, header[prefix.len()..].trim().to_string(), index)
                                })
                        })
                })
                .collect();
            for attribute in &mapping.entity_attributes {
                if attribute.column.is_empty() && attribute.key.trim().is_empty() {
                    continue;
                }
                if attribute.column.is_empty() || attribute.key.trim().is_empty() {
                    return Err(
                        "mapped entity attributes need both a source column and a property name"
                            .into(),
                    );
                }
                let index = headers
                    .iter()
                    .position(|header| header.eq_ignore_ascii_case(&attribute.column))
                    .ok_or_else(|| {
                        format!(
                            "mapped entity attribute column is missing: {}",
                            attribute.column
                        )
                    })?;
                if attribute_columns.iter().any(|(source_side, key, _)| {
                    *source_side == attribute.source
                        && key.eq_ignore_ascii_case(attribute.key.trim())
                }) {
                    return Err(format!(
                        "entity attribute is mapped more than once: {}",
                        attribute.key.trim()
                    ));
                }
                attribute_columns.push((attribute.source, attribute.key.trim().to_owned(), index));
            }
            let mut row_filters = Vec::new();
            for filter in &mapping.row_filters {
                if filter.column.is_empty() && filter.value.is_empty() {
                    continue;
                }
                if filter.column.is_empty() {
                    return Err("row filters need a column".into());
                }
                let index = headers
                    .iter()
                    .position(|header| header.eq_ignore_ascii_case(&filter.column))
                    .ok_or_else(|| format!("row filter column is missing: {}", filter.column))?;
                row_filters.push((index, filter.operator, filter.value.clone()));
            }
            let mut created = 0;
            let mut updated = 0;
            let mut skipped = 0;
            let mut imported_ids = HashSet::new();
            let mut imported_natural_keys = HashSet::new();
            let mut imported_entity_values: HashMap<(Uuid, String), String> = HashMap::new();
            let immutable_sources: HashSet<String> = self
                .case
                .snapshots
                .iter()
                .map(|snapshot| snapshot.source_key.clone())
                .collect();
            let mut entities = self.case.entities.clone();
            let mut entity_index = EntityIndex::new(&entities);
            let mut links = self.case.links.clone();
            let mut links_by_id: HashMap<(Option<String>, String), Vec<usize>> = HashMap::new();
            let mut links_by_natural_key: HashMap<
                (String, String, String, String, bool),
                Vec<usize>,
            > = HashMap::new();
            for (index, link) in links.iter().enumerate() {
                if !link.connection_id.is_empty() {
                    links_by_id
                        .entry((link.ingest_source.clone(), link.connection_id.clone()))
                        .or_default()
                        .push(index);
                } else if let Some(ingest_source) = &link.ingest_source {
                    let source_name = entity_index
                        .by_id
                        .get(&link.from)
                        .and_then(|position| entities.get(*position))
                        .map(|entity| entity.name.as_str());
                    let target_name = entity_index
                        .by_id
                        .get(&link.to)
                        .and_then(|position| entities.get(*position))
                        .map(|entity| entity.name.as_str());
                    if let Some((source_name, target_name)) = source_name.zip(target_name) {
                        links_by_natural_key
                            .entry(indexed_link_natural_key(
                                ingest_source,
                                source_name,
                                target_name,
                                &link.label,
                                link.directed,
                            ))
                            .or_default()
                            .push(index);
                    }
                }
            }
            for row in reader.records() {
                let row = row.map_err(|e| e.to_string())?;
                let row: csv::StringRecord = if mapping.trim_whitespace {
                    row.iter().map(str::trim).collect()
                } else {
                    row
                };
                let included = row_filters.iter().all(|(index, operator, expected)| {
                    let value = row.get(*index).unwrap_or("");
                    match operator {
                        ImportFilterOperator::Equals => value == expected,
                        ImportFilterOperator::Contains => value.contains(expected),
                        ImportFilterOperator::IsEmpty => value.trim().is_empty(),
                        ImportFilterOperator::IsNotEmpty => !value.trim().is_empty(),
                    }
                });
                if !included {
                    skipped += 1;
                    continue;
                }
                let row_type = record_type
                    .and_then(|index| row.get(index))
                    .unwrap_or("connection")
                    .trim();
                let entity_only = row_type.eq_ignore_ascii_case("entity");
                if !entity_only
                    && !row_type.is_empty()
                    && !row_type.eq_ignore_ascii_case("connection")
                {
                    return Err(format!("CSV record_type is unsupported: {row_type}"));
                }
                let source_name = row.get(source).unwrap_or("").trim();
                let target_name = target.and_then(|index| row.get(index)).unwrap_or("").trim();
                if source_name.is_empty() || (!entity_only && target_name.is_empty()) {
                    skipped += 1;
                    continue;
                }
                let external_id = connection_id
                    .and_then(|i| row.get(i))
                    .unwrap_or("")
                    .trim()
                    .to_owned();
                let ingest_source = ingest_source
                    .and_then(|i| row.get(i))
                    .map(str::trim)
                    .filter(|source| !source.is_empty())
                    .map(str::to_owned);
                if !external_id.is_empty()
                    && !imported_ids.insert((ingest_source.clone(), external_id.clone()))
                {
                    return Err(format!(
                        "CSV contains duplicate connection_id: {external_id}"
                    ));
                }
                let source_type_value = source_type.and_then(|i| row.get(i)).unwrap_or("").trim();
                let target_type_value = target_type.and_then(|i| row.get(i)).unwrap_or("").trim();
                let parse_entity_id =
                    |column: Option<usize>, side: &str| -> Result<Option<Uuid>, String> {
                        column
                            .and_then(|index| row.get(index))
                            .map(str::trim)
                            .filter(|value| !value.is_empty())
                            .map(|value| {
                                Uuid::parse_str(value).map_err(|error| {
                                    format!("CSV {side}_entity_id is invalid: {error}")
                                })
                            })
                            .transpose()
                    };
                let source_id = parse_entity_id(source_entity_id, "source")?;
                let target_id = if entity_only {
                    source_id
                } else {
                    parse_entity_id(target_entity_id, "target")?
                };
                let a = entity_index.find_or_add_entity_with_id(
                    source_name,
                    source_type_value,
                    source_id,
                    &mut entities,
                )?;
                let b = if entity_only {
                    a
                } else {
                    entity_index.find_or_add_entity_with_id(
                        target_name,
                        target_type_value,
                        target_id,
                        &mut entities,
                    )?
                };
                for (entity_id, is_source) in [(a, true), (b, false)] {
                    let entity_position = entity_index.by_id[&entity_id];
                    let entity_name = entities[entity_position].name.clone();
                    let external_ids_column = if is_source {
                        source_external_ids
                    } else {
                        target_external_ids
                    };
                    if let Some(index) = external_ids_column {
                        let encoded = row.get(index).unwrap_or("");
                        if !encoded.is_empty() {
                            let external_ids = serde_json::from_str::<Vec<(String, String)>>(encoded)
                                .map_err(|error| format!("CSV has invalid {}_external_ids_json for {entity_name}: {error}", if is_source { "source" } else { "target" }))?;
                            let mut normalized = Vec::new();
                            for (source, external_id) in external_ids {
                                let source = source.trim().to_owned();
                                let external_id = external_id.trim().to_owned();
                                if source.is_empty() || external_id.is_empty() {
                                    return Err(format!(
                                        "CSV has an empty {} external identity for {entity_name}",
                                        if is_source { "source" } else { "target" }
                                    ));
                                }
                                if !normalized.contains(&(source.clone(), external_id.clone())) {
                                    normalized.push((source, external_id));
                                }
                            }
                            let canonical = serde_json::to_string(&normalized)
                                .map_err(|error| error.to_string())?;
                            record_imported_entity_value(
                                &mut imported_entity_values,
                                entity_id,
                                "external_ids",
                                &canonical,
                                &entity_name,
                            )?;
                            entities[entity_position].external_ids = normalized;
                        }
                    }
                    let color_column = if is_source {
                        source_color
                    } else {
                        target_color
                    };
                    if let Some(index) = color_column {
                        let value = row.get(index).unwrap_or("").trim();
                        if !value.is_empty() {
                            let color = parse_entity_color(value).map_err(|error| {
                                format!(
                                    "CSV has invalid {}_color for {entity_name}: {error}",
                                    if is_source { "source" } else { "target" }
                                )
                            })?;
                            let canonical_color =
                                format!("#{:02X}{:02X}{:02X}", color[0], color[1], color[2]);
                            record_imported_entity_value(
                                &mut imported_entity_values,
                                entity_id,
                                "color",
                                &canonical_color,
                                &entity_name,
                            )?;
                            entities[entity_position].color = color;
                        }
                    }
                    for (column, key) in [
                        (if is_source { source_x } else { target_x }, "x"),
                        (if is_source { source_y } else { target_y }, "y"),
                    ] {
                        if let Some(index) = column {
                            let value = row.get(index).unwrap_or("").trim();
                            if !value.is_empty() {
                                let position = parse_entity_position(value).map_err(|error| {
                                    format!(
                                        "CSV has invalid {}_{key} for {entity_name}: {error}",
                                        if is_source { "source" } else { "target" }
                                    )
                                })?;
                                let canonical_position = position.to_string();
                                record_imported_entity_value(
                                    &mut imported_entity_values,
                                    entity_id,
                                    key,
                                    &canonical_position,
                                    &entity_name,
                                )?;
                                match key {
                                    "x" => entities[entity_position].x = position,
                                    _ => entities[entity_position].y = position,
                                }
                            }
                        }
                    }
                    let lock_column = if is_source {
                        source_position_locked
                    } else {
                        target_position_locked
                    };
                    if let Some(index) = lock_column {
                        let value = row.get(index).unwrap_or("").trim();
                        if !value.is_empty() {
                            let locked = parse_import_bool(value).map_err(|error| {
                                format!(
                                    "CSV has invalid {}_position_locked for {entity_name}: {error}",
                                    if is_source { "source" } else { "target" }
                                )
                            })?;
                            record_imported_entity_value(
                                &mut imported_entity_values,
                                entity_id,
                                "position_locked",
                                if locked { "true" } else { "false" },
                                &entity_name,
                            )?;
                            entities[entity_position].position_locked = locked;
                        }
                    }
                    let detail_column = if is_source {
                        source_detail
                    } else {
                        target_detail
                    };
                    if let Some(index) = detail_column {
                        let encoded = row.get(index).unwrap_or("");
                        if !encoded.is_empty() {
                            let detail =
                                serde_json::from_str::<String>(encoded).map_err(|error| {
                                    format!(
                                        "CSV has invalid {}_detail_json for {entity_name}: {error}",
                                        if is_source { "source" } else { "target" }
                                    )
                                })?;
                            record_imported_entity_value(
                                &mut imported_entity_values,
                                entity_id,
                                "detail",
                                &detail,
                                &entity_name,
                            )?;
                            entities[entity_position].detail = detail;
                        }
                    }
                    let aliases_column = if is_source {
                        source_aliases
                    } else {
                        target_aliases
                    };
                    if let Some(index) = aliases_column {
                        let encoded = row.get(index).unwrap_or("");
                        if !encoded.is_empty() {
                            let aliases = serde_json::from_str::<Vec<String>>(encoded).map_err(|error| {
                                format!("CSV has invalid {}_aliases_json for {entity_name}: {error}", if is_source {"source"} else {"target"})
                            })?;
                            let canonical_aliases = serde_json::to_string(&aliases)
                                .map_err(|error| error.to_string())?;
                            record_imported_entity_value(
                                &mut imported_entity_values,
                                entity_id,
                                "aliases",
                                &canonical_aliases,
                                &entity_name,
                            )?;
                            entities[entity_position].aliases = aliases;
                            entity_index.refresh_aliases(entity_id, &entities);
                        }
                    }
                    for (attribute_is_source, name, index) in &attribute_columns {
                        if *attribute_is_source != is_source {
                            continue;
                        }
                        let value = row.get(*index).unwrap_or("").trim();
                        if value.is_empty() {
                            continue;
                        }
                        record_imported_entity_value(
                            &mut imported_entity_values,
                            entity_id,
                            &format!("attribute:{name}"),
                            value,
                            &entity_name,
                        )?;
                        let entity = &mut entities[entity_position];
                        if let Some((_, existing)) = entity
                            .attributes
                            .iter_mut()
                            .find(|(key, _)| key.eq_ignore_ascii_case(name))
                        {
                            *existing = value.into();
                        } else {
                            entity.attributes.push((name.clone(), value.into()));
                        }
                    }
                }
                if entity_only {
                    continue;
                }
                let confidence_value = confidence.and_then(|i| row.get(i)).unwrap_or("").trim();
                let confidence = if confidence_value.is_empty() {
                    2
                } else {
                    confidence_value
                        .parse::<u8>()
                        .ok()
                        .filter(|value| (1..=3).contains(value))
                        .ok_or_else(|| {
                            format!(
                                "CSV confidence must be an integer from 1 to 3, got: {confidence_value}"
                            )
                        })?
                };
                let direction_value = direction
                    .and_then(|i| row.get(i))
                    .unwrap_or("")
                    .trim()
                    .to_ascii_lowercase();
                let directed = match direction_value.as_str() {
                    "" | "true" | "1" | "yes" | "directed" | "→" => true,
                    "false" | "0" | "no" | "undirected" | "↔" => false,
                    value => return Err(format!("CSV direction has unsupported value: {value}")),
                };
                let relationship = rel
                    .and_then(|i| row.get(i))
                    .filter(|value| !value.is_empty())
                    .unwrap_or("ASSOCIATED WITH");
                let natural_key = canonical_connection(
                    source_name,
                    source_type_value,
                    target_name,
                    target_type_value,
                    relationship,
                    directed,
                )?
                .4;
                if external_id.is_empty()
                    && ingest_source.is_some()
                    && !imported_natural_keys.insert((ingest_source.clone(), natural_key.clone()))
                {
                    return Err(format!(
                        "CSV contains ambiguous duplicate connection without connection_id: {} — {} — {}",
                        source_name, relationship, target_name
                    ));
                }
                let mut imported = Link {
                    from: a,
                    to: b,
                    connection_id: external_id.clone(),
                    label: relationship.into(),
                    date: date.and_then(|i| row.get(i)).unwrap_or("").into(),
                    confidence,
                    directed,
                    notes: notes.and_then(|i| row.get(i)).unwrap_or("").into(),
                    source_notes: source_notes.and_then(|i| row.get(i)).unwrap_or("").into(),
                    source_reference: source_reference
                        .and_then(|i| row.get(i))
                        .unwrap_or("")
                        .into(),
                    ingest_source: ingest_source.clone(),
                };
                let existing: Vec<usize> = if !external_id.is_empty() {
                    links_by_id
                        .get(&(ingest_source.clone(), external_id.clone()))
                        .cloned()
                        .unwrap_or_default()
                } else if let Some(source_key) = &ingest_source {
                    links_by_natural_key
                        .get(&indexed_link_natural_key(
                            source_key,
                            source_name,
                            target_name,
                            relationship,
                            directed,
                        ))
                        .into_iter()
                        .flatten()
                        .copied()
                        .filter(|index| {
                            let link = &links[*index];
                            let Some(source_entity) = entity_index
                                .by_id
                                .get(&link.from)
                                .and_then(|position| entities.get(*position))
                            else {
                                return false;
                            };
                            let Some(target_entity) = entity_index
                                .by_id
                                .get(&link.to)
                                .and_then(|position| entities.get(*position))
                            else {
                                return false;
                            };
                            canonical_connection(
                                &source_entity.name,
                                &source_entity.kind,
                                &target_entity.name,
                                &target_entity.kind,
                                &link.label,
                                link.directed,
                            )
                            .is_ok_and(|(_, _, _, _, key)| key == natural_key)
                        })
                        .collect()
                } else {
                    Vec::new()
                };
                if source_notes.is_none()
                    && let [index] = existing.as_slice()
                {
                    imported.source_notes = links[*index].source_notes.clone();
                }
                if ingest_source
                    .as_ref()
                    .is_some_and(|source| immutable_sources.contains(source))
                {
                    match existing.as_slice() {
                        [index] if !same_sql_connection(&links[*index], &imported) => {
                            return Err(format!(
                                "CSV cannot modify SQL-owned connection from {external_id_or_key}; ingest the change from SQLite",
                                external_id_or_key = if external_id.is_empty() {
                                    format!(
                                        "natural key {} — {} — {}",
                                        source_name, relationship, target_name
                                    )
                                } else {
                                    format!("connection_id {external_id}")
                                }
                            ));
                        }
                        [] => {
                            return Err(format!(
                                "CSV contains a new SQL-owned connection from {source}; ingest it from SQLite"
                            ));
                        }
                        _ => {}
                    }
                }
                match existing.as_slice() {
                    [index] => {
                        links[*index] = imported;
                        index_imported_link(
                            &mut links_by_id,
                            &mut links_by_natural_key,
                            *index,
                            &links[*index],
                            &entities,
                            &entity_index,
                        );
                        updated += 1;
                    }
                    [] => {
                        links.push(imported);
                        let index = links.len() - 1;
                        index_imported_link(
                            &mut links_by_id,
                            &mut links_by_natural_key,
                            index,
                            &links[index],
                            &entities,
                            &entity_index,
                        );
                        created += 1;
                    }
                    _ => {
                        return Err(if external_id.is_empty() {
                            format!(
                                "case contains multiple provenance-matched natural-key connections: {} — {} — {}",
                                source_name, relationship, target_name
                            )
                        } else {
                            format!(
                                "case contains multiple connections with connection_id: {external_id}"
                            )
                        });
                    }
                }
            }
            let mut imported_entity_identities = HashMap::<(String, String), Uuid>::new();
            for entity in &entities {
                for identity in &entity.external_ids {
                    if let Some(previous) =
                        imported_entity_identities.insert(identity.clone(), entity.id)
                        && previous != entity.id
                    {
                        return Err(format!(
                            "CSV assigns source entity identity {} to multiple chart entities",
                            identity.1
                        ));
                    }
                }
            }
            Ok((created, updated, skipped, entities, links))
        })();
        self.status = match result {
            Ok((created, updated, skipped, entities, links)) => {
                self.case.entities = entities;
                self.case.links = links;
                self.invalidate_network_groups();
                self.path_result.clear();
                self.duplicate_candidates.clear();
                self.merge_candidate = None;
                format!(
                    "{} IMPORT · {created} NEW · {updated} UPDATED · {skipped} SKIPPED",
                    delimited_text_format(&self.import_path)
                )
            }
            Err(e) => format!("IMPORT FAILED · {e}"),
        };
    }

    fn ingest_sqlite_snapshot(&mut self) {
        self.sql_validation_preview = None;
        let configured_path = self.sql_path.trim().to_string();
        let case_path = self.case_path.clone();
        let series = self.sql_series.trim().to_string();
        let query = self.sql_query.trim().to_string();
        let parameters = parse_sql_parameters(&self.sql_parameter_text);
        let previously_selected = self
            .selected_link
            .and_then(|index| self.case.links.get(index))
            .cloned();
        let result = (|| -> Result<(Vec<Entity>, Vec<Link>, ConnectionSnapshot), String> {
            if configured_path.is_empty() {
                return Err("database path cannot be empty".into());
            }
            if series.is_empty() {
                return Err("snapshot series cannot be empty".into());
            }
            if query.is_empty() {
                return Err("SQL query cannot be empty".into());
            }
            let parameters = parameters?;
            let path = resolve_sqlite_database_path(&configured_path, &case_path)?;
            let path = path.to_string_lossy().into_owned();
            // Enforce read-only access at the SQLite open boundary for every configured job.
            let mut connection =
                Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(|e| e.to_string())?;
            connection
                .busy_timeout(Duration::from_secs(5))
                .map_err(|error| error.to_string())?;
            let interrupt_handle = connection.get_interrupt_handle();
            let transaction = connection
                .transaction_with_behavior(TransactionBehavior::Deferred)
                .map_err(|error| error.to_string())?;
            let captured_at: String = transaction
                .query_row("SELECT strftime('%Y-%m-%dT%H:%M:%fZ','now')", [], |row| {
                    row.get(0)
                })
                .map_err(|e| e.to_string())?;
            let mut statement = transaction.prepare(&query).map_err(|e| e.to_string())?;
            if !statement.readonly() {
                return Err("SQL query must be read-only".into());
            }
            let parameter_values = sql_parameter_values(&statement, &parameters)?;
            let column_names: Vec<String> = statement
                .column_names()
                .map(|name| name.trim().to_ascii_lowercase())
                .collect();
            let col = |field: &str, names: &[&str]| -> Result<Option<usize>, String> {
                let matches: Vec<usize> = column_names
                    .iter()
                    .enumerate()
                    .filter_map(|(index, column)| {
                        names.iter().any(|name| column == name).then_some(index)
                    })
                    .collect();
                if matches.len() > 1 {
                    return Err(format!(
                        "SQL result has ambiguous {field} columns: {}",
                        matches
                            .iter()
                            .map(|index| column_names[*index].as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    ));
                }
                Ok(matches.first().copied())
            };
            let source = col("source", &["source", "from", "entity_a"])?
                .ok_or("SQL result needs a source column")?;
            let target = col("target", &["target", "to", "entity_b"])?
                .ok_or("SQL result needs a target column")?;
            let source_entity_id = col(
                "source entity ID",
                &[
                    "source_entity_id",
                    "source_external_id",
                    "from_entity_id",
                    "from_id",
                    "entity_a_id",
                    "source_person_id",
                    "from_person_id",
                    "source_node_id",
                    "from_node_id",
                ],
            )?;
            let target_entity_id = col(
                "target entity ID",
                &[
                    "target_entity_id",
                    "target_external_id",
                    "target_id",
                    "to_entity_id",
                    "to_id",
                    "entity_b_id",
                    "target_person_id",
                    "to_person_id",
                    "target_node_id",
                    "to_node_id",
                ],
            )?;
            let connection_id = col(
                "connection ID",
                &["connection_id", "link_id", "record_id", "id"],
            )?;
            let source_type = col(
                "source type",
                &["source_type", "from_type", "entity_a_type"],
            )?;
            let target_type = col("target type", &["target_type", "to_type", "entity_b_type"])?;
            let relationship = col("relationship", &["relationship", "link_type", "relation"])?;
            let date = col("date", &["date", "event_date", "datetime", "timestamp"])?;
            let confidence = col("confidence", &["confidence", "strength"])?;
            let directed = col("direction", &["directed", "is_directed", "direction"])?;
            let source_reference = col(
                "source reference",
                &["source_reference", "evidence", "reference", "source_id"],
            )?;
            let notes = col("notes", &["notes", "description"])?;
            let mut facts = Vec::new();
            let mut explicit_ids = HashSet::new();
            let mut skipped_rows = 0;
            let query_deadline = SqlQueryDeadline::new(interrupt_handle);
            let mut rows = statement
                .query(params_from_iter(parameter_values.iter()))
                .map_err(|error| {
                    if query_deadline.expired() {
                        sql_query_timeout_message()
                    } else {
                        error.to_string()
                    }
                })?;
            loop {
                let row = match rows.next() {
                    Ok(Some(row)) => row,
                    Ok(None) => break,
                    Err(_error) if query_deadline.expired() => {
                        return Err(sql_query_timeout_message());
                    }
                    Err(error) => return Err(error.to_string()),
                };
                let source_name = sql_value(row, Some(source))
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
                let target_name = sql_value(row, Some(target))
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
                let source_entity_id = sql_value(row, source_entity_id)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
                let target_entity_id = sql_value(row, target_entity_id)
                    .map_err(|e| e.to_string())?
                    .trim()
                    .to_string();
                if source_name.is_empty() || target_name.is_empty() {
                    skipped_rows += 1;
                    continue;
                }
                let source_kind = sql_value(row, source_type).map_err(|e| e.to_string())?;
                let target_kind = sql_value(row, target_type).map_err(|e| e.to_string())?;
                let relationship = sql_value(row, relationship).map_err(|e| e.to_string())?;
                let date = sql_value(row, date).map_err(|e| e.to_string())?;
                let confidence_value = sql_value(row, confidence).map_err(|e| e.to_string())?;
                let confidence = if confidence_value.trim().is_empty() {
                    2
                } else {
                    confidence_value
                        .parse::<u8>()
                        .ok()
                        .filter(|value| (1..=3).contains(value))
                        .ok_or_else(|| {
                            format!(
                                "SQL confidence must be an integer from 1 to 3, got: {confidence_value}"
                            )
                        })?
                };
                let directed_text = sql_value(row, directed)
                    .map_err(|e| e.to_string())?
                    .to_ascii_lowercase();
                let is_directed = match directed_text.trim() {
                    "" | "true" | "1" | "yes" | "directed" | "→" => true,
                    "false" | "0" | "no" | "undirected" | "↔" => false,
                    value => return Err(format!("SQL direction has unsupported value: {value}")),
                };
                let source_reference =
                    sql_value(row, source_reference).map_err(|e| e.to_string())?;
                let notes = sql_value(row, notes).map_err(|e| e.to_string())?;
                let explicit_id = sql_value(row, connection_id).map_err(|e| e.to_string())?;
                if !explicit_id.trim().is_empty()
                    && !explicit_ids.insert(explicit_id.trim().to_owned())
                {
                    return Err(format!(
                        "SQL result contains duplicate connection_id: {}",
                        explicit_id.trim()
                    ));
                }
                let endpoints_swapped = !is_directed
                    && (source_name.to_lowercase(), source_kind.to_lowercase())
                        > (target_name.to_lowercase(), target_kind.to_lowercase());
                let (source_name, source_kind, target_name, target_kind, natural_key) =
                    canonical_connection(
                        &source_name,
                        &source_kind,
                        &target_name,
                        &target_kind,
                        &relationship,
                        is_directed,
                    )?;
                let (source_entity_id, target_entity_id) = if endpoints_swapped {
                    (target_entity_id, source_entity_id)
                } else {
                    (source_entity_id, target_entity_id)
                };
                let identity = if explicit_id.trim().is_empty() {
                    if source_entity_id.trim().is_empty() && target_entity_id.trim().is_empty() {
                        format!("natural-v2:{natural_key}")
                    } else {
                        format!(
                            "entity-v1:{}",
                            entity_identity_connection_key(
                                &source_name,
                                &source_kind,
                                &source_entity_id,
                                &target_name,
                                &target_kind,
                                &target_entity_id,
                                &relationship,
                                is_directed,
                            )
                        )
                    }
                } else {
                    format!("id:{}", explicit_id.trim())
                };
                facts.push(ConnectionFact {
                    key: identity,
                    connection_id: explicit_id.trim().to_owned(),
                    source_entity_id,
                    target_entity_id,
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
            if query_deadline.expired() {
                return Err(sql_query_timeout_message());
            }
            drop(query_deadline);
            drop(rows);
            drop(statement);
            transaction.commit().map_err(|error| error.to_string())?;
            let mut natural_ids = HashSet::new();
            for fact in &facts {
                if fact.connection_id.is_empty() && !natural_ids.insert(fact.key.clone()) {
                    return Err(format!(
                        "SQL result contains ambiguous duplicate connection without connection_id: {} — {} — {}",
                        fact.source, fact.relationship, fact.target
                    ));
                }
            }
            // Stage chart updates separately and swap them in only after the entire query succeeds.
            let source_key = sql_source_key(&path, &series);
            let analyst_notes: HashMap<String, String> = self
                .case
                .links
                .iter()
                .filter(|link| link.ingest_source.as_deref() == Some(source_key.as_str()))
                .map(|link| (link_refresh_key(link), link.notes.clone()))
                .collect();
            let mut entities = self.case.entities.clone();
            let mut entity_index = EntityIndex::new(&entities);
            let mut links = self.case.links.clone();
            links.retain(|link| link.ingest_source.as_deref() != Some(source_key.as_str()));
            for fact in &facts {
                let from = entity_index.find_or_add_entity_with_external_id(
                    &fact.source,
                    &fact.source_type,
                    &path,
                    &fact.source_entity_id,
                    &mut entities,
                )?;
                let to = entity_index.find_or_add_entity_with_external_id(
                    &fact.target,
                    &fact.target_type,
                    &path,
                    &fact.target_entity_id,
                    &mut entities,
                )?;
                links.push(Link {
                    from,
                    to,
                    connection_id: fact.connection_id.clone(),
                    label: fact.relationship.clone(),
                    date: fact.date.clone(),
                    confidence: fact.confidence,
                    directed: fact.directed,
                    notes: analyst_notes
                        .get(&connection_refresh_key(
                            &fact.connection_id,
                            from,
                            to,
                            &fact.relationship,
                            fact.directed,
                        ))
                        .cloned()
                        .unwrap_or_default(),
                    source_notes: fact.notes.clone(),
                    source_reference: fact.source_reference.clone(),
                    ingest_source: Some(source_key.clone()),
                });
            }
            let snapshot = ConnectionSnapshot {
                id: Uuid::new_v4(),
                series,
                source_key,
                database_path: path,
                configured_database_path: configured_path,
                query,
                parameters,
                captured_at,
                connections: facts,
                skipped_rows,
            };
            Ok((entities, links, snapshot))
        })();
        match result {
            Ok((entities, links, snapshot)) => {
                let count = snapshot.connections.len();
                let skipped_rows = snapshot.skipped_rows;
                self.sql_snapshot_source = Some(snapshot.source_key.clone());
                self.selected_link = previously_selected
                    .as_ref()
                    .and_then(|selected| refreshed_link_index(&links, selected));
                self.case.entities = entities;
                self.case.links = links;
                self.invalidate_network_groups();
                self.case.snapshots.push(snapshot);
                self.sql_snapshot_index = Some(self.case.snapshots.len() - 1);
                self.path_result.clear();
                self.status = if skipped_rows == 0 {
                    format!("SQL SNAPSHOT INGESTED · {count} CONNECTIONS")
                } else {
                    format!(
                        "SQL SNAPSHOT INGESTED · {count} CONNECTIONS · {skipped_rows} ROWS SKIPPED (EMPTY ENDPOINT)"
                    )
                };
            }
            Err(error) => self.status = format!("SQL INGEST FAILED · {error}"),
        }
    }

    fn build_sql_query_from_schema(&mut self) {
        let result = (|| -> Result<Vec<(String, String)>, String> {
            if self.sql_path.trim().is_empty() {
                return Err("database path cannot be empty".into());
            }
            let path = resolve_sqlite_database_path(&self.sql_path, &self.case_path)?;
            let schema = read_sqlite_schema(&path.to_string_lossy())?;
            let templates = sqlite_query_templates(&schema);
            if templates.is_empty() {
                return Err("no table or view has recognizable source and target columns".into());
            }
            Ok(templates)
        })();
        match result {
            Ok(templates) => {
                self.sql_schema_templates = templates;
                self.sql_schema_template_index = 0;
                self.sql_schema_template_source =
                    Some((self.sql_path.clone(), self.case_path.clone()));
                self.apply_sql_schema_template();
            }
            Err(error) => {
                self.sql_schema_templates.clear();
                self.sql_schema_template_index = 0;
                self.sql_schema_template_source = None;
                self.status = format!("SQL SCHEMA INSPECTION FAILED · {error}");
            }
        }
    }

    fn apply_sql_schema_template(&mut self) {
        let Some((object_name, query)) = self
            .sql_schema_templates
            .get(self.sql_schema_template_index)
            .cloned()
        else {
            return;
        };
        self.sql_query = query;
        self.sql_parameter_text.clear();
        self.sql_validation_preview = None;
        self.status = format!("SQL QUERY BUILT FROM {object_name}");
    }

    fn show_sql_schema_template_picker(&mut self, ui: &mut egui::Ui) {
        let current_source = (self.sql_path.clone(), self.case_path.clone());
        if self.sql_schema_template_source.as_ref() != Some(&current_source) {
            self.sql_schema_templates.clear();
            self.sql_schema_template_index = 0;
            self.sql_schema_template_source = None;
            return;
        }
        if self.sql_schema_templates.len() < 2 {
            return;
        }
        let selected_text = self
            .sql_schema_templates
            .get(self.sql_schema_template_index)
            .map(|(table, _)| table.as_str())
            .unwrap_or("Choose table or view");
        let mut selected_index = self.sql_schema_template_index;
        egui::ComboBox::from_id_salt("sql_schema_template")
            .selected_text(selected_text)
            .show_ui(ui, |ui| {
                for (index, (table, _)) in self.sql_schema_templates.iter().enumerate() {
                    ui.selectable_value(&mut selected_index, index, table);
                }
            });
        if selected_index != self.sql_schema_template_index {
            self.sql_schema_template_index = selected_index;
            self.apply_sql_schema_template();
        }
    }

    fn save_automation_job(&mut self) {
        let name = self.automation_job_name.trim().to_string();
        let parameters = match parse_sql_parameters(&self.sql_parameter_text) {
            Ok(parameters) => parameters,
            Err(error) => {
                self.status = format!("JOB NOT SAVED · {error}");
                return;
            }
        };
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
        if let Some(existing) = self.case.automation_jobs.iter().find(|job| {
            Some(job.id) != self.editing_automation_job
                && same_sql_series(
                    &job.database_path,
                    &job.series,
                    self.sql_path.trim(),
                    self.sql_series.trim(),
                    &self.case_path,
                )
        }) {
            self.status = format!(
                "JOB NOT SAVED · database and series already belong to {}",
                existing.name
            );
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
                job.parameters = parameters;
                job.minimum_connections = self.automation_minimum_connections;
                job.maximum_connections = self.automation_maximum_connections;
                job.maximum_regressions = self.automation_maximum_regressions;
                job.maximum_skipped_rows = self.automation_maximum_skipped_rows;
                self.editing_automation_job = None;
                self.automation_editor_before_edit = None;
                self.status = format!("AUTOMATION JOB UPDATED · {name}");
                return;
            }
            self.status = "JOB NOT SAVED · the edited job no longer exists".into();
            return;
        }
        self.case.automation_jobs.push(AutomationJob {
            id: Uuid::new_v4(),
            name: name.clone(),
            database_path: self.sql_path.trim().into(),
            series: self.sql_series.trim().into(),
            query: self.sql_query.trim().into(),
            parameters,
            enabled: true,
            minimum_connections: self.automation_minimum_connections,
            maximum_connections: self.automation_maximum_connections,
            maximum_regressions: self.automation_maximum_regressions,
            maximum_skipped_rows: self.automation_maximum_skipped_rows,
        });
        self.editing_automation_job = None;
        self.automation_editor_before_edit = None;
        self.automation_job_name.clear();
        self.status = format!("AUTOMATION JOB ADDED · {name}");
    }

    fn validate_sql_query(&mut self) -> bool {
        if self.data_mode == DataMode::Automation
            && let Some(existing) = self.case.automation_jobs.iter().find(|job| {
                Some(job.id) != self.editing_automation_job
                    && same_sql_series(
                        &job.database_path,
                        &job.series,
                        self.sql_path.trim(),
                        self.sql_series.trim(),
                        &self.case_path,
                    )
            })
        {
            self.status = format!(
                "SQL QUERY INVALID · database and series already belong to {}",
                existing.name
            );
            return false;
        }
        self.validate_sql_query_with_snapshot_context(false)
    }

    fn validate_sql_query_for_batch(&mut self) -> bool {
        self.validate_sql_query_with_snapshot_context(true)
    }

    fn validate_sql_query_with_snapshot_context(&mut self, keep_candidate_snapshot: bool) -> bool {
        let case_before = self.case.clone();
        let selected_link_before = self.selected_link;
        let snapshot_source_before = self.sql_snapshot_source.clone();
        let snapshot_index_before = self.sql_snapshot_index;
        let path_result_before = self.path_result.clone();
        let snapshot_count_before = self.case.snapshots.len();

        self.ingest_sqlite_snapshot();
        let candidate_snapshot = self.case.snapshots.get(snapshot_count_before).cloned();
        let validation_preview = candidate_snapshot
            .as_ref()
            .map(|snapshot| SqlValidationPreview {
                configured_database_path: snapshot.configured_database_path.clone(),
                series: snapshot.series.clone(),
                query: snapshot.query.clone(),
                parameters: snapshot.parameters.clone(),
                captured_at: snapshot.captured_at.clone(),
                connection_count: snapshot.connections.len(),
                skipped_rows: snapshot.skipped_rows,
                connections: snapshot.connections.iter().take(8).cloned().collect(),
            });
        let validation_message = if self.status.starts_with("SQL SNAPSHOT INGESTED") {
            let snapshot = candidate_snapshot.as_ref();
            let count = snapshot
                .as_ref()
                .map(|snapshot| snapshot.connections.len())
                .unwrap_or_default();
            let skipped_rows = snapshot
                .as_ref()
                .map(|snapshot| snapshot.skipped_rows)
                .unwrap_or_default();
            let skipped = if skipped_rows == 0 {
                String::new()
            } else {
                format!(" · {skipped_rows} ROWS SKIPPED (EMPTY ENDPOINT)")
            };
            let skipped_row_threshold = match self.automation_maximum_skipped_rows {
                Some(maximum) if skipped_rows > maximum => {
                    format!(" · SKIPPED ROW LIMIT EXCEEDED ({skipped_rows}; MAXIMUM {maximum})")
                }
                Some(maximum) => {
                    format!(" · SKIPPED ROW LIMIT MET ({skipped_rows}; MAXIMUM {maximum})")
                }
                None => String::new(),
            };
            let maximum_connection_threshold = match self.automation_maximum_connections {
                Some(maximum) if count > maximum => {
                    format!(" · CONNECTION LIMIT EXCEEDED ({count}; MAXIMUM {maximum})")
                }
                Some(maximum) => format!(" · CONNECTION LIMIT MET ({count}; MAXIMUM {maximum})"),
                None => String::new(),
            };
            let threshold = if count < self.automation_minimum_connections {
                format!(
                    " · BELOW MINIMUM ({} REQUIRED)",
                    self.automation_minimum_connections
                )
            } else {
                " · MINIMUM MET".into()
            };
            let regression_count = snapshot
                .as_ref()
                .map(|snapshot| automation_regression_count(&self.case, &snapshot.source_key))
                .unwrap_or_default();
            let regression_threshold = match self.automation_maximum_regressions {
                Some(maximum) if regression_count > maximum => {
                    format!(" · REGRESSION LIMIT EXCEEDED ({regression_count}; MAXIMUM {maximum})")
                }
                Some(maximum) => {
                    format!(" · REGRESSION LIMIT MET ({regression_count}; MAXIMUM {maximum})")
                }
                None => String::new(),
            };
            let limits_pass = automation_limits_pass(
                count,
                regression_count,
                skipped_rows,
                self.automation_minimum_connections,
                self.automation_maximum_connections,
                self.automation_maximum_regressions,
                self.automation_maximum_skipped_rows,
            );
            let validation_result = if limits_pass {
                "SQL QUERY VALID"
            } else {
                "SQL QUERY LIMITS FAILED"
            };
            format!(
                "{validation_result} · {count} CONNECTIONS{maximum_connection_threshold}{skipped}{skipped_row_threshold}{threshold}{regression_threshold} · CASE UNCHANGED"
            )
        } else {
            self.status
                .strip_prefix("SQL INGEST FAILED · ")
                .map(|error| format!("SQL QUERY INVALID · {error}"))
                .unwrap_or_else(|| "SQL QUERY INVALID".into())
        };

        self.case = case_before;
        if keep_candidate_snapshot && let Some(snapshot) = candidate_snapshot {
            self.case.snapshots.push(snapshot);
        }
        self.sql_validation_preview = validation_preview;
        self.selected_link = selected_link_before;
        self.sql_snapshot_source = snapshot_source_before;
        self.sql_snapshot_index = snapshot_index_before;
        self.path_result = path_result_before;
        self.status = validation_message;
        self.status.starts_with("SQL QUERY VALID · ")
    }

    fn show_sql_validation_preview(
        ui: &mut egui::Ui,
        preview: Option<&SqlValidationPreview>,
        database_path: &str,
        series: &str,
        query: &str,
        parameter_text: &str,
    ) {
        let Some(preview) = preview.filter(|preview| {
            preview.configured_database_path == database_path.trim()
                && preview.series == series.trim()
                && preview.query == query.trim()
                && parse_sql_parameters(parameter_text)
                    .is_ok_and(|parameters| parameters == preview.parameters)
        }) else {
            return;
        };
        egui::CollapsingHeader::new(format!(
            "VALIDATED ROW PREVIEW · {} CONNECTIONS",
            preview.connection_count
        ))
        .id_salt("sql_validation_preview")
        .show(ui, |ui| {
            ui.label(format!(
                "Captured {} · showing up to {} rows{}",
                preview.captured_at,
                preview.connections.len(),
                if preview.skipped_rows == 0 {
                    String::new()
                } else {
                    format!(
                        " · {} rows skipped for empty endpoints",
                        preview.skipped_rows
                    )
                }
            ));
            egui::ScrollArea::horizontal().show(ui, |ui| {
                egui::Grid::new("sql_validation_preview_rows")
                    .striped(true)
                    .show(ui, |ui| {
                        for heading in [
                            "Connection ID",
                            "Source",
                            "Source ID",
                            "Relationship",
                            "Target",
                            "Target ID",
                            "Direction",
                            "Date",
                            "Confidence",
                            "Evidence",
                            "Source notes",
                        ] {
                            ui.strong(heading);
                        }
                        ui.end_row();
                        for connection in &preview.connections {
                            ui.label(if connection.connection_id.is_empty() {
                                "(natural key)"
                            } else {
                                &connection.connection_id
                            });
                            ui.label(format!(
                                "{} · {}",
                                connection.source, connection.source_type
                            ));
                            ui.label(&connection.source_entity_id);
                            ui.label(&connection.relationship);
                            ui.label(format!(
                                "{} · {}",
                                connection.target, connection.target_type
                            ));
                            ui.label(&connection.target_entity_id);
                            ui.label(if connection.directed {
                                "Directed →"
                            } else {
                                "Undirected ↔"
                            });
                            ui.label(&connection.date);
                            ui.label(connection.confidence.to_string());
                            ui.label(&connection.source_reference);
                            ui.label(&connection.notes);
                            ui.end_row();
                        }
                    });
            });
        });
    }

    fn edit_automation_job(&mut self, id: Uuid) {
        if let Some(job) = self
            .case
            .automation_jobs
            .iter()
            .find(|job| job.id == id)
            .cloned()
        {
            self.automation_editor_before_edit = Some((
                self.automation_job_name.clone(),
                self.sql_path.clone(),
                self.sql_series.clone(),
                self.sql_query.clone(),
                self.sql_parameter_text.clone(),
                self.automation_minimum_connections,
                self.automation_maximum_connections,
                self.automation_maximum_regressions,
                self.automation_maximum_skipped_rows,
            ));
            self.automation_job_name = job.name;
            self.sql_path = job.database_path;
            self.sql_series = job.series;
            self.sql_query = job.query;
            self.sql_parameter_text = format_sql_parameters(&job.parameters);
            self.automation_minimum_connections = job.minimum_connections;
            self.automation_maximum_connections = job.maximum_connections;
            self.automation_maximum_regressions = job.maximum_regressions;
            self.automation_maximum_skipped_rows = job.maximum_skipped_rows;
            self.editing_automation_job = Some(id);
            self.status = "EDITING AUTOMATION JOB".into();
        }
    }

    fn cancel_automation_job_edit(&mut self) {
        self.editing_automation_job = None;
        if let Some((
            name,
            path,
            series,
            query,
            parameter_text,
            minimum,
            maximum_connections,
            maximum_regressions,
            maximum_skipped_rows,
        )) = self.automation_editor_before_edit.take()
        {
            self.automation_job_name = name;
            self.sql_path = path;
            self.sql_series = series;
            self.sql_query = query;
            self.sql_parameter_text = parameter_text;
            self.automation_minimum_connections = minimum;
            self.automation_maximum_connections = maximum_connections;
            self.automation_maximum_regressions = maximum_regressions;
            self.automation_maximum_skipped_rows = maximum_skipped_rows;
        }
        self.status = "JOB EDIT CANCELED".into();
    }

    fn start_automation_jobs(&mut self, job_ids: Vec<Uuid>, batch: bool) {
        self.start_automation_work(job_ids, None, batch);
    }

    fn sql_ui_request(&self) -> SqlUiRequest {
        SqlUiRequest {
            database_path: self.sql_path.clone(),
            series: self.sql_series.clone(),
            query: self.sql_query.clone(),
            parameter_text: self.sql_parameter_text.clone(),
            minimum_connections: self.automation_minimum_connections,
            maximum_connections: self.automation_maximum_connections,
            maximum_regressions: self.automation_maximum_regressions,
            maximum_skipped_rows: self.automation_maximum_skipped_rows,
            data_mode: self.data_mode,
            editing_job: self.editing_automation_job,
        }
    }

    fn start_sql_validation(&mut self) {
        self.start_sql_ui_operation(self.sql_ui_request(), true);
    }

    fn start_sql_ingestion(&mut self) {
        self.start_sql_ui_operation(self.sql_ui_request(), false);
    }

    fn start_sql_ui_operation(&mut self, request: SqlUiRequest, validate: bool) {
        if self.automation_worker.is_some() {
            self.status = "SQL OPERATION ALREADY RUNNING".into();
            return;
        }
        let case = self.case.clone();
        let case_path = self.case_path.clone();
        let selected_link_record = self
            .selected_link
            .and_then(|index| self.case.links.get(index))
            .cloned();
        let (sender, receiver) = mpsc::channel();
        let spawn_result = thread::Builder::new()
            .name("arch-notebook-sql-query".into())
            .spawn(move || {
                let mut worker = App::automation_context(case, &case_path);
                worker.selected_link = selected_link_record
                    .as_ref()
                    .and_then(|selected| refreshed_link_index(&worker.case.links, selected));
                worker.sql_path = request.database_path;
                worker.sql_series = request.series;
                worker.sql_query = request.query;
                worker.sql_parameter_text = request.parameter_text;
                worker.automation_minimum_connections = request.minimum_connections;
                worker.automation_maximum_connections = request.maximum_connections;
                worker.automation_maximum_regressions = request.maximum_regressions;
                worker.automation_maximum_skipped_rows = request.maximum_skipped_rows;
                worker.data_mode = request.data_mode;
                worker.editing_automation_job = request.editing_job;
                let result = if validate {
                    worker.validate_sql_query();
                    AutomationWorkerResult::Validation {
                        preview: worker.sql_validation_preview,
                        status: worker.status,
                    }
                } else {
                    worker.ingest_sqlite_snapshot();
                    let selected_snapshot =
                        worker.sql_snapshot_source.zip(worker.sql_snapshot_index);
                    AutomationWorkerResult::UnsavedCase {
                        case: worker.case,
                        status: worker.status,
                        selected_snapshot,
                        selected_link: worker.selected_link,
                    }
                };
                let _ = sender.send(Ok(result));
            });
        match spawn_result {
            Ok(_) => {
                self.automation_worker = Some(receiver);
                self.status = if validate {
                    "SQL QUERY VALIDATION RUNNING · WINDOW REMAINS RESPONSIVE".into()
                } else {
                    "SQL SNAPSHOT INGESTION RUNNING · WINDOW REMAINS RESPONSIVE".into()
                };
            }
            Err(error) => self.status = format!("SQL OPERATION START FAILED · {error}"),
        }
    }

    fn start_automation_work(
        &mut self,
        job_ids: Vec<Uuid>,
        replay_run_id: Option<Uuid>,
        batch: bool,
    ) {
        if self.automation_worker.is_some() {
            self.status = "AUTOMATION ALREADY RUNNING".into();
            return;
        }
        if self.has_unsaved_changes() || self.disk_case_json.is_none() {
            self.status = "SAVE CASE AND JOB CHANGES BEFORE RUNNING AUTOMATION".into();
            return;
        }
        if job_ids.is_empty() && replay_run_id.is_none() {
            self.status = "NO AUTOMATION JOBS TO RUN".into();
            return;
        }
        let case_path = self.case_path.clone();
        let expected_disk_json = self.disk_case_json.clone().unwrap_or_default();
        let selected_link_record = self
            .selected_link
            .and_then(|index| self.case.links.get(index))
            .cloned();
        let progress = Arc::new(Mutex::new("Waiting for the case lock".to_owned()));
        let worker_progress = Arc::clone(&progress);
        let durable_checkpoint = Arc::new(Mutex::new(
            None::<(CaseFile, String, Option<(String, usize)>)>,
        ));
        let worker_checkpoint = Arc::clone(&durable_checkpoint);
        let (sender, receiver) = mpsc::channel();
        let spawn_result = thread::Builder::new()
            .name("arch-notebook-automation".into())
            .spawn(move || {
                let update_progress = |message: String| {
                    if let Ok(mut progress) = worker_progress.lock() {
                        *progress = message;
                    }
                };
                let result = (|| -> Result<AutomationWorkerResult, String> {
                    let path = Path::new(&case_path);
                    let _case_lock = lock_automation_case(path)?;
                    update_progress("Reading the saved case".into());
                    let disk_json = fs::read_to_string(path)
                        .map_err(|error| format!("cannot read case: {error}"))?;
                    if disk_json != expected_disk_json {
                        return Err("case file changed outside this app; reload it before running automation".into());
                    }
                    let mut case = serde_json::from_str::<CaseFile>(&disk_json)
                        .map_err(|error| format!("cannot parse case: {error}"))?;
                    normalize_snapshot_database_paths(&mut case, &case_path);
                    let mut worker = App::automation_context(case, &case_path);
                    worker.selected_link = selected_link_record
                        .as_ref()
                        .and_then(|selected| refreshed_link_index(&worker.case.links, selected));
                    let mut jobs: Vec<_> = job_ids
                        .iter()
                        .filter_map(|id| {
                            worker
                                .case
                                .automation_jobs
                                .iter()
                                .find(|job| job.id == *id)
                                .cloned()
                                .map(|job| (job, None))
                        })
                        .collect();
                    if jobs.len() != job_ids.len() {
                        return Err("one or more automation jobs no longer exist; reload the case".into());
                    }
                    if let Some(run_id) = replay_run_id {
                        let prior_run = worker
                            .case
                            .automation_runs
                            .iter()
                            .find(|run| run.id == run_id)
                            .cloned()
                            .ok_or_else(|| "saved automation run no longer exists; reload the case".to_string())?;
                        jobs.push((automation_job_from_run(&prior_run)?, Some(run_id)));
                    }
                    let job_count = jobs.len();
                    let job_names: Vec<_> = jobs.iter().map(|(job, _)| job.name.clone()).collect();
                    let mut passed = 0;
                    for (index, (job, rerun_of)) in jobs.into_iter().enumerate() {
                        update_progress(format!(
                            "Running job {} of {job_count} · {}",
                            index + 1,
                            job.name
                        ));
                        if worker.run_automation_job_cli(&job).0 {
                            passed += 1;
                        }
                        if let Some(run_id) = rerun_of
                            && let Some(run) = worker.case.automation_runs.last_mut()
                        {
                            run.rerun_of = Some(run_id);
                        }
                        let serialized = serde_json::to_string_pretty(&worker.case)
                            .map_err(|error| error.to_string())?;
                        save_case_atomically(path, &worker.case)?;
                        worker.disk_case_json = Some(serialized.clone());
                        // Publish recovery state only after the atomic replacement succeeds.
                        if let Ok(mut checkpoint) = worker_checkpoint.lock() {
                            *checkpoint = Some((
                                worker.case.clone(),
                                serialized,
                                worker
                                    .sql_snapshot_source
                                    .clone()
                                    .zip(worker.sql_snapshot_index),
                            ));
                        }
                        let run = worker
                            .case
                            .automation_runs
                            .last()
                            .filter(|run| run.job_id == job.id)
                            .ok_or_else(|| "automation checkpoint is missing its run record".to_owned())?;
                        let snapshot = run.snapshot_id.and_then(|id| {
                            worker.case.snapshots.iter().find(|snapshot| snapshot.id == id)
                        });
                        let summary = snapshot.map_or_else(
                            || "no snapshot".to_owned(),
                            |snapshot| {
                                let regressions = run
                                    .snapshot_index
                                    .and_then(|index| previous_snapshot_index(&worker.case, index))
                                    .and_then(|index| worker.case.snapshots.get(index))
                                    .map(|baseline| {
                                        compare_snapshots(baseline, snapshot)
                                            .iter()
                                            .filter(|delta| delta.kinds.iter().any(|kind| matches!(kind, DeltaKind::Removed | DeltaKind::ConfidenceDeclined)))
                                            .count()
                                    })
                                    .unwrap_or_default();
                                format!(
                                    "{} connections · {} skipped · {} · {regressions} regressions",
                                    snapshot.connections.len(),
                                    snapshot.skipped_rows,
                                    snapshot_result_quality_label(snapshot_result_quality(snapshot))
                                )
                            },
                        );
                        update_progress(format!(
                            "Saved checkpoint {} of {job_count} · {} · {} · {summary}",
                            index + 1,
                            job.name,
                            if run.success { "PASS" } else { "FAIL" },
                        ));
                    }
                    let status = if batch {
                        format!(
                            "AUTOMATION RUN COMPLETE · {passed} PASSED · {} FAILED · {job_count} JOBS · CASE SAVED",
                            job_count - passed
                        )
                    } else if replay_run_id.is_some() {
                        format!(
                            "AUTOMATION REPLAY · {} · {} · CASE SAVED",
                            job_names.first().map(String::as_str).unwrap_or("(unnamed)"),
                            if passed == 1 { "PASSED" } else { "FAILED" },
                        )
                    } else {
                        format!(
                            "JOB RUN · {} · {} · CASE SAVED",
                            job_names.first().map(String::as_str).unwrap_or("(unnamed)"),
                            if passed == 1 { "PASSED" } else { "FAILED" },
                        )
                    };
                    let disk_case_json = worker
                        .disk_case_json
                        .unwrap_or(serde_json::to_string(&worker.case).map_err(|error| error.to_string())?);
                    let selected_snapshot = worker
                        .sql_snapshot_source
                        .zip(worker.sql_snapshot_index);
                    Ok(AutomationWorkerResult::SavedCase {
                        case: worker.case,
                        disk_case_json,
                        status,
                        selected_snapshot,
                        selected_link: worker.selected_link,
                    })
                })();
                let result = match result {
                    // Return the last saved case when a later job fails, keeping the open UI in sync.
                    Err(error) => match worker_checkpoint.lock() {
                        Ok(checkpoint) => {
                            if let Some((case, disk_case_json, selected_snapshot)) =
                                checkpoint.clone()
                            {
                                let selected_link = selected_link_record
                                    .as_ref()
                                    .and_then(|selected| refreshed_link_index(&case.links, selected));
                                Ok(AutomationWorkerResult::FailedCase {
                                    case,
                                    disk_case_json,
                                    error,
                                    selected_snapshot,
                                    selected_link,
                                })
                            } else {
                                Err(error)
                            }
                        }
                        Err(_) => Err(error),
                    },
                    result => result,
                };
                let _ = sender.send(result);
            });
        match spawn_result {
            Ok(_) => {
                self.automation_worker = Some(receiver);
                self.automation_progress = Some(progress);
                self.status =
                    "AUTOMATION RUNNING · CASE LOCKED · CHECKPOINTS SAVED AFTER EACH JOB".into();
            }
            Err(error) => self.status = format!("AUTOMATION START FAILED · {error}"),
        }
    }

    fn validate_automation_job(&mut self, id: Uuid) -> bool {
        let Some(job) = self
            .case
            .automation_jobs
            .iter()
            .find(|job| job.id == id)
            .cloned()
        else {
            self.status = "SQL QUERY INVALID · job not found".into();
            return false;
        };
        let mut request = self.sql_ui_request();
        request.database_path = job.database_path;
        request.series = job.series;
        request.query = job.query;
        request.parameter_text = format_sql_parameters(&job.parameters);
        request.minimum_connections = job.minimum_connections;
        request.maximum_connections = job.maximum_connections;
        request.maximum_regressions = job.maximum_regressions;
        request.maximum_skipped_rows = job.maximum_skipped_rows;
        request.data_mode = DataMode::Automation;
        request.editing_job = Some(id);
        self.start_sql_ui_operation(request, true);
        true
    }

    fn run_automation_job_cli(&mut self, job: &AutomationJob) -> (bool, String) {
        self.execute_automation_job(job)
    }

    fn replay_saved_automation_run(&mut self, run_id: Uuid) {
        self.start_automation_work(Vec::new(), Some(run_id), false);
    }

    fn execute_automation_job(&mut self, job: &AutomationJob) -> (bool, String) {
        let elapsed = Instant::now();
        let started_at = epoch_timestamp();
        if let Some(existing) = self.case.automation_jobs.iter().find(|existing| {
            existing.id != job.id
                && same_sql_series(
                    &existing.database_path,
                    &existing.series,
                    &job.database_path,
                    &job.series,
                    &self.case_path,
                )
        }) {
            let message = format!(
                "JOB RUN FAILED · database and series are already assigned to {}",
                existing.name
            );
            self.record_automation_run(
                job,
                started_at,
                elapsed.elapsed().as_millis() as u64,
                false,
                message.clone(),
                None,
            );
            return (false, message);
        }
        let snapshot_count_before = self.case.snapshots.len();
        self.sql_path = job.database_path.clone();
        self.sql_series = job.series.clone();
        self.sql_query = job.query.clone();
        self.sql_parameter_text = format_sql_parameters(&job.parameters);
        self.status = "JOB RUNNING".into();
        self.ingest_sqlite_snapshot();
        let ingestion_succeeded = self.status.starts_with("SQL SNAPSHOT INGESTED");
        let snapshot_index = (self.case.snapshots.len() > snapshot_count_before)
            .then_some(self.case.snapshots.len() - 1);
        let mut success = false;
        if ingestion_succeeded {
            let source_key = snapshot_index
                .and_then(|index| self.case.snapshots.get(index))
                .map(|snapshot| snapshot.source_key.clone())
                .unwrap_or_default();
            let count = snapshot_index
                .and_then(|index| self.case.snapshots.get(index))
                .map(|snapshot| snapshot.connections.len())
                .unwrap_or_default();
            let skipped_rows = snapshot_index
                .and_then(|index| self.case.snapshots.get(index))
                .map(|snapshot| snapshot.skipped_rows)
                .unwrap_or_default();
            let regression_count = automation_regression_count(&self.case, &source_key);
            if count < job.minimum_connections {
                self.status = format!(
                    "{} · BELOW MINIMUM ({} CONNECTIONS; {} REQUIRED)",
                    self.status, count, job.minimum_connections
                );
            }
            if let Some(maximum) = job.maximum_connections {
                let limit_status = if count > maximum {
                    format!("CONNECTION LIMIT EXCEEDED ({count}; MAXIMUM {maximum})")
                } else {
                    format!("CONNECTION LIMIT MET ({count}; MAXIMUM {maximum})")
                };
                self.status = format!("{} · {limit_status}", self.status);
            }
            if let Some(maximum) = job.maximum_regressions
                && regression_count > maximum
            {
                self.status = format!(
                    "{} · REGRESSION LIMIT EXCEEDED ({regression_count} REMOVALS OR CONFIDENCE DECLINES; MAXIMUM {maximum})",
                    self.status
                );
            }
            if let Some(maximum) = job.maximum_skipped_rows
                && skipped_rows > maximum
            {
                self.status = format!(
                    "{} · SKIPPED ROW LIMIT EXCEEDED ({skipped_rows}; MAXIMUM {maximum})",
                    self.status
                );
            } else if let Some(maximum) = job.maximum_skipped_rows {
                self.status = format!(
                    "{} · SKIPPED ROW LIMIT MET ({skipped_rows}; MAXIMUM {maximum})",
                    self.status
                );
            }
            self.status = format!(
                "{} · {}",
                self.status,
                automation_regression_summary(&self.case, &source_key)
            );
            success = automation_limits_pass(
                count,
                regression_count,
                skipped_rows,
                job.minimum_connections,
                job.maximum_connections,
                job.maximum_regressions,
                job.maximum_skipped_rows,
            );
        }
        let message = self.status.clone();
        self.record_automation_run(
            job,
            started_at,
            elapsed.elapsed().as_millis() as u64,
            success,
            message.clone(),
            snapshot_index,
        );
        (success, message)
    }

    fn record_automation_run(
        &mut self,
        job: &AutomationJob,
        started_at: String,
        duration_ms: u64,
        success: bool,
        message: String,
        snapshot_index: Option<usize>,
    ) {
        let snapshot_id = snapshot_index
            .and_then(|index| self.case.snapshots.get(index))
            .map(|snapshot| snapshot.id);
        self.case.automation_runs.push(AutomationRun {
            id: Uuid::new_v4(),
            job_id: job.id,
            job_name: job.name.clone(),
            database_path: job.database_path.clone(),
            series: job.series.clone(),
            query: job.query.clone(),
            parameters: job.parameters.clone(),
            minimum_connections: job.minimum_connections,
            maximum_connections: job.maximum_connections,
            maximum_regressions: job.maximum_regressions,
            maximum_skipped_rows: job.maximum_skipped_rows,
            enabled: job.enabled,
            configuration_recorded: true,
            started_at,
            finished_at: epoch_timestamp(),
            duration_ms: Some(duration_ms),
            success,
            message,
            snapshot_index,
            snapshot_id,
            rerun_of: None,
        });
        if self.case.automation_runs.len() > 200 {
            self.case
                .automation_runs
                .drain(..self.case.automation_runs.len() - 200);
        }
    }

    fn start_enabled_automation_jobs(&mut self) {
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
        self.start_automation_jobs(jobs, true);
    }

    fn export_delimited(&mut self) {
        let temporary_path = (!self.export_path.trim().is_empty()).then(|| {
            let mut path = self.export_path.trim().to_string();
            path.push_str(&format!(".tmp-{}", Uuid::new_v4()));
            PathBuf::from(path)
        });
        let result = (|| -> Result<(), String> {
            if self.export_path.trim().is_empty() {
                return Err("export path cannot be empty".into());
            }
            if paths_refer_to_same_file(self.export_path.trim(), &self.case_path) {
                return Err("SQL comparison output path matches the case file".into());
            }
            self.ensure_export_path_is_not_sql_source()?;
            let mut attribute_columns: Vec<(bool, String)> = Vec::new();
            let known_entities: HashSet<Uuid> =
                self.case.entities.iter().map(|entity| entity.id).collect();
            let linked_entities: HashSet<Uuid> = self
                .case
                .links
                .iter()
                .filter(|link| {
                    known_entities.contains(&link.from) && known_entities.contains(&link.to)
                })
                .flat_map(|link| [link.from, link.to])
                .collect();
            for link in &self.case.links {
                let Some(source) = self.case.entities.iter().find(|e| e.id == link.from) else {
                    continue;
                };
                let Some(target) = self.case.entities.iter().find(|e| e.id == link.to) else {
                    continue;
                };
                for (is_source, entity) in [(true, source), (false, target)] {
                    for (key, _) in &entity.attributes {
                        if !attribute_columns.iter().any(|(side, existing)| {
                            *side == is_source && existing.eq_ignore_ascii_case(key)
                        }) {
                            attribute_columns.push((is_source, key.clone()));
                        }
                    }
                }
            }
            for entity in self
                .case
                .entities
                .iter()
                .filter(|entity| !linked_entities.contains(&entity.id))
            {
                for (key, _) in &entity.attributes {
                    if !attribute_columns
                        .iter()
                        .any(|(side, existing)| *side && existing.eq_ignore_ascii_case(key))
                    {
                        attribute_columns.push((true, key.clone()));
                    }
                }
            }
            attribute_columns.sort_by(|(a_side, a_key), (b_side, b_key)| {
                b_side
                    .cmp(a_side)
                    .then_with(|| a_key.to_lowercase().cmp(&b_key.to_lowercase()))
            });
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delimited_text_delimiter(&self.export_path))
                .from_path(temporary_path.as_ref().expect("non-empty export path"))
                .map_err(|e| e.to_string())?;
            let mut headers = vec![
                "record_type".to_string(),
                "connection_id".to_string(),
                "source".to_string(),
                "source_entity_id".to_string(),
                "source_external_ids_json".to_string(),
                "source_type".to_string(),
                "source_detail_json".to_string(),
                "source_aliases_json".to_string(),
                "source_color".to_string(),
                "source_x".to_string(),
                "source_y".to_string(),
                "relationship".to_string(),
                "target".to_string(),
                "target_entity_id".to_string(),
                "target_external_ids_json".to_string(),
                "target_type".to_string(),
                "target_detail_json".to_string(),
                "target_aliases_json".to_string(),
                "target_color".to_string(),
                "target_x".to_string(),
                "target_y".to_string(),
                "date".to_string(),
                "confidence".to_string(),
                "directed".to_string(),
                "notes".to_string(),
                "source_notes".to_string(),
                "source_reference".to_string(),
                "ingest_source".to_string(),
                "source_position_locked".to_string(),
                "target_position_locked".to_string(),
            ];
            headers.extend(attribute_columns.iter().map(|(is_source, key)| {
                format!(
                    "{}_attribute:{key}",
                    if *is_source { "source" } else { "target" }
                )
            }));
            writer.write_record(&headers).map_err(|e| e.to_string())?;
            for link in &self.case.links {
                let a = self.case.entities.iter().find(|e| e.id == link.from);
                let b = self.case.entities.iter().find(|e| e.id == link.to);
                if let (Some(a), Some(b)) = (a, b) {
                    let conf = link.confidence.to_string();
                    let source_entity_id = a.id.to_string();
                    let target_entity_id = b.id.to_string();
                    let source_external_ids = serde_json::to_string(&a.external_ids)
                        .map_err(|error| error.to_string())?;
                    let target_external_ids = serde_json::to_string(&b.external_ids)
                        .map_err(|error| error.to_string())?;
                    let source_detail =
                        serde_json::to_string(&a.detail).map_err(|error| error.to_string())?;
                    let source_aliases =
                        serde_json::to_string(&a.aliases).map_err(|error| error.to_string())?;
                    let target_detail =
                        serde_json::to_string(&b.detail).map_err(|error| error.to_string())?;
                    let target_aliases =
                        serde_json::to_string(&b.aliases).map_err(|error| error.to_string())?;
                    let source_color =
                        format!("#{:02X}{:02X}{:02X}", a.color[0], a.color[1], a.color[2]);
                    let target_color =
                        format!("#{:02X}{:02X}{:02X}", b.color[0], b.color[1], b.color[2]);
                    let source_x = a.x.to_string();
                    let source_y = a.y.to_string();
                    let target_x = b.x.to_string();
                    let target_y = b.y.to_string();
                    let mut values = vec![
                        "connection",
                        link.connection_id.as_str(),
                        a.name.as_str(),
                        source_entity_id.as_str(),
                        source_external_ids.as_str(),
                        a.kind.as_str(),
                        source_detail.as_str(),
                        source_aliases.as_str(),
                        source_color.as_str(),
                        source_x.as_str(),
                        source_y.as_str(),
                        link.label.as_str(),
                        b.name.as_str(),
                        target_entity_id.as_str(),
                        target_external_ids.as_str(),
                        b.kind.as_str(),
                        target_detail.as_str(),
                        target_aliases.as_str(),
                        target_color.as_str(),
                        target_x.as_str(),
                        target_y.as_str(),
                        link.date.as_str(),
                        conf.as_str(),
                        if link.directed { "true" } else { "false" },
                        link.notes.as_str(),
                        link.source_notes.as_str(),
                        link.source_reference.as_str(),
                        link.ingest_source.as_deref().unwrap_or(""),
                        if a.position_locked { "true" } else { "false" },
                        if b.position_locked { "true" } else { "false" },
                    ];
                    for (is_source, key) in &attribute_columns {
                        let entity = if *is_source { a } else { b };
                        values.push(
                            entity
                                .attributes
                                .iter()
                                .find(|(attribute, _)| attribute.eq_ignore_ascii_case(key))
                                .map(|(_, value)| value.as_str())
                                .unwrap_or(""),
                        );
                    }
                    writer.write_record(values).map_err(|e| e.to_string())?;
                }
            }
            for entity in self
                .case
                .entities
                .iter()
                .filter(|entity| !linked_entities.contains(&entity.id))
            {
                let entity_id = entity.id.to_string();
                let external_ids = serde_json::to_string(&entity.external_ids)
                    .map_err(|error| error.to_string())?;
                let detail =
                    serde_json::to_string(&entity.detail).map_err(|error| error.to_string())?;
                let aliases =
                    serde_json::to_string(&entity.aliases).map_err(|error| error.to_string())?;
                let color = format!(
                    "#{:02X}{:02X}{:02X}",
                    entity.color[0], entity.color[1], entity.color[2]
                );
                let mut values = vec![
                    "entity".to_string(),
                    String::new(),
                    entity.name.clone(),
                    entity_id,
                    external_ids,
                    entity.kind.clone(),
                    detail,
                    aliases,
                    color,
                    entity.x.to_string(),
                    entity.y.to_string(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    String::new(),
                    entity.position_locked.to_string(),
                    String::new(),
                ];
                for (is_source, key) in &attribute_columns {
                    values.push(if *is_source {
                        entity
                            .attributes
                            .iter()
                            .find(|(attribute, _)| attribute.eq_ignore_ascii_case(key))
                            .map(|(_, value)| value.clone())
                            .unwrap_or_default()
                    } else {
                        String::new()
                    });
                }
                writer
                    .write_record(values)
                    .map_err(|error| error.to_string())?;
            }
            writer.flush().map_err(|e| e.to_string())?;
            drop(writer);
            fs::rename(
                temporary_path.as_ref().expect("non-empty export path"),
                self.export_path.trim(),
            )
            .map_err(|error| format!("cannot replace export file: {error}"))?;
            Ok(())
        })();
        if result.is_err()
            && let Some(temporary_path) = temporary_path
        {
            let _ = fs::remove_file(temporary_path);
        }
        self.status = match result {
            Ok(()) => format!(
                "{} EXPORTED {}",
                delimited_text_format(&self.export_path),
                self.export_path
            ),
            Err(e) => format!("EXPORT FAILED · {e}"),
        };
    }

    fn export_graphml(&mut self) {
        let output_path = self.graphml_export_path.trim().to_owned();
        let temporary_path = (!output_path.is_empty()).then(|| {
            let mut name = output_path.clone();
            name.push_str(&format!(".tmp-{}", Uuid::new_v4()));
            PathBuf::from(name)
        });
        let result = (|| -> Result<(), String> {
            if output_path.is_empty() {
                return Err("GraphML export path cannot be empty".into());
            }
            if paths_refer_to_same_file(&output_path, &self.case_path) {
                return Err("GraphML output path matches the case file".into());
            }
            self.ensure_path_is_not_sql_source(&output_path)?;
            let valid_entities: HashSet<Uuid> =
                self.case.entities.iter().map(|entity| entity.id).collect();
            if valid_entities.len() != self.case.entities.len() {
                return Err(
                    "cannot export GraphML because the case contains duplicate entity IDs".into(),
                );
            }
            for (index, link) in self.case.links.iter().enumerate() {
                if !valid_entities.contains(&link.from) || !valid_entities.contains(&link.to) {
                    return Err(format!(
                        "cannot export relationship {} because an endpoint entity is missing",
                        index + 1
                    ));
                }
            }

            let mut xml = String::from(
                "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<graphml xmlns=\"http://graphml.graphdrawing.org/xmlns\">\n",
            );
            for (id, scope, name, kind) in [
                ("graph_title", "graph", "title", "string"),
                ("graph_description", "graph", "description", "string"),
                ("node_uuid", "node", "arch_notebook_uuid", "string"),
                ("node_label", "node", "label", "string"),
                ("node_name", "node", "name", "string"),
                ("node_type", "node", "entity_type", "string"),
                ("node_detail", "node", "detail", "string"),
                ("node_aliases", "node", "aliases_json", "string"),
                ("node_external_ids", "node", "external_ids_json", "string"),
                ("node_attributes", "node", "attributes_json", "string"),
                (
                    "node_merged_records",
                    "node",
                    "merged_records_json",
                    "string",
                ),
                ("node_color", "node", "color", "string"),
                ("node_x", "node", "x", "double"),
                ("node_y", "node", "y", "double"),
                ("node_position_locked", "node", "position_locked", "boolean"),
                ("edge_connection_id", "edge", "connection_id", "string"),
                ("edge_label", "edge", "label", "string"),
                ("edge_relationship", "edge", "relationship", "string"),
                ("edge_date", "edge", "date", "string"),
                ("edge_confidence", "edge", "confidence", "int"),
                (
                    "edge_source_reference",
                    "edge",
                    "source_reference",
                    "string",
                ),
                ("edge_source_notes", "edge", "source_notes", "string"),
                ("edge_analyst_notes", "edge", "analyst_notes", "string"),
                ("edge_ingest_source", "edge", "ingest_source", "string"),
            ] {
                xml.push_str(&format!(
                    "  <key id=\"{id}\" for=\"{scope}\" attr.name=\"{name}\" attr.type=\"{kind}\"/>\n"
                ));
            }
            xml.push_str("  <graph id=\"arch_notebook\" edgedefault=\"undirected\">\n");
            graphml_data(&mut xml, "graph_title", &self.case.title);
            graphml_data(&mut xml, "graph_description", &self.case.description);
            for entity in &self.case.entities {
                xml.push_str(&format!("    <node id=\"n_{}\">\n", entity.id));
                graphml_data(&mut xml, "node_uuid", &entity.id.to_string());
                graphml_data(&mut xml, "node_label", &entity.name);
                graphml_data(&mut xml, "node_name", &entity.name);
                graphml_data(&mut xml, "node_type", &entity.kind);
                graphml_data(&mut xml, "node_detail", &entity.detail);
                graphml_data(
                    &mut xml,
                    "node_aliases",
                    &serde_json::to_string(&entity.aliases).map_err(|error| error.to_string())?,
                );
                graphml_data(
                    &mut xml,
                    "node_external_ids",
                    &serde_json::to_string(&entity.external_ids)
                        .map_err(|error| error.to_string())?,
                );
                graphml_data(
                    &mut xml,
                    "node_attributes",
                    &serde_json::to_string(&entity.attributes)
                        .map_err(|error| error.to_string())?,
                );
                graphml_data(
                    &mut xml,
                    "node_merged_records",
                    &serde_json::to_string(&entity.merged_records)
                        .map_err(|error| error.to_string())?,
                );
                graphml_data(
                    &mut xml,
                    "node_color",
                    &format!(
                        "#{:02X}{:02X}{:02X}",
                        entity.color[0], entity.color[1], entity.color[2]
                    ),
                );
                graphml_data(&mut xml, "node_x", &entity.x.to_string());
                graphml_data(&mut xml, "node_y", &entity.y.to_string());
                graphml_data(
                    &mut xml,
                    "node_position_locked",
                    if entity.position_locked {
                        "true"
                    } else {
                        "false"
                    },
                );
                xml.push_str("    </node>\n");
            }
            for (index, link) in self.case.links.iter().enumerate() {
                xml.push_str(&format!(
                    "    <edge id=\"e{}\" source=\"n_{}\" target=\"n_{}\" directed=\"{}\">\n",
                    index + 1,
                    link.from,
                    link.to,
                    link.directed
                ));
                graphml_data(&mut xml, "edge_connection_id", &link.connection_id);
                graphml_data(&mut xml, "edge_label", &link.label);
                graphml_data(&mut xml, "edge_relationship", &link.label);
                graphml_data(&mut xml, "edge_date", &link.date);
                graphml_data(&mut xml, "edge_confidence", &link.confidence.to_string());
                graphml_data(&mut xml, "edge_source_reference", &link.source_reference);
                graphml_data(&mut xml, "edge_source_notes", &link.source_notes);
                graphml_data(&mut xml, "edge_analyst_notes", &link.notes);
                graphml_data(
                    &mut xml,
                    "edge_ingest_source",
                    link.ingest_source.as_deref().unwrap_or_default(),
                );
                xml.push_str("    </edge>\n");
            }
            xml.push_str("  </graph>\n</graphml>\n");
            let temporary_path = temporary_path.as_ref().expect("non-empty GraphML path");
            fs::write(temporary_path, xml).map_err(|error| {
                format!(
                    "cannot write temporary GraphML file {}: {error}",
                    temporary_path.display()
                )
            })?;
            fs::rename(temporary_path, &output_path)
                .map_err(|error| format!("cannot replace GraphML export: {error}"))?;
            Ok(())
        })();
        if result.is_err()
            && let Some(temporary_path) = temporary_path
        {
            let _ = fs::remove_file(temporary_path);
        }
        self.status = match result {
            Ok(()) => format!("GRAPHML EXPORTED · {output_path}"),
            Err(error) => format!("GRAPHML EXPORT FAILED · {error}"),
        };
    }

    fn export_sql_comparison(
        &mut self,
        baseline: &ConnectionSnapshot,
        current: &ConnectionSnapshot,
    ) {
        let temporary_path = (!self.export_path.trim().is_empty()).then(|| {
            let mut path = self.export_path.trim().to_string();
            path.push_str(&format!(".tmp-{}", Uuid::new_v4()));
            PathBuf::from(path)
        });
        let result = (|| -> Result<usize, String> {
            if self.export_path.trim().is_empty() {
                return Err("export path cannot be empty".into());
            }
            self.ensure_export_path_is_not_sql_source()?;
            let deltas = compare_snapshots(baseline, current);
            let mut writer = csv::WriterBuilder::new()
                .delimiter(delimited_text_delimiter(&self.export_path))
                .from_path(temporary_path.as_ref().expect("non-empty export path"))
                .map_err(|error| error.to_string())?;
            writer
                .write_record([
                    "change",
                    "removed",
                    "confidence_declined",
                    "date_changed",
                    "connection_details_changed",
                    "direction_changed",
                    "evidence_changed",
                    "source_notes_changed",
                    "added",
                    "confidence_increased",
                    "connection_id",
                    "connection_key",
                    "source_before",
                    "source_type_before",
                    "source_entity_id_before",
                    "relationship_before",
                    "target_before",
                    "target_type_before",
                    "target_entity_id_before",
                    "directed_before",
                    "confidence_before",
                    "date_before",
                    "source_reference_before",
                    "source_notes_before",
                    "source",
                    "source_type",
                    "source_entity_id",
                    "relationship",
                    "target",
                    "target_type",
                    "target_entity_id",
                    "directed",
                    "confidence",
                    "date",
                    "source_reference",
                    "source_notes",
                    "database_path",
                    "series",
                    "baseline_snapshot_id",
                    "baseline_captured_at",
                    "baseline_skipped_rows",
                    "current_snapshot_id",
                    "current_captured_at",
                    "current_skipped_rows",
                ])
                .map_err(|error| error.to_string())?;
            for delta in &deltas {
                let previous = delta.previous_connection.as_ref();
                let current_fact =
                    (!delta.kinds.contains(&DeltaKind::Removed)).then_some(&delta.connection);
                let changes = delta
                    .kinds
                    .iter()
                    .map(|kind| match kind {
                        DeltaKind::Removed => "REMOVED",
                        DeltaKind::ConfidenceDeclined => "CONFIDENCE DOWN",
                        DeltaKind::DateChanged => "DATE CHANGED",
                        DeltaKind::Updated => "CONNECTION DETAILS",
                        DeltaKind::DirectionChanged => "DIRECTION CHANGED",
                        DeltaKind::EvidenceChanged => "EVIDENCE CHANGED",
                        DeltaKind::SourceNotesChanged => "SOURCE NOTES CHANGED",
                        DeltaKind::Added => "ADDED",
                        DeltaKind::ConfidenceIncreased => "CONFIDENCE UP",
                    })
                    .collect::<Vec<_>>()
                    .join(" · ");
                let mut record = vec![changes];
                record.extend(
                    [
                        DeltaKind::Removed,
                        DeltaKind::ConfidenceDeclined,
                        DeltaKind::DateChanged,
                        DeltaKind::Updated,
                        DeltaKind::DirectionChanged,
                        DeltaKind::EvidenceChanged,
                        DeltaKind::SourceNotesChanged,
                        DeltaKind::Added,
                        DeltaKind::ConfidenceIncreased,
                    ]
                    .iter()
                    .map(|kind| delta.kinds.contains(kind).to_string()),
                );
                record.push(delta.connection.connection_id.clone());
                record.push(comparison_key(&delta.connection));
                let append_fact = |record: &mut Vec<String>, fact: Option<&ConnectionFact>| {
                    record.extend([
                        fact.map(|value| value.source.clone()).unwrap_or_default(),
                        fact.map(|value| value.source_type.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.source_entity_id.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.relationship.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.target.clone()).unwrap_or_default(),
                        fact.map(|value| value.target_type.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.target_entity_id.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.directed.to_string())
                            .unwrap_or_default(),
                        fact.map(|value| value.confidence.to_string())
                            .unwrap_or_default(),
                        fact.map(|value| value.date.clone()).unwrap_or_default(),
                        fact.map(|value| value.source_reference.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.notes.clone()).unwrap_or_default(),
                    ]);
                };
                append_fact(&mut record, previous);
                append_fact(&mut record, current_fact);
                record.extend([
                    current.database_path.clone(),
                    current.series.clone(),
                    baseline.id.to_string(),
                    baseline.captured_at.clone(),
                    baseline.skipped_rows.to_string(),
                    current.id.to_string(),
                    current.captured_at.clone(),
                    current.skipped_rows.to_string(),
                ]);
                writer
                    .write_record(record)
                    .map_err(|error| error.to_string())?;
            }
            writer.flush().map_err(|error| error.to_string())?;
            drop(writer);
            fs::rename(
                temporary_path.as_ref().expect("non-empty export path"),
                self.export_path.trim(),
            )
            .map_err(|error| format!("cannot replace export file: {error}"))?;
            Ok(deltas.len())
        })();
        if result.is_err()
            && let Some(temporary_path) = temporary_path
        {
            let _ = fs::remove_file(temporary_path);
        }
        self.status = match result {
            Ok(count) => format!(
                "SQL COMPARISON EXPORTED · {} · {count} CHANGES · {}",
                delimited_text_format(&self.export_path),
                self.export_path
            ),
            Err(error) => format!("SQL COMPARISON EXPORT FAILED · {error}"),
        };
    }

    fn ensure_export_path_is_not_sql_source(&self) -> Result<(), String> {
        self.ensure_path_is_not_sql_source(self.export_path.trim())
    }

    fn ensure_path_is_not_sql_source(&self, path: &str) -> Result<(), String> {
        let output_path = Path::new(path);
        let Ok(output_path) = fs::canonicalize(output_path) else {
            return Ok(());
        };
        let path_matches =
            |path: &str| fs::canonicalize(path).is_ok_and(|source_path| source_path == output_path);
        let is_sql_source = self
            .case
            .snapshots
            .iter()
            .any(|snapshot| path_matches(&snapshot.database_path))
            || self.case.automation_jobs.iter().any(|job| {
                resolve_sqlite_database_path(&job.database_path, &self.case_path)
                    .is_ok_and(|source_path| source_path == output_path)
            })
            || resolve_sqlite_database_path(&self.sql_path, &self.case_path)
                .is_ok_and(|source_path| source_path == output_path);
        if is_sql_source {
            Err("export path matches an SQLite source database".into())
        } else {
            Ok(())
        }
    }

    fn auto_layout(&mut self) {
        self.chart_focus_group = None;
        let n = self.case.entities.len();
        if n == 0 {
            return;
        }
        let movable_count = self
            .case
            .entities
            .iter()
            .filter(|entity| !entity.position_locked)
            .count();
        if movable_count == 0 {
            self.status = "LAYOUT UNCHANGED · ALL ENTITY POSITIONS ARE LOCKED".into();
            return;
        }
        let entity_positions: HashMap<Uuid, usize> = self
            .case
            .entities
            .iter()
            .enumerate()
            .map(|(index, entity)| (entity.id, index))
            .collect();
        match self.layout_mode {
            LayoutMode::Connectivity => {
                let degrees = connected_entity_counts(&self.case);
                let mut order: Vec<usize> = (0..n)
                    .filter(|index| !self.case.entities[*index].position_locked)
                    .collect();
                order.sort_by_key(|index| {
                    (
                        std::cmp::Reverse(
                            degrees
                                .get(&self.case.entities[*index].id)
                                .copied()
                                .unwrap_or(0),
                        ),
                        self.case.entities[*index].id,
                    )
                });
                for (rank, index) in order.into_iter().enumerate() {
                    let angle = std::f32::consts::TAU * rank as f32 / movable_count as f32
                        - std::f32::consts::FRAC_PI_2;
                    self.case.entities[index].x = 0.5 + angle.cos() * 0.36;
                    self.case.entities[index].y = 0.5 + angle.sin() * 0.36;
                }
            }
            LayoutMode::Grid => {
                let mut order: Vec<usize> = (0..n)
                    .filter(|index| !self.case.entities[*index].position_locked)
                    .collect();
                order.sort_by(|a, b| {
                    self.case.entities[*a]
                        .name
                        .to_lowercase()
                        .cmp(&self.case.entities[*b].name.to_lowercase())
                        .then_with(|| self.case.entities[*a].id.cmp(&self.case.entities[*b].id))
                });
                let columns = (movable_count as f32).sqrt().ceil() as usize;
                let rows = movable_count.div_ceil(columns);
                for (rank, index) in order.into_iter().enumerate() {
                    let column = rank % columns;
                    let row = rank / columns;
                    self.case.entities[index].x = if columns == 1 {
                        0.5
                    } else {
                        0.08 + column as f32 * 0.84 / (columns - 1) as f32
                    };
                    self.case.entities[index].y = if rows == 1 {
                        0.5
                    } else {
                        0.08 + row as f32 * 0.84 / (rows - 1) as f32
                    };
                }
            }
            LayoutMode::Hierarchy => {
                let degrees = connected_entity_counts(&self.case);
                let mut seeds: Vec<Uuid> =
                    self.case.entities.iter().map(|entity| entity.id).collect();
                seeds.sort_by_key(|id| {
                    (
                        std::cmp::Reverse(degrees.get(id).copied().unwrap_or(0)),
                        *id,
                    )
                });
                let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
                for link in &self.case.links {
                    if link.from != link.to {
                        adjacency.entry(link.from).or_default().push(link.to);
                        adjacency.entry(link.to).or_default().push(link.from);
                    }
                }
                for neighbors in adjacency.values_mut() {
                    neighbors.sort_unstable();
                    neighbors.dedup();
                }
                let mut visited = HashSet::new();
                let mut components = Vec::new();
                for seed in seeds {
                    if !visited.insert(seed) {
                        continue;
                    }
                    let mut levels = vec![vec![seed]];
                    let mut queue = VecDeque::from([(seed, 0usize)]);
                    while let Some((current, depth)) = queue.pop_front() {
                        for neighbor in adjacency.get(&current).into_iter().flatten() {
                            if visited.insert(*neighbor) {
                                if levels.len() <= depth + 1 {
                                    levels.push(Vec::new());
                                }
                                levels[depth + 1].push(*neighbor);
                                queue.push_back((*neighbor, depth + 1));
                            }
                        }
                    }
                    for level in &mut levels {
                        level.sort_unstable();
                    }
                    components.push(levels);
                }
                let component_width = 0.9 / components.len() as f32;
                for (component_index, levels) in components.iter().enumerate() {
                    let left = 0.05 + component_index as f32 * component_width;
                    for (depth, level) in levels.iter().enumerate() {
                        let x = if levels.len() == 1 {
                            left + component_width / 2.0
                        } else {
                            left + depth as f32 * component_width / (levels.len() - 1) as f32
                        };
                        for (rank, id) in level.iter().enumerate() {
                            let y = if level.len() == 1 {
                                0.5
                            } else {
                                0.1 + rank as f32 * 0.8 / (level.len() - 1) as f32
                            };
                            if let Some(index) = entity_positions
                                .get(id)
                                .filter(|index| !self.case.entities[**index].position_locked)
                            {
                                self.case.entities[*index].x = x;
                                self.case.entities[*index].y = y;
                            }
                        }
                    }
                }
            }
        }
        self.chart_zoom = 1.0;
        self.chart_pan = egui::Vec2::ZERO;
        self.chart_focus = None;
        self.chart_focus_link = None;
        self.chart_fit_entities = None;
        self.chart_fit_path = None;
        self.chart_focus_group = None;
        let layout_label = match self.layout_mode {
            LayoutMode::Connectivity => "LAYOUT ARRANGED BY CONNECTIVITY",
            LayoutMode::Hierarchy => "LAYOUT ARRANGED BY CONNECTION DEPTH",
            LayoutMode::Grid => "LAYOUT ARRANGED IN ALPHABETICAL GRID",
        };
        let pinned_count = n - movable_count;
        self.status = if pinned_count == 0 {
            layout_label.into()
        } else {
            format!("{layout_label} · {pinned_count} POSITIONS PINNED")
        };
    }

    fn delete_selected(&mut self) {
        if let Some(id) = self.selected.take() {
            self.case.entities.retain(|e| e.id != id);
            self.case.links.retain(|l| l.from != id && l.to != id);
            self.invalidate_network_groups();
            self.selected_link = None;
            self.path_result.clear();
            self.status = "ENTITY AND ITS RELATIONSHIPS REMOVED".into();
        } else if let Some(i) = self.selected_link.take() {
            if i < self.case.links.len() {
                self.case.links.remove(i);
                self.invalidate_network_groups();
                self.status = "RELATIONSHIP REMOVED".into();
            } else {
                self.status = "RELATIONSHIP NO LONGER EXISTS".into();
            }
            self.selected_link = None;
            self.chart_focus_link = None;
            self.path_result.clear();
        }
    }

    fn new_case(&mut self) {
        self.case = CaseFile {
            format_version: CURRENT_CASE_FORMAT_VERSION,
            title: "UNTITLED CASE".into(),
            description: String::new(),
            entities: Vec::new(),
            links: Vec::new(),
            snapshots: Vec::new(),
            automation_jobs: Vec::new(),
            automation_runs: Vec::new(),
            import_specifications: Vec::new(),
        };
        self.network_groups = None;
        self.chart_focus_group = None;
        self.sql_snapshot_source = None;
        self.sql_snapshot_index = None;
        self.sql_source_focus = None;
        self.data_source_filter = None;
        self.editing_automation_job = None;
        self.automation_editor_before_edit = None;
        self.selected_import_spec = None;
        self.import_spec_name.clear();
        self.show_import_wizard = false;
        self.chart_zoom = 1.0;
        self.chart_pan = egui::Vec2::ZERO;
        self.chart_pan_origin = egui::Vec2::ZERO;
        self.chart_focus = None;
        self.chart_focus_link = None;
        self.chart_fit_entities = None;
        self.chart_fit_path = None;
        self.chart_focus_group = None;
        self.case_path = app_data_dir()
            .join("untitled.case.json")
            .to_string_lossy()
            .into_owned();
        self.case_path_input = self.case_path.clone();
        self.disk_case_json = fs::read_to_string(&self.case_path).ok();
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
        self.chart_focus_group = None;
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
        self.invalidate_network_groups();
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
        if !compatible_entity_types(&self.case.entities[keep_index].kind, &removed.kind) {
            self.merge_candidate = None;
            self.status = "MERGE STOPPED · ENTITY TYPES DIFFER".into();
            return;
        }
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
        if removed.position_locked && !self.case.entities[keep_index].position_locked {
            self.case.entities[keep_index].x = removed.x;
            self.case.entities[keep_index].y = removed.y;
            self.case.entities[keep_index].position_locked = true;
        }
        let keep = &mut self.case.entities[keep_index];
        for external_id in &removed.external_ids {
            if !keep.external_ids.contains(external_id) {
                keep.external_ids.push(external_id.clone());
            }
        }
        if (keep.kind.trim().is_empty() || keep.kind.eq_ignore_ascii_case("ENTITY"))
            && !removed.kind.trim().is_empty()
            && !removed.kind.eq_ignore_ascii_case("ENTITY")
        {
            keep.kind = removed.kind.clone();
        }
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
        self.chart_focus = Some(keep_id);
        self.chart_focus_group = None;
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

fn record_imported_entity_value(
    values: &mut HashMap<(Uuid, String), String>,
    entity_id: Uuid,
    property: &str,
    value: &str,
    entity_name: &str,
) -> Result<(), String> {
    let key = (entity_id, property.to_lowercase());
    if let Some(previous) = values.get(&key) {
        if previous != value {
            return Err(format!(
                "CSV has conflicting values for entity property {entity_name} · {property}: {previous} / {value}"
            ));
        }
    } else {
        values.insert(key, value.to_string());
    }
    Ok(())
}

fn indexed_link_natural_key(
    ingest_source: &str,
    source_name: &str,
    target_name: &str,
    relationship: &str,
    directed: bool,
) -> (String, String, String, String, bool) {
    let (source_name, target_name) =
        if !directed && target_name.to_lowercase() < source_name.to_lowercase() {
            (target_name, source_name)
        } else {
            (source_name, target_name)
        };
    (
        ingest_source.to_owned(),
        source_name.to_lowercase(),
        target_name.to_lowercase(),
        relationship.trim().to_lowercase(),
        directed,
    )
}

fn index_imported_link(
    links_by_id: &mut HashMap<(Option<String>, String), Vec<usize>>,
    links_by_natural_key: &mut HashMap<(String, String, String, String, bool), Vec<usize>>,
    index: usize,
    link: &Link,
    entities: &[Entity],
    entity_index: &EntityIndex,
) {
    if !link.connection_id.is_empty() {
        let indices = links_by_id
            .entry((link.ingest_source.clone(), link.connection_id.clone()))
            .or_default();
        if !indices.contains(&index) {
            indices.push(index);
        }
    } else if let Some(ingest_source) = &link.ingest_source {
        let source_name = entity_index
            .by_id
            .get(&link.from)
            .and_then(|position| entities.get(*position))
            .map(|entity| entity.name.as_str());
        let target_name = entity_index
            .by_id
            .get(&link.to)
            .and_then(|position| entities.get(*position))
            .map(|entity| entity.name.as_str());
        if let Some((source_name, target_name)) = source_name.zip(target_name) {
            let indices = links_by_natural_key
                .entry(indexed_link_natural_key(
                    ingest_source,
                    source_name,
                    target_name,
                    &link.label,
                    link.directed,
                ))
                .or_default();
            if !indices.contains(&index) {
                indices.push(index);
            }
        }
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
            if !compatible_entity_types(&entities[i].kind, &b_entity.kind) {
                continue;
            }
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

fn compatible_entity_types(a: &str, b: &str) -> bool {
    let a = a.trim();
    let b = b.trim();
    let generic = |kind: &str| kind.is_empty() || kind.eq_ignore_ascii_case("ENTITY");
    generic(a) || generic(b) || a.eq_ignore_ascii_case(b)
}

#[derive(Default)]
struct EntityIndex {
    by_name: HashMap<String, Vec<usize>>,
    by_alias: HashMap<String, Vec<usize>>,
    by_id: HashMap<Uuid, usize>,
    by_external_id: HashMap<(String, String), Vec<usize>>,
}

impl EntityIndex {
    fn new(entities: &[Entity]) -> Self {
        let mut index = Self::default();
        for (position, entity) in entities.iter().enumerate() {
            index
                .by_name
                .entry(entity.name.to_lowercase())
                .or_default()
                .push(position);
            index.by_id.insert(entity.id, position);
            index.register_aliases(position, entity);
            for (source, external_id) in &entity.external_ids {
                index
                    .by_external_id
                    .entry((source.clone(), external_id.clone()))
                    .or_default()
                    .push(position);
            }
        }
        index
    }

    fn register_aliases(&mut self, position: usize, entity: &Entity) {
        for alias in &entity.aliases {
            let positions = self.by_alias.entry(alias.to_lowercase()).or_default();
            if !positions.contains(&position) {
                positions.push(position);
            }
        }
    }

    fn refresh_aliases(&mut self, id: Uuid, entities: &[Entity]) {
        if let Some(position) = self.by_id.get(&id).copied() {
            self.register_aliases(position, &entities[position]);
        }
    }

    fn find_or_add_entity_with_id(
        &mut self,
        name: &str,
        kind: &str,
        requested_id: Option<Uuid>,
        entities: &mut Vec<Entity>,
    ) -> Result<Uuid, String> {
        let Some(id) = requested_id else {
            return Ok(self.find_or_add_entity(name, kind, entities));
        };
        let is_generic =
            |value: &str| value.trim().is_empty() || value.eq_ignore_ascii_case("ENTITY");
        if let Some(position) = self.by_id.get(&id).copied() {
            let existing = &mut entities[position];
            if !existing.name.eq_ignore_ascii_case(name)
                || !compatible_entity_types(&existing.kind, kind)
            {
                return Err(format!(
                    "CSV entity ID {id} conflicts with existing entity {} [{}]",
                    existing.name, existing.kind
                ));
            }
            if is_generic(&existing.kind) && !is_generic(kind) {
                existing.kind = kind.into();
            }
            return Ok(id);
        }

        let position = entities.len();
        let entity = Entity {
            id,
            name: name.into(),
            kind: if is_generic(kind) { "ENTITY" } else { kind }.into(),
            detail: String::new(),
            attributes: Vec::new(),
            aliases: Vec::new(),
            external_ids: Vec::new(),
            merged_records: Vec::new(),
            x: 0.12 + (position % 4) as f32 * 0.23,
            y: 0.15 + (position / 4) as f32 * 0.2,
            position_locked: false,
            color: [65, 125, 153],
        };
        self.by_name
            .entry(name.to_lowercase())
            .or_default()
            .push(position);
        self.by_id.insert(id, position);
        entities.push(entity);
        Ok(id)
    }

    fn find_or_add_entity(&mut self, name: &str, kind: &str, entities: &mut Vec<Entity>) -> Uuid {
        let is_generic =
            |value: &str| value.trim().is_empty() || value.eq_ignore_ascii_case("ENTITY");
        let normalized_name = name.to_lowercase();
        let exact: Vec<usize> = self
            .by_name
            .get(&normalized_name)
            .into_iter()
            .flatten()
            .copied()
            .filter(|index| {
                entities[*index].name.eq_ignore_ascii_case(name)
                    && compatible_entity_types(&entities[*index].kind, kind)
            })
            .collect();
        let exact_match = if !is_generic(kind) {
            let typed: Vec<usize> = exact
                .iter()
                .copied()
                .filter(|index| entities[*index].kind.eq_ignore_ascii_case(kind))
                .collect();
            if typed.len() == 1 {
                typed.first().copied()
            } else {
                let generic: Vec<usize> = exact
                    .iter()
                    .copied()
                    .filter(|index| is_generic(&entities[*index].kind))
                    .collect();
                (generic.len() == 1).then(|| generic[0])
            }
        } else if exact.len() == 1 {
            exact.first().copied()
        } else {
            let generic: Vec<usize> = exact
                .iter()
                .copied()
                .filter(|index| is_generic(&entities[*index].kind))
                .collect();
            (generic.len() == 1).then(|| generic[0])
        };
        let alias_match = || {
            let aliases: Vec<usize> = self
                .by_alias
                .get(&normalized_name)
                .into_iter()
                .flatten()
                .copied()
                .filter(|index| {
                    compatible_entity_types(&entities[*index].kind, kind)
                        && entities[*index]
                            .aliases
                            .iter()
                            .any(|alias| alias.eq_ignore_ascii_case(name))
                })
                .collect();
            (aliases.len() == 1).then(|| aliases[0])
        };
        if let Some(position) = exact_match.or_else(alias_match) {
            if is_generic(&entities[position].kind) && !is_generic(kind) {
                entities[position].kind = kind.into();
            }
            return entities[position].id;
        }

        self.add_new_entity(name, kind, entities)
    }

    fn find_or_add_entity_with_external_id(
        &mut self,
        name: &str,
        kind: &str,
        source: &str,
        external_id: &str,
        entities: &mut Vec<Entity>,
    ) -> Result<Uuid, String> {
        let external_id = external_id.trim();
        if external_id.is_empty() {
            return Ok(self.find_or_add_entity(name, kind, entities));
        }
        let key = (source.to_owned(), external_id.to_owned());
        if let Some(positions) = self.by_external_id.get(&key) {
            if positions.len() != 1 {
                return Err(format!(
                    "SQL entity identity {external_id} is assigned to multiple chart entities"
                ));
            }
            let position = positions[0];
            if !compatible_entity_types(&entities[position].kind, kind) {
                return Err(format!(
                    "SQL entity identity {external_id} conflicts with entity type {}",
                    entities[position].kind
                ));
            }
            if entities[position].kind.eq_ignore_ascii_case("ENTITY")
                && !kind.trim().is_empty()
                && !kind.eq_ignore_ascii_case("ENTITY")
            {
                entities[position].kind = kind.to_owned();
            }
            if !entities[position].name.eq_ignore_ascii_case(name)
                && !entities[position]
                    .aliases
                    .iter()
                    .any(|alias| alias.eq_ignore_ascii_case(name))
            {
                entities[position].aliases.push(name.to_owned());
                self.register_aliases(position, &entities[position]);
            }
            return Ok(entities[position].id);
        }

        let source_identity_already_named = self
            .by_name
            .get(&name.to_lowercase())
            .into_iter()
            .flatten()
            .any(|position| {
                entities[*position].name.eq_ignore_ascii_case(name)
                    && compatible_entity_types(&entities[*position].kind, kind)
                    && entities[*position]
                        .external_ids
                        .iter()
                        .any(|(existing_source, _)| existing_source == source)
            });
        let entity_id = if source_identity_already_named {
            self.add_new_entity(name, kind, entities)
        } else {
            self.find_or_add_entity(name, kind, entities)
        };
        let position = self.by_id[&entity_id];
        entities[position]
            .external_ids
            .push((source.to_owned(), external_id.to_owned()));
        self.by_external_id.entry(key).or_default().push(position);
        Ok(entity_id)
    }

    fn add_new_entity(&mut self, name: &str, kind: &str, entities: &mut Vec<Entity>) -> Uuid {
        let is_generic =
            |value: &str| value.trim().is_empty() || value.eq_ignore_ascii_case("ENTITY");
        let normalized_name = name.to_lowercase();
        let id = Uuid::new_v4();
        let position = entities.len();
        let entity = Entity {
            id,
            name: name.into(),
            kind: if is_generic(kind) { "ENTITY" } else { kind }.into(),
            detail: String::new(),
            attributes: Vec::new(),
            aliases: Vec::new(),
            external_ids: Vec::new(),
            merged_records: Vec::new(),
            x: 0.12 + (position % 4) as f32 * 0.23,
            y: 0.15 + (position / 4) as f32 * 0.2,
            position_locked: false,
            color: [65, 125, 153],
        };
        self.by_name
            .entry(normalized_name)
            .or_default()
            .push(position);
        self.by_id.insert(id, position);
        entities.push(entity);
        id
    }
}

fn canonical_connection(
    source: &str,
    source_type: &str,
    target: &str,
    target_type: &str,
    relationship: &str,
    directed: bool,
) -> Result<(String, String, String, String, String), String> {
    let ordered = !directed
        && (source.to_lowercase(), source_type.to_lowercase())
            > (target.to_lowercase(), target_type.to_lowercase());
    let (canonical_source, canonical_source_type, canonical_target, canonical_target_type) =
        if ordered {
            (target, target_type, source, source_type)
        } else {
            (source, source_type, target, target_type)
        };
    let key = serde_json::to_string(&(
        canonical_source.to_lowercase(),
        canonical_source_type.to_lowercase(),
        relationship.trim().to_lowercase(),
        canonical_target.to_lowercase(),
        canonical_target_type.to_lowercase(),
        directed,
    ))
    .map_err(|error| error.to_string())?;
    Ok((
        canonical_source.to_string(),
        canonical_source_type.to_string(),
        canonical_target.to_string(),
        canonical_target_type.to_string(),
        key,
    ))
}

fn grouped_connections(
    entities: &[Entity],
    links: &[Link],
    indices: &[usize],
) -> Vec<ConnectionGroup> {
    let mut groups: HashMap<(Uuid, Uuid), Vec<usize>> = HashMap::new();
    let entity_order: HashMap<Uuid, (String, String)> = entities
        .iter()
        .map(|entity| {
            (
                entity.id,
                (entity.name.to_lowercase(), entity.kind.to_lowercase()),
            )
        })
        .collect();
    for index in indices {
        let Some(link) = links.get(*index) else {
            continue;
        };
        let order_key = |id: Uuid| {
            entity_order
                .get(&id)
                .cloned()
                .unwrap_or_else(|| ("(missing)".into(), String::new()))
        };
        let pair = if (order_key(link.from), link.from) <= (order_key(link.to), link.to) {
            (link.from, link.to)
        } else {
            (link.to, link.from)
        };
        groups.entry(pair).or_default().push(*index);
    }
    let mut groups: Vec<ConnectionGroup> = groups
        .into_iter()
        .map(|((first, second), mut link_indices)| {
            link_indices.sort_unstable();
            ConnectionGroup {
                first,
                second,
                link_indices,
            }
        })
        .collect();
    groups.sort_by_key(|group| (group.first, group.second));
    groups
}

fn connected_entity_counts(case: &CaseFile) -> HashMap<Uuid, usize> {
    connected_entity_counts_for_relationship(case, None)
}

fn linked_entities_for(
    case: &CaseFile,
    entity_id: Uuid,
) -> Vec<(Uuid, String, String, Vec<usize>)> {
    let mut links_by_entity: HashMap<Uuid, Vec<usize>> = HashMap::new();
    for (index, link) in case.links.iter().enumerate() {
        let counterpart = if link.from == entity_id {
            Some(link.to)
        } else if link.to == entity_id {
            Some(link.from)
        } else {
            None
        };
        if let Some(counterpart) = counterpart {
            links_by_entity.entry(counterpart).or_default().push(index);
        }
    }
    let entities_by_id: HashMap<Uuid, &Entity> = case
        .entities
        .iter()
        .map(|entity| (entity.id, entity))
        .collect();
    let mut linked_entities: Vec<_> = links_by_entity
        .into_iter()
        .map(|(counterpart_id, mut link_indices)| {
            let (name, kind) = entities_by_id
                .get(&counterpart_id)
                .map(|entity| (entity.name.clone(), entity.kind.clone()))
                .unwrap_or_else(|| ("(missing entity)".into(), String::new()));
            link_indices.sort_by(|first, second| {
                let first = &case.links[*first];
                let second = &case.links[*second];
                first
                    .date
                    .cmp(&second.date)
                    .then_with(|| first.label.to_lowercase().cmp(&second.label.to_lowercase()))
                    .then_with(|| first.connection_id.cmp(&second.connection_id))
            });
            (counterpart_id, name, kind, link_indices)
        })
        .collect();
    linked_entities.sort_by(|first, second| {
        first
            .1
            .to_lowercase()
            .cmp(&second.1.to_lowercase())
            .then_with(|| first.2.to_lowercase().cmp(&second.2.to_lowercase()))
    });
    linked_entities
}

fn chart_fit_transform(
    case: &CaseFile,
    entity_ids: &[Uuid],
    rect: egui::Rect,
) -> Option<(f32, egui::Vec2)> {
    let entity_positions: HashMap<Uuid, egui::Pos2> = case
        .entities
        .iter()
        .map(|entity| {
            (
                entity.id,
                egui::pos2(
                    rect.left() + entity.x * rect.width(),
                    rect.top() + entity.y * rect.height(),
                ),
            )
        })
        .collect();
    let mut points = entity_ids
        .iter()
        .filter_map(|id| entity_positions.get(id).copied());
    let first = points.next()?;
    let (min, max) = points.fold((first, first), |(min, max), point| {
        (
            egui::pos2(min.x.min(point.x), min.y.min(point.y)),
            egui::pos2(max.x.max(point.x), max.y.max(point.y)),
        )
    });
    let content_width = (max.x - min.x + 142.0).max(142.0);
    let content_height = (max.y - min.y + 56.0).max(56.0);
    let zoom = ((rect.width() - 48.0) / content_width)
        .min((rect.height() - 48.0) / content_height)
        .clamp(0.25, 3.0);
    let pan = -(min.lerp(max, 0.5) - rect.center()) * zoom;
    Some((zoom, pan))
}

fn connected_entity_counts_for_relationship(
    case: &CaseFile,
    relationship_filter: Option<&str>,
) -> HashMap<Uuid, usize> {
    let mut neighbors: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
    for link in &case.links {
        if relationship_filter.is_some_and(|filter| !link.label.eq_ignore_ascii_case(filter)) {
            continue;
        }
        if link.from == link.to {
            continue;
        }
        neighbors.entry(link.from).or_default().insert(link.to);
        neighbors.entry(link.to).or_default().insert(link.from);
    }
    neighbors
        .into_iter()
        .map(|(entity, linked)| (entity, linked.len()))
        .collect()
}

fn most_connected_counts(
    case: &CaseFile,
    measure: MostConnectedMeasure,
    relationship_filter: Option<&str>,
) -> HashMap<Uuid, usize> {
    if measure == MostConnectedMeasure::ConnectedEntities {
        return connected_entity_counts_for_relationship(case, relationship_filter);
    }
    let mut counts: HashMap<Uuid, usize> =
        case.entities.iter().map(|entity| (entity.id, 0)).collect();
    for link in &case.links {
        if relationship_filter.is_some_and(|filter| !link.label.eq_ignore_ascii_case(filter)) {
            continue;
        }
        match measure {
            MostConnectedMeasure::ConnectedEntities => {}
            MostConnectedMeasure::LinkRecords => {
                *counts.entry(link.from).or_default() += 1;
                if link.to != link.from {
                    *counts.entry(link.to).or_default() += 1;
                }
            }
            MostConnectedMeasure::IncomingLinks if link.directed => {
                *counts.entry(link.to).or_default() += 1;
            }
            MostConnectedMeasure::OutgoingLinks if link.directed => {
                *counts.entry(link.from).or_default() += 1;
            }
            MostConnectedMeasure::IncomingLinks | MostConnectedMeasure::OutgoingLinks => {}
        }
    }
    counts
}

fn connected_components(case: &CaseFile) -> Vec<Vec<Uuid>> {
    let names: HashMap<Uuid, String> = case
        .entities
        .iter()
        .map(|entity| (entity.id, entity.name.to_lowercase()))
        .collect();
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = case
        .entities
        .iter()
        .map(|entity| (entity.id, Vec::new()))
        .collect();
    for link in &case.links {
        if link.from == link.to
            || !adjacency.contains_key(&link.from)
            || !adjacency.contains_key(&link.to)
        {
            continue;
        }
        adjacency.entry(link.from).or_default().push(link.to);
        adjacency.entry(link.to).or_default().push(link.from);
    }
    for neighbors in adjacency.values_mut() {
        neighbors.sort_by(|a, b| names.get(a).cmp(&names.get(b)).then_with(|| a.cmp(b)));
        neighbors.dedup();
    }

    let mut seeds: Vec<Uuid> = case.entities.iter().map(|entity| entity.id).collect();
    seeds.sort_by(|a, b| names.get(a).cmp(&names.get(b)).then_with(|| a.cmp(b)));
    let mut visited = HashSet::new();
    let mut groups = Vec::new();
    for seed in seeds {
        if !visited.insert(seed) {
            continue;
        }
        let mut queue = VecDeque::from([seed]);
        let mut group = Vec::new();
        while let Some(entity) = queue.pop_front() {
            group.push(entity);
            if let Some(neighbors) = adjacency.get(&entity) {
                for neighbor in neighbors {
                    if visited.insert(*neighbor) {
                        queue.push_back(*neighbor);
                    }
                }
            }
        }
        group.sort_by(|a, b| names.get(a).cmp(&names.get(b)).then_with(|| a.cmp(b)));
        groups.push(group);
    }
    groups.sort_by(|a, b| {
        b.len()
            .cmp(&a.len())
            .then_with(|| names.get(&a[0]).cmp(&names.get(&b[0])))
            .then_with(|| a[0].cmp(&b[0]))
    });
    groups
}

fn refreshed_link_index(links: &[Link], selected: &Link) -> Option<usize> {
    if !selected.connection_id.is_empty() {
        return links.iter().position(|link| {
            link.connection_id == selected.connection_id
                && link.ingest_source == selected.ingest_source
        });
    }
    if selected.ingest_source.is_some() {
        return links.iter().position(|link| {
            link.ingest_source == selected.ingest_source
                && link.from == selected.from
                && link.to == selected.to
                && link.label == selected.label
                && link.directed == selected.directed
        });
    }
    links.iter().position(|link| {
        link.ingest_source.is_none()
            && link.from == selected.from
            && link.to == selected.to
            && link.connection_id == selected.connection_id
            && link.label == selected.label
            && link.date == selected.date
            && link.confidence == selected.confidence
            && link.directed == selected.directed
            && link.notes == selected.notes
            && link.source_reference == selected.source_reference
    })
}

fn connection_refresh_key(
    connection_id: &str,
    from: Uuid,
    to: Uuid,
    relationship: &str,
    directed: bool,
) -> String {
    if !connection_id.is_empty() {
        return format!("id:{connection_id}");
    }
    let (from, to) = if !directed && to < from {
        (to, from)
    } else {
        (from, to)
    };
    format!(
        "natural:{from}:{to}:{}:{directed}",
        relationship.to_lowercase()
    )
}

fn link_refresh_key(link: &Link) -> String {
    connection_refresh_key(
        &link.connection_id,
        link.from,
        link.to,
        &link.label,
        link.directed,
    )
}

fn same_sql_connection(existing: &Link, imported: &Link) -> bool {
    existing.from == imported.from
        && existing.to == imported.to
        && existing.connection_id == imported.connection_id
        && existing.label == imported.label
        && existing.date == imported.date
        && existing.confidence == imported.confidence
        && existing.directed == imported.directed
        && existing.source_notes == imported.source_notes
        && existing.source_reference == imported.source_reference
        && existing.ingest_source == imported.ingest_source
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

fn normalize_sql_parameter_name(name: &str) -> String {
    name.trim()
        .trim_start_matches(|character| matches!(character, ':' | '@' | '$'))
        .to_owned()
}

fn valid_sql_parameter_name(name: &str) -> bool {
    let mut characters = name.chars();
    matches!(characters.next(), Some('a'..='z' | 'A'..='Z' | '_'))
        && characters.all(|character| character.is_ascii_alphanumeric() || character == '_')
}

fn parse_sql_parameters(text: &str) -> Result<Vec<(String, String)>, String> {
    let mut parameters = Vec::new();
    let mut names = HashSet::new();
    for (line_index, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((raw_name, value)) = line.split_once('=') else {
            return Err(format!(
                "SQL parameter line {} must use name=value",
                line_index + 1
            ));
        };
        let name = normalize_sql_parameter_name(raw_name);
        if !valid_sql_parameter_name(&name) {
            return Err(format!(
                "SQL parameter line {} needs a name starting with a letter or underscore",
                line_index + 1
            ));
        }
        if !names.insert(name.clone()) {
            return Err(format!("duplicate SQL parameter: {name}"));
        }
        parameters.push((name, value.trim().to_owned()));
    }
    Ok(parameters)
}

fn format_sql_parameters(parameters: &[(String, String)]) -> String {
    parameters
        .iter()
        .map(|(name, value)| format!("{name}={value}"))
        .collect::<Vec<_>>()
        .join("\n")
}

fn sql_parameter_value(name: &str, value: &str) -> Result<Value, String> {
    if let Some(text) = value.strip_prefix("text:") {
        return Ok(Value::Text(text.to_owned()));
    }
    if let Some(integer) = value.strip_prefix("int:") {
        return integer
            .parse::<i64>()
            .map(Value::Integer)
            .map_err(|_| format!("SQL parameter {name} must use int:<64-bit integer>"));
    }
    if let Some(real) = value.strip_prefix("real:") {
        let number = real
            .parse::<f64>()
            .map_err(|_| format!("SQL parameter {name} must use real:<finite number>"))?;
        if !number.is_finite() {
            return Err(format!(
                "SQL parameter {name} must use real:<finite number>"
            ));
        }
        return Ok(Value::Real(number));
    }
    if let Some(boolean) = value.strip_prefix("bool:") {
        return match boolean.trim().to_ascii_lowercase().as_str() {
            "true" | "1" => Ok(Value::Integer(1)),
            "false" | "0" => Ok(Value::Integer(0)),
            _ => Err(format!(
                "SQL parameter {name} must use bool:true or bool:false"
            )),
        };
    }
    if value == "null:" {
        return Ok(Value::Null);
    }
    Ok(Value::Text(value.to_owned()))
}

fn sql_parameter_values(
    statement: &rusqlite::Statement<'_>,
    parameters: &[(String, String)],
) -> Result<Vec<Value>, String> {
    let configured: HashMap<&str, &str> = parameters
        .iter()
        .map(|(name, value)| (name.as_str(), value.as_str()))
        .collect();
    let mut used = HashSet::new();
    let mut values = Vec::with_capacity(statement.parameter_count());
    for index in 1..=statement.parameter_count() {
        let Some(raw_name) = statement.parameter_name(index) else {
            return Err("SQL queries must use named parameters (:name, @name, or $name)".into());
        };
        if !matches!(raw_name.chars().next(), Some(':' | '@' | '$')) {
            return Err("SQL queries must use named parameters (:name, @name, or $name)".into());
        }
        let name = normalize_sql_parameter_name(raw_name);
        let Some(value) = configured.get(name.as_str()) else {
            return Err(format!("missing SQL parameter value: {name}"));
        };
        used.insert(name.clone());
        values.push(sql_parameter_value(&name, value)?);
    }
    if let Some((name, _)) = parameters.iter().find(|(name, _)| !used.contains(name)) {
        return Err(format!("SQL parameter is not used by the query: {name}"));
    }
    Ok(values)
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
    let old: HashMap<String, &ConnectionFact> = baseline
        .connections
        .iter()
        .map(|connection| (comparison_key(connection), connection))
        .collect();
    let new: HashMap<String, &ConnectionFact> = current
        .connections
        .iter()
        .map(|connection| (comparison_key(connection), connection))
        .collect();
    let mut deltas = Vec::new();
    for (key, previous) in &old {
        match new.get(key) {
            None => deltas.push(ConnectionDelta {
                kinds: vec![DeltaKind::Removed],
                connection: (*previous).clone(),
                previous_connection: Some((*previous).clone()),
                previous_confidence: Some(previous.confidence),
                previous_date: Some(previous.date.clone()),
            }),
            Some(now) => {
                let mut kinds = Vec::new();
                if now.confidence < previous.confidence {
                    kinds.push(DeltaKind::ConfidenceDeclined);
                } else if now.confidence > previous.confidence {
                    kinds.push(DeltaKind::ConfidenceIncreased);
                }
                if now.date != previous.date {
                    kinds.push(DeltaKind::DateChanged);
                }
                if now.source != previous.source
                    || now.source_type != previous.source_type
                    || now.target != previous.target
                    || now.target_type != previous.target_type
                    || now.source_entity_id != previous.source_entity_id
                    || now.target_entity_id != previous.target_entity_id
                    || now.relationship != previous.relationship
                    || now.directed != previous.directed
                {
                    kinds.push(DeltaKind::Updated);
                }
                if now.directed != previous.directed {
                    kinds.push(DeltaKind::DirectionChanged);
                }
                if now.source_reference != previous.source_reference {
                    kinds.push(DeltaKind::EvidenceChanged);
                }
                if now.notes != previous.notes {
                    kinds.push(DeltaKind::SourceNotesChanged);
                }
                if !kinds.is_empty() {
                    deltas.push(ConnectionDelta {
                        kinds,
                        connection: (*now).clone(),
                        previous_connection: Some((*previous).clone()),
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
                kinds: vec![DeltaKind::Added],
                connection: (*connection).clone(),
                previous_connection: None,
                previous_confidence: None,
                previous_date: None,
            });
        }
    }
    deltas.sort_by(|a, b| {
        delta_order(&a.kinds)
            .cmp(&delta_order(&b.kinds))
            .then_with(|| a.connection.source.cmp(&b.connection.source))
            .then_with(|| a.connection.target.cmp(&b.connection.target))
            .then_with(|| comparison_key(&a.connection).cmp(&comparison_key(&b.connection)))
    });
    deltas
}

fn connection_delta_matches_filter(
    delta: &ConnectionDelta,
    filter: SnapshotDeltaFilter,
    search: &str,
) -> bool {
    let has = |kind| delta.kinds.contains(&kind);
    let matches_category = match filter {
        SnapshotDeltaFilter::All => true,
        SnapshotDeltaFilter::Regressions => {
            has(DeltaKind::Removed) || has(DeltaKind::ConfidenceDeclined)
        }
        SnapshotDeltaFilter::Direction => has(DeltaKind::DirectionChanged),
        SnapshotDeltaFilter::Evidence => has(DeltaKind::EvidenceChanged),
        SnapshotDeltaFilter::SourceNotes => has(DeltaKind::SourceNotesChanged),
        SnapshotDeltaFilter::Other => {
            !has(DeltaKind::Removed)
                && !has(DeltaKind::ConfidenceDeclined)
                && !has(DeltaKind::DirectionChanged)
                && !has(DeltaKind::EvidenceChanged)
                && !has(DeltaKind::SourceNotesChanged)
        }
    };
    if !matches_category {
        return false;
    }
    let search = search.trim().to_lowercase();
    if search.is_empty() {
        return true;
    }
    std::iter::once(&delta.connection)
        .chain(delta.previous_connection.iter())
        .any(|fact| {
            format!(
                "{} {} {} {} {} {} {} {} {} {} {} {}",
                comparison_key(fact),
                fact.connection_id,
                fact.source,
                fact.source_type,
                fact.relationship,
                fact.target,
                fact.target_type,
                fact.date,
                fact.confidence,
                fact.directed,
                fact.source_reference,
                fact.notes
            )
            .to_lowercase()
            .contains(&search)
        })
}

fn entity_identity_connection_key(
    source: &str,
    source_type: &str,
    source_entity_id: &str,
    target: &str,
    target_type: &str,
    target_entity_id: &str,
    relationship: &str,
    directed: bool,
) -> String {
    let endpoint = |name: &str, kind: &str, external_id: &str| {
        if external_id.trim().is_empty() {
            format!("name:{}:{}", name.to_lowercase(), kind.to_lowercase())
        } else {
            format!("id:{}", external_id.trim())
        }
    };
    let mut source_key = endpoint(source, source_type, source_entity_id);
    let mut target_key = endpoint(target, target_type, target_entity_id);
    if !directed && source_key > target_key {
        std::mem::swap(&mut source_key, &mut target_key);
    }
    serde_json::to_string(&(
        source_key,
        relationship.trim().to_lowercase(),
        target_key,
        directed,
    ))
    .unwrap_or_default()
}

fn comparison_key(connection: &ConnectionFact) -> String {
    if !connection.connection_id.trim().is_empty() {
        return format!("id:{}", connection.connection_id.trim());
    }
    if !connection.source_entity_id.trim().is_empty()
        || !connection.target_entity_id.trim().is_empty()
    {
        return format!(
            "entity:{}",
            entity_identity_connection_key(
                &connection.source,
                &connection.source_type,
                &connection.source_entity_id,
                &connection.target,
                &connection.target_type,
                &connection.target_entity_id,
                &connection.relationship,
                connection.directed,
            )
        );
    }
    if let Some(key) = connection.key.strip_prefix("natural-v2:") {
        return format!("natural:{key}");
    }
    let (_, _, _, _, key) = canonical_connection(
        &connection.source,
        &connection.source_type,
        &connection.target,
        &connection.target_type,
        &connection.relationship,
        connection.directed,
    )
    .unwrap_or_default();
    format!("natural:{key}")
}

fn snapshot_case_link_index(
    case: &CaseFile,
    database_path: &str,
    source_key: &str,
    fact: &ConnectionFact,
) -> Option<usize> {
    if !fact.connection_id.is_empty() {
        let matches: Vec<usize> = case
            .links
            .iter()
            .enumerate()
            .filter(|(_, link)| {
                link.ingest_source.as_deref() == Some(source_key)
                    && link.connection_id == fact.connection_id
            })
            .map(|(index, _)| index)
            .collect();
        return (matches.len() == 1).then(|| matches[0]);
    }

    let find_endpoint = |name: &str, kind: &str, external_id: &str| {
        let matches: Vec<Uuid> = case
            .entities
            .iter()
            .filter(|entity| {
                if !external_id.trim().is_empty() {
                    entity
                        .external_ids
                        .iter()
                        .any(|(source, id)| source == database_path && id == external_id)
                } else {
                    (entity.name.eq_ignore_ascii_case(name)
                        || entity
                            .aliases
                            .iter()
                            .any(|alias| alias.eq_ignore_ascii_case(name)))
                        && compatible_entity_types(&entity.kind, kind)
                }
            })
            .map(|entity| entity.id)
            .collect();
        (matches.len() == 1).then(|| matches[0])
    };
    let source = find_endpoint(&fact.source, &fact.source_type, &fact.source_entity_id)?;
    let target = find_endpoint(&fact.target, &fact.target_type, &fact.target_entity_id)?;
    let matches: Vec<usize> = case
        .links
        .iter()
        .enumerate()
        .filter(|(_, link)| {
            link.ingest_source.as_deref() == Some(source_key)
                && link.connection_id.is_empty()
                && link.label.eq_ignore_ascii_case(&fact.relationship)
                && link.directed == fact.directed
                && ((link.from == source && link.to == target)
                    || (!fact.directed && link.from == target && link.to == source))
        })
        .map(|(index, _)| index)
        .collect();
    (matches.len() == 1).then(|| matches[0])
}

fn delta_order(kinds: &[DeltaKind]) -> u8 {
    [
        DeltaKind::Removed,
        DeltaKind::ConfidenceDeclined,
        DeltaKind::DateChanged,
        DeltaKind::Updated,
        DeltaKind::EvidenceChanged,
        DeltaKind::SourceNotesChanged,
        DeltaKind::Added,
        DeltaKind::ConfidenceIncreased,
    ]
    .iter()
    .position(|kind| kinds.contains(kind))
    .unwrap_or(usize::MAX) as u8
}

fn automation_run_matches_filter(run: &AutomationRun, filter: &str, failures_only: bool) -> bool {
    let result = if run.success { "PASS" } else { "FAIL" };
    (!failures_only || !run.success)
        && (filter.is_empty()
            || format!(
                "{} {} {} {} {} {} {} {} {}",
                result,
                run.job_name,
                run.database_path,
                run.series,
                run.query,
                run.message,
                run.started_at,
                run.finished_at,
                run.id,
            )
            .to_lowercase()
            .contains(filter))
}

fn automation_regression_count(case: &CaseFile, source_key: &str) -> usize {
    let mut snapshots = case
        .snapshots
        .iter()
        .rev()
        .filter(|snapshot| snapshot.source_key == source_key);
    let Some(current) = snapshots.next() else {
        return 0;
    };
    let Some(previous) = snapshots.next() else {
        return 0;
    };
    compare_snapshots(previous, current)
        .iter()
        .filter(|delta| {
            delta
                .kinds
                .iter()
                .any(|kind| matches!(kind, DeltaKind::Removed | DeltaKind::ConfidenceDeclined))
        })
        .count()
}

fn automation_limits_pass(
    connections: usize,
    regressions: usize,
    skipped_rows: usize,
    minimum_connections: usize,
    maximum_connections: Option<usize>,
    maximum_regressions: Option<usize>,
    maximum_skipped_rows: Option<usize>,
) -> bool {
    connections >= minimum_connections
        && maximum_connections.is_none_or(|maximum| connections <= maximum)
        && maximum_regressions.is_none_or(|maximum| regressions <= maximum)
        && maximum_skipped_rows.is_none_or(|maximum| skipped_rows <= maximum)
}

fn previous_snapshot_index(case: &CaseFile, current_index: usize) -> Option<usize> {
    let current = case.snapshots.get(current_index)?;
    (0..current_index)
        .rev()
        .find(|index| case.snapshots[*index].source_key == current.source_key)
}

fn snapshot_delta_json(
    current: &ConnectionSnapshot,
    previous: Option<&ConnectionSnapshot>,
) -> serde_json::Value {
    let Some(previous) = previous else {
        return serde_json::json!({
            "baseline": true,
            "compared_to_snapshot_id": null,
            "counts": {
                "removed": 0,
                "confidence_declined": 0,
                "date_changed": 0,
                "connection_updated": 0,
                "direction_changed": 0,
                "evidence_changed": 0,
                "source_notes_changed": 0,
                "added": 0,
                "confidence_increased": 0,
                "changed_records": 0
            },
            "changes": []
        });
    };
    let deltas = compare_snapshots(previous, current);
    let count = |kind| {
        deltas
            .iter()
            .filter(|delta| delta.kinds.contains(&kind))
            .count()
    };
    let changes: Vec<_> = deltas
        .iter()
        .map(|delta| {
            let kinds: Vec<&str> = delta
                .kinds
                .iter()
                .map(|kind| match kind {
                    DeltaKind::Removed => "removed",
                    DeltaKind::ConfidenceDeclined => "confidence_declined",
                    DeltaKind::DateChanged => "date_changed",
                    DeltaKind::Updated => "connection_updated",
                    DeltaKind::DirectionChanged => "direction_changed",
                    DeltaKind::EvidenceChanged => "evidence_changed",
                    DeltaKind::SourceNotesChanged => "source_notes_changed",
                    DeltaKind::Added => "added",
                    DeltaKind::ConfidenceIncreased => "confidence_increased",
                })
                .collect();
            let fact = |connection: &ConnectionFact| {
                serde_json::json!({
                    "key": comparison_key(connection),
                    "connection_id": connection.connection_id,
                    "source": connection.source,
                    "source_type": connection.source_type,
                    "source_entity_id": connection.source_entity_id,
                    "relationship": connection.relationship,
                    "target": connection.target,
                    "target_type": connection.target_type,
                    "target_entity_id": connection.target_entity_id,
                    "date": connection.date,
                    "confidence": connection.confidence,
                    "directed": connection.directed,
                    "source_reference": connection.source_reference,
                    "source_notes": connection.notes
                })
            };
            serde_json::json!({
                "kinds": kinds,
                "connection": fact(&delta.connection),
                "previous": delta.previous_connection.as_ref().map(fact)
            })
        })
        .collect();
    serde_json::json!({
        "baseline": false,
        "compared_to_snapshot_id": previous.id,
        "counts": {
            "removed": count(DeltaKind::Removed),
            "confidence_declined": count(DeltaKind::ConfidenceDeclined),
            "date_changed": count(DeltaKind::DateChanged),
            "connection_updated": count(DeltaKind::Updated),
            "direction_changed": count(DeltaKind::DirectionChanged),
            "evidence_changed": count(DeltaKind::EvidenceChanged),
            "source_notes_changed": count(DeltaKind::SourceNotesChanged),
            "added": count(DeltaKind::Added),
            "confidence_increased": count(DeltaKind::ConfidenceIncreased),
            "changed_records": deltas.len()
        },
        "changes": changes
    })
}

fn snapshot_result_quality(snapshot: &ConnectionSnapshot) -> &'static str {
    if snapshot.connections.is_empty() {
        if snapshot.skipped_rows == 0 {
            "empty_result"
        } else {
            "all_rows_skipped"
        }
    } else if snapshot.skipped_rows == 0 {
        "usable"
    } else {
        "partially_skipped"
    }
}

fn snapshot_result_quality_label(quality: &str) -> &'static str {
    match quality {
        "empty_result" => "EMPTY RESULT",
        "all_rows_skipped" => "ALL ROWS SKIPPED",
        "partially_skipped" => "PARTIAL SKIPS",
        _ => "USABLE RESULT",
    }
}

fn automation_outcome_json(
    job: &AutomationJob,
    success: bool,
    message: &str,
    run_id: Option<Uuid>,
    snapshot: Option<&ConnectionSnapshot>,
    previous: Option<&ConnectionSnapshot>,
) -> serde_json::Value {
    let delta = snapshot.map(|snapshot| snapshot_delta_json(snapshot, previous));
    let regression_count = delta
        .as_ref()
        .and_then(|delta| {
            delta
                .pointer("/counts/removed")
                .and_then(serde_json::Value::as_u64)
        })
        .unwrap_or_default() as usize
        + delta
            .as_ref()
            .and_then(|delta| {
                delta
                    .pointer("/counts/confidence_declined")
                    .and_then(serde_json::Value::as_u64)
            })
            .unwrap_or_default() as usize;
    let connection_count = snapshot.map(|snapshot| snapshot.connections.len());
    let skipped_rows = snapshot.map(|snapshot| snapshot.skipped_rows);
    let result_quality = snapshot.map(snapshot_result_quality);
    serde_json::json!({
        "schema_version": 1,
        "status": if success { "PASS" } else { "FAIL" },
        "job": job.name,
        "job_id": job.id,
        "run_id": run_id,
        "duration_ms": serde_json::Value::Null,
        "database_path": job.database_path,
        "series": job.series,
        "message": message,
        "query_valid": snapshot.is_some(),
        "result_quality": result_quality,
        "has_usable_connections": connection_count.is_some_and(|count| count > 0),
        "capture_limits_pass": snapshot.map(|_| {
            automation_limits_pass(
                connection_count.unwrap_or_default(),
                regression_count,
                skipped_rows.unwrap_or_default(),
                job.minimum_connections,
                job.maximum_connections,
                job.maximum_regressions,
                job.maximum_skipped_rows,
            )
        }),
        "snapshot_id": snapshot.map(|snapshot| snapshot.id),
        "captured_at": snapshot.map(|snapshot| snapshot.captured_at.as_str()),
        "source_key": snapshot.map(|snapshot| snapshot.source_key.as_str()),
        "connection_count": connection_count,
        "skipped_rows": skipped_rows,
        "minimum_connections": job.minimum_connections,
        "maximum_connections": job.maximum_connections,
        "maximum_regressions": job.maximum_regressions,
        "maximum_skipped_rows": job.maximum_skipped_rows,
        "query": job.query,
        "parameters": job.parameters,
        "limits": {
            "minimum_connections": {
                "required": job.minimum_connections,
                "observed": connection_count,
                "passed": connection_count.map(|count| count >= job.minimum_connections)
            },
            "maximum_connections": {
                "limit": job.maximum_connections,
                "observed": connection_count,
                "passed": connection_count.map(|count| job.maximum_connections.is_none_or(|limit| count <= limit))
            },
            "maximum_regressions": {
                "limit": job.maximum_regressions,
                "observed": snapshot.map(|_| regression_count),
                "passed": snapshot.map(|_| job.maximum_regressions.is_none_or(|limit| regression_count <= limit))
            },
            "maximum_skipped_rows": {
                "limit": job.maximum_skipped_rows,
                "observed": skipped_rows,
                "passed": skipped_rows.map(|count| job.maximum_skipped_rows.is_none_or(|limit| count <= limit))
            }
        },
        "delta": delta
    })
}

fn automation_regression_summary(case: &CaseFile, source_key: &str) -> String {
    let mut snapshots = case
        .snapshots
        .iter()
        .rev()
        .filter(|snapshot| snapshot.source_key == source_key);
    let Some(current) = snapshots.next() else {
        return "NO SNAPSHOT COMPARISON".into();
    };
    let Some(baseline) = snapshots.next() else {
        return "BASELINE · no previous snapshot".into();
    };
    let deltas = compare_snapshots(baseline, current);
    let count = |kind| {
        deltas
            .iter()
            .filter(|delta| delta.kinds.contains(&kind))
            .count()
    };
    let removed = count(DeltaKind::Removed);
    let declined = count(DeltaKind::ConfidenceDeclined);
    let dates = count(DeltaKind::DateChanged);
    let added = count(DeltaKind::Added);
    let increased = count(DeltaKind::ConfidenceIncreased);
    let details = count(DeltaKind::Updated);
    let direction = count(DeltaKind::DirectionChanged);
    let evidence = count(DeltaKind::EvidenceChanged);
    let source_notes = count(DeltaKind::SourceNotesChanged);
    if deltas.is_empty() {
        "NO CONNECTION CHANGES".into()
    } else {
        format!(
            "SNAPSHOT DELTA · {removed} removed · {declined} confidence declines · {dates} date changes · {details} connection detail changes · {direction} direction changes · {evidence} evidence changes · {source_notes} source note changes · {added} added · {increased} confidence increases"
        )
    }
}

fn shortest_path(case: &CaseFile, start: Uuid, goal: Uuid) -> Option<Vec<Uuid>> {
    if !case.entities.iter().any(|entity| entity.id == start)
        || !case.entities.iter().any(|entity| entity.id == goal)
    {
        return None;
    }
    let mut adjacency: HashMap<Uuid, Vec<Uuid>> = HashMap::new();
    for link in &case.links {
        adjacency.entry(link.from).or_default().push(link.to);
        if !link.directed {
            adjacency.entry(link.to).or_default().push(link.from);
        }
    }
    let mut queue = VecDeque::from([start]);
    let mut previous: HashMap<Uuid, Uuid> = HashMap::new();
    let mut seen = std::collections::HashSet::from([start]);
    while let Some(current) = queue.pop_front() {
        if current == goal {
            break;
        }
        for next in adjacency.get(&current).into_iter().flatten() {
            if seen.insert(*next) {
                previous.insert(*next, current);
                queue.push_back(*next);
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
        if let Some(worker_state) = self.automation_worker.as_ref().map(Receiver::try_recv) {
            match worker_state {
                Ok(Ok(result)) => {
                    self.automation_worker = None;
                    self.automation_progress = None;
                    match result {
                        AutomationWorkerResult::SavedCase {
                            case,
                            disk_case_json,
                            status,
                            selected_snapshot,
                            selected_link,
                        } => {
                            self.case = case;
                            self.disk_case_json = Some(disk_case_json);
                            self.saved_case_json = self.current_case_json();
                            self.selected_link = selected_link;
                            if let Some((source, index)) = selected_snapshot {
                                self.sql_snapshot_source = Some(source);
                                self.sql_snapshot_index = Some(index);
                            }
                            self.status = status;
                            self.invalidate_network_groups();
                        }
                        AutomationWorkerResult::FailedCase {
                            case,
                            disk_case_json,
                            error,
                            selected_snapshot,
                            selected_link,
                        } => {
                            self.case = case;
                            self.disk_case_json = Some(disk_case_json);
                            self.saved_case_json = self.current_case_json();
                            self.selected_link = selected_link;
                            if let Some((source, index)) = selected_snapshot {
                                self.sql_snapshot_source = Some(source);
                                self.sql_snapshot_index = Some(index);
                            }
                            self.status =
                                format!("AUTOMATION FAILED · {error} · SAVED CHECKPOINTS RELOADED");
                            self.invalidate_network_groups();
                        }
                        AutomationWorkerResult::UnsavedCase {
                            case,
                            status,
                            selected_snapshot,
                            selected_link,
                        } => {
                            self.case = case;
                            self.selected_link = selected_link;
                            if let Some((source, index)) = selected_snapshot {
                                self.sql_snapshot_source = Some(source);
                                self.sql_snapshot_index = Some(index);
                            }
                            self.status = status;
                            self.invalidate_network_groups();
                        }
                        AutomationWorkerResult::Validation { preview, status } => {
                            self.sql_validation_preview = preview;
                            self.status = status;
                        }
                    }
                }
                Ok(Err(error)) => {
                    self.automation_worker = None;
                    self.automation_progress = None;
                    self.status = format!("AUTOMATION FAILED · {error}");
                }
                Err(mpsc::TryRecvError::Disconnected) => {
                    self.automation_worker = None;
                    self.automation_progress = None;
                    self.status =
                        "AUTOMATION FAILED · background worker stopped unexpectedly".into();
                }
                Err(mpsc::TryRecvError::Empty) => {
                    ctx.request_repaint_after(Duration::from_millis(250));
                    egui::CentralPanel::default().show(ctx, |ui| {
                        ui.vertical_centered(|ui| {
                            ui.add_space(ui.available_height() * 0.35);
                            ui.heading("SQL operation is running");
                            ui.add_space(10.0);
                            ui.add(egui::Spinner::new());
                            ui.add_space(10.0);
                            ui.label("The window remains responsive. Editing is paused until the query finishes.");
                            if self.status.starts_with("AUTOMATION RUNNING") {
                                ui.label("Automation saves a checkpoint after each job.");
                            }
                            if let Some(progress) = self
                                .automation_progress
                                .as_ref()
                                .and_then(|progress| progress.lock().ok().map(|value| value.clone()))
                            {
                                ui.add_space(8.0);
                                ui.label(progress);
                            }
                            ui.add_space(12.0);
                            ui.label(&self.status);
                        });
                    });
                    return;
                }
            }
        }
        let (save, save_as, delete, escape, workspace_shortcut) = ctx.input(|input| {
            (
                input.key_pressed(egui::Key::S) && input.modifiers.command,
                input.key_pressed(egui::Key::S) && input.modifiers.command && input.modifiers.shift,
                input.key_pressed(egui::Key::Delete),
                input.key_pressed(egui::Key::Escape),
                if input.modifiers.command && input.key_pressed(egui::Key::Num1) {
                    Some((WorkspacePage::Chart, false))
                } else if input.modifiers.command && input.key_pressed(egui::Key::Num2) {
                    Some((WorkspacePage::Chart, true))
                } else if input.modifiers.command && input.key_pressed(egui::Key::Num3) {
                    Some((WorkspacePage::Data, false))
                } else {
                    None
                },
            )
        });
        if save_as {
            self.save_as();
        } else if save {
            self.save();
        }
        if !ctx.wants_keyboard_input()
            && let Some((page, show_timeline)) = workspace_shortcut
        {
            self.page = page;
            self.show_timeline = show_timeline;
        }
        if delete && self.pending_case_action.is_none() && !ctx.wants_keyboard_input() {
            self.delete_selected();
        }
        if escape {
            if self.pending_case_action.is_some() {
                self.pending_case_action = None;
            } else if !ctx.wants_keyboard_input() {
                self.selected = None;
                self.selected_link = None;
                self.chart_focus = None;
                self.chart_focus_link = None;
                self.chart_fit_entities = None;
                self.chart_fit_path = None;
                self.chart_focus_group = None;
            }
        }

        let connected_counts = connected_entity_counts(&self.case);
        let network_groups = self.network_groups.clone();
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
                            if ui.button("Save as").clicked() { self.save_as(); }
                            if ui.button("Open case").clicked() { self.request_case_action(PendingCaseAction::Open); }
                            if ui.button("New case").clicked() { self.request_case_action(PendingCaseAction::New); }
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
                        let item=Entity { id:Uuid::new_v4(),name:"New entity".into(),kind:"PERSON".into(),detail:"Add an analyst note".into(),attributes:Vec::new(),aliases:Vec::new(),external_ids:Vec::new(),merged_records:Vec::new(),x:0.5,y:0.5,position_locked:false,color:[55,128,162] };
                        self.selected=Some(item.id); self.selected_link=None; self.case.entities.push(item);self.duplicate_candidates.clear();
                        self.invalidate_network_groups();
                    }
                    let link_entities: Vec<(Uuid, String)> = self
                        .case
                        .entities
                        .iter()
                        .map(|entity| (entity.id, format!("{} · {}", entity.name, entity.kind)))
                        .collect();
                    if self.new_link_from.is_none_or(|id| !link_entities.iter().any(|(entity, _)| *entity == id)) {
                        self.new_link_from = self
                            .selected
                            .filter(|id| link_entities.iter().any(|(entity, _)| entity == id))
                            .or_else(|| link_entities.first().map(|(id, _)| *id));
                    }
                    if self.new_link_to.is_none_or(|id| !link_entities.iter().any(|(entity, _)| *entity == id))
                        || self.new_link_to == self.new_link_from
                    {
                        self.new_link_to = link_entities
                            .iter()
                            .map(|(id, _)| *id)
                            .find(|id| Some(*id) != self.new_link_from);
                    }
                    ui.menu_button("＋ Link", |ui| {
                        if link_entities.len() < 2 {
                            ui.label("Add another entity before creating a link.");
                            return;
                        }
                        let entity_label = |id: Option<Uuid>| {
                            id.and_then(|id| link_entities.iter().find(|(entity, _)| *entity == id))
                                .map(|(_, label)| label.as_str())
                                .unwrap_or("Choose an entity")
                        };
                        ui.label("From");
                        egui::ComboBox::from_id_salt("new_link_from")
                            .selected_text(entity_label(self.new_link_from))
                            .show_ui(ui, |ui| {
                                for (id, label) in &link_entities {
                                    ui.selectable_value(&mut self.new_link_from, Some(*id), label);
                                }
                            });
                        if self.new_link_to == self.new_link_from {
                            self.new_link_to = link_entities
                                .iter()
                                .map(|(id, _)| *id)
                                .find(|id| Some(*id) != self.new_link_from);
                        }
                        ui.label("To");
                        egui::ComboBox::from_id_salt("new_link_to")
                            .selected_text(entity_label(self.new_link_to))
                            .show_ui(ui, |ui| {
                                for (id, label) in &link_entities {
                                    if Some(*id) != self.new_link_from {
                                        ui.selectable_value(&mut self.new_link_to, Some(*id), label);
                                    }
                                }
                            });
                        let endpoints = self
                            .new_link_from
                            .zip(self.new_link_to)
                            .filter(|(from, to)| from != to);
                        if ui.add_enabled(endpoints.is_some(), egui::Button::new("Create link")).clicked() {
                            if let Some((from, to)) = endpoints {
                                self.case.links.push(Link {
                                    from,
                                    to,
                                    connection_id: String::new(),
                                    label: "ASSOCIATED WITH".into(),
                                    date: "2025-02-15".into(),
                                    confidence: 2,
                                    directed: true,
                                    notes: String::new(),
                                    source_notes: String::new(),
                                    source_reference: String::new(),
                                    ingest_source: None,
                                });
                                self.invalidate_network_groups();
                                self.selected = None;
                                self.selected_link = Some(self.case.links.len() - 1);
                                self.duplicate_candidates.clear();
                                ui.close_menu();
                            }
                        }
                    });
                    if ui.button("Import CSV/TSV").clicked(){self.inspect_delimited_import();}
                    if ui.button("Import GraphML").clicked(){self.import_graphml();}
                    if ui.button("Export CSV/TSV").clicked(){self.export_delimited();}
                    if ui.button("Export GraphML").clicked(){self.export_graphml();}
                    egui::ComboBox::from_id_salt("layout_mode")
                        .selected_text(match self.layout_mode {
                            LayoutMode::Connectivity => "Connectivity layout",
                            LayoutMode::Hierarchy => "Hierarchy layout",
                            LayoutMode::Grid => "Grid layout",
                        })
                        .show_ui(ui, |ui| {
                            ui.selectable_value(
                                &mut self.layout_mode,
                                LayoutMode::Connectivity,
                                "Connectivity",
                            );
                            ui.selectable_value(
                                &mut self.layout_mode,
                                LayoutMode::Hierarchy,
                                "Hierarchy",
                            );
                            ui.selectable_value(&mut self.layout_mode, LayoutMode::Grid, "Grid");
                        });
                    if ui.button("Arrange chart").clicked(){self.auto_layout();}
                    if self.page == WorkspacePage::Chart {
                        if ui
                            .add_enabled(
                                !self.case.entities.is_empty(),
                                egui::Button::new("Fit all"),
                            )
                            .clicked()
                        {
                            self.chart_fit_entities = Some(
                                self.case.entities.iter().map(|entity| entity.id).collect(),
                            );
                            self.chart_fit_path = None;
                            self.chart_focus = None;
                            self.chart_focus_link = None;
                            self.chart_focus_group = None;
                            self.sql_source_focus = None;
                            self.path_result.clear();
                        }
                        if ui.small_button("−").clicked() {
                            self.chart_zoom = (self.chart_zoom / 1.2).clamp(0.25, 3.0);
                        }
                        ui.label(format!("{:.0}%", self.chart_zoom * 100.0));
                        if ui.small_button("＋").clicked() {
                            self.chart_zoom = (self.chart_zoom * 1.2).clamp(0.25, 3.0);
                        }
                        if ui.button("Reset view").clicked() {
                            self.chart_zoom = 1.0;
                            self.chart_pan = egui::Vec2::ZERO;
                            self.chart_focus = None;
                            self.chart_focus_link = None;
                            self.chart_fit_entities = None;
                            self.chart_fit_path = None;
                            self.chart_focus_group = None;
                        }
                    }
                    if self.page==WorkspacePage::Chart && self.sql_source_focus.is_some() {ui.label(egui::RichText::new("SOURCE HIGHLIGHT").small().color(cyan));if ui.small_button("Clear source").clicked(){self.sql_source_focus=None;}}
                    if self.page==WorkspacePage::Data && (self.selected.is_some() || self.selected_link.is_some()) && ui.button("View on chart").clicked(){self.chart_focus=self.selected;self.chart_focus_link=self.selected_link;self.chart_focus_group=None;self.page=WorkspacePage::Chart;self.show_timeline=false;}
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
                            EntitySort::Connections=>connected_counts.get(&b.id).copied().unwrap_or_default().cmp(&connected_counts.get(&a.id).copied().unwrap_or_default()),
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
                            ui.horizontal(|ui|{ui.label(egui::RichText::new("CASE RECORDS").strong().color(cyan));ui.selectable_value(&mut self.data_mode,DataMode::Entities,"Entities");ui.selectable_value(&mut self.data_mode,DataMode::Relationships,"Connections");ui.selectable_value(&mut self.data_mode,DataMode::SqlRegressions,"SQL regressions");ui.selectable_value(&mut self.data_mode,DataMode::Automation,"Automation");if self.data_mode==DataMode::Entities||self.data_mode==DataMode::Relationships{ui.label(egui::RichText::new("CSV/TSV carries connection identity, provenance, entity properties and colors").small().color(muted));}});
                            ui.horizontal(|ui|{ui.label("Case file");ui.add(egui::TextEdit::singleline(&mut self.case_path_input).desired_width(260.0));ui.label("Description");ui.add(egui::TextEdit::singleline(&mut self.case.description).desired_width(260.0));});
                            ui.horizontal(|ui|{ui.label("Import path");ui.add(egui::TextEdit::singleline(&mut self.import_path).desired_width(180.0));ui.label("Export path");ui.add(egui::TextEdit::singleline(&mut self.export_path).desired_width(180.0));ui.label("GraphML");ui.add(egui::TextEdit::singleline(&mut self.graphml_export_path).desired_width(180.0));});
                            if self.data_mode==DataMode::Automation {
                                ui.label(egui::RichText::new("REPEATABLE SQL INGESTION JOBS").strong().color(cyan));
                                ui.label(egui::RichText::new("Jobs are saved in this case file. Run them here or from a script with the command-line runner.").small().color(muted));
                                ui.horizontal(|ui|{ui.label("Job name");ui.add(egui::TextEdit::singleline(&mut self.automation_job_name).desired_width(220.0));ui.label("Database");ui.add(egui::TextEdit::singleline(&mut self.sql_path).hint_text("Relative to case file").desired_width(220.0));});
                                ui.horizontal(|ui|{ui.label("Snapshot series");ui.add(egui::TextEdit::singleline(&mut self.sql_series).desired_width(220.0));ui.label("Minimum connections");ui.add(egui::DragValue::new(&mut self.automation_minimum_connections).range(0..=1_000_000));});
                                ui.horizontal(|ui| {
                                    let mut limit_connections = self.automation_maximum_connections.is_some();
                                    if ui.checkbox(&mut limit_connections, "Limit connections").changed() {
                                        self.automation_maximum_connections = limit_connections.then_some(0);
                                    }
                                    if let Some(maximum) = self.automation_maximum_connections.as_mut() {
                                        ui.label("Maximum captured connections");
                                        ui.add(egui::DragValue::new(maximum).range(0..=1_000_000));
                                    }
                                });
                                ui.horizontal(|ui| {
                                    let mut limit_skipped_rows =
                                        self.automation_maximum_skipped_rows.is_some();
                                    if ui
                                        .checkbox(&mut limit_skipped_rows, "Limit skipped rows")
                                        .changed()
                                    {
                                        self.automation_maximum_skipped_rows =
                                            limit_skipped_rows.then_some(0);
                                    }
                                    if let Some(maximum) =
                                        self.automation_maximum_skipped_rows.as_mut()
                                    {
                                        ui.label("Maximum empty-endpoint rows");
                                        ui.add(egui::DragValue::new(maximum).range(0..=1_000_000));
                                    }
                                });
                                ui.horizontal(|ui|{let mut limit_regressions=self.automation_maximum_regressions.is_some();if ui.checkbox(&mut limit_regressions,"Limit regressions").changed(){self.automation_maximum_regressions=limit_regressions.then_some(0);}if let Some(maximum)=self.automation_maximum_regressions.as_mut(){ui.label("Maximum removals/confidence declines");ui.add(egui::DragValue::new(maximum).range(0..=1_000_000));}if ui.button("Build query from schema").clicked(){self.build_sql_query_from_schema();}if ui.button("Validate query").clicked(){self.start_sql_validation();}if ui.button(if self.editing_automation_job.is_some(){"Save job changes"}else{"Add job"}).clicked(){self.save_automation_job();}if self.editing_automation_job.is_some()&&ui.button("Cancel edit").clicked(){self.cancel_automation_job_edit();}});
                                if self.sql_schema_templates.len() > 1 {
                                    ui.horizontal(|ui| {
                                        ui.label("Query source table or view");
                                        self.show_sql_schema_template_picker(ui);
                                    });
                                }
                                ui.label(egui::RichText::new("A regression is a removed connection or confidence decline. Runs over either configured limit are saved as failed, with their snapshots retained.").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_query).desired_width(canvas_w-20.0).desired_rows(5).code_editor());
                                ui.label(egui::RichText::new("Named SQL parameters · name=value per line; bind as text with :name, @name, or $name").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_parameter_text).desired_width(canvas_w-20.0).desired_rows(2).code_editor());
                                Self::show_sql_validation_preview(ui, self.sql_validation_preview.as_ref(), &self.sql_path, &self.sql_series, &self.sql_query, &self.sql_parameter_text);
                                let enabled_job_count = self
                                    .case
                                    .automation_jobs
                                    .iter()
                                    .filter(|job| job.enabled)
                                    .count();
                                ui.horizontal(|ui| {
                                    ui.label(format!("Batch includes {enabled_job_count} enabled jobs"));
                                    if ui
                                        .add_enabled(
                                            enabled_job_count > 0,
                                            egui::Button::new("Run enabled jobs and save"),
                                        )
                                        .clicked()
                                    {
                                        self.start_enabled_automation_jobs();
                                    }
                                });
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("JOBS").small().strong().color(cyan));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.automation_job_filter)
                                            .hint_text("Find by job, database, series, or SQL")
                                            .desired_width(280.0),
                                    );
                                    if ui.small_button("Clear").clicked() {
                                        self.automation_job_filter.clear();
                                    }
                                });
                                let job_filter = self.automation_job_filter.trim().to_lowercase();
                                let visible_job_ids: HashSet<Uuid> = self
                                    .case
                                    .automation_jobs
                                    .iter()
                                    .filter(|job| {
                                        job_filter.is_empty()
                                            || format!(
                                                "{} {} {} {}",
                                                job.name,
                                                job.database_path,
                                                job.series,
                                                job.query
                                            )
                                            .to_lowercase()
                                            .contains(&job_filter)
                                    })
                                    .map(|job| job.id)
                                    .collect();
                                ui.label(egui::RichText::new(format!(
                                    "{} OF {} JOBS",
                                    visible_job_ids.len(),
                                    self.case.automation_jobs.len()
                                )).small().color(muted));
                                let mut action: Option<(String,Uuid)> = None;
                                for job in &mut self.case.automation_jobs {
                                    if !visible_job_ids.contains(&job.id) {
                                        continue;
                                    }
                                    let latest_run = self
                                        .case
                                        .automation_runs
                                        .iter()
                                        .rev()
                                        .find(|run| run.job_id == job.id)
                                        .map(|run| {
                                            (
                                                run.success,
                                                run.finished_at.clone(),
                                                run.message.clone(),
                                                run.duration_ms,
                                            )
                                        });
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut job.enabled, "");
                                        ui.label(format!("{} · {} · {} · min {} · max connections {} · max regressions {} · max skipped rows {}",job.name,job.series,job.database_path,job.minimum_connections,job.maximum_connections.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),job.maximum_regressions.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),job.maximum_skipped_rows.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into())));
                                        if ui.small_button("Validate").clicked(){action=Some(("validate".into(),job.id));}
                                        if ui.small_button("Edit").clicked(){action=Some(("edit".into(),job.id));}
                                        if ui.small_button("Run and save").clicked(){action=Some(("run".into(),job.id));}
                                        if ui.small_button("Delete").clicked(){action=Some(("delete".into(),job.id));}
                                    });
                                    if let Some((success, finished_at, message, duration_ms)) = latest_run {
                                        let result = if success { "PASS" } else { "FAIL" };
                                        ui.colored_label(
                                            if success { egui::Color32::from_rgb(102, 190, 146) } else { egui::Color32::from_rgb(222, 112, 104) },
                                            egui::RichText::new(format!(
                                                "LAST RUN · {result} · {finished_at} · {} ms",
                                                duration_ms
                                                    .map(|value| value.to_string())
                                                    .unwrap_or_else(|| "unknown duration".into())
                                            )).small(),
                                        ).on_hover_text(message);
                                    } else {
                                        ui.label(egui::RichText::new("LAST RUN · NEVER RUN").small().color(muted));
                                    }
                                    egui::CollapsingHeader::new("SQL query and capture configuration")
                                        .id_salt(("automation_job_query", job.id))
                                        .show(ui, |ui| {
                                            ui.label(format!("SQLite {} · series {}", job.database_path, job.series));
                                            ui.label(format!("Parameters: {}", if job.parameters.is_empty() { "(none)".to_owned() } else { format_sql_parameters(&job.parameters) }));
                                            let mut query = job.query.clone();
                                            ui.add(
                                                egui::TextEdit::multiline(&mut query)
                                                    .desired_width(canvas_w - 30.0)
                                                    .desired_rows(3)
                                                    .code_editor()
                                                    .interactive(false),
                                            );
                                            Self::show_sql_validation_preview(
                                                ui,
                                                self.sql_validation_preview.as_ref(),
                                                &job.database_path,
                                                &job.series,
                                                &job.query,
                                                &format_sql_parameters(&job.parameters),
                                            );
                                        });
                                }
                                if let Some((verb,id))=action {match verb.as_str(){"validate"=>{self.validate_automation_job(id);},"edit"=>self.edit_automation_job(id),"run"=>self.start_automation_jobs(vec![id],false),"delete"=>{if self.editing_automation_job==Some(id){self.cancel_automation_job_edit();}self.case.automation_jobs.retain(|job|job.id!=id);self.status="AUTOMATION JOB DELETED".into();},_=>{}}}
                                ui.add_space(8.0);
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("RUN HISTORY · newest first").small().strong().color(cyan));
                                    ui.add(
                                        egui::TextEdit::singleline(&mut self.automation_run_filter)
                                            .hint_text("Find by job, result, database, series, SQL, or message")
                                            .desired_width(300.0),
                                    );
                                    ui.checkbox(&mut self.automation_failed_runs_only, "Failures only");
                                    if ui.small_button("Clear history filters").clicked() {
                                        self.automation_run_filter.clear();
                                        self.automation_failed_runs_only = false;
                                    }
                                });
                                let run_filter = self.automation_run_filter.trim().to_lowercase();
                                let visible_run_count = self
                                    .case
                                    .automation_runs
                                    .iter()
                                    .filter(|run| {
                                        automation_run_matches_filter(
                                            run,
                                            &run_filter,
                                            self.automation_failed_runs_only,
                                        )
                                    })
                                    .count();
                                ui.label(egui::RichText::new(format!(
                                    "{visible_run_count} OF {} RECORDED RUNS",
                                    self.case.automation_runs.len()
                                )).small().color(muted));
                                let mut review_snapshot = None;
                                let mut replay_run = None;
                                egui::ScrollArea::vertical().max_height(210.0).show(ui, |ui| {
                                    for run in self
                                        .case
                                        .automation_runs
                                        .iter()
                                        .rev()
                                        .filter(|run| {
                                            automation_run_matches_filter(
                                                run,
                                                &run_filter,
                                                self.automation_failed_runs_only,
                                            )
                                        })
                                    {
                                        let result = if run.success { "PASS" } else { "FAIL" };
                                        let snapshot = run
                                            .snapshot_id
                                            .and_then(|id| self.case.snapshots.iter().find(|snapshot| snapshot.id == id))
                                            .or_else(|| run.snapshot_index.and_then(|index| self.case.snapshots.get(index)));
                                        let quality = snapshot
                                            .map(snapshot_result_quality)
                                            .map(snapshot_result_quality_label)
                                            .unwrap_or("NO SNAPSHOT");
                                        egui::CollapsingHeader::new(format!(
                                            "{result} · {quality} · {} · {}",
                                            run.job_name,
                                            run.started_at
                                        ))
                                        .id_salt(("automation_run", run.id))
                                        .show(ui, |ui| {
                                            ui.label(format!("Finished {}", run.finished_at));
                                            if let Some(source_run) = run.rerun_of {
                                                ui.label(format!("Replayed from run {source_run}"));
                                            }
                                            if let Some(duration_ms) = run.duration_ms {
                                                ui.label(format!("Query and validation time: {duration_ms} ms"));
                                            }
                                            ui.label(&run.message);
                                            let snapshot_index = run
                                                .snapshot_id
                                                .and_then(|id| {
                                                    self.case.snapshots.iter().position(|snapshot| snapshot.id == id)
                                                })
                                                .or_else(|| {
                                                    run.snapshot_index.filter(|index| *index < self.case.snapshots.len())
                                                });
                                            if let Some(index) = snapshot_index {
                                                let snapshot = &self.case.snapshots[index];
                                                let previous = previous_snapshot_index(&self.case, index)
                                                    .and_then(|previous_index| self.case.snapshots.get(previous_index));
                                                let delta = snapshot_delta_json(snapshot, previous);
                                                let counts = &delta["counts"];
                                                ui.label(format!(
                                                    "CAPTURE · {} connections · {} skipped rows · {} · {} changed records",
                                                    snapshot.connections.len(),
                                                    snapshot.skipped_rows,
                                                    snapshot_result_quality_label(snapshot_result_quality(snapshot)),
                                                    counts["changed_records"].as_u64().unwrap_or(0),
                                                ));
                                                if let Some(previous) = previous {
                                                    ui.label(format!(
                                                        "Compared with {} · removed {} · confidence declines {} · date changes {} · updates {} · added {}",
                                                        previous.captured_at,
                                                        counts["removed"].as_u64().unwrap_or(0),
                                                        counts["confidence_declined"].as_u64().unwrap_or(0),
                                                        counts["date_changed"].as_u64().unwrap_or(0),
                                                        counts["connection_updated"].as_u64().unwrap_or(0),
                                                        counts["added"].as_u64().unwrap_or(0),
                                                    ));
                                                } else {
                                                    ui.label("This is the baseline capture for its database and series.");
                                                }
                                                ui.label(egui::RichText::new(format!(
                                                    "SNAPSHOT ID · {}",
                                                    snapshot.id
                                                )).small().color(muted));
                                            } else if run.success {
                                                ui.label("This run has no linked snapshot record.");
                                            }
                                            if run.configuration_recorded {
                                                ui.label(format!(
                                                    "Minimum {} connections · maximum connections {} · maximum regressions {} · maximum skipped rows {} · job {}",
                                                    run.minimum_connections,
                                                    run.maximum_connections.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),
                                                    run.maximum_regressions.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),
                                                    run.maximum_skipped_rows.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),
                                                    if run.enabled { "enabled" } else { "disabled" }
                                                ));
                                                ui.label(format!(
                                                    "SQLite {} · series {}",
                                                    run.database_path, run.series
                                                ));
                                                ui.label(format!(
                                                    "SQL parameters: {}",
                                                    if run.parameters.is_empty() {
                                                        "(none)".to_owned()
                                                    } else {
                                                        format_sql_parameters(&run.parameters)
                                                    }
                                                ));
                                            } else {
                                                ui.label(egui::RichText::new(
                                                    "This older run did not save its job configuration.",
                                                ).small().color(muted));
                                            }
                                            if run.configuration_recorded && !run.query.is_empty() {
                                                ui.label(egui::RichText::new("QUERY AT RUN TIME").small().strong().color(cyan));
                                                let mut query = run.query.clone();
                                                ui.add(
                                                    egui::TextEdit::multiline(&mut query)
                                                        .desired_width(canvas_w - 30.0)
                                                        .desired_rows(3)
                                                        .code_editor()
                                                        .interactive(false),
                                                );
                                            }
                                            if run.configuration_recorded
                                                && ui.small_button("Replay saved configuration").clicked()
                                            {
                                                replay_run = Some(run.id);
                                            }
                                            if let Some(index) = snapshot_index {
                                                if ui.small_button(format!("Review snapshot #{}", index + 1)).clicked() {
                                                    review_snapshot = Some(index);
                                                }
                                            }
                                        });
                                    }
                                    if visible_run_count == 0 {
                                        ui.label(egui::RichText::new("No automation runs match these filters.").small().color(muted));
                                    }
                                });
                                if let Some(run_id) = replay_run {
                                    self.replay_saved_automation_run(run_id);
                                }
                                if let Some(index)=review_snapshot {self.sql_snapshot_source=Some(self.case.snapshots[index].source_key.clone());self.sql_snapshot_index=Some(index);self.data_mode=DataMode::SqlRegressions;}
                                return;
                            }
                            if self.data_mode==DataMode::SqlRegressions {
                                let mut sources: Vec<(String, String, usize, usize)> = Vec::new();
                                for snapshot in self.case.snapshots.iter().rev() {
                                    if sources.iter().all(|(key, _, _, _)| key != &snapshot.source_key) {
                                        let source_links: Vec<&Link> = self
                                            .case
                                            .links
                                            .iter()
                                            .filter(|link| {
                                                link.ingest_source.as_deref()
                                                    == Some(snapshot.source_key.as_str())
                                            })
                                            .collect();
                                        let source_entities: HashSet<Uuid> = source_links
                                            .iter()
                                            .flat_map(|link| [link.from, link.to])
                                            .collect();
                                        sources.push((
                                            snapshot.source_key.clone(),
                                            format!("{} · {}", snapshot.series, snapshot.database_path),
                                            source_links.len(),
                                            source_entities.len(),
                                        ));
                                    }
                                }
                                if self.sql_snapshot_source.as_ref().is_none_or(|selected| {
                                    sources.iter().all(|(key, _, _, _)| key != selected)
                                }) {
                                    self.sql_snapshot_source = sources.first().map(|(key, _, _, _)| key.clone());
                                }
                                if !sources.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.label("Review saved series");
                                        egui::ComboBox::from_id_salt("sql_snapshot_source")
                                            .selected_text(
                                                self.sql_snapshot_source
                                                    .as_ref()
                                                    .and_then(|selected| sources.iter().find(|(key, _, _, _)| key == selected))
                                                    .map(|(_, label, links, entities)| format!("{label} · {links} links · {entities} entities"))
                                                    .as_deref()
                                                    .unwrap_or("Choose a series"),
                                            )
                                            .show_ui(ui, |ui| {
                                                for (key, label, links, entities) in &sources {
                                                    ui.selectable_value(
                                                        &mut self.sql_snapshot_source,
                                                        Some(key.clone()),
                                                        format!("{label} · {links} links · {entities} entities"),
                                                    );
                                                }
                                            });
                                    });
                                }
                                let source_key = self.sql_snapshot_source.as_deref();
                                let latest_index = self.case.snapshots.iter().rposition(|snapshot| {
                                    Some(snapshot.source_key.as_str()) == source_key
                                });
                                let selected_index = self.sql_snapshot_index.filter(|index| {
                                    self.case.snapshots.get(*index).is_some_and(|snapshot| {
                                        Some(snapshot.source_key.as_str()) == source_key
                                    })
                                }).or(latest_index);
                                let current_is_latest = selected_index == latest_index;
                                let baseline_index = selected_index.and_then(|current| {
                                    (0..current).rev().find(|index| {
                                        self.case.snapshots[*index].source_key == self.case.snapshots[current].source_key
                                    })
                                });
                                let current_snapshot = selected_index.and_then(|index|self.case.snapshots.get(index)).cloned();
                                let baseline = baseline_index.and_then(|index|self.case.snapshots.get(index)).cloned();
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("INGEST NEW SNAPSHOT").small().strong().color(cyan));
                                ui.horizontal(|ui|{ui.label("SQLite file");ui.add(egui::TextEdit::singleline(&mut self.sql_path).hint_text("Relative to case file").desired_width(260.0));ui.label("Series");ui.add(egui::TextEdit::singleline(&mut self.sql_series).desired_width(260.0));});
                                ui.label(egui::RichText::new("Read-only SQL query · required columns: source, target").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_query).desired_width(canvas_w-20.0).desired_rows(4).code_editor());
                                ui.label(egui::RichText::new("Named SQL parameters · name=value per line; bind as text with :name, @name, or $name").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_parameter_text).desired_width(canvas_w-20.0).desired_rows(2).code_editor());
                                ui.horizontal(|ui|{if ui.button("Build query from schema").clicked(){self.build_sql_query_from_schema();}if ui.button("Validate query").clicked(){self.start_sql_validation();}if ui.button("Ingest SQL snapshot").clicked(){self.start_sql_ingestion();}});
                                if self.sql_schema_templates.len() > 1 {
                                    ui.horizontal(|ui| {
                                        ui.label("Query source table or view");
                                        self.show_sql_schema_template_picker(ui);
                                    });
                                }
                                Self::show_sql_validation_preview(ui, self.sql_validation_preview.as_ref(), &self.sql_path, &self.sql_series, &self.sql_query, &self.sql_parameter_text);
                                if let Some(current)=current_snapshot.clone() {
                                    let chart_source_key = current.source_key.clone();
                                    let chart_database_path = current.database_path.clone();
                                    let capture_label=if selected_index==latest_index{"Latest snapshot"}else{"Selected historical snapshot"};
                                    let skipped = if current.skipped_rows == 0 {
                                        String::new()
                                    } else {
                                        format!(" · {} rows skipped", current.skipped_rows)
                                    };
                                    ui.label(format!("{capture_label} #{}: {} · {} · {} connections{skipped}",selected_index.map(|index|index+1).unwrap_or_default(),current.captured_at,current.series,current.connections.len()));
                                    ui.horizontal(|ui| {
                                        if ui.small_button("Highlight source on chart").clicked() {
                                            self.sql_source_focus = Some(current.source_key.clone());
                                            self.chart_focus_group = None;
                                            self.path_result.clear();
                                            let mut source_entities: Vec<Uuid> = self
                                                .case
                                                .links
                                                .iter()
                                                .filter(|link| {
                                                    link.ingest_source.as_deref()
                                                        == Some(current.source_key.as_str())
                                                })
                                                .flat_map(|link| [link.from, link.to])
                                                .collect::<HashSet<_>>()
                                                .into_iter()
                                                .collect();
                                            source_entities.sort_unstable();
                                            if !source_entities.is_empty() {
                                                self.chart_fit_entities = Some(source_entities);
                                                self.chart_fit_path = None;
                                            }
                                            self.page = WorkspacePage::Chart;
                                            self.show_timeline = false;
                                            self.selected = None;
                                            self.selected_link = None;
                                            self.chart_focus = None;
                                            self.chart_focus_link = None;
                                        }
                                        if self.sql_source_focus.as_deref() == Some(current.source_key.as_str()) {
                                            ui.label(egui::RichText::new("Source highlight is active").small().color(cyan));
                                        }
                                        if ui.small_button("View source records").clicked() {
                                            self.data_query.clear();
                                            self.data_source_filter = Some(current.source_key.clone());
                                            self.data_mode = DataMode::Relationships;
                                            self.page = WorkspacePage::Data;
                                            self.connections_for_selected = false;
                                            self.selected = None;
                                            self.selected_link = None;
                                        }
                                    });
                                    egui::CollapsingHeader::new("Captured SQL configuration")
                                        .id_salt(("snapshot_sql_configuration", current.id))
                                        .show(ui, |ui| {
                                            ui.label(egui::RichText::new("QUERY AT CAPTURE").small().strong().color(cyan));
                                            let mut query = current.query.clone();
                                            ui.add(
                                                egui::TextEdit::multiline(&mut query)
                                                    .desired_width(canvas_w - 30.0)
                                                    .desired_rows(3)
                                                    .code_editor()
                                                    .interactive(false),
                                            );
                                            ui.label(format!(
                                                "Parameters: {}",
                                                if current.parameters.is_empty() {
                                                    "(none)".to_owned()
                                                } else {
                                                    format_sql_parameters(&current.parameters)
                                                }
                                            ));
                                        });
                                    if let Some(base)=baseline {
                                        let deltas=compare_snapshots(&base,&current);
                                        let count=|kind|deltas.iter().filter(|delta|delta.kinds.contains(&kind)).count();
                                        let removed=count(DeltaKind::Removed);
                                        let declined=count(DeltaKind::ConfidenceDeclined);
                                        let dates=count(DeltaKind::DateChanged);
                                        let updated=count(DeltaKind::Updated);
                                        let direction=count(DeltaKind::DirectionChanged);
                                        let evidence=count(DeltaKind::EvidenceChanged);
                                        let source_notes=count(DeltaKind::SourceNotesChanged);
                                        let added=count(DeltaKind::Added);
                                        let increased=count(DeltaKind::ConfidenceIncreased);
                                        ui.label(egui::RichText::new(format!("Baseline {} → current {} · {removed} removed · {declined} confidence declines · {dates} date changes · {updated} connection detail changes · {direction} direction changes · {evidence} evidence changes · {source_notes} source note changes · {added} added · {increased} confidence increases",base.captured_at,current.captured_at)).strong().color(if removed+declined>0{egui::Color32::from_rgb(238,139,115)}else{cyan}));
                                        ui.horizontal(|ui| {
                                            egui::ComboBox::from_id_salt("sql_delta_filter")
                                                .selected_text(match self.sql_delta_filter {
                                                    SnapshotDeltaFilter::All => "All changes",
                                                    SnapshotDeltaFilter::Regressions => "Regressions",
                                                    SnapshotDeltaFilter::Direction => "Direction",
                                                    SnapshotDeltaFilter::Evidence => "Evidence",
                                                    SnapshotDeltaFilter::SourceNotes => "Source notes",
                                                    SnapshotDeltaFilter::Other => "Other changes",
                                                })
                                                .show_ui(ui, |ui| {
                                                    ui.selectable_value(&mut self.sql_delta_filter, SnapshotDeltaFilter::All, "All changes");
                                                    ui.selectable_value(&mut self.sql_delta_filter, SnapshotDeltaFilter::Regressions, "Regressions");
                                                    ui.selectable_value(&mut self.sql_delta_filter, SnapshotDeltaFilter::Direction, "Direction");
                                                    ui.selectable_value(&mut self.sql_delta_filter, SnapshotDeltaFilter::Evidence, "Evidence");
                                                    ui.selectable_value(&mut self.sql_delta_filter, SnapshotDeltaFilter::SourceNotes, "Source notes");
                                                    ui.selectable_value(&mut self.sql_delta_filter, SnapshotDeltaFilter::Other, "Other changes");
                                                });
                                            ui.add(egui::TextEdit::singleline(&mut self.sql_delta_search)
                                                .hint_text("Search IDs, entities, evidence, or notes")
                                                .desired_width(300.0));
                                        });
                                        let visible_deltas: Vec<&ConnectionDelta> = deltas.iter()
                                            .filter(|delta| connection_delta_matches_filter(delta, self.sql_delta_filter, &self.sql_delta_search))
                                            .collect();
                                        ui.label(egui::RichText::new(format!("Showing {} of {} changes · export includes all changes", visible_deltas.len(), deltas.len())).small().color(muted));
                                        if ui.button("Export comparison CSV/TSV").clicked() {
                                            self.export_sql_comparison(&base, &current);
                                        }
                                        egui::ScrollArea::both().show(ui,|ui|egui::Grid::new("sql_regression_table").striped(true).show(ui,|ui|{
                                            for heading in ["CHANGE","CONNECTION","CONFIDENCE","DATE","EVIDENCE","NOTES","CHART"]{ui.label(egui::RichText::new(heading).small().strong().color(cyan));}ui.end_row();
                                            for delta in visible_deltas {
                                                let change=delta.kinds.iter().map(|kind|match kind{DeltaKind::Removed=>"REMOVED",DeltaKind::ConfidenceDeclined=>"CONFIDENCE DOWN",DeltaKind::DateChanged=>"DATE CHANGED",DeltaKind::Updated=>"CONNECTION DETAILS",DeltaKind::DirectionChanged=>"DIRECTION CHANGED",DeltaKind::EvidenceChanged=>"EVIDENCE CHANGED",DeltaKind::SourceNotesChanged=>"SOURCE NOTES CHANGED",DeltaKind::Added=>"ADDED",DeltaKind::ConfidenceIncreased=>"CONFIDENCE UP"}).collect::<Vec<_>>().join(" · ");
                                                let previous=delta.previous_connection.as_ref();
                                                let current=(!delta.kinds.contains(&DeltaKind::Removed)).then_some(&delta.connection);
                                                let describe_connection=|fact:&ConnectionFact|format!("{} [{}] — {} — {} [{}] {}",fact.source,fact.source_type,fact.relationship,fact.target,fact.target_type,if fact.directed{"→"}else{"↔"});
                                                let previous_connection=previous.map(describe_connection).unwrap_or_else(||"—".into());
                                                let current_connection=current.map(describe_connection).unwrap_or_else(||"—".into());
                                                let previous_evidence=previous.map(|fact|fact.source_reference.as_str()).filter(|value|!value.is_empty()).unwrap_or("—");
                                                let current_evidence=current.map(|fact|fact.source_reference.as_str()).filter(|value|!value.is_empty()).unwrap_or("—");
                                                let previous_notes=previous.map(|fact|fact.notes.as_str()).filter(|value|!value.is_empty()).unwrap_or("—");
                                                let current_notes=current.map(|fact|fact.notes.as_str()).filter(|value|!value.is_empty()).unwrap_or("—");
                                                ui.label(change);
                                                ui.label(format!("{previous_connection} → {current_connection}"));
                                                ui.label(format!("{} → {}",delta.previous_confidence.map(|value|value.to_string()).unwrap_or_else(||"—".into()),current.map(|fact|fact.confidence.to_string()).unwrap_or_else(||"—".into())));
                                                ui.label(format!("{} → {}",delta.previous_date.as_deref().unwrap_or("—"),current.map(|fact|fact.date.as_str()).filter(|value|!value.is_empty()).unwrap_or("—")));
                                                ui.label(format!("{previous_evidence} → {current_evidence}"));
                                                ui.label(format!("{previous_notes} → {current_notes}"));
                                                if current_is_latest
                                                    && let Some(fact) = current
                                                    && ui.small_button("Show").clicked()
                                                {
                                                    if let Some(index) = snapshot_case_link_index(&self.case, &chart_database_path, &chart_source_key, fact) {
                                                        self.selected = None;
                                                        self.selected_link = Some(index);
                                                        self.chart_focus = None;
                                                        self.chart_focus_link = Some(index);
                                                        self.chart_focus_group = None;
                                                        self.chart_fit_entities = None;
                                                        self.chart_fit_path = None;
                                                        self.sql_source_focus = None;
                                                        self.path_result.clear();
                                                        self.page = WorkspacePage::Chart;
                                                        self.show_timeline = false;
                                                    } else {
                                                        self.status = "CURRENT CONNECTION IS NOT PRESENT ON THE CHART".into();
                                                    }
                                                } else {
                                                    ui.label(if current.is_some() { "Historical" } else { "—" });
                                                }
                                                ui.end_row();
                                            }
                                        }));
                                    } else {ui.label(egui::RichText::new("This is the baseline snapshot for its database and series. Ingest the next period with the same series to compare connections.").color(muted));}
                                } else {ui.label(egui::RichText::new("No SQL snapshots yet. Ingest a baseline result, then ingest a later result using the same database and series label.").color(muted));}
                                ui.add_space(8.0);ui.label(egui::RichText::new("SNAPSHOT HISTORY · select a capture to compare").small().strong().color(cyan));
                                for (index,snapshot) in self.case.snapshots.iter().enumerate().rev(){if self.sql_snapshot_source.as_deref()==Some(snapshot.source_key.as_str()){let skipped=if snapshot.skipped_rows==0{String::new()}else{format!(" · {} skipped",snapshot.skipped_rows)};if ui.selectable_label(selected_index==Some(index),format!("Snapshot #{} · {} · {} · {} connections{skipped} · {}",index+1,snapshot.captured_at,snapshot.series,snapshot.connections.len(),snapshot.database_path)).clicked(){self.sql_snapshot_index=Some(index);}}}
                                return;
                            }
                            ui.horizontal(|ui|{ui.add(egui::TextEdit::singleline(&mut self.data_query).hint_text(if self.data_mode==DataMode::Entities{"Filter entities…"}else{"Filter connections…"}).desired_width(240.0));if self.data_mode==DataMode::Relationships{let selected_entity=self.selected.is_some_and(|id|self.case.entities.iter().any(|entity|entity.id==id));ui.add_enabled(selected_entity,egui::Checkbox::new(&mut self.connections_for_selected,"Selected entity connections"));egui::ComboBox::from_id_salt("link_sort").selected_text(match self.link_sort {LinkSort::RecordId=>"Sort: record ID",LinkSort::Source=>"Sort: source",LinkSort::Date=>"Sort: date",LinkSort::Relationship=>"Sort: relationship",LinkSort::Target=>"Sort: target",LinkSort::Confidence=>"Sort: confidence",LinkSort::SourceReference=>"Sort: source reference"}).show_ui(ui,|ui|{ui.selectable_value(&mut self.link_sort,LinkSort::RecordId,"Record ID");ui.selectable_value(&mut self.link_sort,LinkSort::Source,"Source");ui.selectable_value(&mut self.link_sort,LinkSort::Relationship,"Relationship");ui.selectable_value(&mut self.link_sort,LinkSort::Target,"Target");ui.selectable_value(&mut self.link_sort,LinkSort::Date,"Date");ui.selectable_value(&mut self.link_sort,LinkSort::Confidence,"Confidence");ui.selectable_value(&mut self.link_sort,LinkSort::SourceReference,"Source reference");});}});
                            if self.data_mode == DataMode::Relationships
                                && let Some(source_filter) = self.data_source_filter.clone()
                            {
                                let label = self
                                    .case
                                    .snapshots
                                    .iter()
                                    .rev()
                                    .find(|snapshot| snapshot.source_key == source_filter)
                                    .map(|snapshot| {
                                        format!("{} · {}", snapshot.series, snapshot.database_path)
                                    })
                                    .unwrap_or_else(|| source_filter.clone());
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new(format!("SQL source filter · {label}")).small().color(cyan));
                                    if ui.small_button("Clear source filter").clicked() {
                                        self.data_source_filter = None;
                                    }
                                });
                            }
                            if self.data_mode==DataMode::Entities {
                                egui::ScrollArea::both().show(ui,|ui|egui::Grid::new("entities_table").striped(true).show(ui,|ui|{
                                    for h in ["ENTITY","TYPE","NOTES","CONNECTIONS","ALIASES","MERGED RECORDS","ATTRIBUTES"]{ui.label(egui::RichText::new(h).small().strong().color(cyan));}ui.end_row();
                                    let mut entities:Vec<&Entity>=self.case.entities.iter().collect();
                                    entities.sort_by(|a,b|match self.entity_sort{EntitySort::Name=>a.name.to_lowercase().cmp(&b.name.to_lowercase()),EntitySort::Type=>a.kind.to_lowercase().cmp(&b.kind.to_lowercase()),EntitySort::Connections=>connected_counts.get(&b.id).copied().unwrap_or_default().cmp(&connected_counts.get(&a.id).copied().unwrap_or_default())});
                                    for entity in entities{if !format!("{} {} {} {:?} {:?}",entity.name,entity.kind,entity.detail,entity.attributes,entity.aliases).to_lowercase().contains(&self.data_query.to_lowercase()){continue;}let selected=self.selected==Some(entity.id);if ui.selectable_label(selected,&entity.name).clicked(){self.selected=Some(entity.id);self.selected_link=None;}ui.label(&entity.kind);ui.label(&entity.detail);let connection_count=connected_counts.get(&entity.id).copied().unwrap_or_default();if ui.selectable_label(false,connection_count.to_string()).on_hover_text("Open this entity's connections").clicked(){self.selected=Some(entity.id);self.selected_link=None;self.connections_for_selected=true;self.data_mode=DataMode::Relationships;}ui.label(entity.aliases.join(" · "));ui.label(entity.merged_records.len().to_string());ui.label(entity.attributes.iter().map(|(k,v)|format!("{k}: {v}")).collect::<Vec<_>>().join(" · "));ui.end_row();}
                                }));
                                return;
                            }
                            if self.connections_for_selected && self.selected.is_none_or(|id|!self.case.entities.iter().any(|entity|entity.id==id)){self.connections_for_selected=false;}
                            let scoped_to=self.connections_for_selected.then_some(self.selected).flatten();
                            let filter=self.data_query.trim().to_lowercase();
                            let entity_names:HashMap<Uuid,(String,String)>=self.case.entities.iter().map(|entity|(entity.id,(entity.name.clone(),entity.kind.clone()))).collect();
                            let mut indices:Vec<usize>=(0..self.case.links.len()).collect();
                            indices.retain(|i|{let link=&self.case.links[*i];if scoped_to.is_some_and(|id|link.from!=id&&link.to!=id){return false;}if self.data_source_filter.as_deref().is_some_and(|source|link.ingest_source.as_deref()!=Some(source)){return false;}let endpoint=|id|entity_names.get(&id).map(|(name,kind)|format!("{name} {kind}")).unwrap_or_else(||"(missing)".into());let source=endpoint(link.from);let target=endpoint(link.to);let direction=if link.directed{"directed →"}else{"undirected ↔"};let confidence=link.confidence.to_string();format!("{} {} {} {} {} {} {} {} {} {} {}",link.connection_id,source,link.label,target,link.date,confidence,direction,link.source_reference,link.source_notes,link.notes,link.ingest_source.as_deref().unwrap_or("manual")).to_lowercase().contains(&filter)});
                            indices.sort_by(|a,b|{let a=&self.case.links[*a];let b=&self.case.links[*b];let from=|link:&Link|entity_names.get(&link.from).map(|(name,_)|name.to_lowercase()).unwrap_or_default();let to=|link:&Link|entity_names.get(&link.to).map(|(name,_)|name.to_lowercase()).unwrap_or_default();match self.link_sort {LinkSort::RecordId=>a.connection_id.to_lowercase().cmp(&b.connection_id.to_lowercase()),LinkSort::Source=>from(a).cmp(&from(b)),LinkSort::Date=>a.date.cmp(&b.date),LinkSort::Relationship=>a.label.to_lowercase().cmp(&b.label.to_lowercase()),LinkSort::Target=>to(a).cmp(&to(b)),LinkSort::Confidence=>b.confidence.cmp(&a.confidence),LinkSort::SourceReference=>a.source_reference.to_lowercase().cmp(&b.source_reference.to_lowercase())}.then_with(||a.connection_id.cmp(&b.connection_id)).then_with(||a.from.cmp(&b.from)).then_with(||a.to.cmp(&b.to))});
                            ui.horizontal(|ui|{ui.label(format!("{} link records",indices.len()));if let Some(id)=scoped_to{if let Some(entity)=self.case.entities.iter().find(|entity|entity.id==id){ui.label(format!("Connected to {} · {}",entity.name,entity.kind));}}ui.label("Select a row to inspect; use View on chart to locate its endpoints.");});
                            ui.horizontal(|ui|{ui.selectable_value(&mut self.link_table_view,LinkTableView::Connections,"Connections");ui.selectable_value(&mut self.link_table_view,LinkTableView::LinkRecords,"Link records");});
                            if self.link_table_view==LinkTableView::Connections{
                                let mut groups=grouped_connections(&self.case.entities,&self.case.links,&indices);
                                ui.label(format!("{} entity-pair connections",groups.len()));
                                groups.sort_by(|a,b|{
                                    let entity_name=|id:Uuid|entity_names.get(&id).map(|(name,_)|name.to_lowercase()).unwrap_or_default();
                                    let link_text=|group:&ConnectionGroup,field:fn(&Link)->&str|group.link_indices.iter().filter_map(|index|self.case.links.get(*index).map(field)).map(str::to_lowercase).min().unwrap_or_default();
                                    let comparison=match self.link_sort{
                                        LinkSort::RecordId=>link_text(a,|link|&link.connection_id).cmp(&link_text(b,|link|&link.connection_id)),
                                        LinkSort::Source=>entity_name(a.first).cmp(&entity_name(b.first)),
                                        LinkSort::Date=>link_text(a,|link|&link.date).cmp(&link_text(b,|link|&link.date)),
                                        LinkSort::Relationship=>link_text(a,|link|&link.label).cmp(&link_text(b,|link|&link.label)),
                                        LinkSort::Target=>entity_name(a.second).cmp(&entity_name(b.second)),
                                        LinkSort::Confidence=>a.link_indices.iter().filter_map(|index|self.case.links.get(*index).map(|link|link.confidence)).max().cmp(&b.link_indices.iter().filter_map(|index|self.case.links.get(*index).map(|link|link.confidence)).max()).reverse(),
                                        LinkSort::SourceReference=>link_text(a,|link|&link.source_reference).cmp(&link_text(b,|link|&link.source_reference)),
                                    };
                                    comparison.then_with(||a.first.cmp(&b.first)).then_with(||a.second.cmp(&b.second))
                                });
                                egui::ScrollArea::both().auto_shrink([false,false]).show(ui,|ui|{
                                    egui::Grid::new("connections_table").striped(true).min_col_width(96.0).show(ui,|ui|{
                                        for heading in ["CONNECTION · ENTITY PAIR","LINK RECORDS","RELATIONSHIP TYPES"]{ui.label(egui::RichText::new(heading).small().strong().color(cyan));}ui.end_row();
                                        for group in &groups{
                                            let first=entity_names.get(&group.first).map(|(name,kind)|format!("{name} · {kind}")).unwrap_or_else(||"(missing entity)".into());
                                            let second=entity_names.get(&group.second).map(|(name,kind)|format!("{name} · {kind}")).unwrap_or_else(||"(missing entity)".into());
                                            let mut relationships:Vec<String>=group.link_indices.iter().filter_map(|index|self.case.links.get(*index).map(|link|link.label.clone())).collect();relationships.sort_by_key(|label|label.to_lowercase());relationships.dedup_by(|a,b|a.eq_ignore_ascii_case(b));
                                            egui::CollapsingHeader::new(format!("{first}  ↔  {second}")).id_salt((group.first,group.second)).show(ui,|ui|{
                                                egui::Grid::new(("connection_records",group.first,group.second)).striped(true).show(ui,|ui|{
                                                    for heading in ["RECORD ID","RELATIONSHIP","DATE","CONFIDENCE","DIRECTION","SOURCE REFERENCE","SOURCE NOTES","ANALYST NOTES","INGEST SOURCE"]{ui.label(egui::RichText::new(heading).small().strong().color(cyan));}ui.end_row();
                                                    for index in &group.link_indices{if let Some(link)=self.case.links.get(*index){let selected=self.selected_link==Some(*index);let confidence=link.confidence.to_string();let ingest_source=link.ingest_source.as_deref().unwrap_or("Manual");let values=[if link.connection_id.is_empty(){"(no ID)"}else{&link.connection_id},&link.label,&link.date,&confidence,if link.directed{"Directed →"}else{"Undirected ↔"},&link.source_reference,&link.source_notes,&link.notes,ingest_source];let mut clicked=false;for value in values{if ui.selectable_label(selected,value).clicked(){clicked=true;}}if clicked{self.selected_link=Some(*index);self.selected=None;self.chart_focus=None;self.chart_focus_link=None;}ui.end_row();}}
                                                });
                                            });
                                            ui.label(group.link_indices.len().to_string());ui.label(relationships.join(" · "));ui.end_row();
                                        }
                                    });
                                });
                                return;
                            }
                            egui::ScrollArea::both().auto_shrink([false,false]).show(ui,|ui|{
                                egui::Grid::new("links_table").striped(true).min_col_width(72.0).show(ui,|ui|{
                                    for h in ["RECORD ID","SOURCE ENTITY","RELATIONSHIP","TARGET ENTITY","DATE","CONFIDENCE","DIRECTION","SOURCE REFERENCE","SOURCE NOTES","ANALYST NOTES","INGEST SOURCE"] {ui.label(egui::RichText::new(h).small().strong().color(cyan));}ui.end_row();
                                    for i in indices {let link=&self.case.links[i];let from_label=entity_names.get(&link.from).map(|(name,kind)|format!("{name} · {kind}")).unwrap_or_else(||"(missing)".into());let to_label=entity_names.get(&link.to).map(|(name,kind)|format!("{name} · {kind}")).unwrap_or_else(||"(missing)".into());let selected=self.selected_link==Some(i);let id_label=if link.connection_id.is_empty(){"(no ID)"}else{&link.connection_id};let cells=[id_label.to_owned(),from_label,link.label.clone(),to_label,link.date.clone(),link.confidence.to_string(),if link.directed{"Directed →".into()}else{"Undirected ↔".into()},link.source_reference.clone(),link.source_notes.clone(),link.notes.clone(),link.ingest_source.clone().unwrap_or_else(||"Manual".into())];let mut row_clicked=false;for (column,value) in cells.iter().enumerate(){let response=ui.selectable_label(selected,value);if response.clicked(){row_clicked=true;}if column==8&&!link.source_notes.is_empty(){response.on_hover_text(&link.source_notes);}else if column==9&&!link.notes.is_empty(){response.on_hover_text(&link.notes);}else if column==10&&link.ingest_source.is_some(){response.on_hover_text(link.ingest_source.as_deref().unwrap_or_default());}}if row_clicked{self.selected_link=Some(i);self.selected=None;self.chart_focus=None;self.chart_focus_link=None;}ui.end_row();}
                                });
                            });
                            return;
                        }
                        let (rect,response)=ui.allocate_exact_size(egui::vec2(canvas_w,avail.y),egui::Sense::click_and_drag());
                        let p=ui.painter_at(rect);
                        if response.hovered() {
                            let scroll=ctx.input(|input|input.smooth_scroll_delta.y);
                            if scroll!=0.0 {
                                let pointer=response.hover_pos().unwrap_or_else(||rect.center());
                                let from_center=pointer-rect.center();
                                let old_zoom=self.chart_zoom;
                                let new_zoom=(old_zoom*(1.0+scroll*0.001)).clamp(0.25,3.0);
                                self.chart_pan=from_center-(from_center-self.chart_pan)*(new_zoom/old_zoom);
                                self.chart_zoom=new_zoom;
                            }
                        }
                        if let Some(entity_ids) = self
                            .chart_fit_entities
                            .take()
                            .or_else(|| self.chart_fit_path.take())
                        {
                            if let Some((zoom, pan)) = chart_fit_transform(&self.case, &entity_ids, rect)
                            {
                                self.chart_zoom = zoom;
                                self.chart_pan = pan;
                            }
                            self.chart_focus=None;
                            self.chart_focus_link=None;
                        } else if let Some(id)=self.chart_focus.take(){
                            if let Some(entity)=self.case.entities.iter().find(|e|e.id==id){
                                let base=egui::pos2(rect.left()+entity.x*rect.width(),rect.top()+entity.y*rect.height());
                                self.chart_pan=-(base-rect.center())*self.chart_zoom;
                            }
                            self.chart_focus_link=None;
                        } else if let Some(index)=self.chart_focus_link.take(){
                            if let Some(link)=self.case.links.get(index){
                                if let (Some(from),Some(to))=(self.case.entities.iter().find(|entity|entity.id==link.from),self.case.entities.iter().find(|entity|entity.id==link.to)){
                                    let base_from=egui::pos2(rect.left()+from.x*rect.width(),rect.top()+from.y*rect.height());
                                    let base_to=egui::pos2(rect.left()+to.x*rect.width(),rect.top()+to.y*rect.height());
                                    self.chart_pan=-(base_from.lerp(base_to,0.5)-rect.center())*self.chart_zoom;
                                }
                            }
                        }
                        p.rect_filled(rect,4.0,egui::Color32::from_rgb(14,23,32));
                        for i in 0..20 { let x=rect.left()+i as f32*48.0; p.line_segment([egui::pos2(x,rect.top()),egui::pos2(x,rect.bottom())],(1.0,egui::Color32::from_rgb(24,38,49))); }
                        for i in 0..14 { let y=rect.top()+i as f32*48.0; p.line_segment([egui::pos2(rect.left(),y),egui::pos2(rect.right(),y)],(1.0,egui::Color32::from_rgb(24,38,49))); }
                        let chart_zoom = self.chart_zoom;
                        let chart_pan = self.chart_pan;
                        let center=|e:&Entity|{let base=egui::pos2(rect.left()+e.x*rect.width(),rect.top()+e.y*rect.height());rect.center()+(base-rect.center())*chart_zoom+chart_pan};
                        let link_lanes = link_lane_offsets(&self.case.links);
                        let entity_indices: HashMap<Uuid, usize> = self
                            .case
                            .entities
                            .iter()
                            .enumerate()
                            .map(|(index, entity)| (entity.id, index))
                            .collect();
                        let path_edges: HashSet<(Uuid, Uuid)> = self
                            .path_result
                            .windows(2)
                            .map(|pair| (pair[0], pair[1]))
                            .collect();
                        let path_entities: HashSet<Uuid> = self.path_result.iter().copied().collect();
                        let focused_source_entities: HashSet<Uuid> = self
                            .case
                            .links
                            .iter()
                            .filter(|link| {
                                self.sql_source_focus.as_deref() == link.ingest_source.as_deref()
                            })
                            .flat_map(|link| [link.from, link.to])
                            .collect();
                        let has_source_focus = self.sql_source_focus.is_some();
                        let has_group_focus = self.chart_focus_group.is_some();
                        for (link_index, link) in self.case.links.iter().enumerate() {
                            if let (Some(&a_index), Some(&b_index)) =
                                (entity_indices.get(&link.from), entity_indices.get(&link.to))
                            {
                                let a = &self.case.entities[a_index];
                                let b = &self.case.entities[b_index];
                                let ca=center(a);let cb=center(b);let in_path=path_edges.contains(&(link.from,link.to))||(!link.directed&&path_edges.contains(&(link.to,link.from)));let in_source=self.sql_source_focus.as_deref()==link.ingest_source.as_deref();let in_group=self.chart_focus_group.as_ref().is_some_and(|group|group.contains(&link.from)||group.contains(&link.to));let col=if self.selected_link==Some(link_index){cyan}else if in_path{egui::Color32::from_rgb(236,174,83)}else if in_source{egui::Color32::from_rgb(91,190,178)}else if in_group{egui::Color32::from_rgb(161,132,218)}else if has_source_focus||has_group_focus{egui::Color32::from_rgb(35,52,63)}else{egui::Color32::from_rgb(72,104,120)};
                                let points = connection_curve_points(
                                    ca,
                                    cb,
                                    link_lanes[link_index] * self.chart_zoom,
                                );
                                for (segment_index, segment) in points.windows(2).enumerate() {
                                    if link.confidence != 1 || segment_index % 4 < 2 {
                                        p.line_segment(
                                            [segment[0], segment[1]],
                                            (if link.confidence == 1 { 1.0 } else { 2.0 } * self.chart_zoom, col),
                                        );
                                    }
                                }
                                if link.directed {
                                    let tangent = (points[points.len() - 1] - points[points.len() - 2]).normalized();
                                    let tip = cb - tangent * 75.0 * self.chart_zoom;
                                    let wing = egui::vec2(-tangent.y, tangent.x) * 5.0 * self.chart_zoom;
                                    p.line_segment([tip, tip - tangent * 10.0 * self.chart_zoom + wing], (1.5 * self.chart_zoom, col));
                                    p.line_segment([tip, tip - tangent * 10.0 * self.chart_zoom - wing], (1.5 * self.chart_zoom, col));
                                }
                                let mid=points[points.len()/2]; let label_rect=egui::Rect::from_center_size(mid,egui::vec2(110.0,18.0)*self.chart_zoom);
                                p.rect_filled(label_rect,3.0,if self.selected_link==Some(link_index){egui::Color32::from_rgb(34,61,70)}else{egui::Color32::from_rgb(19,31,41)});
                                p.text(mid,egui::Align2::CENTER_CENTER,&link.label,egui::FontId::proportional(9.0*self.chart_zoom),if in_source{cyan}else if in_group{egui::Color32::from_rgb(190,169,227)}else if has_source_focus||has_group_focus{egui::Color32::from_rgb(65,78,87)}else{muted});
                            }
                        }
                        for e in &self.case.entities {
                            let c=center(e); let fill=egui::Color32::from_rgb(e.color[0],e.color[1],e.color[2]);
                            let r=egui::Rect::from_center_size(c,egui::vec2(142.0,56.0)*self.chart_zoom);
                            let picked=self.selected==Some(e.id);
                            let on_path=path_entities.contains(&e.id);let in_source=focused_source_entities.contains(&e.id);let in_group=self.chart_focus_group.as_ref().is_some_and(|group|group.contains(&e.id));let dimmed=(has_source_focus&&!in_source)||(has_group_focus&&!in_group);p.rect_filled(r,7.0,if on_path{egui::Color32::from_rgb(55,48,38)}else if in_group{egui::Color32::from_rgb(48,38,61)}else if in_source{egui::Color32::from_rgb(34,53,60)}else if dimmed{egui::Color32::from_rgb(17,27,35)}else{egui::Color32::from_rgb(25,39,51)});p.rect_stroke(r,7.0,egui::Stroke::new(if picked{2.0_f32}else{1.0_f32},if picked{cyan}else if on_path{egui::Color32::from_rgb(236,174,83)}else if in_group{egui::Color32::from_rgb(161,132,218)}else if in_source{egui::Color32::from_rgb(91,190,178)}else if dimmed{egui::Color32::from_rgb(35,48,56)}else{fill}),egui::StrokeKind::Inside);
                            p.circle_filled(c+egui::vec2(-58.0,0.0)*self.chart_zoom,6.0*self.chart_zoom,if dimmed{egui::Color32::from_rgb(53,66,73)}else{fill});
                            p.text(c+egui::vec2(-44.0,-8.0)*self.chart_zoom,egui::Align2::LEFT_CENTER,&e.name,egui::FontId::proportional(12.0*self.chart_zoom),if dimmed{egui::Color32::from_rgb(81,95,103)}else{egui::Color32::WHITE});
                            p.text(c+egui::vec2(-44.0,11.0)*self.chart_zoom,egui::Align2::LEFT_CENTER,&e.kind,egui::FontId::proportional(8.0*self.chart_zoom),if dimmed{egui::Color32::from_rgb(68,82,90)}else{muted});
                        }
                        let navigator_rect = egui::Rect::from_min_size(
                            rect.right_top() + egui::vec2(-166.0, 12.0),
                            egui::vec2(154.0, 116.0),
                        );
                        let navigator_map = egui::Rect::from_min_max(
                            navigator_rect.min + egui::vec2(8.0, 26.0),
                            navigator_rect.max - egui::vec2(8.0, 8.0),
                        );
                        p.rect_filled(
                            navigator_rect,
                            6.0,
                            egui::Color32::from_rgba_unmultiplied(15, 25, 34, 238),
                        );
                        p.rect_stroke(
                            navigator_rect,
                            6.0,
                            egui::Stroke::new(1.0_f32, egui::Color32::from_rgb(57, 83, 98)),
                            egui::StrokeKind::Inside,
                        );
                        p.text(
                            navigator_rect.left_top() + egui::vec2(9.0, 8.0),
                            egui::Align2::LEFT_TOP,
                            "NAVIGATOR · CLICK TO JUMP",
                            egui::FontId::proportional(8.0),
                            muted,
                        );
                        let map_point = |entity: &Entity| {
                            egui::pos2(
                                navigator_map.left() + entity.x * navigator_map.width(),
                                navigator_map.top() + entity.y * navigator_map.height(),
                            )
                        };
                        for link in &self.case.links {
                            if let (Some(from), Some(to)) = (
                                entity_indices
                                    .get(&link.from)
                                    .and_then(|index| self.case.entities.get(*index)),
                                entity_indices
                                    .get(&link.to)
                                    .and_then(|index| self.case.entities.get(*index)),
                            ) {
                                p.line_segment(
                                    [map_point(from), map_point(to)],
                                    (0.7, egui::Color32::from_rgba_unmultiplied(87, 112, 129, 120)),
                                );
                            }
                        }
                        for entity in &self.case.entities {
                            let in_group = self
                                .chart_focus_group
                                .as_ref()
                                .is_some_and(|group| group.contains(&entity.id));
                            p.circle_filled(
                                map_point(entity),
                                if self.selected == Some(entity.id) { 3.2 } else { 2.2 },
                                if in_group {
                                    egui::Color32::from_rgb(184, 151, 232)
                                } else if self.selected == Some(entity.id) {
                                    cyan
                                } else {
                                    egui::Color32::from_rgb(
                                        entity.color[0],
                                        entity.color[1],
                                        entity.color[2],
                                    )
                                },
                            );
                        }
                        let visible_min = rect.center()
                            + (rect.min - rect.center() - self.chart_pan) / self.chart_zoom;
                        let visible_max = rect.center()
                            + (rect.max - rect.center() - self.chart_pan) / self.chart_zoom;
                        let viewport_point = |point: egui::Pos2| {
                            egui::pos2(
                                navigator_map.left()
                                    + ((point.x - rect.left()) / rect.width()) * navigator_map.width(),
                                navigator_map.top()
                                    + ((point.y - rect.top()) / rect.height()) * navigator_map.height(),
                            )
                        };
                        let viewport = egui::Rect::from_two_pos(
                            viewport_point(visible_min),
                            viewport_point(visible_max),
                        )
                        .intersect(navigator_map);
                        p.rect_stroke(
                            viewport,
                            1.0,
                            egui::Stroke::new(1.2_f32, cyan),
                            egui::StrokeKind::Inside,
                        );
                        if response.clicked()
                            && let Some(pos) = response.interact_pointer_pos()
                        {
                            if navigator_rect.contains(pos) {
                                if navigator_map.contains(pos) {
                                    let x = ((pos.x - navigator_map.left())
                                        / navigator_map.width())
                                    .clamp(0.0, 1.0);
                                    let y = ((pos.y - navigator_map.top())
                                        / navigator_map.height())
                                    .clamp(0.0, 1.0);
                                    let target = egui::pos2(
                                        rect.left() + x * rect.width(),
                                        rect.top() + y * rect.height(),
                                    );
                                    self.chart_pan = -(target - rect.center()) * self.chart_zoom;
                                    self.chart_focus = None;
                                    self.chart_focus_link = None;
                                }
                            } else if self.show_timeline && pos.y >= rect.bottom() - 105.0 {
                                let timeline_rect = egui::Rect::from_min_size(
                                    rect.left_bottom() - egui::vec2(0.0, 105.0),
                                    egui::vec2(rect.width(), 105.0),
                                );
                                self.selected_link = timeline_event_positions(
                                    &self.case.links,
                                    timeline_rect,
                                )
                                .into_iter()
                                .map(|(index, point)| (index, point.distance(pos)))
                                .min_by(|a, b| a.1.total_cmp(&b.1))
                                .filter(|(_, distance)| *distance < 20.0)
                                .map(|(index, _)| index);
                                self.selected = None;
                            } else {
                                let lanes = link_lane_offsets(&self.case.links);
                                let nearest_link = self
                                    .case
                                    .links
                                    .iter()
                                    .enumerate()
                                    .filter_map(|(index, link)| {
                                        let from = self
                                            .case
                                            .entities
                                            .get(*entity_indices.get(&link.from)?)?;
                                        let to = self
                                            .case
                                            .entities
                                            .get(*entity_indices.get(&link.to)?)?;
                                        let points = connection_curve_points(
                                            center(from),
                                            center(to),
                                            lanes[index] * self.chart_zoom,
                                        );
                                        Some((index, polyline_distance(pos, &points)))
                                    })
                                    .filter(|(_, distance)| *distance < 10.0 * self.chart_zoom)
                                    .min_by(|a, b| a.1.total_cmp(&b.1))
                                    .filter(|(_, distance)| *distance < 22.0 * self.chart_zoom)
                                    .map(|(index, _)| index);
                                let entity = self
                                    .case
                                    .entities
                                    .iter()
                                    .find(|entity| center(entity).distance(pos) < 75.0 * self.chart_zoom)
                                    .map(|entity| entity.id);
                                if let Some(index) = nearest_link {
                                    self.selected = None;
                                    self.selected_link = Some(index);
                                } else if let Some(entity) = entity {
                                    self.selected = Some(entity);
                                    self.selected_link = None;
                                } else {
                                    self.selected = None;
                                    self.selected_link = None;
                                }
                            }
                        }
                        if response.drag_started() { if let Some(pos)=response.interact_pointer_pos(){self.chart_navigator_dragging=navigator_rect.contains(pos);if !self.chart_navigator_dragging{if let Some(entity)=self.case.entities.iter().find(|e|center(e).distance(pos)<75.0*self.chart_zoom){self.selected=Some(entity.id);self.selected_link=None;}else{self.selected=None;self.chart_pan_origin=self.chart_pan;}}} }
                        if response.dragged() && !self.chart_navigator_dragging { if let Some(pos)=response.interact_pointer_pos(){if let Some(id)=self.selected{if let Some(e)=self.case.entities.iter_mut().find(|e|e.id==id){let local=(pos-rect.center()-self.chart_pan)/self.chart_zoom+rect.size()*0.5;e.x=(local.x/rect.width()).clamp(0.05,0.95);e.y=(local.y/rect.height()).clamp(0.08,0.92);}}else{self.chart_pan=self.chart_pan_origin+response.drag_delta();}} }
                        if self.chart_navigator_dragging && !ctx.input(|input| input.pointer.any_down()) {
                            self.chart_navigator_dragging = false;
                        }
                        if self.show_timeline {
                            let tr=egui::Rect::from_min_size(rect.left_bottom()-egui::vec2(0.0,105.0),egui::vec2(rect.width(),105.0));
                            p.rect_filled(tr,4.0,egui::Color32::from_rgb(20,32,43));p.text(tr.left_top()+egui::vec2(14.0,14.0),egui::Align2::LEFT_TOP,"EVENT TIMELINE",egui::FontId::proportional(10.0),cyan);
                            let y=tr.center().y+12.0;p.line_segment([egui::pos2(tr.left()+25.0,y),egui::pos2(tr.right()-25.0,y)],(1.0,muted));
                            for (index,point) in timeline_event_positions(&self.case.links,tr){let l=&self.case.links[index];let event_color=if self.selected_link==Some(index){cyan}else if l.confidence==1{egui::Color32::from_rgb(223,137,105)}else{egui::Color32::from_rgb(236,174,83)};p.circle_filled(point,5.0,event_color);p.text(point-egui::vec2(0.0,8.0),egui::Align2::CENTER_BOTTOM,&l.label,egui::FontId::proportional(7.0),muted);p.text(point+egui::vec2(0.0,8.0),egui::Align2::CENTER_TOP,&l.date,egui::FontId::proportional(8.0),muted);}
                        }
                    });
                    ui.allocate_ui_with_layout(egui::vec2(right,avail.y),egui::Layout::top_down(egui::Align::Min),|ui|{
                    egui::Frame::new().fill(panel).inner_margin(12).show(ui,|ui|{
                        ui.set_min_width(right);ui.set_max_width(right);
                        ui.label(egui::RichText::new("ANALYSIS").size(10.0).strong().color(muted));ui.add_space(10.0);
                        let mut relationship_types: Vec<String> = self
                            .case
                            .links
                            .iter()
                            .map(|link| link.label.clone())
                            .collect();
                        relationship_types.sort_by_key(|label| label.to_lowercase());
                        relationship_types
                            .dedup_by(|first, second| first.eq_ignore_ascii_case(second));
                        if self.most_connected_relationship.as_ref().is_some_and(|selected| {
                            relationship_types
                                .iter()
                                .all(|label| !label.eq_ignore_ascii_case(selected))
                        }) {
                            self.most_connected_relationship = None;
                        }
                        ui.label(egui::RichText::new("MOST CONNECTED").size(9.0).strong().color(cyan));
                        ui.horizontal_wrapped(|ui| {
                            ui.label("Rank by");
                            egui::ComboBox::from_id_salt("most_connected_measure")
                                .selected_text(match self.most_connected_measure {
                                    MostConnectedMeasure::ConnectedEntities => "Distinct entities",
                                    MostConnectedMeasure::LinkRecords => "Link records",
                                    MostConnectedMeasure::IncomingLinks => "Incoming links",
                                    MostConnectedMeasure::OutgoingLinks => "Outgoing links",
                                })
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.most_connected_measure,
                                        MostConnectedMeasure::ConnectedEntities,
                                        "Distinct entities",
                                    );
                                    ui.selectable_value(
                                        &mut self.most_connected_measure,
                                        MostConnectedMeasure::LinkRecords,
                                        "Link records",
                                    );
                                    ui.selectable_value(
                                        &mut self.most_connected_measure,
                                        MostConnectedMeasure::IncomingLinks,
                                        "Incoming links",
                                    );
                                    ui.selectable_value(
                                        &mut self.most_connected_measure,
                                        MostConnectedMeasure::OutgoingLinks,
                                        "Outgoing links",
                                    );
                                });
                            ui.label("Connected by");
                            egui::ComboBox::from_id_salt("most_connected_relationship")
                                .selected_text(
                                    self.most_connected_relationship
                                        .as_deref()
                                        .map(|label| {
                                            if label.is_empty() {
                                                "(Unlabeled)"
                                            } else {
                                                label
                                            }
                                        })
                                        .unwrap_or("Any relationship"),
                                )
                                .show_ui(ui, |ui| {
                                    ui.selectable_value(
                                        &mut self.most_connected_relationship,
                                        None,
                                        "Any relationship",
                                    );
                                    for relationship_type in &relationship_types {
                                        ui.selectable_value(
                                            &mut self.most_connected_relationship,
                                            Some(relationship_type.clone()),
                                            if relationship_type.is_empty() {
                                                "(Unlabeled)"
                                            } else {
                                                relationship_type
                                            },
                                        );
                                    }
                                });
                        });
                        let measure_note = match self.most_connected_measure {
                            MostConnectedMeasure::ConnectedEntities => {
                                "Distinct counterpart entities; parallel links count once."
                            }
                            MostConnectedMeasure::LinkRecords => {
                                "Every link record counts; self-links count once."
                            }
                            MostConnectedMeasure::IncomingLinks => {
                                "Directed links ending at each entity; undirected links are excluded."
                            }
                            MostConnectedMeasure::OutgoingLinks => {
                                "Directed links starting at each entity; undirected links are excluded."
                            }
                        };
                        ui.label(egui::RichText::new(measure_note).small().color(muted));
                        if let Some(relationship_type) = self.most_connected_relationship.as_deref()
                        {
                            ui.label(
                                egui::RichText::new(if relationship_type.is_empty() {
                                    "Connected by: unlabeled relationships."
                                } else {
                                    relationship_type
                                })
                                .small()
                                .color(muted),
                            );
                        }
                        let ranking_counts = most_connected_counts(
                            &self.case,
                            self.most_connected_measure,
                            self.most_connected_relationship.as_deref(),
                        );
                        let mut degrees: Vec<_> = self
                            .case
                            .entities
                            .iter()
                            .map(|entity| {
                                (
                                    entity,
                                    ranking_counts.get(&entity.id).copied().unwrap_or_default(),
                                )
                            })
                            .collect();
                        degrees.sort_by(|(first, first_count), (second, second_count)| {
                            second_count.cmp(first_count).then_with(|| {
                                first
                                    .name
                                    .to_lowercase()
                                    .cmp(&second.name.to_lowercase())
                            })
                        });
                        degrees.retain(|(_, count)| *count > 0);
                        if degrees.is_empty() {
                            ui.label(
                                egui::RichText::new(
                                    "No connected entities match this measure and relationship type.",
                                )
                                .small()
                                .color(muted),
                            );
                        }
                        for (rank, (entity, count)) in degrees.iter().take(5).enumerate() {
                            ui.horizontal(|ui| {
                                let label = format!(
                                    "{:02}  {} · {}",
                                    rank + 1,
                                    entity.name,
                                    entity.kind
                                );
                                if ui
                                    .selectable_label(self.selected == Some(entity.id), label)
                                    .clicked()
                                {
                                    self.selected = Some(entity.id);
                                    self.selected_link = None;
                                    self.chart_focus = Some(entity.id);
                                    self.chart_focus_group = None;
                                    self.page = WorkspacePage::Chart;
                                    self.show_timeline = false;
                                }
                                ui.with_layout(
                                    egui::Layout::right_to_left(egui::Align::Center),
                                    |ui| ui.label(count.to_string()),
                                );
                            });
                        }
                        if ui
                            .add_enabled(
                                !degrees.is_empty(),
                                egui::Button::new("Highlight top 10 on chart"),
                            )
                            .clicked()
                        {
                            let top_entities: Vec<Uuid> = degrees
                                .iter()
                                .take(10)
                                .map(|(entity, _)| entity.id)
                                .collect();
                            self.chart_fit_entities = Some(top_entities.clone());
                            self.chart_fit_path = None;
                            self.chart_focus_group = Some(top_entities.iter().copied().collect());
                            self.chart_focus = None;
                            self.chart_focus_link = None;
                            self.path_result.clear();
                            self.sql_source_focus = None;
                            self.selected = top_entities.first().copied();
                            self.selected_link = None;
                            self.page = WorkspacePage::Chart;
                            self.show_timeline = false;
                        }
                        ui.add_space(16.0);
                        ui.label(egui::RichText::new("NETWORK GROUPS").size(9.0).strong().color(cyan));
                        if network_groups.is_none() {
                            ui.label(egui::RichText::new("Find weakly connected groups across all links.").small().color(muted));
                            if ui.button("Find groups").clicked() {
                                self.network_groups = Some(std::sync::Arc::new(connected_components(&self.case)));
                            }
                        }
                        let linked_groups: Vec<&Vec<Uuid>> = network_groups
                            .as_deref()
                            .map(|groups| {
                                groups
                                    .iter()
                                    .filter(|group| group.len() > 1)
                                    .collect()
                            })
                            .unwrap_or_default();
                        if network_groups.is_some() {
                            ui.horizontal(|ui| {
                                ui.label(egui::RichText::new(format!("{} connected groups · link direction ignored", linked_groups.len())).small().color(muted));
                                if ui.small_button("Refresh").clicked() {
                                    self.network_groups = Some(std::sync::Arc::new(connected_components(&self.case)));
                                }
                            });
                        }
                        for (index, group) in linked_groups.iter().take(4).enumerate() {
                            let group_ids = (**group).clone();
                            let names = group
                                .iter()
                                .take(3)
                                .filter_map(|id| self.case.entities.iter().find(|entity| entity.id == *id).map(|entity| entity.name.as_str()))
                                .collect::<Vec<_>>()
                                .join(" · ");
                            ui.horizontal(|ui| {
                                ui.label(format!("{:02} · {} entities", index + 1, group.len()));
                                if ui.small_button("Show").clicked() {
                                    self.chart_fit_entities = Some(group_ids.clone());
                                    self.chart_fit_path = None;
                                    self.chart_focus_group = Some(group_ids.iter().copied().collect());
                                    self.chart_focus = None;
                                    self.chart_focus_link = None;
                                    self.path_result.clear();
                                    self.selected = group_ids.first().copied();
                                    self.selected_link = None;
                                    self.sql_source_focus = None;
                                    self.page = WorkspacePage::Chart;
                                    self.show_timeline = false;
                                }
                            });
                            ui.label(egui::RichText::new(if group.len() > 3 { format!("{names} · +{}", group.len() - 3) } else { names }).small().color(muted));
                        }
                        ui.add_space(16.0);ui.label(egui::RichText::new("SHORTEST PATH").size(9.0).strong().color(cyan));
                        egui::ComboBox::from_id_salt("path_start").selected_text(self.path_start.and_then(|id|self.case.entities.iter().find(|e|e.id==id).map(|e|format!("{} · {}",e.name,e.kind))).unwrap_or_else(||"Start entity".into())).show_ui(ui,|ui|{for e in &self.case.entities{ui.selectable_value(&mut self.path_start,Some(e.id),format!("{} · {}",e.name,e.kind));}});
                        egui::ComboBox::from_id_salt("path_end").selected_text(self.path_end.and_then(|id|self.case.entities.iter().find(|e|e.id==id).map(|e|format!("{} · {}",e.name,e.kind))).unwrap_or_else(||"End entity".into())).show_ui(ui,|ui|{for e in &self.case.entities{ui.selectable_value(&mut self.path_end,Some(e.id),format!("{} · {}",e.name,e.kind));}});
                        if ui.button("Analyze path").clicked(){self.analyze_path();}
                        if !self.path_result.is_empty(){let labels:Vec<_>=self.path_result.iter().filter_map(|id|self.case.entities.iter().find(|e|e.id==*id).map(|e|format!("{} · {}",e.name,e.kind))).collect();ui.label(egui::RichText::new(labels.join(" → ")).small().color(egui::Color32::from_rgb(236,174,83)));ui.horizontal(|ui|{if ui.small_button("Show path on chart").clicked(){self.chart_fit_path=Some(self.path_result.clone());self.chart_fit_entities=None;self.chart_focus=None;self.chart_focus_link=None;self.chart_focus_group=None;self.page=WorkspacePage::Chart;self.show_timeline=false;self.selected=self.path_result.first().copied();self.selected_link=None;}if ui.small_button("Clear path").clicked(){self.path_result.clear();self.chart_fit_path=None;self.chart_fit_entities=None;}});}
                        ui.add_space(16.0);ui.label(egui::RichText::new("POSSIBLE DUPLICATES").size(9.0).strong().color(cyan));
                        if ui.button("Find likely matches").clicked(){self.find_duplicates();}
                        for (a,b) in self.duplicate_candidates.clone().iter().take(5){let an=self.case.entities.iter().find(|e|e.id==*a).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());let bn=self.case.entities.iter().find(|e|e.id==*b).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());if ui.selectable_label(self.merge_candidate==Some((*a,*b)),format!("{an}  ↔  {bn}")).clicked(){self.merge_candidate=Some((*a,*b));}}
                        if let Some((keep,remove))=self.merge_candidate{let keep_name=self.case.entities.iter().find(|e|e.id==keep).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());let remove_name=self.case.entities.iter().find(|e|e.id==remove).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());ui.label(egui::RichText::new(format!("Keep {keep_name}; merge {remove_name} into it. Aliases, notes, attributes, and links will be retained.")).small().color(muted));if ui.button(format!("Merge {remove_name} into {keep_name}")).clicked(){self.merge_candidate_into_first();}}
                        ui.add_space(20.0);ui.label(egui::RichText::new("SELECTED ITEM").size(9.0).strong().color(cyan));ui.add_space(5.0);
                        let mut focus_linked_entity = None;
                        let mut focus_linked_record = None;
                        if let Some(id) = self.selected {
                            let linked_entities = linked_entities_for(&self.case, id);
                            if let Some(entity) = self.case.entities.iter().find(|entity| entity.id == id) {
                                if !entity.aliases.is_empty() {
                                    ui.label(
                                        egui::RichText::new("KNOWN ALIASES")
                                            .size(9.0)
                                            .strong()
                                            .color(cyan),
                                    );
                                    for alias in &entity.aliases {
                                        ui.label(alias);
                                    }
                                }
                                if !entity.merged_records.is_empty() {
                                    egui::CollapsingHeader::new(format!(
                                        "Merged record history ({})",
                                        entity.merged_records.len()
                                    ))
                                    .show(ui, |ui| {
                                        for record in &entity.merged_records {
                                            ui.label(format!(
                                                "{} · {} · {}",
                                                record.name, record.kind, record.id
                                            ));
                                            if !record.detail.is_empty() {
                                                ui.label(&record.detail);
                                            }
                                            for (key, value) in &record.attributes {
                                                ui.label(format!("{key}: {value}"));
                                            }
                                        }
                                    });
                                }
                            }
                            if let Some(entity) =
                                self.case.entities.iter_mut().find(|entity| entity.id == id)
                            {
                                ui.label(egui::RichText::new("TYPE").size(9.0).color(muted));
                                ui.add(
                                    egui::TextEdit::singleline(&mut entity.kind)
                                        .desired_width(230.0),
                                );
                                ui.label(egui::RichText::new("LABEL").size(9.0).color(muted));
                                ui.add(
                                    egui::TextEdit::singleline(&mut entity.name)
                                        .desired_width(230.0),
                                );
                                ui.horizontal(|ui| {
                                    ui.label(egui::RichText::new("COLOR").size(9.0).color(muted));
                                    ui.color_edit_button_srgb(&mut entity.color);
                                    ui.label(format!(
                                        "#{:02X}{:02X}{:02X}",
                                        entity.color[0], entity.color[1], entity.color[2]
                                    ));
                                });
                                ui.checkbox(
                                    &mut entity.position_locked,
                                    "Pin position during automatic layouts",
                                );
                                ui.label(egui::RichText::new("NOTES").size(9.0).color(muted));
                                ui.add(
                                    egui::TextEdit::multiline(&mut entity.detail)
                                        .desired_width(230.0)
                                        .desired_rows(3),
                                );
                                ui.label(
                                    egui::RichText::new("ATTRIBUTES")
                                        .size(9.0)
                                        .strong()
                                        .color(cyan),
                                );
                                let mut remove_attribute = None;
                                for (index, (key, value)) in
                                    entity.attributes.iter_mut().enumerate()
                                {
                                    ui.horizontal(|ui| {
                                        ui.add(
                                            egui::TextEdit::singleline(key)
                                                .hint_text("Attribute")
                                                .desired_width(90.0),
                                        );
                                        ui.add(
                                            egui::TextEdit::singleline(value)
                                                .hint_text("Value")
                                                .desired_width(100.0),
                                        );
                                        if ui.small_button("×").clicked() {
                                            remove_attribute = Some(index);
                                        }
                                    });
                                }
                                if let Some(index) = remove_attribute {
                                    entity.attributes.remove(index);
                                }
                                if ui.small_button("＋ Attribute").clicked() {
                                    entity.attributes.push((String::new(), String::new()));
                                }
                            }
                            egui::CollapsingHeader::new(format!(
                                "LINKED ENTITIES · {}",
                                linked_entities.len()
                            ))
                            .default_open(false)
                            .id_salt(("linked_entities", id))
                            .show(ui, |ui| {
                                if linked_entities.is_empty() {
                                    ui.label("This entity has no linked entities.");
                                }
                                egui::ScrollArea::vertical()
                                    .max_height(190.0)
                                    .show(ui, |ui| {
                                        for (counterpart_id, name, kind, link_indices) in
                                            &linked_entities
                                        {
                                            let label = format!(
                                                "{name} · {kind} · {} link records",
                                                link_indices.len()
                                            );
                                            egui::CollapsingHeader::new(label)
                                                .id_salt(("linked_counterpart", id, counterpart_id))
                                                .show(ui, |ui| {
                                                    if ui.small_button("Focus entity on chart").clicked() {
                                                        focus_linked_entity = Some(*counterpart_id);
                                                    }
                                                    for index in link_indices {
                                                        let Some(link) =
                                                            self.case.links.get(*index)
                                                        else {
                                                            continue;
                                                        };
                                                        let direction = if !link.directed {
                                                            "UNDIRECTED ↔"
                                                        } else if link.from == id {
                                                            "OUTGOING →"
                                                        } else {
                                                            "INCOMING ←"
                                                        };
                                                        let connection_id = if link.connection_id.is_empty() {
                                                            "no ID"
                                                        } else {
                                                            link.connection_id.as_str()
                                                        };
                                                        let date = if link.date.is_empty() {
                                                            "undated"
                                                        } else {
                                                            link.date.as_str()
                                                        };
                                                        let response = ui.selectable_label(
                                                            self.selected_link == Some(*index),
                                                            format!(
                                                                "{direction} · {} · {date} · {connection_id}",
                                                                link.label
                                                            ),
                                                        );
                                                        if response.clicked() {
                                                            focus_linked_record = Some(*index);
                                                        }
                                                        if !link.source_reference.is_empty() {
                                                            response.on_hover_text(format!(
                                                                "Evidence: {}",
                                                                link.source_reference
                                                            ));
                                                        }
                                                    }
                                                });
                                        }
                                    });
                            });
                        } else if let Some(index) = self.selected_link {
                            if let Some(link) = self.case.links.get_mut(index) {
                                ui.label(
                                    egui::RichText::new("RELATIONSHIP")
                                        .size(9.0)
                                        .color(muted),
                                );
                                let from = link.from;
                                let to = link.to;
                                ui.horizontal(|ui| {
                                    if let Some(entity) =
                                        self.case.entities.iter().find(|entity| entity.id == from)
                                        && ui
                                            .small_button(format!("Source · {}", entity.name))
                                            .clicked()
                                    {
                                        self.selected = Some(from);
                                        self.selected_link = None;
                                        self.chart_focus = Some(from);
                                    }
                                    if let Some(entity) =
                                        self.case.entities.iter().find(|entity| entity.id == to)
                                        && ui
                                            .small_button(format!("Target · {}", entity.name))
                                            .clicked()
                                    {
                                        self.selected = Some(to);
                                        self.selected_link = None;
                                        self.chart_focus = Some(to);
                                    }
                                });
                                ui.add(egui::TextEdit::singleline(&mut link.label));
                                ui.horizontal(|ui| {
                                    ui.label("Date");
                                    ui.add(
                                        egui::TextEdit::singleline(&mut link.date)
                                            .desired_width(100.0),
                                    );
                                });
                                ui.add(
                                    egui::Slider::new(&mut link.confidence, 1..=3)
                                        .text("Confidence"),
                                );
                                ui.checkbox(&mut link.directed, "Directed relationship");
                                ui.label("Source reference");
                                ui.add(
                                    egui::TextEdit::singleline(&mut link.source_reference)
                                        .desired_width(230.0),
                                );
                                if !link.source_notes.is_empty() {
                                    ui.label(
                                        egui::RichText::new("SOURCE NOTES")
                                            .size(9.0)
                                            .color(muted),
                                    );
                                    ui.label(&link.source_notes);
                                }
                                ui.label("Analyst notes");
                                ui.add(
                                    egui::TextEdit::multiline(&mut link.notes).desired_rows(4),
                                );
                            }
                        } else {
                            ui.label(
                                egui::RichText::new(
                                    "Select an entity on the chart to inspect its attributes and linked entities. Select a relationship in Data to edit its properties.",
                                )
                                .small()
                                .color(muted),
                            );
                        }
                        if let Some(counterpart_id) = focus_linked_entity {
                            self.selected = Some(counterpart_id);
                            self.selected_link = None;
                            self.chart_focus = Some(counterpart_id);
                            self.chart_focus_link = None;
                            self.chart_focus_group = None;
                            self.chart_fit_entities = None;
                            self.chart_fit_path = None;
                            self.path_result.clear();
                            self.sql_source_focus = None;
                            self.show_timeline = false;
                        } else if let Some(index) = focus_linked_record {
                            self.selected = None;
                            self.selected_link = Some(index);
                            self.chart_focus = None;
                            self.chart_focus_link = Some(index);
                            self.chart_focus_group = None;
                            self.chart_fit_entities = None;
                            self.chart_fit_path = None;
                            self.path_result.clear();
                            self.sql_source_focus = None;
                            self.show_timeline = false;
                        }
                        ui.add_space(20.0);ui.label(egui::RichText::new("LEGEND").size(9.0).strong().color(cyan));ui.label(egui::RichText::new("Solid link  ·  corroborated").small().color(muted));ui.label(egui::RichText::new("Dashed link  ·  unverified").small().color(muted));
                    });
                    });
                });
            });
        });
        if self.show_import_wizard {
            let mut open = self.show_import_wizard;
            let mut import_now = false;
            let mut cancel_import = false;
            let mut save_import_spec = false;
            let import_specs = self.case.import_specifications.clone();
            egui::Window::new("Import design")
                .open(&mut open)
                .resizable(true)
                .default_width(560.0)
                .show(ctx, |ui| {
                    ui.label(format!("Map columns from {} to chart properties.", self.import_path));
                    ui.label(egui::RichText::new("Existing header names are detected automatically. Choose a different column when the file uses its own schema.").small());
                    ui.add_space(6.0);
                    ui.checkbox(
                        &mut self.import_column_mapping.trim_whitespace,
                        "Trim whitespace from imported values",
                    );
                    ui.horizontal(|ui| {
                        ui.label("Saved design");
                        egui::ComboBox::from_id_salt("saved_import_design")
                            .selected_text(self.selected_import_spec
                                .and_then(|id| import_specs.iter().find(|spec| spec.id == id))
                                .map(|spec| spec.name.as_str())
                                .unwrap_or("Current mapping"))
                            .show_ui(ui, |ui| {
                                for spec in &import_specs {
                                    if ui.selectable_label(self.selected_import_spec == Some(spec.id), &spec.name).clicked() {
                                        self.selected_import_spec = Some(spec.id);
                                        self.import_spec_name = spec.name.clone();
                                        self.import_column_mapping = spec.mapping.clone();
                                        remap_import_mapping_headers(
                                            &mut self.import_column_mapping,
                                            &self.import_headers,
                                        );
                                    }
                                }
                            });
                        ui.label("Design name");
                        ui.add(egui::TextEdit::singleline(&mut self.import_spec_name).desired_width(180.0));
                        if ui.button("Save design").clicked() {
                            save_import_spec = true;
                        }
                    });
                    egui::CollapsingHeader::new("Preview first rows")
                        .default_open(false)
                        .show(ui, |ui| {
                            ui.horizontal_wrapped(|ui| {
                                for header in &self.import_headers {
                                    ui.label(egui::RichText::new(header).strong());
                                    ui.separator();
                                }
                            });
                            for row in &self.import_preview {
                                ui.horizontal_wrapped(|ui| {
                                    for value in row {
                                        ui.label(value);
                                        ui.separator();
                                    }
                                });
                            }
                        });
                    egui::ScrollArea::vertical().max_height(440.0).show(ui, |ui| {
                        let fields: [(&str, &mut String); 30] = [
                            ("Source entity", &mut self.import_column_mapping.source),
                            ("Source entity ID", &mut self.import_column_mapping.source_entity_id),
                            ("Target entity", &mut self.import_column_mapping.target),
                            ("Target entity ID", &mut self.import_column_mapping.target_entity_id),
                            ("Source type", &mut self.import_column_mapping.source_type),
                            ("Target type", &mut self.import_column_mapping.target_type),
                            ("Relationship", &mut self.import_column_mapping.relationship),
                            ("Date", &mut self.import_column_mapping.date),
                            ("Confidence", &mut self.import_column_mapping.confidence),
                            ("Direction", &mut self.import_column_mapping.direction),
                            ("Connection ID", &mut self.import_column_mapping.connection_id),
                            ("Analyst notes", &mut self.import_column_mapping.notes),
                            ("Source notes", &mut self.import_column_mapping.source_notes),
                            ("Evidence reference", &mut self.import_column_mapping.source_reference),
                            ("Ingest source", &mut self.import_column_mapping.ingest_source),
                            ("Record type", &mut self.import_column_mapping.record_type),
                            ("Source detail JSON", &mut self.import_column_mapping.source_detail),
                            ("Target detail JSON", &mut self.import_column_mapping.target_detail),
                            ("Source aliases JSON", &mut self.import_column_mapping.source_aliases),
                            ("Target aliases JSON", &mut self.import_column_mapping.target_aliases),
                            ("Source external IDs JSON", &mut self.import_column_mapping.source_external_ids),
                            ("Target external IDs JSON", &mut self.import_column_mapping.target_external_ids),
                            ("Source color", &mut self.import_column_mapping.source_color),
                            ("Target color", &mut self.import_column_mapping.target_color),
                            ("Source chart X", &mut self.import_column_mapping.source_x),
                            ("Source chart Y", &mut self.import_column_mapping.source_y),
                            ("Target chart X", &mut self.import_column_mapping.target_x),
                            ("Target chart Y", &mut self.import_column_mapping.target_y),
                            ("Source position locked", &mut self.import_column_mapping.source_position_locked),
                            ("Target position locked", &mut self.import_column_mapping.target_position_locked),
                        ];
                        egui::Grid::new("import_column_mapping_grid").striped(true).show(ui, |ui| {
                            for (index, (label, selected)) in fields.into_iter().enumerate() {
                                ui.label(label);
                                egui::ComboBox::from_id_salt(("import_column", index))
                                    .selected_text(if selected.is_empty() { "Auto detect" } else { selected.as_str() })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(selected, String::new(), "Auto detect");
                                        for header in &self.import_headers {
                                            ui.selectable_value(selected, header.clone(), header);
                                        }
                                    });
                                ui.end_row();
                            }
                        });
                        ui.separator();
                        ui.label(egui::RichText::new("ENTITY ATTRIBUTE MAPPINGS").strong());
                        let mut remove_attribute = None;
                        for (index, attribute) in self
                            .import_column_mapping
                            .entity_attributes
                            .iter_mut()
                            .enumerate()
                        {
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt(("attribute_column", index))
                                    .selected_text(if attribute.column.is_empty() { "Choose column" } else { &attribute.column })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut attribute.column, String::new(), "Choose column");
                                        for header in &self.import_headers {
                                            ui.selectable_value(&mut attribute.column, header.clone(), header);
                                        }
                                    });
                                egui::ComboBox::from_id_salt(("attribute_side", index))
                                    .selected_text(if attribute.source { "Source entity" } else { "Target entity" })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut attribute.source, true, "Source entity");
                                        ui.selectable_value(&mut attribute.source, false, "Target entity");
                                    });
                                ui.add(egui::TextEdit::singleline(&mut attribute.key).hint_text("Property name").desired_width(130.0));
                                if ui.small_button("×").clicked() {
                                    remove_attribute = Some(index);
                                }
                            });
                        }
                        if let Some(index) = remove_attribute {
                            self.import_column_mapping.entity_attributes.remove(index);
                        }
                        if ui.small_button("＋ Entity attribute mapping").clicked() {
                            self.import_column_mapping.entity_attributes.push(ImportAttributeMapping {
                                column: String::new(),
                                key: String::new(),
                                source: true,
                            });
                        }
                        ui.separator();
                        ui.label(egui::RichText::new("INCLUDE ROWS WHERE ALL RULES MATCH").strong());
                        let mut remove_filter = None;
                        for (index, filter) in self
                            .import_column_mapping
                            .row_filters
                            .iter_mut()
                            .enumerate()
                        {
                            ui.horizontal(|ui| {
                                egui::ComboBox::from_id_salt(("filter_column", index))
                                    .selected_text(if filter.column.is_empty() { "Choose column" } else { &filter.column })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut filter.column, String::new(), "Choose column");
                                        for header in &self.import_headers {
                                            ui.selectable_value(&mut filter.column, header.clone(), header);
                                        }
                                    });
                                egui::ComboBox::from_id_salt(("filter_operator", index))
                                    .selected_text(match filter.operator {
                                        ImportFilterOperator::Equals => "equals",
                                        ImportFilterOperator::Contains => "contains",
                                        ImportFilterOperator::IsEmpty => "is empty",
                                        ImportFilterOperator::IsNotEmpty => "is not empty",
                                    })
                                    .show_ui(ui, |ui| {
                                        ui.selectable_value(&mut filter.operator, ImportFilterOperator::Equals, "equals");
                                        ui.selectable_value(&mut filter.operator, ImportFilterOperator::Contains, "contains");
                                        ui.selectable_value(&mut filter.operator, ImportFilterOperator::IsEmpty, "is empty");
                                        ui.selectable_value(&mut filter.operator, ImportFilterOperator::IsNotEmpty, "is not empty");
                                    });
                                if matches!(filter.operator, ImportFilterOperator::Equals | ImportFilterOperator::Contains) {
                                    ui.add(egui::TextEdit::singleline(&mut filter.value).hint_text("Value").desired_width(120.0));
                                }
                                if ui.small_button("×").clicked() {
                                    remove_filter = Some(index);
                                }
                            });
                        }
                        if let Some(index) = remove_filter {
                            self.import_column_mapping.row_filters.remove(index);
                        }
                        if ui.small_button("＋ Row filter").clicked() {
                            self.import_column_mapping.row_filters.push(ImportRowFilter::default());
                        }
                    });
                    ui.add_space(8.0);
                    ui.horizontal(|ui| {
                        if ui.button("Import into chart").clicked() {
                            import_now = true;
                        }
                        if ui.button("Cancel").clicked() {
                            cancel_import = true;
                        }
                        ui.label(format!("{} columns detected", self.import_headers.len()));
                    });
                });
            if cancel_import {
                open = false;
            }
            self.show_import_wizard = open;
            if save_import_spec {
                self.save_import_specification();
            }
            if import_now {
                self.import_delimited();
                self.show_import_wizard = false;
            }
        }
        if let Some(action) = self.pending_case_action {
            egui::Window::new("Unsaved changes")
                .collapsible(false)
                .resizable(false)
                .show(ctx, |ui| {
                    ui.label("This case has unsaved changes. Discard them and continue?");
                    ui.horizontal(|ui| {
                        let label = match action {
                            PendingCaseAction::Open => "Discard and open",
                            PendingCaseAction::New => "Discard and create new",
                        };
                        if ui.button(label).clicked() {
                            self.perform_case_action(action);
                        }
                        if ui.button("Cancel").clicked() {
                            self.pending_case_action = None;
                        }
                    });
                });
        }
    }
}

fn link_lane_offsets(links: &[Link]) -> Vec<f32> {
    let mut groups: HashMap<(Uuid, Uuid), Vec<usize>> = HashMap::new();
    for (index, link) in links.iter().enumerate() {
        let pair = if link.from <= link.to {
            (link.from, link.to)
        } else {
            (link.to, link.from)
        };
        groups.entry(pair).or_default().push(index);
    }

    let mut offsets = vec![0.0; links.len()];
    for ((first, second), indices) in groups {
        for (lane, index) in indices.iter().enumerate() {
            let lane_offset = if first == second {
                (lane as f32 + 1.0) * 18.0
            } else {
                (lane as f32 - (indices.len() - 1) as f32 / 2.0) * 18.0
            };
            offsets[*index] = if links[*index].from == first {
                lane_offset
            } else {
                -lane_offset
            };
        }
    }
    offsets
}

fn connection_curve_points(start: egui::Pos2, end: egui::Pos2, lane: f32) -> Vec<egui::Pos2> {
    const STEPS: usize = 16;
    let direction = (end - start).normalized();
    let normal = if direction.length_sq() == 0.0 {
        egui::vec2(1.0, 0.0)
    } else {
        egui::vec2(-direction.y, direction.x)
    };
    let control = start.lerp(end, 0.5) + normal * lane * 2.0;
    (0..=STEPS)
        .map(|step| {
            let t = step as f32 / STEPS as f32;
            let inverse = 1.0 - t;
            let point = start.to_vec2() * (inverse * inverse)
                + control.to_vec2() * (2.0 * inverse * t)
                + end.to_vec2() * (t * t);
            egui::pos2(point.x, point.y)
        })
        .collect()
}

fn polyline_distance(point: egui::Pos2, points: &[egui::Pos2]) -> f32 {
    points
        .windows(2)
        .map(|segment| segment_distance(point, segment[0], segment[1]))
        .fold(f32::INFINITY, f32::min)
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

fn parse_iso_date_ordinal(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() < 10 || bytes[4] != b'-' || bytes[7] != b'-' {
        return None;
    }
    let number = |slice: &[u8]| -> Option<i64> {
        if slice.iter().all(u8::is_ascii_digit) {
            std::str::from_utf8(slice).ok()?.parse().ok()
        } else {
            None
        }
    };
    let year = number(&bytes[..4])?;
    let month = number(&bytes[5..7])?;
    let day = number(&bytes[8..10])?;
    if year == 0 || !(1..=12).contains(&month) {
        return None;
    }
    let leap = year % 4 == 0 && (year % 100 != 0 || year % 400 == 0);
    let month_days = [
        31,
        if leap { 29 } else { 28 },
        31,
        30,
        31,
        30,
        31,
        31,
        30,
        31,
        30,
        31,
    ];
    if day < 1 || day > month_days[(month - 1) as usize] {
        return None;
    }
    let adjusted_year = year - i64::from(month <= 2);
    let era = adjusted_year / 400;
    let year_of_era = adjusted_year - era * 400;
    let adjusted_month = month + if month > 2 { -3 } else { 9 };
    let day_of_year = (153 * adjusted_month + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    Some(era * 146_097 + day_of_era)
}

fn timeline_event_positions(links: &[Link], rect: egui::Rect) -> Vec<(usize, egui::Pos2)> {
    let mut events: Vec<usize> = (0..links.len()).collect();
    events.sort_by(|a, b| links[*a].date.cmp(&links[*b].date));
    let ordinals: Vec<Option<i64>> = events
        .iter()
        .map(|index| parse_iso_date_ordinal(&links[*index].date))
        .collect();
    let all_dated = !events.is_empty() && ordinals.iter().all(Option::is_some);
    let earliest = ordinals.iter().flatten().copied().min();
    let latest = ordinals.iter().flatten().copied().max();
    let proportional = all_dated && earliest.zip(latest).is_some();
    let fallback_denominator = events.len().saturating_sub(1).max(1) as f32;
    let mut same_day_occurrences: HashMap<i64, usize> = HashMap::new();
    events
        .into_iter()
        .enumerate()
        .map(|(rank, index)| {
            let x = if proportional {
                let date = ordinals[rank].unwrap();
                let (first, last) = earliest.zip(latest).unwrap();
                if first == last {
                    rect.center().x
                } else {
                    rect.left()
                        + 25.0
                        + (date - first) as f32 / (last - first) as f32 * (rect.width() - 50.0)
                }
            } else {
                rect.left() + 25.0 + rank as f32 * (rect.width() - 50.0) / fallback_denominator
            };
            let lane = if proportional {
                let occurrence = same_day_occurrences
                    .entry(ordinals[rank].unwrap())
                    .or_insert(0);
                let lane = *occurrence;
                *occurrence += 1;
                lane % 3
            } else {
                0
            };
            let y = rect.center().y + 12.0 + lane as f32 * 7.0;
            (index, egui::pos2(x, y))
        })
        .collect()
}

fn save_case_atomically(path: &Path, case: &CaseFile) -> Result<(), String> {
    let serialized = serde_json::to_string_pretty(case).map_err(|error| error.to_string())?;
    let mut temporary_name = path.as_os_str().to_os_string();
    temporary_name.push(format!(".tmp-{}", std::process::id()));
    let temporary_path = PathBuf::from(temporary_name);
    fs::write(&temporary_path, serialized).map_err(|error| {
        format!(
            "cannot write temporary case file {}: {error}",
            temporary_path.display()
        )
    })?;
    if let Err(error) = fs::rename(&temporary_path, path) {
        let _ = fs::remove_file(&temporary_path);
        return Err(format!(
            "cannot replace case file {}: {error}",
            path.display()
        ));
    }
    Ok(())
}

fn save_case_atomically_with_lock(path: &Path, case: &CaseFile) -> Result<String, String> {
    let _case_lock = lock_automation_case(path)?;
    let serialized = serde_json::to_string_pretty(case).map_err(|error| error.to_string())?;
    save_case_atomically(path, case)?;
    Ok(serialized)
}

fn save_case_atomically_if_unchanged(
    path: &Path,
    case: &CaseFile,
    expected_disk_json: Option<&str>,
) -> Result<String, String> {
    let _case_lock = lock_automation_case(path)?;
    match fs::read_to_string(path) {
        Ok(current_disk_json) if Some(current_disk_json.as_str()) == expected_disk_json => {}
        Ok(_) => {
            return Err(
                "case file changed outside this app; reload it before saving to prevent overwriting newer data".into(),
            );
        }
        Err(error)
            if error.kind() == std::io::ErrorKind::NotFound && expected_disk_json.is_none() => {}
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Err("case file was removed outside this app; reload or use Save as".into());
        }
        Err(error) => return Err(format!("cannot check case file before saving: {error}")),
    }
    let serialized = serde_json::to_string_pretty(case).map_err(|error| error.to_string())?;
    save_case_atomically(path, case)?;
    Ok(serialized)
}

fn delimited_text_delimiter(path: &str) -> u8 {
    match Path::new(path)
        .extension()
        .and_then(|extension| extension.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "tsv" | "tab" => b'\t',
        _ => b',',
    }
}

fn validate_delimited_headers<I>(headers: I) -> Result<(), String>
where
    I: IntoIterator,
    I::Item: AsRef<str>,
{
    let mut seen = HashSet::new();
    for header in headers {
        let header = header.as_ref();
        let normalized = header.trim().to_ascii_lowercase();
        if !normalized.is_empty() && !seen.insert(normalized) {
            return Err(format!(
                "delimited file has an ambiguous duplicate column header: {}",
                header.trim()
            ));
        }
    }
    Ok(())
}

fn delimited_header_signature(headers: &[String]) -> Vec<String> {
    let mut signature: Vec<String> = headers
        .iter()
        .map(|header| header.trim().to_ascii_lowercase())
        .collect();
    signature.sort();
    signature
}

fn remap_import_mapping_headers(mapping: &mut ImportColumnMapping, headers: &[String]) {
    let columns = [
        &mut mapping.source,
        &mut mapping.target,
        &mut mapping.source_type,
        &mut mapping.target_type,
        &mut mapping.relationship,
        &mut mapping.date,
        &mut mapping.confidence,
        &mut mapping.direction,
        &mut mapping.connection_id,
        &mut mapping.notes,
        &mut mapping.source_notes,
        &mut mapping.source_reference,
        &mut mapping.ingest_source,
        &mut mapping.record_type,
        &mut mapping.source_entity_id,
        &mut mapping.target_entity_id,
        &mut mapping.source_external_ids,
        &mut mapping.target_external_ids,
        &mut mapping.source_detail,
        &mut mapping.target_detail,
        &mut mapping.source_aliases,
        &mut mapping.target_aliases,
        &mut mapping.source_color,
        &mut mapping.target_color,
        &mut mapping.source_x,
        &mut mapping.source_y,
        &mut mapping.target_x,
        &mut mapping.target_y,
        &mut mapping.source_position_locked,
        &mut mapping.target_position_locked,
    ];
    for column in columns {
        remap_import_column(column, headers);
    }
    for attribute in &mut mapping.entity_attributes {
        remap_import_column(&mut attribute.column, headers);
    }
    for filter in &mut mapping.row_filters {
        remap_import_column(&mut filter.column, headers);
    }
}

fn remap_import_column(column: &mut String, headers: &[String]) {
    if column.is_empty() {
        return;
    }
    if let Some(header) = headers
        .iter()
        .find(|header| header.trim().eq_ignore_ascii_case(column.trim()))
    {
        *column = header.clone();
    }
}

fn delimited_text_format(path: &str) -> &'static str {
    if delimited_text_delimiter(path) == b'\t' {
        "TSV"
    } else {
        "CSV"
    }
}

#[derive(Clone, Copy)]
enum GraphMlScope {
    Graph,
    Node,
    Edge,
}

fn graphml_xml_attributes(
    start: &quick_xml::events::BytesStart<'_>,
    decoder: quick_xml::encoding::Decoder,
) -> Result<HashMap<String, String>, String> {
    let mut attributes = HashMap::new();
    for attribute in start.attributes() {
        let attribute = attribute.map_err(|error| error.to_string())?;
        let name = String::from_utf8_lossy(attribute.key.as_ref()).into_owned();
        let value = attribute
            .decoded_and_normalized_value(quick_xml::XmlVersion::Implicit1_0, decoder)
            .map_err(|error| error.to_string())?
            .into_owned();
        attributes.insert(name, value);
    }
    Ok(attributes)
}

fn graphml_store_data(
    scope: GraphMlScope,
    key: String,
    value: String,
    keys: &HashMap<String, (String, String)>,
    document: &mut GraphMlDocument,
    nodes: &mut Vec<GraphMlRecord>,
    edge: &mut Option<GraphMlRecord>,
) -> Result<(), String> {
    let (declared_scope, property) = keys
        .get(&key)
        .cloned()
        .unwrap_or_else(|| (String::new(), key));
    let scope_name = match scope {
        GraphMlScope::Graph => "graph",
        GraphMlScope::Node => "node",
        GraphMlScope::Edge => "edge",
    };
    if !declared_scope.is_empty() && declared_scope != scope_name && declared_scope != "all" {
        return Err(format!(
            "GraphML key {property} is declared for {declared_scope} but used on {scope_name}"
        ));
    }
    match scope {
        GraphMlScope::Graph => {
            document.graph_data.insert(property, value);
        }
        GraphMlScope::Node => {
            nodes
                .last_mut()
                .ok_or("GraphML node data is outside a node")?
                .data
                .insert(property, value);
        }
        GraphMlScope::Edge => {
            edge.as_mut()
                .ok_or("GraphML edge data is outside an edge")?
                .data
                .insert(property, value);
        }
    }
    Ok(())
}

fn resolve_graphml_reference(reference: &str) -> Result<String, String> {
    if let Some(value) = resolve_predefined_entity(reference) {
        return Ok(value.to_owned());
    }
    let codepoint = if let Some(hex) = reference.strip_prefix("#x") {
        u32::from_str_radix(hex, 16).map_err(|error| error.to_string())?
    } else if let Some(decimal) = reference.strip_prefix('#') {
        decimal.parse::<u32>().map_err(|error| error.to_string())?
    } else {
        return Err(format!(
            "GraphML uses unsupported custom XML entity &{reference};"
        ));
    };
    char::from_u32(codepoint)
        .map(|character| character.to_string())
        .ok_or_else(|| format!("GraphML contains invalid XML character reference &{reference};"))
}

fn parse_graphml(path: &str) -> Result<GraphMlDocument, String> {
    if path.trim().is_empty() {
        return Err("GraphML import path cannot be empty".into());
    }
    let mut reader = XmlReader::from_file(path).map_err(|error| error.to_string())?;
    reader.config_mut().trim_text(false);
    let decoder = reader.decoder();
    let mut document = GraphMlDocument::default();
    let mut keys: HashMap<String, (String, String)> = HashMap::new();
    let mut nodes: Vec<GraphMlRecord> = Vec::new();
    let mut edge: Option<GraphMlRecord> = None;
    let mut active_data: Option<(GraphMlScope, String, String)> = None;
    let mut root_seen = false;
    let mut graph_count = 0;
    let mut graphs: Vec<GraphMlGraphContext> = Vec::new();
    let mut buffer = Vec::new();
    loop {
        let event = reader
            .read_event_into(&mut buffer)
            .map_err(|error| format!("invalid GraphML XML: {error}"))?;
        match event {
            XmlEvent::Start(start) => match start.local_name().as_ref() {
                b"graphml" => root_seen = true,
                b"key" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    if let Some(id) = attributes.get("id") {
                        keys.insert(
                            id.clone(),
                            (
                                attributes.get("for").cloned().unwrap_or_default(),
                                attributes
                                    .get("attr.name")
                                    .cloned()
                                    .unwrap_or_else(|| id.clone()),
                            ),
                        );
                    }
                }
                b"graph" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let default_directed = match attributes
                        .get("edgedefault")
                        .map(String::as_str)
                        .unwrap_or("undirected")
                    {
                        "directed" => true,
                        "undirected" => false,
                        value => {
                            return Err(format!(
                                "GraphML edgedefault must be directed or undirected, got {value}"
                            ));
                        }
                    };
                    if graphs.is_empty() {
                        if !nodes.is_empty() || edge.is_some() {
                            return Err("top-level GraphML graph begins inside another item".into());
                        }
                        graph_count += 1;
                        document.default_directed = default_directed;
                        graphs.push(GraphMlGraphContext {
                            default_directed,
                            parent_node_id: None,
                        });
                    } else {
                        if nodes.len() != graphs.len() || edge.is_some() {
                            return Err(
                                "nested GraphML graph must be directly inside a node".into()
                            );
                        }
                        let parent_node_id = nodes.last().map(|node| node.id.clone());
                        graphs.push(GraphMlGraphContext {
                            default_directed,
                            parent_node_id,
                        });
                    }
                }
                b"node" => {
                    if graphs.is_empty() || nodes.len() + 1 != graphs.len() || edge.is_some() {
                        return Err("GraphML nodes must be direct children of a graph".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let mut record = GraphMlRecord {
                        id: attributes.get("id").cloned().unwrap_or_default(),
                        ..GraphMlRecord::default()
                    };
                    if let Some(parent_node_id) = graphs
                        .last()
                        .and_then(|graph| graph.parent_node_id.as_ref())
                    {
                        record
                            .data
                            .insert("arch_notebook_parent_id".into(), parent_node_id.clone());
                    }
                    nodes.push(record);
                }
                b"edge" => {
                    if graphs.is_empty() || nodes.len() + 1 != graphs.len() {
                        return Err("GraphML edges must be direct children of a graph".into());
                    }
                    if edge.is_some() {
                        return Err("nested GraphML edges are not supported".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let mut record = graphml_edge_record(attributes)?;
                    record.directed.get_or_insert(
                        graphs
                            .last()
                            .map(|graph| graph.default_directed)
                            .unwrap_or(document.default_directed),
                    );
                    edge = Some(record);
                }
                b"data" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let key = attributes
                        .get("key")
                        .cloned()
                        .ok_or("GraphML data element is missing its key")?;
                    let scope = if edge.is_some() {
                        GraphMlScope::Edge
                    } else if nodes.len() == graphs.len() {
                        GraphMlScope::Node
                    } else {
                        GraphMlScope::Graph
                    };
                    active_data = Some((scope, key, String::new()));
                }
                _ => {}
            },
            XmlEvent::Empty(start) => match start.local_name().as_ref() {
                b"key" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    if let Some(id) = attributes.get("id") {
                        keys.insert(
                            id.clone(),
                            (
                                attributes.get("for").cloned().unwrap_or_default(),
                                attributes
                                    .get("attr.name")
                                    .cloned()
                                    .unwrap_or_else(|| id.clone()),
                            ),
                        );
                    }
                }
                b"graph" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let default_directed = match attributes
                        .get("edgedefault")
                        .map(String::as_str)
                        .unwrap_or("undirected")
                    {
                        "directed" => true,
                        "undirected" => false,
                        value => {
                            return Err(format!(
                                "GraphML edgedefault must be directed or undirected, got {value}"
                            ));
                        }
                    };
                    if graphs.is_empty() {
                        if !nodes.is_empty() || edge.is_some() {
                            return Err("top-level GraphML graph begins inside another item".into());
                        }
                        graph_count += 1;
                        document.default_directed = default_directed;
                    } else if nodes.len() != graphs.len() || edge.is_some() {
                        return Err("nested GraphML graph must be directly inside a node".into());
                    }
                }
                b"node" => {
                    if graphs.is_empty() || nodes.len() + 1 != graphs.len() || edge.is_some() {
                        return Err("GraphML nodes must be direct children of a graph".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let mut record = GraphMlRecord {
                        id: attributes.get("id").cloned().unwrap_or_default(),
                        ..GraphMlRecord::default()
                    };
                    if let Some(parent_node_id) = graphs
                        .last()
                        .and_then(|graph| graph.parent_node_id.as_ref())
                    {
                        record
                            .data
                            .insert("arch_notebook_parent_id".into(), parent_node_id.clone());
                    }
                    document.nodes.push(record);
                }
                b"edge" => {
                    if graphs.is_empty() || nodes.len() + 1 != graphs.len() {
                        return Err("GraphML edges must be direct children of a graph".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let mut record = graphml_edge_record(attributes)?;
                    record.directed.get_or_insert(
                        graphs
                            .last()
                            .map(|graph| graph.default_directed)
                            .unwrap_or(document.default_directed),
                    );
                    document.edges.push(record);
                }
                b"data" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let key = attributes
                        .get("key")
                        .cloned()
                        .ok_or("GraphML data element is missing its key")?;
                    let scope = if edge.is_some() {
                        GraphMlScope::Edge
                    } else if nodes.len() == graphs.len() {
                        GraphMlScope::Node
                    } else {
                        GraphMlScope::Graph
                    };
                    graphml_store_data(
                        scope,
                        key,
                        String::new(),
                        &keys,
                        &mut document,
                        &mut nodes,
                        &mut edge,
                    )?;
                }
                _ => {}
            },
            XmlEvent::Text(text) => {
                if let Some((_, _, value)) = active_data.as_mut() {
                    let decoded = text.decode().map_err(|error| error.to_string())?;
                    value.push_str(&decoded);
                }
            }
            XmlEvent::CData(text) => {
                if let Some((_, _, value)) = active_data.as_mut() {
                    value.push_str(&text.decode().map_err(|error| error.to_string())?);
                }
            }
            XmlEvent::GeneralRef(reference) => {
                if let Some((_, _, value)) = active_data.as_mut() {
                    value.push_str(&resolve_graphml_reference(
                        &reference.decode().map_err(|error| error.to_string())?,
                    )?);
                }
            }
            XmlEvent::End(end) => match end.local_name().as_ref() {
                b"data" => {
                    let Some((scope, key, value)) = active_data.take() else {
                        return Err("GraphML data end tag has no matching start".into());
                    };
                    graphml_store_data(
                        scope,
                        key,
                        value,
                        &keys,
                        &mut document,
                        &mut nodes,
                        &mut edge,
                    )?;
                }
                b"node" => {
                    document
                        .nodes
                        .push(nodes.pop().ok_or("GraphML node end tag has no start")?);
                }
                b"edge" => {
                    document
                        .edges
                        .push(edge.take().ok_or("GraphML edge end tag has no start")?);
                }
                b"graph" => {
                    graphs.pop().ok_or("GraphML graph end tag has no start")?;
                }
                _ => {}
            },
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_seen || graph_count == 0 {
        return Err("file does not contain a GraphML graph".into());
    }
    if !graphs.is_empty() || !nodes.is_empty() || edge.is_some() || active_data.is_some() {
        return Err("GraphML document ended inside an element".into());
    }
    Ok(document)
}

fn graphml_edge_record(attributes: HashMap<String, String>) -> Result<GraphMlRecord, String> {
    let directed = attributes
        .get("directed")
        .map(|value| match value.as_str() {
            "true" | "1" => Ok(true),
            "false" | "0" => Ok(false),
            _ => Err(format!("invalid GraphML edge direction: {value}")),
        })
        .transpose()?;
    Ok(GraphMlRecord {
        id: attributes.get("id").cloned().unwrap_or_default(),
        source: attributes.get("source").cloned().unwrap_or_default(),
        target: attributes.get("target").cloned().unwrap_or_default(),
        directed,
        data: HashMap::new(),
    })
}

fn graphml_property<'a>(record: &'a GraphMlRecord, keys: &[&str]) -> Option<&'a str> {
    keys.iter().find_map(|key| {
        record
            .data
            .iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(key))
            .map(|(_, value)| value.as_str())
    })
}

fn graphml_json_property<T: serde::Serialize>(value: &T) -> Result<String, String> {
    serde_json::to_string(value).map_err(|error| error.to_string())
}

fn graphml_csv_bytes(path: &str) -> Result<Vec<u8>, String> {
    let mut document = parse_graphml(path)?;
    let source_identity = fs::canonicalize(path)
        .unwrap_or_else(|_| PathBuf::from(path))
        .to_string_lossy()
        .into_owned();
    let mut namespace_bytes = [0u8; 16];
    let mut hash = 0xcbf29ce484222325u64;
    for byte in source_identity.bytes() {
        hash = (hash ^ u64::from(byte)).wrapping_mul(0x100000001b3);
    }
    namespace_bytes[..8].copy_from_slice(&hash.to_be_bytes());
    namespace_bytes[8..].copy_from_slice(
        &hash
            .rotate_left(29)
            .wrapping_mul(0x9e3779b97f4a7c15)
            .to_be_bytes(),
    );
    namespace_bytes[6] = (namespace_bytes[6] & 0x0f) | 0x50;
    namespace_bytes[8] = (namespace_bytes[8] & 0x3f) | 0x80;
    let namespace = Uuid::from_bytes(namespace_bytes);
    let mut node_ids = HashMap::<String, Uuid>::new();
    let mut seen_ids = std::collections::HashSet::new();
    let mut positions = Vec::<(f32, f32)>::new();

    for node in &document.nodes {
        if node.id.trim().is_empty() || !seen_ids.insert(node.id.clone()) {
            return Err(format!(
                "GraphML node ID is empty or duplicated: {}",
                node.id
            ));
        }
        let id = graphml_property(node, &["arch_notebook_uuid", "node_uuid", "uuid"])
            .filter(|value| !value.trim().is_empty())
            .map(Uuid::parse_str)
            .transpose()
            .map_err(|error| format!("GraphML node {} has an invalid UUID: {error}", node.id))?
            .or_else(|| {
                node.id
                    .strip_prefix("n_")
                    .and_then(|value| Uuid::parse_str(value).ok())
            })
            .unwrap_or_else(|| {
                let mut bytes = namespace.into_bytes();
                for (index, byte) in node.id.bytes().enumerate() {
                    let slot = index % bytes.len();
                    bytes[slot] = bytes[slot].wrapping_mul(31).wrapping_add(byte);
                }
                bytes[6] = (bytes[6] & 0x0f) | 0x50;
                bytes[8] = (bytes[8] & 0x3f) | 0x80;
                Uuid::from_bytes(bytes)
            });
        node_ids.insert(node.id.clone(), id);

        let x = graphml_property(node, &["x", "x_coord", "xcoordinate"])
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| value.is_finite());
        let y = graphml_property(node, &["y", "y_coord", "ycoordinate"])
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| value.is_finite());
        if let (Some(x), Some(y)) = (x, y) {
            positions.push((x, y));
        }
    }

    // Normalize external layout coordinates only when they are outside the app's unit chart.
    let normalize = positions
        .iter()
        .any(|(x, y)| !(0.0..=1.0).contains(x) || !(0.0..=1.0).contains(y));
    let bounds = positions
        .iter()
        .fold(None::<(f32, f32, f32, f32)>, |bounds, (x, y)| {
            Some(match bounds {
                Some((min_x, min_y, max_x, max_y)) => {
                    (min_x.min(*x), min_y.min(*y), max_x.max(*x), max_y.max(*y))
                }
                None => (*x, *y, *x, *y),
            })
        });
    let coordinate = |value: Option<&str>, axis: usize| -> String {
        let Some(mut value) = value
            .and_then(|value| value.parse::<f32>().ok())
            .filter(|value| value.is_finite())
        else {
            return String::new();
        };
        if normalize {
            if let Some((min_x, min_y, max_x, max_y)) = bounds {
                let (min, max) = if axis == 0 {
                    (min_x, max_x)
                } else {
                    (min_y, max_y)
                };
                value = if max > min {
                    0.1 + 0.8 * (value - min) / (max - min)
                } else {
                    0.5
                };
            }
        }
        value.clamp(0.0, 1.0).to_string()
    };

    let mut headers = vec![
        "record_type",
        "connection_id",
        "source",
        "source_entity_id",
        "source_external_ids_json",
        "source_type",
        "source_detail_json",
        "source_aliases_json",
        "source_color",
        "source_x",
        "source_y",
        "source_position_locked",
        "relationship",
        "target",
        "target_entity_id",
        "target_external_ids_json",
        "target_type",
        "target_detail_json",
        "target_aliases_json",
        "target_color",
        "target_x",
        "target_y",
        "target_position_locked",
        "date",
        "confidence",
        "directed",
        "notes",
        "source_notes",
        "source_reference",
        "ingest_source",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    let mut attribute_keys = document
        .nodes
        .iter()
        .flat_map(|node| node.data.keys())
        .filter(|key| {
            ![
                "label",
                "name",
                "entity_name",
                "entity_type",
                "type",
                "kind",
                "detail",
                "description",
                "aliases_json",
                "aliases",
                "external_ids_json",
                "external_ids",
                "merged_records_json",
                "attributes_json",
                "color",
                "x",
                "y",
                "x_coord",
                "y_coord",
                "position_locked",
            ]
            .iter()
            .any(|reserved| key.eq_ignore_ascii_case(reserved))
        })
        .cloned()
        .collect::<Vec<_>>();
    attribute_keys.push("merged_records_json".to_owned());
    attribute_keys.sort_by_key(|key| key.to_ascii_lowercase());
    attribute_keys.dedup_by(|first, second| first.eq_ignore_ascii_case(second));
    for key in &attribute_keys {
        headers.push(format!("source_attribute:{key}"));
    }
    for key in &attribute_keys {
        headers.push(format!("target_attribute:{key}"));
    }

    let mut writer = csv::WriterBuilder::new().from_writer(Vec::new());
    writer
        .write_record(&headers)
        .map_err(|error| error.to_string())?;
    for node in &document.nodes {
        let name = graphml_property(node, &["label", "name", "entity_name"])
            .filter(|value| !value.is_empty())
            .unwrap_or(&node.id);
        let kind = graphml_property(node, &["entity_type", "type", "kind"]).unwrap_or("");
        let detail = graphml_property(node, &["detail", "description"]).unwrap_or("");
        let aliases = graphml_property(node, &["aliases_json", "aliases"])
            .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
            .or_else(|| {
                graphml_property(node, &["aliases"]).map(|value| {
                    value
                        .split('|')
                        .map(str::trim)
                        .filter(|part| !part.is_empty())
                        .map(str::to_owned)
                        .collect()
                })
            })
            .unwrap_or_default();
        let external_ids = graphml_property(node, &["external_ids_json", "external_ids"])
            .map(serde_json::from_str::<Vec<(String, String)>>)
            .transpose()
            .map_err(|error| format!("GraphML node {} has invalid external IDs: {error}", node.id))?
            .unwrap_or_default();
        let entity_id = node_ids[&node.id].to_string();
        let source_detail = graphml_json_property(&detail)?;
        let source_aliases = graphml_json_property(&aliases)?;
        let source_external_ids = graphml_json_property(&external_ids)?;
        let mut row = vec![String::new(); headers.len()];
        let fields = [
            ("record_type", "entity"),
            ("source", name),
            ("source_entity_id", entity_id.as_str()),
            ("source_external_ids_json", source_external_ids.as_str()),
            ("source_type", kind),
            ("source_detail_json", source_detail.as_str()),
            ("source_aliases_json", source_aliases.as_str()),
            (
                "source_color",
                graphml_property(node, &["color"]).unwrap_or(""),
            ),
            (
                "source_position_locked",
                graphml_property(node, &["position_locked"]).unwrap_or(""),
            ),
        ];
        for (key, value) in fields {
            if let Some(index) = headers.iter().position(|header| header == key) {
                row[index] = value.to_owned();
            }
        }
        if let Some(index) = headers.iter().position(|header| header == "source_x") {
            row[index] = coordinate(graphml_property(node, &["x", "x_coord", "xcoordinate"]), 0);
        }
        if let Some(index) = headers.iter().position(|header| header == "source_y") {
            row[index] = coordinate(graphml_property(node, &["y", "y_coord", "ycoordinate"]), 1);
        }
        let attributes = graphml_property(node, &["attributes_json"])
            .and_then(|value| serde_json::from_str::<Vec<(String, String)>>(value).ok())
            .unwrap_or_default();
        for (key, value) in &attributes {
            if let Some(index) = headers
                .iter()
                .position(|header| header == &format!("source_attribute:{key}"))
            {
                row[index] = value.clone();
            }
        }
        for key in &attribute_keys {
            let value = node
                .data
                .iter()
                .find(|(name, _)| name.eq_ignore_ascii_case(key))
                .map(|(_, value)| value.clone())
                .or_else(|| {
                    attributes
                        .iter()
                        .find(|(name, _)| name.eq_ignore_ascii_case(key))
                        .map(|(_, value)| value.clone())
                })
                .unwrap_or_default();
            if let Some(index) = headers
                .iter()
                .position(|header| header == &format!("source_attribute:{key}"))
            {
                row[index] = value;
            }
        }
        writer
            .write_record(row)
            .map_err(|error| error.to_string())?;
    }

    let mut edge_ids = std::collections::HashSet::new();
    for (index, edge) in document.edges.iter_mut().enumerate() {
        if !edge.id.is_empty() && !edge_ids.insert(edge.id.clone()) {
            return Err(format!("GraphML edge ID is duplicated: {}", edge.id));
        }
        let from = node_ids.get(&edge.source).ok_or_else(|| {
            format!(
                "GraphML edge {} has missing source node {}",
                edge.id, edge.source
            )
        })?;
        let to = node_ids.get(&edge.target).ok_or_else(|| {
            format!(
                "GraphML edge {} has missing target node {}",
                edge.id, edge.target
            )
        })?;
        let mut row = vec![String::new(); headers.len()];
        let source_node = document
            .nodes
            .iter()
            .find(|node| node.id == edge.source)
            .expect("resolved source");
        let target_node = document
            .nodes
            .iter()
            .find(|node| node.id == edge.target)
            .expect("resolved target");
        let put = |row: &mut Vec<String>, key: &str, value: String| {
            if let Some(position) = headers.iter().position(|header| header == key) {
                row[position] = value;
            }
        };
        for (side, record, id) in [("source", source_node, from), ("target", target_node, to)] {
            let name = graphml_property(record, &["label", "name", "entity_name"])
                .unwrap_or(&record.id)
                .to_owned();
            let kind = graphml_property(record, &["entity_type", "type", "kind"])
                .unwrap_or("")
                .to_owned();
            let detail = graphml_property(record, &["detail", "description"]).unwrap_or("");
            let aliases = graphml_property(record, &["aliases_json", "aliases"])
                .and_then(|value| serde_json::from_str::<Vec<String>>(value).ok())
                .unwrap_or_default();
            put(&mut row, side, name);
            put(&mut row, &format!("{side}_entity_id"), id.to_string());
            put(&mut row, &format!("{side}_type"), kind);
            put(
                &mut row,
                &format!("{side}_detail_json"),
                graphml_json_property(&detail)?,
            );
            put(
                &mut row,
                &format!("{side}_aliases_json"),
                graphml_json_property(&aliases)?,
            );
            put(
                &mut row,
                &format!("{side}_color"),
                graphml_property(record, &["color"])
                    .unwrap_or("")
                    .to_owned(),
            );
            put(
                &mut row,
                &format!("{side}_x"),
                coordinate(
                    graphml_property(record, &["x", "x_coord", "xcoordinate"]),
                    0,
                ),
            );
            put(
                &mut row,
                &format!("{side}_y"),
                coordinate(
                    graphml_property(record, &["y", "y_coord", "ycoordinate"]),
                    1,
                ),
            );
            put(
                &mut row,
                &format!("{side}_position_locked"),
                graphml_property(record, &["position_locked"])
                    .unwrap_or("")
                    .to_owned(),
            );
            let attributes = graphml_property(record, &["attributes_json"])
                .and_then(|value| serde_json::from_str::<Vec<(String, String)>>(value).ok())
                .unwrap_or_default();
            for key in &attribute_keys {
                let value = record
                    .data
                    .iter()
                    .find(|(name, _)| name.eq_ignore_ascii_case(key))
                    .map(|(_, value)| value.clone())
                    .or_else(|| {
                        attributes
                            .iter()
                            .find(|(name, _)| name.eq_ignore_ascii_case(key))
                            .map(|(_, value)| value.clone())
                    })
                    .unwrap_or_default();
                put(&mut row, &format!("{side}_attribute:{key}"), value);
            }
        }
        put(&mut row, "record_type", "connection".into());
        put(
            &mut row,
            "relationship",
            graphml_property(edge, &["relationship", "label", "name"])
                .unwrap_or("")
                .to_owned(),
        );
        put(
            &mut row,
            "connection_id",
            graphml_property(edge, &["connection_id"])
                .unwrap_or("")
                .to_owned(),
        );
        put(
            &mut row,
            "date",
            graphml_property(edge, &["date"]).unwrap_or("").to_owned(),
        );
        put(
            &mut row,
            "confidence",
            graphml_property(edge, &["confidence"])
                .unwrap_or("2")
                .to_owned(),
        );
        put(
            &mut row,
            "directed",
            edge.directed
                .unwrap_or(document.default_directed)
                .to_string(),
        );
        put(
            &mut row,
            "notes",
            graphml_property(edge, &["notes", "analyst_notes"])
                .unwrap_or("")
                .to_owned(),
        );
        put(
            &mut row,
            "source_notes",
            graphml_property(edge, &["source_notes"])
                .unwrap_or("")
                .to_owned(),
        );
        put(
            &mut row,
            "source_reference",
            graphml_property(edge, &["source_reference", "evidence"])
                .unwrap_or("")
                .to_owned(),
        );
        let ingest_source = graphml_property(edge, &["ingest_source"])
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| format!("graphml:{path}"));
        let connection_id = graphml_property(edge, &["connection_id"])
            .filter(|value| !value.is_empty())
            .map(str::to_owned)
            .unwrap_or_else(|| {
                format!(
                    "graphml:{}",
                    if edge.id.is_empty() {
                        format!("edge-{index}")
                    } else {
                        edge.id.clone()
                    }
                )
            });
        put(&mut row, "connection_id", connection_id);
        put(&mut row, "ingest_source", ingest_source);
        writer
            .write_record(row)
            .map_err(|error| error.to_string())?;
    }
    writer
        .into_inner()
        .map_err(|error| error.into_error().to_string())
}

fn write_graphml_csv_temp(contents: Vec<u8>) -> Result<PathBuf, String> {
    use std::fs::OpenOptions;
    let path = env::temp_dir().join(format!(
        "arch-notebook-graphml-import-{}.csv",
        Uuid::new_v4()
    ));
    #[cfg(unix)]
    use std::os::unix::fs::OpenOptionsExt;
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    options.mode(0o600);
    let mut file = options
        .open(&path)
        .map_err(|error| format!("cannot create temporary GraphML import file: {error}"))?;
    if let Err(error) = file.write_all(&contents) {
        let _ = fs::remove_file(&path);
        return Err(format!(
            "cannot write temporary GraphML import file: {error}"
        ));
    }
    Ok(path)
}

fn graphml_data(xml: &mut String, key: &str, value: &str) {
    xml.push_str("      <data key=\"");
    xml.push_str(key);
    xml.push_str("\">");
    xml.push_str(&escape_xml_text(value));
    xml.push_str("</data>\n");
}

fn paths_refer_to_same_file(first: &str, second: &str) -> bool {
    let first = Path::new(first);
    let second = Path::new(second);
    first == second
        || fs::canonicalize(first)
            .ok()
            .zip(fs::canonicalize(second).ok())
            .is_some_and(|(first, second)| first == second)
}

fn escape_xml_text(value: &str) -> String {
    let mut escaped = String::with_capacity(value.len());
    for character in value.chars() {
        match character {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&apos;"),
            '\u{9}' | '\u{A}' | '\u{D}' => escaped.push(character),
            character
                if ('\u{20}'..='\u{D7FF}').contains(&character)
                    || ('\u{E000}'..='\u{FFFD}').contains(&character)
                    || ('\u{10000}'..='\u{10FFFF}').contains(&character) =>
            {
                escaped.push(character);
            }
            _ => escaped.push('\u{FFFD}'),
        }
    }
    escaped
}

fn parse_entity_color(value: &str) -> Result<[u8; 3], String> {
    let hex = value.strip_prefix('#').unwrap_or(value);
    if hex.len() != 6 || !hex.bytes().all(|byte| byte.is_ascii_hexdigit()) {
        return Err("expected a six-digit RGB hex value such as #3B82A2".into());
    }
    let channel =
        |start| u8::from_str_radix(&hex[start..start + 2], 16).map_err(|error| error.to_string());
    Ok([channel(0)?, channel(2)?, channel(4)?])
}

fn parse_entity_position(value: &str) -> Result<f32, String> {
    let position = value
        .parse::<f32>()
        .map_err(|error| format!("expected a normalized chart position: {error}"))?;
    if !position.is_finite() || !(0.0..=1.0).contains(&position) {
        return Err("expected a finite value from 0 to 1".into());
    }
    Ok(position)
}

fn parse_import_bool(value: &str) -> Result<bool, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        "true" | "1" | "yes" => Ok(true),
        "false" | "0" | "no" => Ok(false),
        _ => Err(format!("expected true/false, 1/0, or yes/no; got {value}")),
    }
}

fn resolve_sqlite_database_path(database_path: &str, case_path: &str) -> Result<PathBuf, String> {
    let database_path = Path::new(database_path);
    let resolved = if database_path.is_absolute() {
        database_path.to_path_buf()
    } else {
        let case_parent = Path::new(case_path)
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        case_parent.join(database_path)
    };
    fs::canonicalize(&resolved).map_err(|error| {
        format!(
            "cannot resolve SQLite database {}: {error}",
            resolved.display()
        )
    })
}

fn sqlite_database_identity_path(database_path: &str, case_path: &str) -> PathBuf {
    if let Ok(path) = resolve_sqlite_database_path(database_path, case_path) {
        return path;
    }

    let database_path = Path::new(database_path);
    let resolved = if database_path.is_absolute() {
        database_path.to_path_buf()
    } else {
        let case_parent = Path::new(case_path)
            .parent()
            .filter(|parent| !parent.as_os_str().is_empty())
            .unwrap_or_else(|| Path::new("."));
        case_parent.join(database_path)
    };

    let mut ancestor = resolved.as_path();
    let mut missing = Vec::new();
    loop {
        if let Ok(canonical_ancestor) = fs::canonicalize(ancestor) {
            let mut identity = canonical_ancestor;
            for component in missing.iter().rev() {
                identity.push(component);
            }
            return identity;
        }
        let Some(name) = ancestor.file_name() else {
            return resolved;
        };
        missing.push(name.to_os_string());
        let Some(parent) = ancestor.parent() else {
            return resolved;
        };
        ancestor = parent;
    }
}

fn same_sql_series(
    first_database: &str,
    first_series: &str,
    second_database: &str,
    second_series: &str,
    case_path: &str,
) -> bool {
    if first_series.trim() != second_series.trim() {
        return false;
    }
    sqlite_database_identity_path(first_database, case_path)
        == sqlite_database_identity_path(second_database, case_path)
}

fn sql_source_key(database_path: &str, series: &str) -> String {
    format!(
        "sqlite-v2:{}:{}{}",
        database_path.len(),
        database_path,
        series
    )
}

fn normalize_snapshot_database_paths(case: &mut CaseFile, case_path: &str) {
    let mut source_key_rewrites = HashMap::new();
    let mut database_path_rewrites = HashMap::new();
    let mut ambiguous_database_paths = HashSet::new();
    let mut ambiguous_source_keys = HashSet::new();
    for snapshot in &mut case.snapshots {
        let old_database_path = snapshot.database_path.clone();
        let configured_path = (!snapshot.configured_database_path.trim().is_empty())
            .then(|| snapshot.configured_database_path.as_str());
        let path = configured_path
            .and_then(|path| resolve_sqlite_database_path(path, case_path).ok())
            .or_else(|| resolve_sqlite_database_path(&snapshot.database_path, case_path).ok());
        if let Some(path) = path {
            let old_source_key = snapshot.source_key.clone();
            snapshot.database_path = path.to_string_lossy().into_owned();
            let new_source_key = sql_source_key(&snapshot.database_path, &snapshot.series);
            if old_database_path != snapshot.database_path {
                if let Some(previous_path) = database_path_rewrites.get(&old_database_path) {
                    if previous_path != &snapshot.database_path {
                        database_path_rewrites.remove(&old_database_path);
                        ambiguous_database_paths.insert(old_database_path);
                    }
                } else if !ambiguous_database_paths.contains(&old_database_path) {
                    database_path_rewrites
                        .insert(old_database_path, snapshot.database_path.clone());
                }
            }
            if old_source_key != new_source_key {
                if let Some(previous_key) = source_key_rewrites.get(&old_source_key) {
                    if previous_key != &new_source_key {
                        source_key_rewrites.remove(&old_source_key);
                        ambiguous_source_keys.insert(old_source_key);
                    }
                } else if !ambiguous_source_keys.contains(&old_source_key) {
                    source_key_rewrites.insert(old_source_key, new_source_key.clone());
                }
            }
            snapshot.source_key = new_source_key;
        }
    }
    for entity in &mut case.entities {
        for (source, _) in &mut entity.external_ids {
            if let Some(rewritten) = database_path_rewrites.get(source) {
                *source = rewritten.clone();
            }
        }
    }
    for link in &mut case.links {
        if let Some(source_key) = link.ingest_source.as_mut() {
            if let Some(rewritten) = source_key_rewrites.get(source_key) {
                *source_key = rewritten.clone();
            }
        }
    }
    if case.format_version < 2 {
        let snapshots = &case.snapshots;
        let entities = &case.entities;
        for link in &mut case.links {
            let Some(source_key) = link.ingest_source.as_deref() else {
                continue;
            };
            let Some(snapshot) = snapshots
                .iter()
                .rev()
                .find(|snapshot| snapshot.source_key == source_key)
            else {
                continue;
            };
            let fact = if !link.connection_id.is_empty() {
                snapshot
                    .connections
                    .iter()
                    .find(|fact| fact.connection_id == link.connection_id)
            } else {
                let source = entities.iter().find(|entity| entity.id == link.from);
                let target = entities.iter().find(|entity| entity.id == link.to);
                source
                    .zip(target)
                    .and_then(|(source, target)| {
                        canonical_connection(
                            &source.name,
                            &source.kind,
                            &target.name,
                            &target.kind,
                            &link.label,
                            link.directed,
                        )
                        .ok()
                    })
                    .map(|(_, _, _, _, key)| format!("natural:{key}"))
                    .and_then(|key| {
                        snapshot
                            .connections
                            .iter()
                            .find(|fact| comparison_key(fact) == key)
                    })
            };
            if let Some(fact) = fact {
                link.source_notes = fact.notes.clone();
                if link.notes == fact.notes {
                    link.notes.clear();
                }
            }
        }
    }
    case.format_version = CURRENT_CASE_FORMAT_VERSION;
    for run in &mut case.automation_runs {
        if run.snapshot_id.is_none() {
            run.snapshot_id = run
                .snapshot_index
                .and_then(|index| case.snapshots.get(index))
                .map(|snapshot| snapshot.id);
        }
    }
}

fn run_automation_cli(
    case_path: &str,
    job_name: Option<&str>,
    selected_jobs: Option<&[String]>,
    parameter_overrides: &[(String, String)],
    scoped_parameter_overrides: &[(String, String, String)],
    skip_recent_success: bool,
    json_output: bool,
) -> Result<bool, String> {
    let _case_lock = lock_automation_case(Path::new(case_path))?;
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let mut jobs: Vec<AutomationJob> = if let Some(name) = job_name {
        let matches: Vec<AutomationJob> = case
            .automation_jobs
            .iter()
            .filter(|job| {
                job.name.eq_ignore_ascii_case(name) || job.id.to_string().eq_ignore_ascii_case(name)
            })
            .cloned()
            .collect();
        if matches.len() > 1 {
            return Err(format!("job selector is ambiguous: {name}"));
        }
        vec![
            matches
                .into_iter()
                .next()
                .ok_or_else(|| format!("automation job not found: {name}"))?,
        ]
    } else if let Some(selected_jobs) = selected_jobs {
        select_enabled_automation_jobs(&case, selected_jobs)?
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
    apply_sql_parameter_overrides(&mut jobs, parameter_overrides, scoped_parameter_overrides)?;
    let mut app = App::automation_context(case, case_path);
    normalize_snapshot_database_paths(&mut app.case, &app.case_path);
    let mut failures = 0;
    let mut skipped = Vec::new();
    let mut outcomes = Vec::new();
    for job in jobs.iter_mut() {
        let latest_job_run = skip_recent_success
            .then(|| {
                app.case
                    .automation_runs
                    .iter()
                    .rev()
                    .find(|run| run.job_id == job.id)
            })
            .flatten();
        let latest_run_matches_current_job = latest_job_run.is_some_and(|run| {
            run.configuration_recorded
                && run.database_path == job.database_path
                && run.series == job.series
                && run.query == job.query
                && run.parameters == job.parameters
                && run.minimum_connections == job.minimum_connections
                && run.maximum_connections == job.maximum_connections
                && run.maximum_regressions == job.maximum_regressions
                && run.maximum_skipped_rows == job.maximum_skipped_rows
        });
        if let Some(run) =
            latest_job_run.filter(|run| latest_run_matches_current_job && run.success)
        {
            skipped.push(serde_json::json!({
                "job_id": job.id,
                "job": job.name,
                "status": "SKIPPED",
                "reason": "latest run for this configuration succeeded",
                "database_path": job.database_path,
                "series": job.series,
                "query": job.query,
                "parameters": job.parameters,
                "minimum_connections": job.minimum_connections,
                "maximum_connections": job.maximum_connections,
                "maximum_regressions": job.maximum_regressions,
                "maximum_skipped_rows": job.maximum_skipped_rows,
                "run_id": run.id,
                "finished_at": run.finished_at,
                "duration_ms": run.duration_ms,
                "snapshot_id": run.snapshot_id,
            }));
            if !json_output {
                println!(
                    "SKIP · {} · latest run for this configuration succeeded",
                    job.name
                );
            }
            continue;
        }
        let (success, message) = app.run_automation_job_cli(&job);
        if !success {
            failures += 1;
        }
        let run = app.case.automation_runs.last();
        let run_id = run.map(|run| run.id);
        let duration_ms = run.and_then(|run| run.duration_ms);
        let snapshot_id = run.and_then(|run| run.snapshot_id);
        let snapshot =
            snapshot_id.and_then(|id| app.case.snapshots.iter().find(|snapshot| snapshot.id == id));
        let snapshot_index = snapshot_id.and_then(|id| {
            app.case
                .snapshots
                .iter()
                .position(|snapshot| snapshot.id == id)
        });
        let previous = snapshot_index
            .and_then(|index| previous_snapshot_index(&app.case, index))
            .and_then(|index| app.case.snapshots.get(index));
        save_case_atomically(Path::new(case_path), &app.case)?;
        if json_output {
            outcomes.push(automation_outcome_json(
                &job, success, &message, run_id, snapshot, previous,
            ));
            if let Some(outcome) = outcomes.last_mut() {
                outcome["duration_ms"] = serde_json::json!(duration_ms);
            }
        } else {
            println!(
                "{} · {} · {}",
                if success { "PASS" } else { "FAIL" },
                job.name,
                message
            );
        }
    }
    if outcomes.is_empty() && !skipped.is_empty() {
        if json_output {
            println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":1,"ok":true,"saved":false,"case":case_path,"passed":0,"failed":0,"skipped":skipped.len(),"outcomes":outcomes,"skipped_jobs":skipped})).map_err(|error|error.to_string())?);
        }
        return Ok(true);
    }
    if json_output {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":1,"ok":failures==0,"saved":!outcomes.is_empty(),"case":case_path,"passed":outcomes.iter().filter(|outcome|outcome["status"]=="PASS").count(),"failed":failures,"skipped":skipped.len(),"outcomes":outcomes,"skipped_jobs":skipped})).map_err(|error|error.to_string())?);
    }
    Ok(failures == 0)
}

fn apply_sql_parameter_overrides(
    jobs: &mut [AutomationJob],
    parameter_overrides: &[(String, String)],
    scoped_parameter_overrides: &[(String, String, String)],
) -> Result<(), String> {
    for job in jobs.iter_mut() {
        for (name, value) in parameter_overrides {
            if let Some((_, configured_value)) = job
                .parameters
                .iter_mut()
                .find(|(configured_name, _)| configured_name == name)
            {
                *configured_value = value.clone();
            } else {
                job.parameters.push((name.clone(), value.clone()));
            }
        }
    }
    let mut assigned = HashSet::new();
    for (selector, name, value) in scoped_parameter_overrides {
        let matches: Vec<&mut AutomationJob> = jobs
            .iter_mut()
            .filter(|job| {
                job.name.eq_ignore_ascii_case(selector)
                    || job.id.to_string().eq_ignore_ascii_case(selector)
            })
            .collect();
        if matches.len() > 1 {
            return Err(format!("job selector is ambiguous: {selector}"));
        }
        let Some(job) = matches.into_iter().next() else {
            return Err(format!(
                "job-specific parameter selector is not in the selected batch: {selector}"
            ));
        };
        if !assigned.insert((job.id, name.clone())) {
            return Err(format!(
                "parameter {name} is overridden more than once for job {}",
                job.name
            ));
        }
        if let Some((_, configured_value)) = job
            .parameters
            .iter_mut()
            .find(|(configured_name, _)| configured_name == name)
        {
            *configured_value = value.clone();
        } else {
            job.parameters.push((name.clone(), value.clone()));
        }
    }
    Ok(())
}

fn automation_job_from_run(prior_run: &AutomationRun) -> Result<AutomationJob, String> {
    if !prior_run.configuration_recorded {
        return Err(format!(
            "automation run {} does not contain a saved configuration",
            prior_run.id
        ));
    }
    Ok(AutomationJob {
        id: prior_run.job_id,
        name: prior_run.job_name.clone(),
        database_path: prior_run.database_path.clone(),
        series: prior_run.series.clone(),
        query: prior_run.query.clone(),
        parameters: prior_run.parameters.clone(),
        enabled: prior_run.enabled,
        minimum_connections: prior_run.minimum_connections,
        maximum_connections: prior_run.maximum_connections,
        maximum_regressions: prior_run.maximum_regressions,
        maximum_skipped_rows: prior_run.maximum_skipped_rows,
    })
}

fn validate_saved_automation_configuration_cli(
    case_path: &str,
    run_id: Uuid,
    json_output: bool,
) -> Result<bool, String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let prior_run = case
        .automation_runs
        .iter()
        .find(|run| run.id == run_id)
        .ok_or_else(|| format!("automation run not found: {run_id}"))?;
    let job = automation_job_from_run(prior_run)?;
    let mut app = App::automation_context(case, case_path);
    normalize_snapshot_database_paths(&mut app.case, &app.case_path);
    app.sql_path = job.database_path.clone();
    app.sql_series = job.series.clone();
    app.sql_query = job.query.clone();
    app.sql_parameter_text = format_sql_parameters(&job.parameters);
    app.automation_minimum_connections = job.minimum_connections;
    app.automation_maximum_regressions = job.maximum_regressions;
    app.automation_maximum_skipped_rows = job.maximum_skipped_rows;
    let started = Instant::now();
    let snapshot_count_before = app.case.snapshots.len();
    let success = app.validate_sql_query_for_batch();
    let message = app.status.clone();
    let snapshot = app.case.snapshots.get(snapshot_count_before);
    let previous = snapshot
        .and_then(|_| previous_snapshot_index(&app.case, snapshot_count_before))
        .and_then(|index| app.case.snapshots.get(index));
    let duration_ms = started.elapsed().as_millis() as u64;
    if json_output {
        let mut outcome =
            automation_outcome_json(&job, success, &message, None, snapshot, previous);
        outcome["duration_ms"] = serde_json::json!(duration_ms);
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "ok": success,
                "saved": false,
                "case": case_path,
                "validated_run_id": run_id,
                "passed": usize::from(success),
                "failed": usize::from(!success),
                "outcomes": [outcome],
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{} · {} · {}",
            if success { "PASS" } else { "FAIL" },
            job.name,
            message
        );
    }
    Ok(success)
}

fn rerun_saved_automation_configuration_cli(
    case_path: &str,
    run_id: Uuid,
    json_output: bool,
) -> Result<bool, String> {
    let _case_lock = lock_automation_case(Path::new(case_path))?;
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let prior_run = case
        .automation_runs
        .iter()
        .find(|run| run.id == run_id)
        .ok_or_else(|| format!("automation run not found: {run_id}"))?;
    let job = automation_job_from_run(prior_run)?;
    let mut app = App::automation_context(case, case_path);
    normalize_snapshot_database_paths(&mut app.case, &app.case_path);
    let (success, message) = app.run_automation_job_cli(&job);
    if let Some(run) = app.case.automation_runs.last_mut() {
        run.rerun_of = Some(run_id);
    }
    let new_run = app
        .case
        .automation_runs
        .last()
        .ok_or_else(|| "automation rerun did not produce run history".to_owned())?;
    let new_run_id = new_run.id;
    let duration_ms = new_run.duration_ms;
    let snapshot_id = new_run.snapshot_id;
    let snapshot =
        snapshot_id.and_then(|id| app.case.snapshots.iter().find(|snapshot| snapshot.id == id));
    let snapshot_index = snapshot_id.and_then(|id| {
        app.case
            .snapshots
            .iter()
            .position(|snapshot| snapshot.id == id)
    });
    let previous = snapshot_index
        .and_then(|index| previous_snapshot_index(&app.case, index))
        .and_then(|index| app.case.snapshots.get(index));
    save_case_atomically(Path::new(case_path), &app.case)?;
    if json_output {
        let mut outcome = automation_outcome_json(
            &job,
            success,
            &message,
            Some(new_run_id),
            snapshot,
            previous,
        );
        outcome["duration_ms"] = serde_json::json!(duration_ms);
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "ok": success,
                "saved": true,
                "case": case_path,
                "rerun_of": run_id,
                "passed": usize::from(success),
                "failed": usize::from(!success),
                "outcomes": [outcome],
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{} · rerun of {} · {} · {}",
            if success { "PASS" } else { "FAIL" },
            run_id,
            job.name,
            message
        );
    }
    Ok(success)
}

fn select_enabled_automation_jobs(
    case: &CaseFile,
    selectors: &[String],
) -> Result<Vec<AutomationJob>, String> {
    let mut selected_ids = HashSet::new();
    for selector in selectors {
        let matches: Vec<&AutomationJob> = case
            .automation_jobs
            .iter()
            .filter(|job| {
                job.name.eq_ignore_ascii_case(selector)
                    || job.id.to_string().eq_ignore_ascii_case(selector)
            })
            .collect();
        if matches.len() > 1 {
            return Err(format!("job selector is ambiguous: {selector}"));
        }
        let Some(job) = matches.first() else {
            return Err(format!("automation job not found: {selector}"));
        };
        if !job.enabled {
            return Err(format!("automation job is disabled: {}", job.name));
        }
        if !selected_ids.insert(job.id) {
            return Err(format!(
                "automation job selected more than once: {}",
                job.name
            ));
        }
    }
    Ok(case
        .automation_jobs
        .iter()
        .filter(|job| job.enabled && selected_ids.contains(&job.id))
        .cloned()
        .collect())
}

fn export_graphml_cli(case_path: &str, output_path: &str, json_output: bool) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let entity_count = case.entities.len();
    let relationship_count = case.links.len();
    let mut app = App::automation_context(case, case_path);
    app.graphml_export_path = output_path.to_owned();
    app.export_graphml();
    if let Some(error) = app.status.strip_prefix("GRAPHML EXPORT FAILED · ") {
        return Err(error.to_owned());
    }
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "ok": true,
                "case": case_path,
                "output": output_path,
                "entities": entity_count,
                "relationships": relationship_count,
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", app.status);
    }
    Ok(())
}

fn import_graphml_cli(case_path: &str, input_path: &str, json_output: bool) -> Result<(), String> {
    let case_file = Path::new(case_path);
    let _case_lock = lock_automation_case(case_file)?;
    let content =
        fs::read_to_string(case_file).map_err(|error| format!("cannot read case: {error}"))?;
    let mut case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    normalize_snapshot_database_paths(&mut case, case_path);
    let mut app = App::automation_context(case, case_path);
    app.import_path = input_path.to_owned();
    app.import_graphml();
    if let Some(error) = app.status.strip_prefix("GRAPHML IMPORT FAILED · ") {
        return Err(error.to_owned());
    }
    save_case_atomically(case_file, &app.case)?;
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "ok": true,
                "saved": true,
                "case": case_path,
                "input": input_path,
                "entities": app.case.entities.len(),
                "relationships": app.case.links.len(),
                "message": app.status,
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", app.status);
    }
    Ok(())
}

fn read_sqlite_schema(database_path: &str) -> Result<Vec<serde_json::Value>, String> {
    let connection = Connection::open_with_flags(database_path, OpenFlags::SQLITE_OPEN_READ_ONLY)
        .map_err(|error| format!("cannot open SQLite database read-only: {error}"))?;
    connection
        .busy_timeout(Duration::from_secs(5))
        .map_err(|error| error.to_string())?;
    let objects = {
        let mut statement = connection
            .prepare(
                "SELECT type, name FROM sqlite_schema \
                 WHERE type IN ('table', 'view') AND substr(name, 1, 7) <> 'sqlite_' \
                 ORDER BY type, name",
            )
            .map_err(|error| error.to_string())?;
        statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?
    };
    let mut schema = Vec::with_capacity(objects.len());
    for (object_type, name) in objects {
        let columns = if object_type == "view" {
            let query = format!("SELECT * FROM {} LIMIT 0", quote_sql_identifier(&name));
            let statement = connection
                .prepare(&query)
                .map_err(|error| format!("cannot inspect SQLite view {name}: {error}"))?;
            statement
                .column_names()
                .map(|column| {
                    serde_json::json!({
                        "name": column,
                        "type": "",
                        "not_null": false,
                        "primary_key_order": 0,
                    })
                })
                .collect::<Vec<_>>()
        } else {
            let pragma = format!("PRAGMA table_info({})", quote_sql_identifier(&name));
            let mut statement = connection
                .prepare(&pragma)
                .map_err(|error| error.to_string())?;
            statement
                .query_map([], |row| {
                    Ok(serde_json::json!({
                        "name": row.get::<_, String>(1)?,
                        "type": row.get::<_, String>(2)?,
                        "not_null": row.get::<_, bool>(3)?,
                        "primary_key_order": row.get::<_, i64>(5)?,
                    }))
                })
                .map_err(|error| error.to_string())?
                .collect::<rusqlite::Result<Vec<_>>>()
                .map_err(|error| error.to_string())?
        };
        let foreign_key_pragma =
            format!("PRAGMA foreign_key_list({})", quote_sql_identifier(&name));
        let mut statement = connection
            .prepare(&foreign_key_pragma)
            .map_err(|error| error.to_string())?;
        let foreign_keys = statement
            .query_map([], |row| {
                Ok(serde_json::json!({
                    "id": row.get::<_, i64>(0)?,
                    "sequence": row.get::<_, i64>(1)?,
                    "table": row.get::<_, String>(2)?,
                    "from": row.get::<_, String>(3)?,
                    "to": row.get::<_, Option<String>>(4)?,
                    "on_update": row.get::<_, String>(5)?,
                    "on_delete": row.get::<_, String>(6)?,
                    "match": row.get::<_, String>(7)?,
                }))
            })
            .map_err(|error| error.to_string())?
            .collect::<rusqlite::Result<Vec<_>>>()
            .map_err(|error| error.to_string())?;
        schema.push(serde_json::json!({
            "type": object_type,
            "name": name,
            "columns": columns,
            "foreign_keys": foreign_keys,
        }));
    }
    Ok(schema)
}

fn quote_sql_identifier(identifier: &str) -> String {
    format!("\"{}\"", identifier.replace('"', "\"\""))
}

fn schema_column<'a>(object: &'a serde_json::Value, aliases: &[&str]) -> Option<&'a str> {
    let columns = object["columns"].as_array()?;
    aliases.iter().find_map(|alias| {
        columns.iter().find_map(|column| {
            let name = column["name"].as_str()?;
            name.trim().eq_ignore_ascii_case(alias).then_some(name)
        })
    })
}

fn sqlite_inferred_endpoint(
    link_object: &serde_json::Value,
    schema: &[serde_json::Value],
    id_column: &str,
    join_alias: &str,
) -> Option<(String, String, Option<String>, String)> {
    let link_name = link_object["name"].as_str()?;
    let mut candidates = Vec::new();
    for object in schema {
        let Some(table_name) = object["name"].as_str() else {
            continue;
        };
        if table_name.eq_ignore_ascii_case(link_name) {
            continue;
        }
        let Some(columns) = object["columns"].as_array() else {
            continue;
        };
        let primary_keys: Vec<&str> = columns
            .iter()
            .filter(|column| column["primary_key_order"].as_i64().unwrap_or_default() > 0)
            .filter_map(|column| column["name"].as_str())
            .collect();
        if primary_keys.len() != 1 {
            continue;
        }
        let key_column = primary_keys[0];
        let Some(name_column) = schema_column(
            object,
            &[
                "name",
                "entity_name",
                "person_name",
                "organization_name",
                "display_name",
                "display",
                "full_name",
                "label",
                "title",
                "value",
            ],
        ) else {
            continue;
        };
        let normalized_table_name = table_name.trim().to_ascii_lowercase();
        let priority = match normalized_table_name.as_str() {
            "entity" | "entities" | "node" | "nodes" => 2,
            "person" | "persons" | "people" | "organization" | "organizations" => 1,
            _ => 0,
        };
        candidates.push((priority, table_name, key_column, name_column, object));
    }
    let best_priority = candidates.iter().map(|candidate| candidate.0).max()?;
    let mut best: Vec<_> = candidates
        .into_iter()
        .filter(|candidate| candidate.0 == best_priority)
        .collect();
    if best.len() != 1 {
        return None;
    }
    let (_, table_name, key_column, name_column, object) = best.pop()?;
    let link_alias = quote_sql_identifier("link_row");
    let entity_alias = quote_sql_identifier(join_alias);
    let join = format!(
        "LEFT JOIN {} AS {entity_alias} ON {entity_alias}.{} = {link_alias}.{}",
        quote_sql_identifier(table_name),
        quote_sql_identifier(key_column),
        quote_sql_identifier(id_column),
    );
    let expression = format!("{entity_alias}.{}", quote_sql_identifier(name_column));
    let type_expression = schema_column(object, &["entity_type", "type", "kind"])
        .map(|column| format!("{entity_alias}.{}", quote_sql_identifier(column)));
    let identity_expression = format!("{link_alias}.{}", quote_sql_identifier(id_column));
    Some((expression, join, type_expression, identity_expression))
}

fn sqlite_foreign_key_for_column<'a>(
    link_object: &'a serde_json::Value,
    column: &str,
) -> Option<&'a serde_json::Value> {
    sqlite_foreign_key_group_for_column(link_object, column)?
        .into_iter()
        .next()
}

fn sqlite_foreign_key_group_for_column<'a>(
    link_object: &'a serde_json::Value,
    column: &str,
) -> Option<Vec<&'a serde_json::Value>> {
    let foreign_keys = link_object["foreign_keys"].as_array()?;
    let mut matching_groups = foreign_keys
        .iter()
        .filter(|key| {
            key["from"]
                .as_str()
                .is_some_and(|name| name.eq_ignore_ascii_case(column))
        })
        .filter_map(|key| key["id"].as_i64())
        .collect::<Vec<_>>();
    matching_groups.sort_unstable();
    matching_groups.dedup();
    if matching_groups.len() != 1 {
        return None;
    }
    let foreign_key_id = matching_groups[0];
    let mut group = foreign_keys
        .iter()
        .filter(|key| key["id"].as_i64() == Some(foreign_key_id))
        .collect::<Vec<_>>();
    group.sort_by_key(|key| key["sequence"].as_i64().unwrap_or_default());
    (!group.is_empty()).then_some(group)
}

fn sqlite_declared_endpoint(
    link_object: &serde_json::Value,
    schema: &[serde_json::Value],
    link_column: &str,
    join_alias: &str,
) -> Option<(String, String, Option<String>, String)> {
    let foreign_key_group = sqlite_foreign_key_group_for_column(link_object, link_column)?;
    let foreign_key = *foreign_key_group.first()?;
    let target_table = foreign_key["table"].as_str()?;
    if foreign_key_group.iter().any(|key| {
        key["table"]
            .as_str()
            .is_none_or(|table| !table.eq_ignore_ascii_case(target_table))
    }) {
        return None;
    }
    let target_object = schema.iter().find(|object| {
        object["name"]
            .as_str()
            .is_some_and(|name| name.eq_ignore_ascii_case(target_table))
    })?;
    let declared_target_keys = foreign_key_group
        .iter()
        .map(|key| key["to"].as_str().filter(|column| !column.is_empty()))
        .collect::<Vec<_>>();
    let target_keys = if declared_target_keys.iter().all(Option::is_none) {
        let mut primary_keys = target_object["columns"]
            .as_array()?
            .iter()
            .filter(|column| column["primary_key_order"].as_i64().unwrap_or_default() > 0)
            .filter_map(|column| {
                Some((
                    column["primary_key_order"].as_i64()?,
                    column["name"].as_str()?,
                ))
            })
            .collect::<Vec<_>>();
        primary_keys.sort_by_key(|(order, _)| *order);
        primary_keys
            .into_iter()
            .map(|(_, name)| name)
            .collect::<Vec<_>>()
    } else if declared_target_keys.iter().all(Option::is_some) {
        declared_target_keys.into_iter().flatten().collect()
    } else {
        return None;
    };
    if target_keys.len() != foreign_key_group.len() {
        return None;
    }
    let name_column = schema_column(
        target_object,
        &[
            "name",
            "entity_name",
            "person_name",
            "organization_name",
            "display_name",
            "display",
            "full_name",
            "label",
            "title",
            "value",
        ],
    )?;
    let link_alias = quote_sql_identifier("link_row");
    let entity_alias = quote_sql_identifier(join_alias);
    let join_conditions = foreign_key_group
        .iter()
        .zip(&target_keys)
        .map(|(foreign_key, target_key)| {
            let source_key = foreign_key["from"].as_str()?;
            Some(format!(
                "{entity_alias}.{} = {link_alias}.{}",
                quote_sql_identifier(target_key),
                quote_sql_identifier(source_key),
            ))
        })
        .collect::<Option<Vec<_>>>()?;
    let join = format!(
        "LEFT JOIN {} AS {entity_alias} ON {}",
        quote_sql_identifier(target_table),
        join_conditions.join(" AND "),
    );
    let expression = format!("{entity_alias}.{}", quote_sql_identifier(name_column));
    let type_expression = schema_column(target_object, &["entity_type", "type", "kind"])
        .map(|column| format!("{entity_alias}.{}", quote_sql_identifier(column)));
    let identity_expression = if foreign_key_group.len() == 1 {
        format!("{link_alias}.{}", quote_sql_identifier(link_column))
    } else {
        let encoded = foreign_key_group
            .iter()
            .map(|key| {
                let column = key["from"].as_str()?;
                let value = format!("{link_alias}.{}", quote_sql_identifier(column));
                Some(format!(
                    "typeof({value}) || ':' || coalesce(hex(CAST({value} AS BLOB)), '')"
                ))
            })
            .collect::<Option<Vec<_>>>()?
            .join(" || '|' || ");
        format!("({encoded})")
    };
    Some((expression, join, type_expression, identity_expression))
}

fn sqlite_endpoint_expression(
    link_object: &serde_json::Value,
    schema: &[serde_json::Value],
    direct_aliases: &[&str],
    id_aliases: &[&str],
    join_alias: &str,
) -> Option<(String, Option<String>, Option<String>, Option<String>)> {
    let link_alias = quote_sql_identifier("link_row");
    if let Some(column) = schema_column(link_object, direct_aliases) {
        let direct_expression = format!("{link_alias}.{}", quote_sql_identifier(column));
        if sqlite_foreign_key_for_column(link_object, column).is_some() {
            if let Some((expression, join, type_expression, identity_expression)) =
                sqlite_declared_endpoint(link_object, schema, column, join_alias)
            {
                return Some((
                    expression,
                    Some(join),
                    type_expression,
                    Some(identity_expression),
                ));
            }
            return Some((
                direct_expression.clone(),
                None,
                None,
                Some(direct_expression),
            ));
        }
        let identity_expression = schema_column(link_object, id_aliases)
            .filter(|id| {
                !id.eq_ignore_ascii_case("source_id")
                    || sqlite_foreign_key_for_column(link_object, id).is_some()
            })
            .map(|id| format!("{link_alias}.{}", quote_sql_identifier(id)));
        return Some((direct_expression, None, None, identity_expression));
    }
    let id_column = schema_column(link_object, id_aliases)?;
    let raw_expression = format!("{link_alias}.{}", quote_sql_identifier(id_column));
    let raw_endpoint = || {
        Some((
            raw_expression.clone(),
            None,
            None,
            Some(raw_expression.clone()),
        ))
    };
    if sqlite_foreign_key_for_column(link_object, id_column).is_some() {
        if let Some((expression, join, type_expression, identity_expression)) =
            sqlite_declared_endpoint(link_object, schema, id_column, join_alias)
        {
            return Some((
                expression,
                Some(join),
                type_expression,
                Some(identity_expression),
            ));
        }
        return raw_endpoint();
    }
    let foreign_keys = link_object["foreign_keys"].as_array();
    if foreign_keys.is_none_or(|keys| {
        keys.iter().all(|key| {
            key["from"]
                .as_str()
                .is_none_or(|column| !column.eq_ignore_ascii_case(id_column))
        })
    }) && let Some((expression, join, type_expression, identity_expression)) =
        sqlite_inferred_endpoint(link_object, schema, id_column, join_alias)
    {
        return Some((
            expression,
            Some(join),
            type_expression,
            Some(identity_expression),
        ));
    }
    raw_endpoint()
}

fn sqlite_query_templates(schema: &[serde_json::Value]) -> Vec<(String, String)> {
    let source_aliases = [
        "source",
        "from",
        "entity_a",
        "source_name",
        "from_name",
        "entity_a_name",
        "source_entity_name",
        "from_entity_name",
        "source_person_name",
        "from_person_name",
        "origin",
    ];
    let source_id_aliases = [
        "source_entity_id",
        "source_external_id",
        "from_entity_id",
        "from_id",
        "entity_a_id",
        "source_person_id",
        "from_person_id",
        "source_node_id",
        "from_node_id",
        "source_id",
    ];
    let target_aliases = [
        "target",
        "to",
        "entity_b",
        "target_name",
        "to_name",
        "entity_b_name",
        "target_entity_name",
        "to_entity_name",
        "target_person_name",
        "to_person_name",
        "destination",
    ];
    let target_id_aliases = [
        "target_entity_id",
        "target_external_id",
        "to_entity_id",
        "to_id",
        "entity_b_id",
        "target_id",
        "target_person_id",
        "to_person_id",
        "target_node_id",
        "to_node_id",
    ];
    let fields: [(&str, &[&str]); 7] = [
        (
            "connection_id",
            &[
                "connection_id",
                "link_id",
                "edge_id",
                "relationship_id",
                "record_id",
                "id",
            ],
        ),
        ("relationship", &["relationship", "link_type", "relation"]),
        ("date", &["date", "event_date", "datetime", "timestamp"]),
        ("confidence", &["confidence", "strength"]),
        ("directed", &["directed", "is_directed", "direction"]),
        (
            "source_reference",
            &[
                "source_reference",
                "evidence",
                "reference",
                "source_document_id",
                "evidence_id",
                "source_id",
            ],
        ),
        ("notes", &["notes", "description"]),
    ];
    let mut templates = Vec::new();
    for object in schema {
        let Some(object_name) = object["name"].as_str() else {
            continue;
        };
        let Some((source_expression, source_join, source_type_expression, source_identity)) =
            sqlite_endpoint_expression(
                object,
                schema,
                &source_aliases,
                &source_id_aliases,
                "source_entity",
            )
        else {
            continue;
        };
        let Some((target_expression, target_join, target_type_expression, target_identity)) =
            sqlite_endpoint_expression(
                object,
                schema,
                &target_aliases,
                &target_id_aliases,
                "target_entity",
            )
        else {
            continue;
        };
        let source_id_is_identity = schema_column(object, &["source_id"]).is_some_and(|column| {
            source_identity.as_deref()
                == Some(format!("\"link_row\".{}", quote_sql_identifier(column)).as_str())
        });
        let mut selected = vec![
            format!("{source_expression} AS \"source\""),
            format!("{target_expression} AS \"target\""),
        ];
        if let Some(identity) = source_identity.as_ref() {
            selected.push(format!("{identity} AS \"source_entity_id\""));
        }
        if let Some(identity) = target_identity {
            selected.push(format!("{identity} AS \"target_entity_id\""));
        }
        for (canonical, aliases) in fields {
            if let Some(column) = schema_column(object, aliases) {
                if canonical == "source_reference"
                    && column.eq_ignore_ascii_case("source_id")
                    && source_id_is_identity
                {
                    continue;
                }
                selected.push(format!(
                    "\"link_row\".{} AS \"{canonical}\"",
                    quote_sql_identifier(column)
                ));
            }
        }
        for (canonical, aliases, joined_expression) in [
            (
                "source_type",
                &["source_type", "from_type", "entity_a_type"][..],
                source_type_expression,
            ),
            (
                "target_type",
                &["target_type", "to_type", "entity_b_type"][..],
                target_type_expression,
            ),
        ] {
            let expression = schema_column(object, aliases)
                .map(|column| format!("\"link_row\".{}", quote_sql_identifier(column)))
                .or(joined_expression);
            if let Some(expression) = expression {
                selected.push(format!("{expression} AS \"{canonical}\""));
            }
        }
        let mut joins = Vec::new();
        if let Some(join) = source_join {
            joins.push(join);
        }
        if let Some(join) = target_join {
            if !joins.contains(&join) {
                joins.push(join);
            }
        }
        let mut query = format!(
            "SELECT {}\nFROM {} AS \"link_row\"",
            selected.join(",\n       "),
            quote_sql_identifier(object_name)
        );
        for join in joins {
            query.push('\n');
            query.push_str(&join);
        }
        query.push(';');
        templates.push((object_name.to_owned(), query));
    }
    templates
}

fn inspect_sqlite_schema_cli(database_path: &str, json_output: bool) -> Result<(), String> {
    let schema = read_sqlite_schema(database_path)?;
    let query_templates = sqlite_query_templates(&schema);
    let query_template = query_templates.first();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "ok": true,
                "database": database_path,
                "object_count": schema.len(),
                "objects": schema,
                "query_template": query_template.map(|(table, sql)| {
                    serde_json::json!({ "table": table, "sql": sql })
                }),
                "query_templates": query_templates
                    .iter()
                    .map(|(table, sql)| serde_json::json!({ "table": table, "sql": sql }))
                    .collect::<Vec<_>>(),
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        if schema.is_empty() {
            println!("No user tables or views found.");
        } else {
            for object in &schema {
                let object_type = object["type"].as_str().unwrap_or("object");
                let name = object["name"].as_str().unwrap_or("");
                println!("{object_type} {name}");
                for column in object["columns"].as_array().into_iter().flatten() {
                    println!(
                        "  {} · {}{}",
                        column["name"].as_str().unwrap_or(""),
                        column["type"].as_str().unwrap_or(""),
                        if column["primary_key_order"].as_i64().unwrap_or_default() > 0 {
                            " · primary key"
                        } else if column["not_null"].as_bool().unwrap_or(false) {
                            " · not null"
                        } else {
                            ""
                        }
                    );
                }
                for foreign_key in object["foreign_keys"].as_array().into_iter().flatten() {
                    println!(
                        "  foreign key {} → {}.{}",
                        foreign_key["from"].as_str().unwrap_or(""),
                        foreign_key["table"].as_str().unwrap_or(""),
                        foreign_key["to"].as_str().unwrap_or("primary key")
                    );
                }
            }
        }
        match query_template {
            Some((table, sql)) => println!("Suggested query from {table}:\n{sql}"),
            None if !schema.is_empty() => {
                println!("No table or view has recognizable source and target columns.");
            }
            None => {}
        }
    }
    Ok(())
}

fn export_sql_comparison_cli(
    case_path: &str,
    baseline_id: Uuid,
    current_id: Uuid,
    output_path: &str,
    json_output: bool,
) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let baseline_index = case
        .snapshots
        .iter()
        .position(|snapshot| snapshot.id == baseline_id)
        .ok_or_else(|| format!("baseline snapshot not found: {baseline_id}"))?;
    let current_index = case
        .snapshots
        .iter()
        .position(|snapshot| snapshot.id == current_id)
        .ok_or_else(|| format!("current snapshot not found: {current_id}"))?;
    let baseline = case.snapshots[baseline_index].clone();
    let current = case.snapshots[current_index].clone();
    if baseline_index >= current_index {
        return Err("baseline snapshot must precede the current snapshot".into());
    }
    if baseline.source_key != current.source_key {
        return Err("snapshots must belong to the same database and series".into());
    }
    let change_count = compare_snapshots(&baseline, &current).len();

    let mut app = App::automation_context(case, case_path);
    app.export_path = output_path.to_owned();
    app.export_sql_comparison(&baseline, &current);
    if let Some(error) = app.status.strip_prefix("SQL COMPARISON EXPORT FAILED · ") {
        return Err(error.to_owned());
    }
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "ok": true,
                "case": case_path,
                "output": output_path,
                "format": delimited_text_format(output_path),
                "baseline_snapshot_id": baseline_id,
                "current_snapshot_id": current_id,
                "changes": change_count,
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!("{}", app.status);
    }
    Ok(())
}

fn lock_automation_case(path: &Path) -> Result<fs::File, String> {
    if path.as_os_str().is_empty() {
        return Err("case path cannot be empty".into());
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("invalid case path: {}", path.display()))?;
    // Resolve path aliases first so every scheduler targeting the same case
    // file derives the same sibling lock file.
    let case_path = fs::canonicalize(path)
        .or_else(|_| {
            let parent = path
                .parent()
                .filter(|parent| !parent.as_os_str().is_empty())
                .unwrap_or_else(|| Path::new("."));
            fs::canonicalize(parent).map(|parent| parent.join(file_name))
        })
        .map_err(|error| {
            format!(
                "cannot resolve case path {} for automation lock: {error}",
                path.display()
            )
        })?;
    let file_name = case_path
        .file_name()
        .ok_or_else(|| format!("invalid case path: {}", case_path.display()))?;
    let mut lock_name = file_name.to_os_string();
    lock_name.push(".automation.lock");
    let mut lock_path = case_path;
    lock_path.set_file_name(lock_name);
    let lock_file = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .create(true)
        .truncate(false)
        .open(&lock_path)
        .map_err(|error| {
            format!(
                "cannot open automation lock {}: {error}",
                lock_path.display()
            )
        })?;
    lock_file.lock().map_err(|error| {
        format!(
            "cannot acquire automation lock {}: {error}",
            lock_path.display()
        )
    })?;
    Ok(lock_file)
}

fn validate_enabled_automation_jobs_cli(
    case_path: &str,
    selected_jobs: Option<&[String]>,
    parameter_overrides: &[(String, String)],
    scoped_parameter_overrides: &[(String, String, String)],
    queries_only: bool,
    json_output: bool,
) -> Result<bool, String> {
    // Match the lock used by automation runs and saves so preflight cannot read
    // a case while another process is checkpointing a job result.
    let _case_lock = lock_automation_case(Path::new(case_path))?;
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let mut jobs = if let Some(selected_jobs) = selected_jobs {
        select_enabled_automation_jobs(&case, selected_jobs)?
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
    apply_sql_parameter_overrides(&mut jobs, parameter_overrides, scoped_parameter_overrides)?;

    let mut app = App::automation_context(case, case_path);
    normalize_snapshot_database_paths(&mut app.case, &app.case_path);
    let mut failures = 0;
    let mut outcomes = Vec::new();
    for job in jobs {
        if let Some(existing) = app.case.automation_jobs.iter().find(|existing| {
            existing.id != job.id
                && same_sql_series(
                    &existing.database_path,
                    &existing.series,
                    &job.database_path,
                    &job.series,
                    &app.case_path,
                )
        }) {
            let message = format!(
                "SQL QUERY INVALID · database and series already belong to {}",
                existing.name
            );
            failures += 1;
            if json_output {
                outcomes.push(automation_outcome_json(
                    &job, false, &message, None, None, None,
                ));
            } else {
                println!("FAIL · {} · {}", job.name, message);
            }
            continue;
        }
        app.sql_path = job.database_path.clone();
        app.sql_series = job.series.clone();
        app.sql_query = job.query.clone();
        app.sql_parameter_text = format_sql_parameters(&job.parameters);
        app.automation_minimum_connections = job.minimum_connections;
        app.automation_maximum_connections = job.maximum_connections;
        app.automation_maximum_regressions = job.maximum_regressions;
        app.automation_maximum_skipped_rows = job.maximum_skipped_rows;
        let snapshot_count_before = app.case.snapshots.len();
        let limits_pass = app.validate_sql_query_for_batch();
        let snapshot = app.case.snapshots.get(snapshot_count_before);
        let query_valid = snapshot.is_some();
        let success = if queries_only {
            query_valid
        } else {
            limits_pass
        };
        let message = if queries_only && query_valid && !limits_pass {
            app.status.replacen(
                "SQL QUERY LIMITS FAILED",
                "SQL QUERY VALID · CURRENT CAPTURE LIMITS FAIL",
                1,
            )
        } else {
            app.status.clone()
        };
        let message = if queries_only
            && let Some(snapshot) = snapshot
            && snapshot.connections.is_empty()
        {
            format!(
                "{message} · {} · NO USABLE CONNECTIONS",
                if snapshot.skipped_rows == 0 {
                    "EMPTY QUERY RESULT"
                } else {
                    "ALL RETURNED ROWS SKIPPED"
                }
            )
        } else {
            message
        };
        let previous = snapshot
            .and_then(|_| previous_snapshot_index(&app.case, snapshot_count_before))
            .and_then(|index| app.case.snapshots.get(index));
        if !success {
            failures += 1;
        }
        if json_output {
            outcomes.push(automation_outcome_json(
                &job,
                limits_pass,
                &message,
                None,
                snapshot,
                previous,
            ));
        } else {
            println!(
                "{} · {} · {}",
                if success { "PASS" } else { "FAIL" },
                job.name,
                message
            );
        }
    }
    if json_output {
        let passed = outcomes
            .iter()
            .filter(|outcome| {
                if queries_only {
                    outcome["query_valid"] == true
                } else {
                    outcome["status"] == "PASS"
                }
            })
            .count();
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"schema_version":1,"mode":if queries_only { "query" } else { "capture_limits" },"ok":failures==0,"saved":false,"case":case_path,"passed":passed,"failed":failures,"outcomes":outcomes})).map_err(|error|error.to_string())?);
    }
    Ok(failures == 0)
}

fn list_automation_jobs_cli(
    case_path: &str,
    json_output: bool,
    enabled_only: bool,
) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let total = case.automation_jobs.len();
    let enabled = case
        .automation_jobs
        .iter()
        .filter(|job| job.enabled)
        .count();
    let jobs: Vec<serde_json::Value> = case
        .automation_jobs
        .iter()
        .filter(|job| !enabled_only || job.enabled)
        .map(|job| {
            let latest_run = case
                .automation_runs
                .iter()
                .rev()
                .find(|run| run.job_id == job.id)
                .map(|run| {
                    serde_json::json!({
                        "id": run.id,
                        "status": if run.success { "PASS" } else { "FAIL" },
                        "started_at": run.started_at,
                        "finished_at": run.finished_at,
                        "duration_ms": run.duration_ms,
                        "message": run.message,
                        "snapshot_id": run.snapshot_id,
                    })
                });
            serde_json::json!({
                "id": job.id,
                "name": job.name,
                "enabled": job.enabled,
                "database_path": job.database_path,
                "series": job.series,
                "query": job.query,
                "parameters": job.parameters,
                "minimum_connections": job.minimum_connections,
                "maximum_connections": job.maximum_connections,
                "maximum_regressions": job.maximum_regressions,
                "maximum_skipped_rows": job.maximum_skipped_rows,
                "latest_run": latest_run,
            })
        })
        .collect();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version": 1,
                "case": case_path,
                "filter": {"enabled_only": enabled_only},
                "total": total,
                "enabled": enabled,
                "returned": jobs.len(),
                "jobs": jobs
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        println!(
            "{}",
            serde_json::to_string_pretty(&jobs).map_err(|error| error.to_string())?
        );
    }
    Ok(())
}

fn list_automation_runs_cli(
    case_path: &str,
    json_output: bool,
    job_filter: Option<&str>,
    offset: usize,
    limit: Option<usize>,
) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let snapshots_by_id: HashMap<Uuid, &ConnectionSnapshot> = case
        .snapshots
        .iter()
        .map(|snapshot| (snapshot.id, snapshot))
        .collect();
    let matching_runs: Vec<&AutomationRun> = case
        .automation_runs
        .iter()
        .rev()
        .filter(|run| {
            job_filter.is_none_or(|filter| {
                run.job_name.eq_ignore_ascii_case(filter)
                    || run.job_id.to_string().eq_ignore_ascii_case(filter)
            })
        })
        .collect();
    let total = matching_runs.len();
    let selected_runs = matching_runs
        .into_iter()
        .skip(offset)
        .take(limit.unwrap_or(usize::MAX));
    let runs: Vec<serde_json::Value> = selected_runs
        .map(|run| {
            let snapshot_index = run
                .snapshot_id
                .and_then(|id| case.snapshots.iter().position(|snapshot| snapshot.id == id))
                .or_else(|| {
                    run.snapshot_index
                        .filter(|index| *index < case.snapshots.len())
                });
            let snapshot = snapshot_index
                .and_then(|index| case.snapshots.get(index))
                .or_else(|| {
                    run.snapshot_id
                        .and_then(|id| snapshots_by_id.get(&id).copied())
                });
            let previous = snapshot_index
                .and_then(|index| previous_snapshot_index(&case, index))
                .and_then(|index| case.snapshots.get(index));
            let job = AutomationJob {
                id: run.job_id,
                name: run.job_name.clone(),
                database_path: run.database_path.clone(),
                series: run.series.clone(),
                query: run.query.clone(),
                parameters: run.parameters.clone(),
                enabled: run.enabled,
                minimum_connections: run.minimum_connections,
                maximum_connections: run.maximum_connections,
                maximum_regressions: run.maximum_regressions,
                maximum_skipped_rows: run.maximum_skipped_rows,
            };
            let mut outcome = automation_outcome_json(
                &job,
                run.success,
                &run.message,
                Some(run.id),
                snapshot,
                previous,
            );
            outcome["id"] = serde_json::json!(run.id);
            outcome["started_at"] = serde_json::json!(run.started_at);
            outcome["finished_at"] = serde_json::json!(run.finished_at);
            outcome["duration_ms"] = serde_json::json!(run.duration_ms);
            outcome["enabled"] = serde_json::json!(run.enabled);
            outcome["configuration_recorded"] = serde_json::json!(run.configuration_recorded);
            outcome["rerun_of"] = serde_json::json!(run.rerun_of);
            if !run.configuration_recorded {
                outcome["limits"] = serde_json::Value::Null;
                outcome["capture_limits_pass"] = serde_json::Value::Null;
                outcome["parameters"] = serde_json::Value::Null;
                outcome["minimum_connections"] = serde_json::Value::Null;
                outcome["maximum_connections"] = serde_json::Value::Null;
                outcome["maximum_regressions"] = serde_json::Value::Null;
                outcome["maximum_skipped_rows"] = serde_json::Value::Null;
            }
            outcome
        })
        .collect();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "schema_version":1,
                "case":case_path,
                "filter":{"job":job_filter,"offset":offset,"limit":limit},
                "total":total,
                "returned":runs.len(),
                "next_offset":(offset.saturating_add(runs.len()) < total)
                        .then_some(offset.saturating_add(runs.len())),
                "runs":runs
            }))
            .map_err(|error| error.to_string())?
        );
    } else {
        for run in runs {
            println!(
                "{} · {} · {} · {}",
                run["status"].as_str().unwrap_or("UNKNOWN"),
                run["job"].as_str().unwrap_or("(unknown job)"),
                run["started_at"].as_str().unwrap_or("(unknown time)"),
                run["message"].as_str().unwrap_or_default()
            );
        }
    }
    Ok(())
}

fn report_cli_error(action: &str, error: String, json_output: bool) -> ! {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"schema_version":1,"ok":false,"error":error})
            )
            .unwrap_or_else(|_| {
                "{\"schema_version\":1,\"ok\":false,\"error\":\"failed to serialize CLI error\"}"
                    .into()
            })
        );
    }
    eprintln!("{action}: {error}");
    std::process::exit(1);
}

fn report_cli_usage_error(usage: &str, json_output: bool) -> ! {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(
                &serde_json::json!({"schema_version":1,"ok":false,"error":usage})
            )
            .unwrap_or_else(|_| {
                "{\"schema_version\":1,\"ok\":false,\"error\":\"invalid command arguments\"}".into()
            })
        );
    }
    eprintln!("{usage}");
    std::process::exit(2);
}

fn parse_batch_selection_args(
    args: &[String],
    usage: &str,
    allow_skip_recent_success: bool,
) -> (
    bool,
    Option<Vec<String>>,
    Vec<(String, String)>,
    Vec<(String, String, String)>,
    bool,
) {
    let json_output = args.iter().any(|arg| arg == "--json");
    let mut selectors = Vec::new();
    let mut parameter_options = Vec::new();
    let mut scoped_parameter_options = Vec::new();
    let mut seen_json = false;
    let mut skip_recent_success = false;
    let mut seen_skip_recent_success = false;
    let mut index = 2;
    while index < args.len() {
        match args[index].as_str() {
            "--json" if !seen_json => seen_json = true,
            "--job" => {
                index += 1;
                let Some(value) = args
                    .get(index)
                    .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                else {
                    report_cli_usage_error(
                        "--job requires an enabled job name or UUID",
                        json_output,
                    );
                };
                selectors.push(value.clone());
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
            "--job-param" => {
                index += 1;
                let Some(job_selector) = args
                    .get(index)
                    .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                else {
                    report_cli_usage_error("--job-param requires a job name or UUID", json_output);
                };
                index += 1;
                let Some(value) = args
                    .get(index)
                    .filter(|value| !value.trim().is_empty() && !value.starts_with("--"))
                else {
                    report_cli_usage_error("--job-param requires name=value", json_output);
                };
                scoped_parameter_options.push((job_selector.clone(), value.clone()));
            }
            "--skip-recent-success" if allow_skip_recent_success && !seen_skip_recent_success => {
                skip_recent_success = true;
                seen_skip_recent_success = true;
            }
            _ => report_cli_usage_error(usage, json_output),
        }
        index += 1;
    }
    let parameters = parse_sql_parameters(&parameter_options.join("\n"))
        .unwrap_or_else(|error| report_cli_usage_error(&error, json_output));
    let mut scoped_parameters = Vec::new();
    for (selector, option) in scoped_parameter_options {
        let parsed = parse_sql_parameters(&option)
            .unwrap_or_else(|error| report_cli_usage_error(&error, json_output));
        if parsed.len() != 1 {
            report_cli_usage_error("--job-param requires one name=value pair", json_output);
        }
        let (name, value) = parsed.into_iter().next().expect("one parsed parameter");
        scoped_parameters.push((selector, name, value));
    }
    (
        json_output,
        (!selectors.is_empty()).then_some(selectors),
        parameters,
        scoped_parameters,
        skip_recent_success,
    )
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!(
            "Arch Notebook\n\nUsage:\n  arch-notebook\n  arch-notebook --inspect-sqlite <database.sqlite> [--json]\n  arch-notebook --import-graphml <case.json> <input.graphml> [--json]\n  arch-notebook --export-graphml <case.json> <output.graphml> [--json]\n  arch-notebook --export-sql-comparison <case.json> <baseline-snapshot-id> <current-snapshot-id> <output.csv|tsv> [--json]\n  arch-notebook --list-jobs <case.json> [--enabled-only] [--json]\n  arch-notebook --list-runs <case.json> [--json] [--job <name-or-id>] [--offset <count>] [--limit <count>]\n  arch-notebook --validate-enabled-jobs <case.json> [--json] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]\n  arch-notebook --check-enabled-job-queries <case.json> [--json] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]\n  arch-notebook --validate-run <case.json> <run-id> [--json]\n  arch-notebook --run-enabled-jobs <case.json> [--json] [--skip-recent-success] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]\n  arch-notebook --run-job <case.json> <job-name-or-id> [--param name=value ...] [--json]\n  arch-notebook --rerun-run <case.json> <run-id> [--json]\n\nJob and run listing and validation do not change the case. Run history is newest first and can be filtered or paged. SQL comparisons can be exported from saved snapshots for downstream tools. Batch runs save snapshots and run history to the case file. JSON output is available for scheduler integration."
        );
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--inspect-sqlite") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let Some(database_path) = args.get(1).filter(|path| !path.starts_with("--")) else {
            report_cli_usage_error(
                "usage: arch-notebook --inspect-sqlite <database.sqlite> [--json]",
                json_output,
            );
        };
        if args.len() > 3 || args.get(2).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --inspect-sqlite <database.sqlite> [--json]",
                json_output,
            );
        }
        if let Err(error) = inspect_sqlite_schema_cli(database_path, json_output) {
            report_cli_error("SQLite schema inspection failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--export-graphml") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let (Some(case_path), Some(output_path)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(
                "usage: arch-notebook --export-graphml <case.json> <output.graphml> [--json]",
                json_output,
            );
        };
        if args.len() > 4 || args.get(3).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --export-graphml <case.json> <output.graphml> [--json]",
                json_output,
            );
        }
        if let Err(error) = export_graphml_cli(case_path, output_path, json_output) {
            report_cli_error("GraphML export failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--import-graphml") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let usage = "usage: arch-notebook --import-graphml <case.json> <input.graphml> [--json]";
        let (Some(case_path), Some(input_path)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 4 || args.get(3).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        if let Err(error) = import_graphml_cli(case_path, input_path, json_output) {
            report_cli_error("GraphML import failed", error, json_output);
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
        let usage = "usage: arch-notebook --list-jobs <case.json> [--enabled-only] [--json]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, json_output);
        };
        let mut enabled_only = false;
        let mut seen_json = false;
        let mut seen_enabled_only = false;
        for option in args.iter().skip(2) {
            match option.as_str() {
                "--json" if !seen_json => seen_json = true,
                "--enabled-only" if !seen_enabled_only => {
                    seen_enabled_only = true;
                    enabled_only = true;
                }
                _ => report_cli_usage_error(usage, json_output),
            }
        }
        if let Err(error) = list_automation_jobs_cli(case_path, json_output, enabled_only) {
            report_cli_error("job listing failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--list-runs") {
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(
                "usage: arch-notebook --list-runs <case.json> [--json] [--job <name-or-id>] [--offset <count>] [--limit <count>]",
                args.iter().any(|arg| arg == "--json"),
            );
        };
        let json_output = args.iter().any(|arg| arg == "--json");
        let mut job_filter = None;
        let mut offset = 0usize;
        let mut limit = None;
        let mut seen_json = false;
        let mut seen_job = false;
        let mut seen_offset = false;
        let mut seen_limit = false;
        let mut index = 2;
        while index < args.len() {
            match args[index].as_str() {
                "--json" if !seen_json => seen_json = true,
                "--job" if !seen_job => {
                    index += 1;
                    let Some(value) = args.get(index).filter(|value| !value.trim().is_empty())
                    else {
                        report_cli_usage_error("--job requires a job name or UUID", json_output);
                    };
                    job_filter = Some(value.clone());
                    seen_job = true;
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
                    "usage: arch-notebook --list-runs <case.json> [--json] [--job <name-or-id>] [--offset <count>] [--limit <count>]",
                    json_output,
                ),
            }
            index += 1;
        }
        if let Err(error) =
            list_automation_runs_cli(case_path, json_output, job_filter.as_deref(), offset, limit)
        {
            report_cli_error("run history listing failed", error, json_output);
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
        let (json_output, selected_jobs, parameter_overrides, scoped_parameter_overrides, _) =
            parse_batch_selection_args(&args, usage, false);
        match validate_enabled_automation_jobs_cli(
            case_path,
            selected_jobs.as_deref(),
            &parameter_overrides,
            &scoped_parameter_overrides,
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
        let usage = "usage: arch-notebook --check-enabled-job-queries <case.json> [--json] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, args.iter().any(|arg| arg == "--json"));
        };
        let (json_output, selected_jobs, parameter_overrides, scoped_parameter_overrides, _) =
            parse_batch_selection_args(&args, usage, false);
        match validate_enabled_automation_jobs_cli(
            case_path,
            selected_jobs.as_deref(),
            &parameter_overrides,
            &scoped_parameter_overrides,
            true,
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
        let usage = "usage: arch-notebook --run-enabled-jobs <case.json> [--json] [--skip-recent-success] [--job <name-or-id> ...] [--param name=value ...] [--job-param <name-or-id> <name=value> ...]";
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(usage, args.iter().any(|arg| arg == "--json"));
        };
        let (
            json_output,
            selected_jobs,
            parameter_overrides,
            scoped_parameter_overrides,
            skip_recent_success,
        ) = parse_batch_selection_args(&args, usage, true);
        match run_automation_cli(
            case_path,
            None,
            selected_jobs.as_deref(),
            &parameter_overrides,
            &scoped_parameter_overrides,
            skip_recent_success,
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
        let usage = "usage: arch-notebook --run-job <case.json> <job-name-or-id> [--param name=value ...] [--json]";
        let (Some(case_path), Some(job_name)) = (args.get(1), args.get(2)) else {
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
        match run_automation_cli(
            case_path,
            Some(job_name),
            None,
            &parameter_overrides,
            &[],
            false,
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
        let usage = "usage: arch-notebook --rerun-run <case.json> <run-id> [--json]";
        let (Some(case_path), Some(run_id)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(usage, json_output);
        };
        if args.len() > 4 || args.get(3).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(usage, json_output);
        }
        let run_id = Uuid::parse_str(run_id)
            .unwrap_or_else(|_| report_cli_usage_error("invalid run UUID", json_output));
        match rerun_saved_automation_configuration_cli(case_path, run_id, json_output) {
            Err(error) => report_cli_error("automation rerun failed", error, json_output),
            Ok(false) => {
                eprintln!("automation rerun failed; run history saved");
                std::process::exit(1);
            }
            Ok(true) => {}
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

#[cfg(test)]
mod tests {
    use super::{ConnectionFact, ConnectionSnapshot, canonical_connection, compare_snapshots};

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
            source_entity_id: String::new(),
            target: "Pier 19 Warehouse".into(),
            target_type: "LOCATION".into(),
            target_entity_id: String::new(),
            relationship: "LEASES".into(),
            date: "2025-01-28".into(),
            confidence: 2,
            directed: false,
            source_reference: "LOC-81".into(),
            notes: String::new(),
        };
        let make_snapshot = |key: String| ConnectionSnapshot {
            id: super::Uuid::new_v4(),
            series: "test".into(),
            source_key: "db::test".into(),
            database_path: "db".into(),
            configured_database_path: "db".into(),
            query: String::new(),
            parameters: Vec::new(),
            captured_at: String::new(),
            connections: vec![fact(key)],
            skipped_rows: 0,
        };
        let old = make_snapshot(
            "Northstar Imports|ORGANIZATION|LEASES|Pier 19 Warehouse|LOCATION|false#1".into(),
        );
        let current = make_snapshot(format!("natural-v2:{new_key}"));
        assert!(compare_snapshots(&old, &current).is_empty());
    }
}
