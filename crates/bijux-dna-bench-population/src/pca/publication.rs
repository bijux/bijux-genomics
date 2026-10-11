//! Owner: bijux-dna-bench-population
//! Render normalized tabular population observations.
use std::fmt::Write as _;
// Pca publication.
use crate::pca::model::LocalVcfPcaSmokeRow;

pub(crate) fn build_pca_tsv(rows: &[LocalVcfPcaSmokeRow]) -> String {
    let mut rendered = String::from("sample_id\tpopulation_id\tpopulation_label\tsex\tpc1\tpc2\n");
    for row in rows {
        let _ = writeln!(
            rendered,
            "{}\t{}\t{}\t{}\t{}\t{}",
            row.sample_id, row.population_id, row.population_label, row.sex, row.pc1, row.pc2
        );
    }
    rendered
}
