use super::VcfLocalContainerSmokeRow;
pub(super) fn render_vcf_local_container_smoke_tsv(rows: &[VcfLocalContainerSmokeRow]) -> String {
    let mut rendered = String::from(
        "stage_id\ttool_id\tregistered_binary\ttool_status\tstage_support_status\tscope_state\tscope_detail\tsmoke_path_kind\tsmoke_runtime\tsmoke_tool_id\tsmoke_command\tsmoke_support_path\tsmoke_minimal_cmd\treason\n",
    );
    for row in rows {
        rendered.push_str(&row.stage_id);
        rendered.push('\t');
        rendered.push_str(&row.tool_id);
        rendered.push('\t');
        rendered.push_str(&row.registered_binary);
        rendered.push('\t');
        rendered.push_str(&row.tool_status);
        rendered.push('\t');
        rendered.push_str(&row.stage_support_status);
        rendered.push('\t');
        rendered.push_str(&row.scope_state);
        rendered.push('\t');
        rendered.push_str(&row.scope_detail);
        rendered.push('\t');
        rendered.push_str(&row.smoke_path_kind);
        rendered.push('\t');
        rendered.push_str(&row.smoke_runtime);
        rendered.push('\t');
        rendered.push_str(&row.smoke_tool_id);
        rendered.push('\t');
        rendered.push_str(&row.smoke_command);
        rendered.push('\t');
        rendered.push_str(&row.smoke_support_path);
        rendered.push('\t');
        rendered.push_str(&row.smoke_minimal_cmd);
        rendered.push('\t');
        rendered.push_str(&row.reason);
        rendered.push('\n');
    }
    rendered
}
