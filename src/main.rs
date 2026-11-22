use eframe::egui;
use quick_xml::{
    Reader as XmlReader, escape::resolve_predefined_entity, events::Event as XmlEvent,
};
use rusqlite::{Connection, OpenFlags, TransactionBehavior, types::ValueRef};
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
    env, fs,
    io::Write,
    path::{Path, PathBuf},
    time::Duration,
};
use uuid::Uuid;

const CURRENT_CASE_FORMAT_VERSION: u32 = 9;

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
    enabled: bool,
    #[serde(default)]
    minimum_connections: usize,
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
    minimum_connections: usize,
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
    success: bool,
    message: String,
    #[serde(default)]
    snapshot_index: Option<usize>,
    #[serde(default)]
    snapshot_id: Option<Uuid>,
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
    chart_fit_path: Option<Vec<Uuid>>,
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
    layout_mode: LayoutMode,
    link_sort: LinkSort,
    link_table_view: LinkTableView,
    data_query: String,
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
    sql_snapshot_source: Option<String>,
    sql_snapshot_index: Option<usize>,
    automation_job_name: String,
    automation_minimum_connections: usize,
    automation_maximum_regressions: Option<usize>,
    automation_maximum_skipped_rows: Option<usize>,
    editing_automation_job: Option<Uuid>,
    automation_editor_before_edit: Option<(
        String,
        String,
        String,
        String,
        usize,
        Option<usize>,
        Option<usize>,
    )>,
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
    EvidenceChanged,
    SourceNotesChanged,
    Added,
    ConfidenceIncreased,
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
            chart_fit_path: None,
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
            layout_mode: LayoutMode::Connectivity,
            link_sort: LinkSort::Date,
            link_table_view: LinkTableView::Connections,
            data_query: String::new(),
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
            sql_query: "SELECT connection_id, source, source_type, target, target_type, relationship, event_date AS date, confidence, directed, source_reference, notes FROM connection_observations WHERE snapshot_id = 'baseline' ORDER BY connection_id;".into(),
            sql_snapshot_source: None,
            sql_snapshot_index: None,
            automation_job_name: String::new(),
            automation_minimum_connections: 0,
            automation_maximum_regressions: None,
            automation_maximum_skipped_rows: None,
            editing_automation_job: None,
            automation_editor_before_edit: None,
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
            chart_fit_path: None,
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
            layout_mode: LayoutMode::Connectivity,
            link_sort: LinkSort::Date,
            link_table_view: LinkTableView::Connections,
            data_query: String::new(),
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
            sql_snapshot_source: None,
            sql_snapshot_index: None,
            automation_job_name: String::new(),
            automation_minimum_connections: 0,
            automation_maximum_regressions: None,
            automation_maximum_skipped_rows: None,
            editing_automation_job: None,
            automation_editor_before_edit: None,
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
                self.case_path = requested_path.clone();
                self.case_path_input = requested_path;
                self.disk_case_json = Some(disk_case_json);
                self.saved_case_json = self.current_case_json();
                self.sql_snapshot_source = None;
                self.sql_snapshot_index = None;
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
                self.chart_fit_path = None;
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
            Ok((created, updated, skipped, entities, links))
        })();
        self.status = match result {
            Ok((created, updated, skipped, entities, links)) => {
                self.case.entities = entities;
                self.case.links = links;
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
        let configured_path = self.sql_path.trim().to_string();
        let case_path = self.case_path.clone();
        let series = self.sql_series.trim().to_string();
        let query = self.sql_query.trim().to_string();
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
            let path = resolve_sqlite_database_path(&configured_path, &case_path)?;
            let path = path.to_string_lossy().into_owned();
            // Enforce read-only access at the SQLite open boundary for every configured job.
            let mut connection =
                Connection::open_with_flags(&path, OpenFlags::SQLITE_OPEN_READ_ONLY)
                    .map_err(|e| e.to_string())?;
            connection
                .busy_timeout(Duration::from_secs(5))
                .map_err(|error| error.to_string())?;
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
                let (source_name, source_kind, target_name, target_kind, natural_key) =
                    canonical_connection(
                        &source_name,
                        &source_kind,
                        &target_name,
                        &target_kind,
                        &relationship,
                        is_directed,
                    )?;
                let identity = if explicit_id.trim().is_empty() {
                    format!("natural-v2:{natural_key}")
                } else {
                    format!("id:{}", explicit_id.trim())
                };
                facts.push(ConnectionFact {
                    key: identity,
                    connection_id: explicit_id.trim().to_owned(),
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
                let from =
                    entity_index.find_or_add_entity(&fact.source, &fact.source_type, &mut entities);
                let to =
                    entity_index.find_or_add_entity(&fact.target, &fact.target_type, &mut entities);
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
                job.minimum_connections = self.automation_minimum_connections;
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
            enabled: true,
            minimum_connections: self.automation_minimum_connections,
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
                self.automation_maximum_regressions,
                self.automation_maximum_skipped_rows,
            );
            let validation_result = if limits_pass {
                "SQL QUERY VALID"
            } else {
                "SQL QUERY LIMITS FAILED"
            };
            format!(
                "{validation_result} · {count} CONNECTIONS{skipped}{skipped_row_threshold}{threshold}{regression_threshold} · CASE UNCHANGED"
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
        self.selected_link = selected_link_before;
        self.sql_snapshot_source = snapshot_source_before;
        self.sql_snapshot_index = snapshot_index_before;
        self.path_result = path_result_before;
        self.status = validation_message;
        self.status.starts_with("SQL QUERY VALID · ")
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
                self.automation_minimum_connections,
                self.automation_maximum_regressions,
                self.automation_maximum_skipped_rows,
            ));
            self.automation_job_name = job.name;
            self.sql_path = job.database_path;
            self.sql_series = job.series;
            self.sql_query = job.query;
            self.automation_minimum_connections = job.minimum_connections;
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
            minimum,
            maximum_regressions,
            maximum_skipped_rows,
        )) = self.automation_editor_before_edit.take()
        {
            self.automation_job_name = name;
            self.sql_path = path;
            self.sql_series = series;
            self.sql_query = query;
            self.automation_minimum_connections = minimum;
            self.automation_maximum_regressions = maximum_regressions;
            self.automation_maximum_skipped_rows = maximum_skipped_rows;
        }
        self.status = "JOB EDIT CANCELED".into();
    }

    fn run_automation_job(&mut self, id: Uuid) -> bool {
        let Some(job) = self
            .case
            .automation_jobs
            .iter()
            .find(|job| job.id == id)
            .cloned()
        else {
            self.status = "JOB RUN FAILED · job not found".into();
            return false;
        };
        let editor_state = (
            self.sql_path.clone(),
            self.sql_series.clone(),
            self.sql_query.clone(),
        );
        let (success, message) = self.execute_automation_job(&job);
        (self.sql_path, self.sql_series, self.sql_query) = editor_state;
        self.status = message;
        success
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
        let editor_state = (
            self.sql_path.clone(),
            self.sql_series.clone(),
            self.sql_query.clone(),
            self.automation_minimum_connections,
            self.automation_maximum_regressions,
            self.automation_maximum_skipped_rows,
            self.editing_automation_job,
        );
        self.sql_path = job.database_path;
        self.sql_series = job.series;
        self.sql_query = job.query;
        self.automation_minimum_connections = job.minimum_connections;
        self.automation_maximum_regressions = job.maximum_regressions;
        self.automation_maximum_skipped_rows = job.maximum_skipped_rows;
        self.editing_automation_job = Some(id);
        let valid = self.validate_sql_query();
        (
            self.sql_path,
            self.sql_series,
            self.sql_query,
            self.automation_minimum_connections,
            self.automation_maximum_regressions,
            self.automation_maximum_skipped_rows,
            self.editing_automation_job,
        ) = editor_state;
        valid
    }

    fn run_automation_job_cli(&mut self, job: &AutomationJob) -> (bool, String) {
        self.execute_automation_job(job)
    }

    fn execute_automation_job(&mut self, job: &AutomationJob) -> (bool, String) {
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
            self.record_automation_run(job, started_at, false, message.clone(), None);
            return (false, message);
        }
        let snapshot_count_before = self.case.snapshots.len();
        self.sql_path = job.database_path.clone();
        self.sql_series = job.series.clone();
        self.sql_query = job.query.clone();
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
                job.maximum_regressions,
                job.maximum_skipped_rows,
            );
        }
        let message = self.status.clone();
        self.record_automation_run(job, started_at, success, message.clone(), snapshot_index);
        (success, message)
    }

    fn record_automation_run(
        &mut self,
        job: &AutomationJob,
        started_at: String,
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
            minimum_connections: job.minimum_connections,
            maximum_regressions: job.maximum_regressions,
            maximum_skipped_rows: job.maximum_skipped_rows,
            enabled: job.enabled,
            configuration_recorded: true,
            started_at,
            finished_at: epoch_timestamp(),
            success,
            message,
            snapshot_index,
            snapshot_id,
        });
        if self.case.automation_runs.len() > 200 {
            self.case
                .automation_runs
                .drain(..self.case.automation_runs.len() - 200);
        }
    }

    fn save_automation_results(&mut self, summary: &str) {
        let path = Path::new(self.case_path.trim());
        let result = if path.as_os_str().is_empty() {
            Err("case path cannot be empty".to_string())
        } else {
            save_case_atomically_if_unchanged(path, &self.case, self.disk_case_json.as_deref())
        };
        self.status = match result {
            Ok(disk_case_json) => {
                self.disk_case_json = Some(disk_case_json);
                self.saved_case_json = self.current_case_json();
                format!("{summary} · CASE SAVED")
            }
            Err(error) => format!("{summary} · CASE SAVE FAILED · {error}"),
        };
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
        let mut passed = 0;
        let editor_state = (
            self.sql_path.clone(),
            self.sql_series.clone(),
            self.sql_query.clone(),
        );
        for id in jobs {
            if self.run_automation_job(id) {
                passed += 1;
            }
            self.save_automation_results("AUTOMATION JOB CHECKPOINT");
        }
        (self.sql_path, self.sql_series, self.sql_query) = editor_state;
        let failed = count - passed;
        let summary =
            format!("AUTOMATION RUN COMPLETE · {passed} PASSED · {failed} FAILED · {count} JOBS");
        self.save_automation_results(&summary);
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
                "source_type".to_string(),
                "source_detail_json".to_string(),
                "source_aliases_json".to_string(),
                "source_color".to_string(),
                "source_x".to_string(),
                "source_y".to_string(),
                "relationship".to_string(),
                "target".to_string(),
                "target_entity_id".to_string(),
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
                        a.kind.as_str(),
                        source_detail.as_str(),
                        source_aliases.as_str(),
                        source_color.as_str(),
                        source_x.as_str(),
                        source_y.as_str(),
                        link.label.as_str(),
                        b.name.as_str(),
                        target_entity_id.as_str(),
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
                    "evidence_changed",
                    "source_notes_changed",
                    "added",
                    "confidence_increased",
                    "connection_id",
                    "source_before",
                    "source_type_before",
                    "relationship_before",
                    "target_before",
                    "target_type_before",
                    "directed_before",
                    "confidence_before",
                    "date_before",
                    "source_reference_before",
                    "source_notes_before",
                    "source",
                    "source_type",
                    "relationship",
                    "target",
                    "target_type",
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
                        DeltaKind::EvidenceChanged,
                        DeltaKind::SourceNotesChanged,
                        DeltaKind::Added,
                        DeltaKind::ConfidenceIncreased,
                    ]
                    .iter()
                    .map(|kind| delta.kinds.contains(kind).to_string()),
                );
                record.push(delta.connection.connection_id.clone());
                let append_fact = |record: &mut Vec<String>, fact: Option<&ConnectionFact>| {
                    record.extend([
                        fact.map(|value| value.source.clone()).unwrap_or_default(),
                        fact.map(|value| value.source_type.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.relationship.clone())
                            .unwrap_or_default(),
                        fact.map(|value| value.target.clone()).unwrap_or_default(),
                        fact.map(|value| value.target_type.clone())
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
        self.chart_fit_path = None;
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
            self.selected_link = None;
            self.path_result.clear();
            self.status = "ENTITY AND ITS RELATIONSHIPS REMOVED".into();
        } else if let Some(i) = self.selected_link.take() {
            if i < self.case.links.len() {
                self.case.links.remove(i);
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
        self.sql_snapshot_source = None;
        self.sql_snapshot_index = None;
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
        self.chart_fit_path = None;
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

        let id = Uuid::new_v4();
        let position = entities.len();
        let entity = Entity {
            id,
            name: name.into(),
            kind: if is_generic(kind) { "ENTITY" } else { kind }.into(),
            detail: String::new(),
            attributes: Vec::new(),
            aliases: Vec::new(),
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
    let mut neighbors: HashMap<Uuid, HashSet<Uuid>> = HashMap::new();
    for link in &case.links {
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
                    || now.relationship != previous.relationship
                    || now.directed != previous.directed
                {
                    kinds.push(DeltaKind::Updated);
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

fn comparison_key(connection: &ConnectionFact) -> String {
    if !connection.connection_id.trim().is_empty() {
        return format!("id:{}", connection.connection_id.trim());
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
    maximum_regressions: Option<usize>,
    maximum_skipped_rows: Option<usize>,
) -> bool {
    connections >= minimum_connections
        && maximum_regressions.is_none_or(|maximum| regressions <= maximum)
        && maximum_skipped_rows.is_none_or(|maximum| skipped_rows <= maximum)
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
    let evidence = count(DeltaKind::EvidenceChanged);
    let source_notes = count(DeltaKind::SourceNotesChanged);
    if deltas.is_empty() {
        "NO CONNECTION CHANGES".into()
    } else {
        format!(
            "SNAPSHOT DELTA · {removed} removed · {declined} confidence declines · {dates} date changes · {details} connection detail changes · {evidence} evidence changes · {source_notes} source note changes · {added} added · {increased} confidence increases"
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
                self.chart_fit_path = None;
            }
        }

        let connected_counts = connected_entity_counts(&self.case);
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
                        let item=Entity { id:Uuid::new_v4(),name:"New entity".into(),kind:"PERSON".into(),detail:"Add an analyst note".into(),attributes:Vec::new(),aliases:Vec::new(),merged_records:Vec::new(),x:0.5,y:0.5,position_locked:false,color:[55,128,162] };
                        self.selected=Some(item.id); self.selected_link=None; self.case.entities.push(item);self.duplicate_candidates.clear();
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
                    if self.page==WorkspacePage::Chart {if ui.add_enabled(!self.case.entities.is_empty(),egui::Button::new("Fit all")).clicked(){self.chart_fit_path=Some(self.case.entities.iter().map(|entity|entity.id).collect());self.chart_focus=None;self.chart_focus_link=None;}if ui.small_button("−").clicked(){self.chart_zoom=(self.chart_zoom/1.2).clamp(0.25,3.0);}ui.label(format!("{:.0}%",self.chart_zoom*100.0));if ui.small_button("＋").clicked(){self.chart_zoom=(self.chart_zoom*1.2).clamp(0.25,3.0);}if ui.button("Reset view").clicked(){self.chart_zoom=1.0;self.chart_pan=egui::Vec2::ZERO;self.chart_focus=None;self.chart_focus_link=None;self.chart_fit_path=None;}}
                    if self.page==WorkspacePage::Data && (self.selected.is_some() || self.selected_link.is_some()) && ui.button("View on chart").clicked(){self.chart_focus=self.selected;self.chart_focus_link=self.selected_link;self.page=WorkspacePage::Chart;self.show_timeline=false;}
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
                                ui.horizontal(|ui|{let mut limit_regressions=self.automation_maximum_regressions.is_some();if ui.checkbox(&mut limit_regressions,"Limit regressions").changed(){self.automation_maximum_regressions=limit_regressions.then_some(0);}if let Some(maximum)=self.automation_maximum_regressions.as_mut(){ui.label("Maximum removals/confidence declines");ui.add(egui::DragValue::new(maximum).range(0..=1_000_000));}if ui.button("Validate query").clicked(){self.validate_sql_query();}if ui.button(if self.editing_automation_job.is_some(){"Save job changes"}else{"Add job"}).clicked(){self.save_automation_job();}if self.editing_automation_job.is_some()&&ui.button("Cancel edit").clicked(){self.cancel_automation_job_edit();}});
                                ui.label(egui::RichText::new("A regression is a removed connection or confidence decline. Runs over either configured limit are saved as failed, with their snapshots retained.").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_query).desired_width(canvas_w-20.0).desired_rows(5).code_editor());
                                if ui.button("Run enabled jobs and save").clicked(){self.run_enabled_automation_jobs();}
                                ui.add_space(8.0);ui.label(egui::RichText::new("JOBS").small().strong().color(cyan));
                                let mut action: Option<(String,Uuid)> = None;
                                for job in &mut self.case.automation_jobs {
                                    ui.horizontal(|ui| {
                                        ui.checkbox(&mut job.enabled, "");
                                        ui.label(format!("{} · {} · {} · min {} · max regressions {} · max skipped rows {}",job.name,job.series,job.database_path,job.minimum_connections,job.maximum_regressions.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),job.maximum_skipped_rows.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into())));
                                        if ui.small_button("Validate").clicked(){action=Some(("validate".into(),job.id));}
                                        if ui.small_button("Edit").clicked(){action=Some(("edit".into(),job.id));}
                                        if ui.small_button("Run and save").clicked(){action=Some(("run".into(),job.id));}
                                        if ui.small_button("Delete").clicked(){action=Some(("delete".into(),job.id));}
                                    });
                                    egui::CollapsingHeader::new("SQL query and capture configuration")
                                        .id_salt(("automation_job_query", job.id))
                                        .show(ui, |ui| {
                                            ui.label(format!("SQLite {} · series {}", job.database_path, job.series));
                                            let mut query = job.query.clone();
                                            ui.add(
                                                egui::TextEdit::multiline(&mut query)
                                                    .desired_width(canvas_w - 30.0)
                                                    .desired_rows(3)
                                                    .code_editor()
                                                    .interactive(false),
                                            );
                                        });
                                }
                                if let Some((verb,id))=action {match verb.as_str(){"validate"=>{self.validate_automation_job(id);},"edit"=>self.edit_automation_job(id),"run"=>{let job_name=self.case.automation_jobs.iter().find(|job|job.id==id).map(|job|job.name.clone()).unwrap_or_else(||"(missing job)".into());let passed=self.run_automation_job(id);let summary=format!("JOB RUN {} · {job_name}",if passed{"PASSED"}else{"FAILED"});self.save_automation_results(&summary);},"delete"=>{if self.editing_automation_job==Some(id){self.cancel_automation_job_edit();}self.case.automation_jobs.retain(|job|job.id!=id);self.status="AUTOMATION JOB DELETED".into();},_=>{}}}
                                ui.add_space(8.0);ui.label(egui::RichText::new("RUN HISTORY · newest first").small().strong().color(cyan));
                                let mut review_snapshot = None;
                                egui::ScrollArea::vertical().max_height(210.0).show(ui, |ui| {
                                    for run in self.case.automation_runs.iter().rev() {
                                        let result = if run.success { "PASS" } else { "FAIL" };
                                        egui::CollapsingHeader::new(format!(
                                            "{result} · {} · {}",
                                            run.job_name, run.started_at
                                        ))
                                        .id_salt(("automation_run", run.id))
                                        .show(ui, |ui| {
                                            ui.label(format!("Finished {}", run.finished_at));
                                            ui.label(&run.message);
                                            if run.configuration_recorded {
                                                ui.label(format!(
                                                    "Minimum {} connections · maximum regressions {} · maximum skipped rows {} · job {}",
                                                    run.minimum_connections,
                                                    run.maximum_regressions.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),
                                                    run.maximum_skipped_rows.map(|value|value.to_string()).unwrap_or_else(||"unlimited".into()),
                                                    if run.enabled { "enabled" } else { "disabled" }
                                                ));
                                                ui.label(format!(
                                                    "SQLite {} · series {}",
                                                    run.database_path, run.series
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
                                            let snapshot_index = run
                                                .snapshot_id
                                                .and_then(|id| {
                                                    self.case.snapshots.iter().position(|snapshot| snapshot.id == id)
                                                })
                                                .or_else(|| {
                                                    run.snapshot_index.filter(|index| *index < self.case.snapshots.len())
                                                });
                                            if let Some(index) = snapshot_index {
                                                if ui.small_button(format!("Review snapshot #{}", index + 1)).clicked() {
                                                    review_snapshot = Some(index);
                                                }
                                            }
                                        });
                                    }
                                });
                                if let Some(index)=review_snapshot {self.sql_snapshot_source=Some(self.case.snapshots[index].source_key.clone());self.sql_snapshot_index=Some(index);self.data_mode=DataMode::SqlRegressions;}
                                return;
                            }
                            if self.data_mode==DataMode::SqlRegressions {
                                let mut sources: Vec<(String, String)> = Vec::new();
                                for snapshot in self.case.snapshots.iter().rev() {
                                    if sources.iter().all(|(key, _)| key != &snapshot.source_key) {
                                        sources.push((
                                            snapshot.source_key.clone(),
                                            format!("{} · {}", snapshot.series, snapshot.database_path),
                                        ));
                                    }
                                }
                                if self.sql_snapshot_source.as_ref().is_none_or(|selected| {
                                    sources.iter().all(|(key, _)| key != selected)
                                }) {
                                    self.sql_snapshot_source = sources.first().map(|(key, _)| key.clone());
                                }
                                if !sources.is_empty() {
                                    ui.horizontal(|ui| {
                                        ui.label("Review saved series");
                                        egui::ComboBox::from_id_salt("sql_snapshot_source")
                                            .selected_text(
                                                self.sql_snapshot_source
                                                    .as_ref()
                                                    .and_then(|selected| sources.iter().find(|(key, _)| key == selected))
                                                    .map(|(_, label)| label.as_str())
                                                    .unwrap_or("Choose a series"),
                                            )
                                            .show_ui(ui, |ui| {
                                                for (key, label) in &sources {
                                                    ui.selectable_value(
                                                        &mut self.sql_snapshot_source,
                                                        Some(key.clone()),
                                                        label,
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
                                let baseline_index = selected_index.and_then(|current| {
                                    (0..current).rev().find(|index| {
                                        self.case.snapshots[*index].source_key == self.case.snapshots[current].source_key
                                    })
                                });
                                let latest = selected_index.and_then(|index|self.case.snapshots.get(index)).cloned();
                                let baseline = baseline_index.and_then(|index|self.case.snapshots.get(index)).cloned();
                                ui.add_space(6.0);
                                ui.label(egui::RichText::new("INGEST NEW SNAPSHOT").small().strong().color(cyan));
                                ui.horizontal(|ui|{ui.label("SQLite file");ui.add(egui::TextEdit::singleline(&mut self.sql_path).hint_text("Relative to case file").desired_width(260.0));ui.label("Series");ui.add(egui::TextEdit::singleline(&mut self.sql_series).desired_width(260.0));});
                                ui.label(egui::RichText::new("Read-only SQL query · required columns: source, target").small().color(muted));
                                ui.add(egui::TextEdit::multiline(&mut self.sql_query).desired_width(canvas_w-20.0).desired_rows(4).code_editor());
                                ui.horizontal(|ui|{if ui.button("Validate query").clicked(){self.validate_sql_query();}if ui.button("Ingest SQL snapshot").clicked(){self.ingest_sqlite_snapshot();}});
                                if let Some(current)=latest {
                                    let capture_label=if selected_index==latest_index{"Latest snapshot"}else{"Selected historical snapshot"};
                                    let skipped = if current.skipped_rows == 0 {
                                        String::new()
                                    } else {
                                        format!(" · {} rows skipped", current.skipped_rows)
                                    };
                                    ui.label(format!("{capture_label} #{}: {} · {} · {} connections{skipped}",selected_index.map(|index|index+1).unwrap_or_default(),current.captured_at,current.series,current.connections.len()));
                                    if let Some(base)=baseline {
                                        let deltas=compare_snapshots(&base,&current);
                                        let count=|kind|deltas.iter().filter(|delta|delta.kinds.contains(&kind)).count();
                                        let removed=count(DeltaKind::Removed);
                                        let declined=count(DeltaKind::ConfidenceDeclined);
                                        let dates=count(DeltaKind::DateChanged);
                                        let updated=count(DeltaKind::Updated);
                                        let evidence=count(DeltaKind::EvidenceChanged);
                                        let source_notes=count(DeltaKind::SourceNotesChanged);
                                        let added=count(DeltaKind::Added);
                                        let increased=count(DeltaKind::ConfidenceIncreased);
                                        ui.label(egui::RichText::new(format!("Baseline {} → current {} · {removed} removed · {declined} confidence declines · {dates} date changes · {updated} connection detail changes · {evidence} evidence changes · {source_notes} source note changes · {added} added · {increased} confidence increases",base.captured_at,current.captured_at)).strong().color(if removed+declined>0{egui::Color32::from_rgb(238,139,115)}else{cyan}));
                                        if ui.button("Export comparison CSV/TSV").clicked() {
                                            self.export_sql_comparison(&base, &current);
                                        }
                                        egui::ScrollArea::both().show(ui,|ui|egui::Grid::new("sql_regression_table").striped(true).show(ui,|ui|{
                                            for heading in ["CHANGE","CONNECTION","CONFIDENCE","DATE","EVIDENCE","NOTES"]{ui.label(egui::RichText::new(heading).small().strong().color(cyan));}ui.end_row();
                                            for delta in deltas {
                                                let change=delta.kinds.iter().map(|kind|match kind{DeltaKind::Removed=>"REMOVED",DeltaKind::ConfidenceDeclined=>"CONFIDENCE DOWN",DeltaKind::DateChanged=>"DATE CHANGED",DeltaKind::Updated=>"CONNECTION DETAILS",DeltaKind::EvidenceChanged=>"EVIDENCE CHANGED",DeltaKind::SourceNotesChanged=>"SOURCE NOTES CHANGED",DeltaKind::Added=>"ADDED",DeltaKind::ConfidenceIncreased=>"CONFIDENCE UP"}).collect::<Vec<_>>().join(" · ");
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
                                                ui.label(format!("{} → {}",delta.previous_date.unwrap_or_else(||"—".into()),current.map(|fact|fact.date.as_str()).filter(|value|!value.is_empty()).unwrap_or("—")));
                                                ui.label(format!("{previous_evidence} → {current_evidence}"));
                                                ui.label(format!("{previous_notes} → {current_notes}"));
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
                            indices.retain(|i|{let link=&self.case.links[*i];if scoped_to.is_some_and(|id|link.from!=id&&link.to!=id){return false;}let endpoint=|id|entity_names.get(&id).map(|(name,kind)|format!("{name} {kind}")).unwrap_or_else(||"(missing)".into());let source=endpoint(link.from);let target=endpoint(link.to);let direction=if link.directed{"directed →"}else{"undirected ↔"};let confidence=link.confidence.to_string();format!("{} {} {} {} {} {} {} {} {} {} {}",link.connection_id,source,link.label,target,link.date,confidence,direction,link.source_reference,link.source_notes,link.notes,link.ingest_source.as_deref().unwrap_or("manual")).to_lowercase().contains(&filter)});
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
                        if let Some(path)=self.chart_fit_path.take(){
                            let mut points=path.iter().filter_map(|id|self.case.entities.iter().find(|entity|entity.id==*id).map(|entity|egui::pos2(rect.left()+entity.x*rect.width(),rect.top()+entity.y*rect.height())));
                            if let Some(first)=points.next(){
                                let (min,max)=points.fold((first,first),|(min,max),point|(egui::pos2(min.x.min(point.x),min.y.min(point.y)),egui::pos2(max.x.max(point.x),max.y.max(point.y))));
                                let content_width=(max.x-min.x+142.0).max(142.0);
                                let content_height=(max.y-min.y+56.0).max(56.0);
                                self.chart_zoom=((rect.width()-48.0)/content_width).min((rect.height()-48.0)/content_height).clamp(0.25,3.0);
                                self.chart_pan=-(min.lerp(max,0.5)-rect.center())*self.chart_zoom;
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
                        let center=|e:&Entity|{let base=egui::pos2(rect.left()+e.x*rect.width(),rect.top()+e.y*rect.height());rect.center()+(base-rect.center())*self.chart_zoom+self.chart_pan};
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
                        for (link_index, link) in self.case.links.iter().enumerate() {
                            if let (Some(&a_index), Some(&b_index)) =
                                (entity_indices.get(&link.from), entity_indices.get(&link.to))
                            {
                                let a = &self.case.entities[a_index];
                                let b = &self.case.entities[b_index];
                                let ca=center(a);let cb=center(b);let in_path=path_edges.contains(&(link.from,link.to))||(!link.directed&&path_edges.contains(&(link.to,link.from)));let col=if self.selected_link==Some(link_index){cyan}else if in_path{egui::Color32::from_rgb(236,174,83)}else{egui::Color32::from_rgb(72,104,120)};
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
                                p.text(mid,egui::Align2::CENTER_CENTER,&link.label,egui::FontId::proportional(9.0*self.chart_zoom),muted);
                            }
                        }
                        for e in &self.case.entities {
                            let c=center(e); let fill=egui::Color32::from_rgb(e.color[0],e.color[1],e.color[2]);
                            let r=egui::Rect::from_center_size(c,egui::vec2(142.0,56.0)*self.chart_zoom);
                            let picked=self.selected==Some(e.id);
                            let on_path=path_entities.contains(&e.id);p.rect_filled(r,7.0,if on_path{egui::Color32::from_rgb(55,48,38)}else{egui::Color32::from_rgb(25,39,51)});p.rect_stroke(r,7.0,egui::Stroke::new(if picked{2.0_f32}else{1.0_f32},if picked{cyan}else if on_path{egui::Color32::from_rgb(236,174,83)}else{fill}),egui::StrokeKind::Inside);
                            p.circle_filled(c+egui::vec2(-58.0,0.0)*self.chart_zoom,6.0*self.chart_zoom,fill);
                            p.text(c+egui::vec2(-44.0,-8.0)*self.chart_zoom,egui::Align2::LEFT_CENTER,&e.name,egui::FontId::proportional(12.0*self.chart_zoom),egui::Color32::WHITE);
                            p.text(c+egui::vec2(-44.0,11.0)*self.chart_zoom,egui::Align2::LEFT_CENTER,&e.kind,egui::FontId::proportional(8.0*self.chart_zoom),muted);
                        }
                        if response.clicked() { if let Some(pos)=response.interact_pointer_pos(){
                            if self.show_timeline && pos.y >= rect.bottom()-105.0 {
                                let timeline_rect=egui::Rect::from_min_size(rect.left_bottom()-egui::vec2(0.0,105.0),egui::vec2(rect.width(),105.0));
                                let nearest=timeline_event_positions(&self.case.links,timeline_rect).into_iter().map(|(index,point)|(index,point.distance(pos))).min_by(|a,b|a.1.total_cmp(&b.1)).filter(|(_,distance)|*distance<20.0).map(|(index,_)|index);
                                self.selected=None;self.selected_link=nearest;
                            } else {let lanes=link_lane_offsets(&self.case.links);let nearest=self.case.links.iter().enumerate().filter_map(|(i,l)|{let a=self.case.entities.get(*entity_indices.get(&l.from)?)?;let b=self.case.entities.get(*entity_indices.get(&l.to)?)?;let points=connection_curve_points(center(a),center(b),lanes[i]*self.chart_zoom);Some((i,polyline_distance(pos,&points)))}).filter(|(_,d)|*d<10.0*self.chart_zoom).min_by(|a,b|a.1.total_cmp(&b.1)).filter(|(_,distance)|*distance<22.0*self.chart_zoom).map(|(i,_)|i);let entity=self.case.entities.iter().find(|e|center(e).distance(pos)<75.0*self.chart_zoom).map(|entity|entity.id);if let Some(index)=nearest{self.selected=None;self.selected_link=Some(index);}else if let Some(entity)=entity{self.selected=Some(entity);self.selected_link=None;}else{self.selected=None;self.selected_link=None;}}
                        } }
                        if response.drag_started() { if let Some(pos)=response.interact_pointer_pos(){if let Some(entity)=self.case.entities.iter().find(|e|center(e).distance(pos)<75.0*self.chart_zoom){self.selected=Some(entity.id);self.selected_link=None;}else{self.selected=None;self.chart_pan_origin=self.chart_pan;}} }
                        if response.dragged() { if let Some(pos)=response.interact_pointer_pos(){if let Some(id)=self.selected{if let Some(e)=self.case.entities.iter_mut().find(|e|e.id==id){let local=(pos-rect.center()-self.chart_pan)/self.chart_zoom+rect.size()*0.5;e.x=(local.x/rect.width()).clamp(0.05,0.95);e.y=(local.y/rect.height()).clamp(0.08,0.92);}}else{self.chart_pan=self.chart_pan_origin+response.drag_delta();}} }
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
                        let mut degrees:Vec<_>=self.case.entities.iter().map(|e|(e,connected_counts.get(&e.id).copied().unwrap_or_default())).collect();degrees.sort_by_key(|(_,n)|std::cmp::Reverse(*n));
                        ui.label(egui::RichText::new("MOST CONNECTED").size(9.0).strong().color(cyan));
                        for (rank,(e,n)) in degrees.iter().take(5).enumerate(){ui.horizontal(|ui|{let label=format!("{:02}  {} · {}",rank+1,e.name,e.kind);if ui.selectable_label(self.selected==Some(e.id),label).clicked(){self.selected=Some(e.id);self.selected_link=None;self.chart_focus=Some(e.id);self.page=WorkspacePage::Chart;self.show_timeline=false;}ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(n.to_string());});});}
                        ui.add_space(16.0);ui.label(egui::RichText::new("SHORTEST PATH").size(9.0).strong().color(cyan));
                        egui::ComboBox::from_id_salt("path_start").selected_text(self.path_start.and_then(|id|self.case.entities.iter().find(|e|e.id==id).map(|e|format!("{} · {}",e.name,e.kind))).unwrap_or_else(||"Start entity".into())).show_ui(ui,|ui|{for e in &self.case.entities{ui.selectable_value(&mut self.path_start,Some(e.id),format!("{} · {}",e.name,e.kind));}});
                        egui::ComboBox::from_id_salt("path_end").selected_text(self.path_end.and_then(|id|self.case.entities.iter().find(|e|e.id==id).map(|e|format!("{} · {}",e.name,e.kind))).unwrap_or_else(||"End entity".into())).show_ui(ui,|ui|{for e in &self.case.entities{ui.selectable_value(&mut self.path_end,Some(e.id),format!("{} · {}",e.name,e.kind));}});
                        if ui.button("Analyze path").clicked(){self.analyze_path();}
                        if !self.path_result.is_empty(){let labels:Vec<_>=self.path_result.iter().filter_map(|id|self.case.entities.iter().find(|e|e.id==*id).map(|e|format!("{} · {}",e.name,e.kind))).collect();ui.label(egui::RichText::new(labels.join(" → ")).small().color(egui::Color32::from_rgb(236,174,83)));ui.horizontal(|ui|{if ui.small_button("Show path on chart").clicked(){self.chart_fit_path=Some(self.path_result.clone());self.chart_focus=None;self.chart_focus_link=None;self.page=WorkspacePage::Chart;self.show_timeline=false;self.selected=self.path_result.first().copied();self.selected_link=None;}if ui.small_button("Clear path").clicked(){self.path_result.clear();self.chart_fit_path=None;}});}
                        ui.add_space(16.0);ui.label(egui::RichText::new("POSSIBLE DUPLICATES").size(9.0).strong().color(cyan));
                        if ui.button("Find likely matches").clicked(){self.find_duplicates();}
                        for (a,b) in self.duplicate_candidates.clone().iter().take(5){let an=self.case.entities.iter().find(|e|e.id==*a).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());let bn=self.case.entities.iter().find(|e|e.id==*b).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());if ui.selectable_label(self.merge_candidate==Some((*a,*b)),format!("{an}  ↔  {bn}")).clicked(){self.merge_candidate=Some((*a,*b));}}
                        if let Some((keep,remove))=self.merge_candidate{let keep_name=self.case.entities.iter().find(|e|e.id==keep).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());let remove_name=self.case.entities.iter().find(|e|e.id==remove).map(|e|format!("{} · {}",e.name,e.kind)).unwrap_or_else(||"(missing)".into());ui.label(egui::RichText::new(format!("Keep {keep_name}; merge {remove_name} into it. Aliases, notes, attributes, and links will be retained.")).small().color(muted));if ui.button(format!("Merge {remove_name} into {keep_name}")).clicked(){self.merge_candidate_into_first();}}
                        ui.add_space(20.0);ui.label(egui::RichText::new("SELECTED ITEM").size(9.0).strong().color(cyan));ui.add_space(5.0);
                        if let Some(id)=self.selected{if let Some(entity)=self.case.entities.iter().find(|e|e.id==id){if !entity.aliases.is_empty(){ui.label(egui::RichText::new("KNOWN ALIASES").size(9.0).strong().color(cyan));for alias in &entity.aliases{ui.label(alias);}}if !entity.merged_records.is_empty(){egui::CollapsingHeader::new(format!("Merged record history ({})",entity.merged_records.len())).show(ui,|ui|for record in &entity.merged_records{ui.label(format!("{} · {} · {}",record.name,record.kind,record.id));if !record.detail.is_empty(){ui.label(&record.detail);}for (key,value) in &record.attributes{ui.label(format!("{key}: {value}"));}});}}}
                        if let Some(id)=self.selected { if let Some(e)=self.case.entities.iter_mut().find(|e|e.id==id){ui.label(egui::RichText::new("TYPE").size(9.0).color(muted));ui.add(egui::TextEdit::singleline(&mut e.kind).desired_width(230.0));ui.label(egui::RichText::new("LABEL").size(9.0).color(muted));ui.add(egui::TextEdit::singleline(&mut e.name).desired_width(230.0));ui.horizontal(|ui|{ui.label(egui::RichText::new("COLOR").size(9.0).color(muted));ui.color_edit_button_srgb(&mut e.color);ui.label(format!("#{:02X}{:02X}{:02X}",e.color[0],e.color[1],e.color[2]));});ui.checkbox(&mut e.position_locked,"Pin position during automatic layouts");ui.label(egui::RichText::new("NOTES").size(9.0).color(muted));ui.add(egui::TextEdit::multiline(&mut e.detail).desired_width(230.0).desired_rows(3));ui.label(egui::RichText::new("ATTRIBUTES").size(9.0).strong().color(cyan));let mut remove=None;for (i,(key,value)) in e.attributes.iter_mut().enumerate(){ui.horizontal(|ui|{ui.add(egui::TextEdit::singleline(key).hint_text("Attribute").desired_width(90.0));ui.add(egui::TextEdit::singleline(value).hint_text("Value").desired_width(100.0));if ui.small_button("×").clicked(){remove=Some(i);}});}if let Some(i)=remove{e.attributes.remove(i);}if ui.small_button("＋ Attribute").clicked(){e.attributes.push((String::new(),String::new()));}ui.label(egui::RichText::new("LINKS").size(9.0).strong().color(cyan));for l in self.case.links.iter().filter(|l|l.from==id||l.to==id){ui.label(format!("{}  ·  {}",l.label,l.date));}}}else if let Some(i)=self.selected_link {if let Some(link)=self.case.links.get_mut(i){ui.label(egui::RichText::new("RELATIONSHIP").size(9.0).color(muted));let from=link.from;let to=link.to;ui.horizontal(|ui|{if let Some(entity)=self.case.entities.iter().find(|entity|entity.id==from){if ui.small_button(format!("Source · {}",entity.name)).clicked(){self.selected=Some(from);self.selected_link=None;self.chart_focus=Some(from);}}if let Some(entity)=self.case.entities.iter().find(|entity|entity.id==to){if ui.small_button(format!("Target · {}",entity.name)).clicked(){self.selected=Some(to);self.selected_link=None;self.chart_focus=Some(to);}}});ui.add(egui::TextEdit::singleline(&mut link.label));ui.horizontal(|ui|{ui.label("Date");ui.add(egui::TextEdit::singleline(&mut link.date).desired_width(100.0));});ui.add(egui::Slider::new(&mut link.confidence,1..=3).text("Confidence"));ui.checkbox(&mut link.directed,"Directed relationship");ui.label("Source reference");ui.add(egui::TextEdit::singleline(&mut link.source_reference).desired_width(230.0));if !link.source_notes.is_empty(){ui.label(egui::RichText::new("SOURCE NOTES").size(9.0).color(muted));ui.label(&link.source_notes);}ui.label("Analyst notes");ui.add(egui::TextEdit::multiline(&mut link.notes).desired_rows(4));}}else{ui.label(egui::RichText::new("Select an entity on the chart to inspect its attributes and connections. Select a relationship in Data to edit its properties.").small().color(muted));}
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
                        let fields: [(&str, &mut String); 28] = [
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
    node: &mut Option<GraphMlRecord>,
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
            node.as_mut()
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
    let mut node: Option<GraphMlRecord> = None;
    let mut edge: Option<GraphMlRecord> = None;
    let mut active_data: Option<(GraphMlScope, String, String)> = None;
    let mut root_seen = false;
    let mut graph_count = 0;
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
                    graph_count += 1;
                    if graph_count > 1 {
                        return Err("GraphML import supports one top-level graph".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    document.default_directed = match attributes
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
                }
                b"node" => {
                    if node.is_some() || edge.is_some() {
                        return Err("nested GraphML nodes and edges are not supported".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    node = Some(GraphMlRecord {
                        id: attributes.get("id").cloned().unwrap_or_default(),
                        ..GraphMlRecord::default()
                    });
                }
                b"edge" => {
                    if node.is_some() || edge.is_some() {
                        return Err("nested GraphML nodes and edges are not supported".into());
                    }
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    edge = Some(graphml_edge_record(attributes)?);
                }
                b"data" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let key = attributes
                        .get("key")
                        .cloned()
                        .ok_or("GraphML data element is missing its key")?;
                    let scope = if edge.is_some() {
                        GraphMlScope::Edge
                    } else if node.is_some() {
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
                b"node" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    document.nodes.push(GraphMlRecord {
                        id: attributes.get("id").cloned().unwrap_or_default(),
                        ..GraphMlRecord::default()
                    });
                }
                b"edge" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    document.edges.push(graphml_edge_record(attributes)?);
                }
                b"data" => {
                    let attributes = graphml_xml_attributes(&start, decoder)?;
                    let key = attributes
                        .get("key")
                        .cloned()
                        .ok_or("GraphML data element is missing its key")?;
                    let scope = if edge.is_some() {
                        GraphMlScope::Edge
                    } else if node.is_some() {
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
                        &mut node,
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
                        &mut node,
                        &mut edge,
                    )?;
                }
                b"node" => {
                    document
                        .nodes
                        .push(node.take().ok_or("GraphML node end tag has no start")?);
                }
                b"edge" => {
                    document
                        .edges
                        .push(edge.take().ok_or("GraphML edge end tag has no start")?);
                }
                _ => {}
            },
            XmlEvent::Eof => break,
            _ => {}
        }
        buffer.clear();
    }
    if !root_seen || graph_count != 1 {
        return Err("file does not contain one GraphML graph".into());
    }
    if node.is_some() || edge.is_some() || active_data.is_some() {
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
        let entity_id = node_ids[&node.id].to_string();
        let source_detail = graphml_json_property(&detail)?;
        let source_aliases = graphml_json_property(&aliases)?;
        let mut row = vec![String::new(); headers.len()];
        let fields = [
            ("record_type", "entity"),
            ("source", name),
            ("source_entity_id", entity_id.as_str()),
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
    for snapshot in &mut case.snapshots {
        let configured_path = (!snapshot.configured_database_path.trim().is_empty())
            .then(|| snapshot.configured_database_path.as_str());
        let path = configured_path
            .and_then(|path| resolve_sqlite_database_path(path, case_path).ok())
            .or_else(|| resolve_sqlite_database_path(&snapshot.database_path, case_path).ok());
        if let Some(path) = path {
            let old_source_key = snapshot.source_key.clone();
            snapshot.database_path = path.to_string_lossy().into_owned();
            snapshot.source_key = sql_source_key(&snapshot.database_path, &snapshot.series);
            source_key_rewrites.insert(old_source_key, snapshot.source_key.clone());
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
    json_output: bool,
) -> Result<bool, String> {
    let _case_lock = lock_automation_case(Path::new(case_path))?;
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
    let mut app = App::automation_context(case, case_path);
    normalize_snapshot_database_paths(&mut app.case, &app.case_path);
    let mut failures = 0;
    let mut outcomes = Vec::new();
    for job in jobs {
        let (success, message) = app.run_automation_job_cli(&job);
        if !success {
            failures += 1;
        }
        let run = app.case.automation_runs.last();
        let snapshot_id = run.and_then(|run| run.snapshot_id);
        let snapshot =
            snapshot_id.and_then(|id| app.case.snapshots.iter().find(|snapshot| snapshot.id == id));
        let connection_count = snapshot.map(|snapshot| snapshot.connections.len());
        let skipped_rows = snapshot.map(|snapshot| snapshot.skipped_rows);
        save_case_atomically(Path::new(case_path), &app.case)?;
        if json_output {
            outcomes.push(serde_json::json!({
                "status": if success { "PASS" } else { "FAIL" },
                "job": job.name,
                "message": message,
                "snapshot_id": snapshot_id,
                "connection_count": connection_count,
                "skipped_rows": skipped_rows,
                "minimum_connections": job.minimum_connections,
                "maximum_regressions": job.maximum_regressions,
                "maximum_skipped_rows": job.maximum_skipped_rows,
                "query": job.query,
            }));
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
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"ok":failures==0,"saved":true,"case":case_path,"passed":outcomes.iter().filter(|outcome|outcome["status"]=="PASS").count(),"failed":failures,"outcomes":outcomes})).map_err(|error|error.to_string())?);
    }
    Ok(failures == 0)
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

fn lock_automation_case(path: &Path) -> Result<fs::File, String> {
    if path.as_os_str().is_empty() {
        return Err("case path cannot be empty".into());
    }
    let file_name = path
        .file_name()
        .ok_or_else(|| format!("invalid case path: {}", path.display()))?;
    let mut lock_name = file_name.to_os_string();
    lock_name.push(".automation.lock");
    let mut lock_path = path.to_path_buf();
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
    json_output: bool,
) -> Result<bool, String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let jobs: Vec<AutomationJob> = case
        .automation_jobs
        .iter()
        .filter(|job| job.enabled)
        .cloned()
        .collect();
    if jobs.is_empty() {
        return Err("case contains no enabled automation jobs".into());
    }

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
                outcomes
                    .push(serde_json::json!({"status":"FAIL","job":job.name,"message":message}));
            } else {
                println!("FAIL · {} · {}", job.name, message);
            }
            continue;
        }
        app.sql_path = job.database_path.clone();
        app.sql_series = job.series.clone();
        app.sql_query = job.query.clone();
        app.automation_minimum_connections = job.minimum_connections;
        app.automation_maximum_regressions = job.maximum_regressions;
        app.automation_maximum_skipped_rows = job.maximum_skipped_rows;
        let snapshot_count_before = app.case.snapshots.len();
        let success = app.validate_sql_query_for_batch();
        let message = app.status.clone();
        let snapshot = app.case.snapshots.get(snapshot_count_before);
        if !success {
            failures += 1;
        }
        if json_output {
            outcomes.push(serde_json::json!({
                "status": if success { "PASS" } else { "FAIL" },
                "job": job.name,
                "message": message,
                "snapshot_id": snapshot.map(|snapshot| snapshot.id),
                "connection_count": snapshot.map(|snapshot| snapshot.connections.len()),
                "skipped_rows": snapshot.map(|snapshot| snapshot.skipped_rows),
                "minimum_connections": job.minimum_connections,
                "maximum_regressions": job.maximum_regressions,
                "maximum_skipped_rows": job.maximum_skipped_rows,
                "query": job.query,
            }));
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
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({"ok":failures==0,"saved":false,"case":case_path,"passed":outcomes.iter().filter(|outcome|outcome["status"]=="PASS").count(),"failed":failures,"outcomes":outcomes})).map_err(|error|error.to_string())?);
    }
    Ok(failures == 0)
}

fn list_automation_jobs_cli(case_path: &str, json_output: bool) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let jobs: Vec<serde_json::Value> = case
        .automation_jobs
        .iter()
        .map(|job| {
            serde_json::json!({
                "id": job.id,
                "name": job.name,
                "enabled": job.enabled,
                "database_path": job.database_path,
                "series": job.series,
                "minimum_connections": job.minimum_connections,
                "maximum_regressions": job.maximum_regressions,
                "maximum_skipped_rows": job.maximum_skipped_rows,
            })
        })
        .collect();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({"case":case_path,"jobs":jobs}))
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

fn list_automation_runs_cli(case_path: &str, json_output: bool) -> Result<(), String> {
    let content =
        fs::read_to_string(case_path).map_err(|error| format!("cannot read case: {error}"))?;
    let case = serde_json::from_str::<CaseFile>(&content)
        .map_err(|error| format!("cannot parse case: {error}"))?;
    let snapshots_by_id: HashMap<Uuid, &ConnectionSnapshot> = case
        .snapshots
        .iter()
        .map(|snapshot| (snapshot.id, snapshot))
        .collect();
    let runs: Vec<serde_json::Value> = case
        .automation_runs
        .iter()
        .rev()
        .map(|run| {
            let snapshot = run
                .snapshot_id
                .and_then(|id| snapshots_by_id.get(&id).copied())
                .or_else(|| {
                    run.snapshot_index
                        .and_then(|index| case.snapshots.get(index))
                });
            serde_json::json!({
                "id": run.id,
                "job_id": run.job_id,
                "job": run.job_name,
                "status": if run.success { "PASS" } else { "FAIL" },
                "started_at": run.started_at,
                "finished_at": run.finished_at,
                "database_path": run.database_path,
                "series": run.series,
                "query": run.query,
                "minimum_connections": run.minimum_connections,
                "maximum_regressions": run.maximum_regressions,
                "maximum_skipped_rows": run.maximum_skipped_rows,
                "enabled": run.enabled,
                "snapshot_id": run.snapshot_id,
                "connection_count": snapshot.map(|snapshot| snapshot.connections.len()),
                "skipped_rows": snapshot.map(|snapshot| snapshot.skipped_rows),
                "captured_at": snapshot.map(|snapshot| snapshot.captured_at.as_str()),
                "message": run.message,
                "configuration_recorded": run.configuration_recorded,
            })
        })
        .collect();
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({"case":case_path,"runs":runs}))
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
            serde_json::to_string_pretty(&serde_json::json!({"ok":false,"error":error}))
                .unwrap_or_else(
                    |_| "{\"ok\":false,\"error\":\"failed to serialize CLI error\"}".into()
                )
        );
    }
    eprintln!("{action}: {error}");
    std::process::exit(1);
}

fn report_cli_usage_error(usage: &str, json_output: bool) -> ! {
    if json_output {
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({"ok":false,"error":usage}))
                .unwrap_or_else(|_| "{\"ok\":false,\"error\":\"invalid command arguments\"}".into())
        );
    }
    eprintln!("{usage}");
    std::process::exit(2);
}

fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args
        .first()
        .is_some_and(|arg| arg == "--help" || arg == "-h")
    {
        println!(
            "Arch Notebook\n\nUsage:\n  arch-notebook\n  arch-notebook --export-graphml <case.json> <output.graphml> [--json]\n  arch-notebook --list-jobs <case.json> [--json]\n  arch-notebook --list-runs <case.json> [--json]\n  arch-notebook --validate-enabled-jobs <case.json> [--json]\n  arch-notebook --run-enabled-jobs <case.json> [--json]\n  arch-notebook --run-job <case.json> <job-name> [--json]\n\nJob and run listing and validation do not change the case. Batch runs save snapshots and run history to the case file. JSON output is available for scheduler integration."
        );
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
    if args.first().is_some_and(|arg| arg == "--list-jobs") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(
                "usage: arch-notebook --list-jobs <case.json> [--json]",
                json_output,
            );
        };
        if args.len() > 3 || args.get(2).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --list-jobs <case.json> [--json]",
                json_output,
            );
        }
        if let Err(error) = list_automation_jobs_cli(case_path, json_output) {
            report_cli_error("job listing failed", error, json_output);
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--list-runs") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(
                "usage: arch-notebook --list-runs <case.json> [--json]",
                json_output,
            );
        };
        if args.len() > 3 || args.get(2).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --list-runs <case.json> [--json]",
                json_output,
            );
        }
        if let Err(error) = list_automation_runs_cli(case_path, json_output) {
            report_cli_error("run history listing failed", error, json_output);
        }
        return Ok(());
    }
    if args
        .first()
        .is_some_and(|arg| arg == "--validate-enabled-jobs")
    {
        let json_output = args.iter().any(|arg| arg == "--json");
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(
                "usage: arch-notebook --validate-enabled-jobs <case.json> [--json]",
                json_output,
            );
        };
        if args.len() > 3 || args.get(2).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --validate-enabled-jobs <case.json> [--json]",
                json_output,
            );
        }
        match validate_enabled_automation_jobs_cli(case_path, json_output) {
            Err(error) => report_cli_error("automation validation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation validation failed; case file was not changed");
                std::process::exit(1);
            }
            Ok(true) => {}
        }
        return Ok(());
    }
    if args.first().is_some_and(|arg| arg == "--run-enabled-jobs") {
        let json_output = args.iter().any(|arg| arg == "--json");
        let Some(case_path) = args.get(1) else {
            report_cli_usage_error(
                "usage: arch-notebook --run-enabled-jobs <case.json> [--json]",
                json_output,
            );
        };
        if args.len() > 3 || args.get(2).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --run-enabled-jobs <case.json> [--json]",
                json_output,
            );
        }
        match run_automation_cli(case_path, None, json_output) {
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
        let (Some(case_path), Some(job_name)) = (args.get(1), args.get(2)) else {
            report_cli_usage_error(
                "usage: arch-notebook --run-job <case.json> <job-name> [--json]",
                json_output,
            );
        };
        if args.len() > 4 || args.get(3).is_some_and(|arg| arg != "--json") {
            report_cli_usage_error(
                "usage: arch-notebook --run-job <case.json> <job-name> [--json]",
                json_output,
            );
        }
        match run_automation_cli(case_path, Some(job_name), json_output) {
            Err(error) => report_cli_error("automation failed", error, json_output),
            Ok(false) => {
                eprintln!("automation failed; run history saved");
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
            target: "Pier 19 Warehouse".into(),
            target_type: "LOCATION".into(),
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
