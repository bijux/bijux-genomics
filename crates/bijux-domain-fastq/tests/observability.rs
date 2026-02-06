use bijux_core::foundation::input_assessment::InputAssessmentV1;

#[test]
fn input_assessment_v1_requires_fields() {
    let json = "{}";
    assert!(serde_json::from_str::<InputAssessmentV1>(json).is_err());
}
