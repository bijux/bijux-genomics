//! Order-sensitive native local-smoke OTU membership on admitted FASTQ record values.
//! This deterministic prefix-identity transform does not implement a backend clustering tool.

use crate::params::edna::OtuClusteringEffectiveParams;
use crate::types::FastqRecord;

#[cfg(test)]
mod contracts;

/// Representatives and aligned abundances in first-observed cluster order.
pub(crate) struct OtuClusters {
    pub representatives: Vec<String>,
    pub counts: Vec<u64>,
}

/// Compute memberships without acquisition, publication or parameter admission.
///
/// The first matching representative wins. Only the identity threshold changes
/// membership; other effective parameters retain their caller-owned report meaning.
pub(crate) fn cluster_otu_records(
    reads: &[FastqRecord],
    params: &OtuClusteringEffectiveParams,
) -> OtuClusters {
    let mut representatives = Vec::<String>::new();
    let mut counts = Vec::<u64>::new();
    for record in reads {
        let mut assigned = false;
        for (idx, rep) in representatives.iter().enumerate() {
            if sequence_identity(rep, &record.sequence) >= params.identity_threshold {
                counts[idx] += 1;
                assigned = true;
                break;
            }
        }
        if !assigned {
            representatives.push(record.sequence.clone());
            counts.push(1);
        }
    }
    OtuClusters { representatives, counts }
}

fn sequence_identity(a: &str, b: &str) -> f64 {
    let min_len = a.len().min(b.len());
    if min_len == 0 {
        return 0.0;
    }
    let matches = a.chars().zip(b.chars()).take(min_len).filter(|(x, y)| x == y).count();
    matches as f64 / min_len as f64
}
