/// A binding observation retains multiplicity until the owner checks it.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct BindingKey {
    pub stage_id: String,
    pub tool_id: String,
}
/// A published canonical surface, projected without inventing another wire model.
#[derive(Debug, Clone)]
pub struct BindingSurface {
    pub output_path: String,
    pub row_count: usize,
    pub stage_count: usize,
    pub tool_count: usize,
    pub bindings: Vec<BindingKey>,
}
/// Command and local-job views share one canonical command acquisition.
#[derive(Debug, Clone)]
pub struct CommandSurfaces {
    pub commands: BindingSurface,
    pub argv_output_path: String,
}
