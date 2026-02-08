use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "bijux-dna-image-qa", version, about = "Bijux image QA")]
struct Args {
    #[arg(long)]
    platform: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    bijux_dna_environment_qa::image_qa::run_image_qa(args.platform.as_deref())
}
