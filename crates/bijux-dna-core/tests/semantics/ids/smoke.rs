use bijux_dna_core::ids::StageId;

#[test]
fn stage_id_from_static_round_trips() {
    let id = StageId::from_static("stage.sample");
    assert_eq!(id.as_str(), "stage.sample");
}
