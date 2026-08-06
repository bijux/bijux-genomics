#[derive(Debug, Clone)]
pub struct DockerToolSpec {
    pub name: String,
    pub executable: Option<String>,
    pub version_cmd: String,
    pub help_cmd: Option<String>,
    pub probe_cmd: Option<String>,
    pub probe_expected_exit: Vec<i32>,
}
