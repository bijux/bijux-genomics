#[derive(Clone, Debug)]
pub(super) struct ImagePlan {
    pub(super) image_name: String,
    pub(super) expected_version: String,
    pub(super) probe_cmd: Option<String>,
    pub(super) probe_expected_exit: Vec<i32>,
    pub(super) executable: Option<String>,
}

#[derive(Default)]
pub(super) struct Summary {
    pub(super) pass: usize,
    pub(super) fail: usize,
}

pub(super) enum ImageTestOutcome {
    Pass(ImageProbeKind),
    Fail(ImageFailureReason),
}

pub(super) enum ImageFailureReason {
    ImageNotFound,
    ExecutableMissing,
    ProbeFailed,
    UnexpectedExitCode(i32),
}

impl std::fmt::Display for ImageFailureReason {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageFailureReason::ImageNotFound => write!(f, "image not found"),
            ImageFailureReason::ExecutableMissing => write!(f, "executable missing"),
            ImageFailureReason::ProbeFailed => write!(f, "probe failed"),
            ImageFailureReason::UnexpectedExitCode(code) => {
                write!(f, "unexpected exit code {code}")
            }
        }
    }
}

pub(super) enum ImageProbeKind {
    Version,
    Exec,
}

impl std::fmt::Display for ImageProbeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImageProbeKind::Version => write!(f, "version"),
            ImageProbeKind::Exec => write!(f, "exec"),
        }
    }
}

pub(super) struct ProbeResult {
    pub(super) exit_code: i32,
    pub(super) output: String,
}
