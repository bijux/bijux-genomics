//! Owner: bijux-dna-bench-population
//! Admixture publication.
use crate::admixture::model::LocalVcfAdmixtureSmokeRow;

pub(crate) fn build_admixture_tsv(
    rows: &[LocalVcfAdmixtureSmokeRow],
    cluster_headers: &[String],
) -> String {
    let mut rendered = String::from("sample_id\tpopulation_id\tpopulation_label\tsex\tK\tstatus");
    for cluster_header in cluster_headers {
        rendered.push('\t');
        rendered.push_str(cluster_header);
    }
    rendered.push('\n');
    for row in rows {
        rendered.push_str(&row.sample_id);
        rendered.push('\t');
        rendered.push_str(&row.population_id);
        rendered.push('\t');
        rendered.push_str(&row.population_label);
        rendered.push('\t');
        rendered.push_str(&row.sex);
        rendered.push('\t');
        rendered.push_str(&row.k.to_string());
        rendered.push('\t');
        rendered.push_str(&row.status);
        for cluster_header in cluster_headers {
            rendered.push('\t');
            rendered.push_str(
                &row.clusters
                    .get(cluster_header)
                    .map_or_else(|| "0.000000".to_string(), |value| format!("{value:.6}")),
            );
        }
        rendered.push('\n');
    }
    rendered
}
