use bijux_dna_containers::evidence::{
    ContainerImageInventoryPublication, ContainerImageState, ContainerSupportDisposition,
};

pub(super) fn markdown(inventory: &ContainerImageInventoryPublication) -> String {
    let summary = &inventory.summary;
    let states = summary
        .states
        .iter()
        .map(|(state, count)| format!("**{state} {count}**"))
        .collect::<Vec<_>>()
        .join(", ");
    let mut output = format!(
        "<!-- GENERATED FILE - DO NOT EDIT -->\n<!-- source: bijux-dna-dev containers generate-image-inventory-report -->\n# LUNARC Apptainer image inventory\n\nThis is an integrity and provenance inventory, not a release certificate.\nAn image is current only when its SIF hash, immutable filename, build evidence,\nrecipe-at-commit hash, and active recipe hash agree. Scientific and security\nqualification remain independent requirements.\n\n## Inventory summary\n\n- Catalog tools: **{}**\n- States: {}\n- Historical run dependencies: **{}**\n- Unknown immutable images: **{}**\n- Interrupted staged images: **{}**\n- Unreferenced obsolete images: **{}**\n\n## Catalog coverage\n\n| Tool | Version | Catalog disposition | Image state | Real-run references |\n|---|---|---|---|---:|\n",
        summary.tool_count,
        states,
        summary.historical_dependency_count,
        summary.unknown_image_count,
        summary.staged_image_count,
        summary.unreferenced_obsolete_count,
    );
    for tool in &inventory.tools {
        let references =
            tool.candidates.iter().map(|candidate| candidate.real_run_references).sum::<u64>();
        output.push_str(&format!(
            "| `{}` | `{}` | {} | **{}** | {references} |\n",
            markdown_code(&tool.tool),
            markdown_code(&tool.version),
            support_disposition(&tool.support_disposition),
            image_state(&tool.state),
        ));
    }
    output.push_str(
        "\n`missing` means no admitted SIF exists for the catalog version. `ambiguous`\nmeans admitted candidates exist but none can be selected uniquely. `historical`\nmeans one admitted image is selected but does not represent the active recipe.\n`unsupported` is an explicit catalog decision, not a failed build.\nA historical run dependency is preserved because a final governed run records\nits immutable path and digest; it is not current support or release evidence.\n\nThe machine-readable companion is [image-inventory.json](https://github.com/bijux/bijux-genomics/blob/main/containers/reports/image-inventory.json).\n",
    );
    output
}

fn support_disposition(disposition: &ContainerSupportDisposition) -> &'static str {
    match disposition {
        ContainerSupportDisposition::Retained => "retained",
        ContainerSupportDisposition::Unsupported => "unsupported",
        ContainerSupportDisposition::Retired => "retired",
        ContainerSupportDisposition::Excluded => "excluded",
    }
}

fn image_state(state: &ContainerImageState) -> &'static str {
    match state {
        ContainerImageState::Current => "current",
        ContainerImageState::Historical => "historical",
        ContainerImageState::Ambiguous => "ambiguous",
        ContainerImageState::Missing => "missing",
        ContainerImageState::Unsupported => "unsupported",
    }
}

fn markdown_code(value: &str) -> String {
    value.replace('|', "\\|").replace('`', "&#96;")
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;

    use bijux_dna_containers::evidence::{
        ContainerImageInventoryPublication, ContainerImageInventorySummary,
        ContainerImageInventoryTool, ContainerImageState, ContainerSupportDisposition,
        ImageInventoryPublicationIdentity,
    };

    use super::markdown;

    #[test]
    fn report_separates_inventory_from_qualification() {
        let publication = ContainerImageInventoryPublication {
            schema_version: "bijux.container.image_inventory_publication.v1".to_string(),
            catalog_sha256: "a".repeat(64),
            repository_commit: "b".repeat(40),
            publication: ImageInventoryPublicationIdentity {
                raw_schema_version: "bijux.container.image_inventory_scan.v1".to_string(),
                raw_sha256: "c".repeat(64),
            },
            summary: ContainerImageInventorySummary {
                tool_count: 1,
                states: BTreeMap::from([("missing".to_string(), 1)]),
                historical_dependency_count: 0,
                unknown_image_count: 0,
                staged_image_count: 0,
                unreferenced_obsolete_count: 0,
            },
            tools: vec![ContainerImageInventoryTool {
                tool: "samtools".to_string(),
                version: "1.20".to_string(),
                support_disposition: ContainerSupportDisposition::Retained,
                apptainer_supported: true,
                state: ContainerImageState::Missing,
                selected_image_sha256: None,
                candidates: Vec::new(),
            }],
            historical_images: Vec::new(),
            unknown_images: Vec::new(),
            staged_images: Vec::new(),
        };

        let report = markdown(&publication);

        assert!(report.contains("not a release certificate"));
        assert!(report.contains("Scientific and security"));
        assert!(report.contains("| `samtools` | `1.20` | retained | **missing** | 0 |"));
        assert!(report.ends_with('\n'));
    }
}
