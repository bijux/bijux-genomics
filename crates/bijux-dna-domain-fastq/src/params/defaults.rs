use super::detect_adapters::DetectAdaptersEffectiveParams;
use super::filter::FilterEffectiveParams;
use super::merge::MergeEffectiveParams;
use super::preprocess::PreprocessEffectiveParams;
use super::qc_post::QcPostEffectiveParams;
use super::screen::ScreenEffectiveParams;
use super::trim::TrimEffectiveParams;
use super::validate::ValidateEffectiveParams;
use super::PairedMode;

fn paired_mode(paired: bool) -> PairedMode {
    if paired {
        PairedMode::PairedEnd
    } else {
        PairedMode::SingleEnd
    }
}

#[must_use]
pub fn validate_defaults(paired: bool) -> ValidateEffectiveParams {
    ValidateEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        q_cutoff: None,
    }
}

#[must_use]
pub fn detect_adapters_defaults(paired: bool) -> DetectAdaptersEffectiveParams {
    DetectAdaptersEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        sample_reads: None,
    }
}

#[must_use]
pub fn trim_defaults(paired: bool) -> TrimEffectiveParams {
    TrimEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        min_len: 0,
        q_cutoff: None,
        adapter_policy: "none".to_string(),
        damage_mode: None,
        polyx_policy: None,
        n_policy: None,
        contaminant_policy: None,
    }
}

#[must_use]
pub fn filter_defaults(paired: bool) -> FilterEffectiveParams {
    FilterEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        max_n: None,
        max_n_fraction: None,
        max_n_count: None,
        low_complexity_threshold: None,
        entropy_threshold: None,
        contaminant_db: None,
        n_policy: None,
        polyx_policy: None,
        damage_mode: None,
    }
}

#[must_use]
pub fn qc_post_defaults(paired: bool) -> QcPostEffectiveParams {
    QcPostEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
    }
}

#[must_use]
pub fn preprocess_defaults(paired: bool) -> PreprocessEffectiveParams {
    PreprocessEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        stages: vec![
            "fastq.validate_pre".to_string(),
            "fastq.detect_adapters".to_string(),
            "fastq.trim".to_string(),
            "fastq.filter".to_string(),
            "fastq.stats_neutral".to_string(),
            "fastq.qc_post".to_string(),
        ],
        enable_contaminant_removal: false,
    }
}

#[must_use]
pub fn merge_defaults(paired: bool) -> MergeEffectiveParams {
    MergeEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        merge_overlap: None,
        min_len: None,
    }
}

#[must_use]
pub fn screen_defaults(paired: bool) -> ScreenEffectiveParams {
    ScreenEffectiveParams {
        paired_mode: paired_mode(paired),
        threads: 1,
        contaminant_db: None,
    }
}
