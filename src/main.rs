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
            Ok(bundle) => inspect_bundle(bundle),
            Err(err) => println!("Error: {}", err),
        }
    }
}

fn inspect_bundle (mut bundle: Bundle) {
    let cab = bundle.get_cab();
    let whatever = bundle.files();

    for (what, bundle_file) in whatever {
        println!("what is this: {:#?}", what);
        match bundle_file {
        BundleFile::Assets(asset_file) => {
                
            let my_materials = asset_file.assets.iter().filter_map(|asset| {
                match asset {
                    Asset::Material(material) => Some(material),
                    _ => None,
                }
            });

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
    let mutbundle: Option<&mut BundleFile> = bundle.get_mut("CAB-d14402932a96240520bc659742cab91e");
    if let Some(bundle_file) = mutbundle {
        println!("hi let's mutate this thing");
        match bundle_file {
            BundleFile::Assets(asset_file) => {
                println!("hello");
                asset_file.assets.iter_mut().for_each(|asset| {
                    match asset {
                        Asset::Material(material) => {
                            let shader = &mut material.shader;
                            println!("Shader: {:#?}", shader);
                            shader.file_id = 1337;
                            shader.path_id = 42;
                        },
                        Asset::Bundle(bundle) => {
                            println!("Bundle name: {:#?}", bundle.name);
                            println!("Bundle deps: {:#?}", bundle.dependencies);
                        },
                        _ => {} // println!("Other don't care"),
                    }
                });
            },
            _ => {}
        }
    }
    // bundle.save("hithere").expect("whateve");
    // println!("CAB: {:#?}", cab);
    // println!("Bundle: {:#?}", bundle);
}