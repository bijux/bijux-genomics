use bijux_dna_core::id_catalog::BAM_DAMAGE;
pub const DEFAULT_BAM_DAMAGE_COMPLETE_PATH: &str =
    "artifacts/benchmark-smoke/readiness/bam/stages/bam.damage.complete.json";

pub(super) const BAM_DAMAGE_COMPLETE_SCHEMA_VERSION: &str =
    "bijux.bench.readiness.bam_damage_complete.v1";

pub(super) const EXPECTED_STAGE_ID: &str = BAM_DAMAGE;

pub(super) const EXPECTED_LOCAL_SMOKE_SCHEMA_VERSION: &str =
    "bijux.bam.damage.local_smoke.report.v1";

pub(super) const EXPECTED_SUMMARY_SCHEMA_VERSION: &str = "bijux.bam.damage_evidence.v1";

pub(super) const EXPECTED_STAGE_METRICS_SCHEMA_VERSION: &str = "bijux.bam.damage.stage_metrics.v1";

pub(super) const EXPECTED_PARSER_OUTPUT_SCHEMA_VERSION: &str = "bijux.bam.damage.parser_output.v1";

pub(super) const EXPECTED_SAMPLE_ID: &str = "adna_damage_non_udg";

pub(super) const EXPECTED_METHOD: &str = "mapdamage2";

pub(super) const EXPECTED_TERMINAL_C_TO_T_5P: f64 = 0.18;

pub(super) const EXPECTED_TERMINAL_G_TO_A_3P: f64 = 0.11;

pub(super) const EXPECTED_SHORT_FRAGMENT_FRACTION: f64 = 1.0;

pub(super) const EXPECTED_DAMAGE_SIGNAL: &str = "moderate";

pub(super) const EXPECTED_STRICT_PROFILE_UPGRADED: bool = false;

pub(super) const CHECKED_SURFACE_COUNT: usize = 13;

pub(super) const EXPECTED_TOOL_IDS: [&str; 5] =
    ["addeam", "damageprofiler", "mapdamage2", "pmdtools", "pydamage"];

pub(super) const REQUIRED_OUTPUT_IDS: [&str; 4] =
    ["damage_report", "terminal_position_metrics", "parser_output", "stage_metrics"];

pub(super) const REQUIRED_SUMMARY_METRIC_NAMES: [&str; 7] = [
    "terminal_c_to_t_5p",
    "terminal_g_to_a_3p",
    "short_fragment_fraction",
    "damage_signal",
    "strict_profile_upgraded",
    "advisory_boundary",
    "notes",
];

pub(super) const REQUIRED_UNIFIED_METRIC_NAMES: [&str; 3] =
    ["canonical", "tools_seen", "comparison"];

pub(super) const REQUIRED_PARSER_OUTPUT_FIELDS: [&str; 3] =
    ["schema_version", "stage_id", "parsed_tools"];

pub(super) const REQUIRED_NORMALIZED_METRIC_NAMES: [&str; 16] = [
    "expected_terminal_c_to_t_5p",
    "terminal_c_to_t_5p",
    "terminal_c_to_t_5p_delta",
    "expected_terminal_g_to_a_3p",
    "terminal_g_to_a_3p",
    "terminal_g_to_a_3p_delta",
    "expected_short_fragment_fraction",
    "short_fragment_fraction",
    "short_fragment_fraction_delta",
    "expected_damage_signal",
    "damage_signal",
    "expected_strict_profile_upgraded",
    "strict_profile_upgraded",
    "expectation_matched",
    "tool_id",
    "tools_seen",
];
