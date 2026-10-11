use std::collections::BTreeSet;
pub(super) fn sorted_status_label(statuses: &BTreeSet<String>) -> String {
    statuses.iter().cloned().collect::<Vec<_>>().join(",")
}
