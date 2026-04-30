use std::path::PathBuf;

use anyhow::Result;
use bijux_dna_domain_compiler::{
    build_domain_registry_bundle, load_domain_registry_bundle, query_domain_registry_bundle,
    DomainRegistryQuery, DomainRegistryQueryKind, DEFAULT_DOMAIN_DIR,
};
use clap::{Parser, ValueEnum};

#[derive(Debug, Clone, Copy, ValueEnum)]
enum QueryKindArg {
    Domains,
    Stages,
    Tools,
    Metrics,
    Artifacts,
    Defaults,
    Deprecations,
    Fixtures,
}

impl From<QueryKindArg> for DomainRegistryQueryKind {
    fn from(value: QueryKindArg) -> Self {
        match value {
            QueryKindArg::Domains => Self::Domains,
            QueryKindArg::Stages => Self::Stages,
            QueryKindArg::Tools => Self::Tools,
            QueryKindArg::Metrics => Self::Metrics,
            QueryKindArg::Artifacts => Self::Artifacts,
            QueryKindArg::Defaults => Self::Defaults,
            QueryKindArg::Deprecations => Self::Deprecations,
            QueryKindArg::Fixtures => Self::Fixtures,
        }
    }
}

#[derive(Parser, Debug)]
#[command(name = "domain_registry_query")]
struct Args {
    #[arg(long)]
    bundle: Option<PathBuf>,
    #[arg(long, default_value = DEFAULT_DOMAIN_DIR)]
    domain_dir: PathBuf,
    #[arg(long, value_enum, default_value_t = QueryKindArg::Domains)]
    kind: QueryKindArg,
    #[arg(long)]
    domain: Option<String>,
    #[arg(long)]
    stage_id: Option<String>,
    #[arg(long)]
    tool_id: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let bundle = if let Some(path) = args.bundle.as_deref() {
        load_domain_registry_bundle(path)?
    } else {
        build_domain_registry_bundle(&args.domain_dir, "workspace-local")?
    };
    let result = query_domain_registry_bundle(
        &bundle,
        &DomainRegistryQuery {
            kind: args.kind.into(),
            domain_id: args.domain,
            stage_id: args.stage_id,
            tool_id: args.tool_id,
        },
    );
    println!("{}", serde_json::to_string_pretty(&result)?);
    Ok(())
}
