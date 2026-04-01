#[derive(Debug, Clone)]
pub struct CommandOutputV1 {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub runtime_s: f64,
    pub command: String,
}
