use std::path::PathBuf;

use clap::{Parser};

use astra_formats::Bundle;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    bundle: Option<PathBuf>,
}

fn main() {
    let cli = Cli::parse();

    if let Some(bundle_path) = cli.bundle.as_deref() {
        println!("Value for bundle_path: {}", bundle_path.display());
        let bundle = Bundle::load(bundle_path);
        match bundle {
            Ok(bundle) => println!("Bundle: {:#?}", bundle),
            Err(err) => println!("Error: {}", err),
        }
    }
}