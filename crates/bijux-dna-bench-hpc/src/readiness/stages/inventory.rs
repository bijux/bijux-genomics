use crate::readiness::assessment::fail_check;
use crate::readiness::assessment::ok_check;
use crate::readiness::context::ReadinessContext;
use crate::readiness::ports::ReadinessDomain as BenchLocalDomain;
use bijux_dna_bench_model::BenchmarkLocalHpcSubmissionReadinessGoalCheck as BenchLocalHpcSubmissionReadyGoalCheck;

pub(in crate::readiness) fn evaluate_fastq_inventory_goal(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<usize> {
    let repo_root = context.repo_root;
    match context.backend.load_local_stage_inventory(repo_root, BenchLocalDomain::Fastq) {
        Ok(inventory) if inventory.stage_count == 27 => {
            checks.push(ok_check(
                1,
                "stage_matrices",
                "FASTQ stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("FASTQ inventory contains {} stages", inventory.stage_count),
            ));
            Some(inventory.stage_count)
        }
        Ok(inventory) => {
            checks.push(fail_check(
                1,
                "stage_matrices",
                "FASTQ stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("expected 27 FASTQ stages but found {}", inventory.stage_count),
            ));
            None
        }
        Err(err) => {
            checks.push(fail_check(
                1,
                "stage_matrices",
                "FASTQ stage matrix covers the governed local inventory",
                Some("configs/bench/local/fastq-stage-matrix.toml".to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}

pub(in crate::readiness) fn evaluate_bam_inventory_goal(
    context: &ReadinessContext<'_>,
    checks: &mut Vec<BenchLocalHpcSubmissionReadyGoalCheck>,
) -> Option<usize> {
    let repo_root = context.repo_root;
    match context.backend.load_local_stage_inventory(repo_root, BenchLocalDomain::Bam) {
        Ok(inventory) if inventory.stage_count == 24 => {
            checks.push(ok_check(
                28,
                "stage_matrices",
                "BAM stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("BAM inventory contains {} stages", inventory.stage_count),
            ));
            Some(inventory.stage_count)
        }
        Ok(inventory) => {
            checks.push(fail_check(
                28,
                "stage_matrices",
                "BAM stage matrix covers the governed local inventory",
                Some(inventory.stage_matrix_path),
                format!("expected 24 BAM stages but found {}", inventory.stage_count),
            ));
            None
        }
        Err(err) => {
            checks.push(fail_check(
                28,
                "stage_matrices",
                "BAM stage matrix covers the governed local inventory",
                Some("configs/bench/local/bam-stage-matrix.toml".to_string()),
                format!("{err:#}"),
            ));
            None
        }
    }
}
