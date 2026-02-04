use anyhow::Result;
use clap::Parser;

#[derive(Debug, Parser)]
#[command(name = "bijux-image-qa", version, about = "Bijux image QA")]
struct Args {
    #[arg(long)]
    platform: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    bijux_api::v1::run::run_image_qa(args.platform.as_deref())
}
