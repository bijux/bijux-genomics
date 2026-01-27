use anyhow::Result;
use clap::Parser;

#[allow(dead_code)]
#[path = "../image_qa/mod.rs"]
mod image_qa;
#[allow(dead_code)]
#[path = "../utils/mod.rs"]
mod utils;

#[derive(Debug, Parser)]
#[command(name = "bijux-image-qa", version, about = "Bijux image QA")]
struct Args {
    #[arg(long)]
    platform: Option<String>,
}

fn main() -> Result<()> {
    let args = Args::parse();
    image_qa::run_image_qa(args.platform.as_deref())
}
