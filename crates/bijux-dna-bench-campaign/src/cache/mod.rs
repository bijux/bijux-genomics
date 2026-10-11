//! `SortMeRNA` input options, shared index layout and cache lifecycle.
mod cloning;
mod layout;
mod lifecycle;
mod options;
mod warmup;
pub(crate) use layout::{sortmerna_shared_index_dir, sortmerna_shared_index_seeded};
pub(crate) use lifecycle::{
    prepare_sortmerna_sample_workdir, promote_sortmerna_sample_index_cache,
    prune_sortmerna_sample_payload,
};
pub(crate) use options::{resolve_deplete_rrna_stage_options, DepleteRrnaStageOptions};
pub(crate) use warmup::warm_sortmerna_shared_index_cache;

#[cfg(test)]
mod contracts;
