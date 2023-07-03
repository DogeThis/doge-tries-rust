use std::path::PathBuf;

use clap::{Parser};

use astra_formats::{Bundle, BundleFile, Asset};

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
            Ok(bundle) => inspect_bundle(&bundle),
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn inspect_bundle (bundle: &Bundle) {
    let cab = bundle.get_cab();
    let whatever = bundle.files();
    for (_what, bundle_file) in whatever {
        match bundle_file {
        BundleFile::Assets(asset_file) => {
                
            asset_file.assets.iter().for_each(|asset| {
                    match asset {
                        Asset::Material(material) => {
                            let shader = &material.shader;
                            println!("Shader: {:#?}", shader);
                        },
                        Asset::Bundle(bundle) => {
                            println!("Bundle name: {:#?}", bundle.name);
                            println!("Bundle deps: {:#?}", bundle.dependencies);
                        },
                        _ => {} // println!("Other don't care"),
                    }
                });
            },
            _ => {} //println!("whatever"),
        }
    }
    println!("CAB: {:#?}", cab);
    // println!("Bundle: {:#?}", bundle);
}