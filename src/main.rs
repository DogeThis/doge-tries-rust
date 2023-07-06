use std::path::PathBuf;

use clap::{Parser};

use astra_formats::{Bundle, BundleFile, Asset, UString};

use std::collections::HashMap;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(short, long)]
    target_bundle_path: PathBuf,

    #[arg(short, long)]
    output_path: Option<PathBuf>,
    
    #[clap(long, short, action)]
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();
    let bundle = Bundle::load(cli.target_bundle_path);
    match bundle {
        Ok(bundle) => make_bundle_compatible(bundle, cli.output_path, cli.dry_run),
        Err(err) => println!("Error: {}", err),
    }
}

#[derive(Debug)]
struct Dependency {
    path: String,
    game_node: DependencyNode,
    custom_bundle_node: DependencyNode,
}

#[derive(Debug)]
struct DependencyNode {
    cab: String,
    path_id: i64
}

impl DependencyNode {
    fn get_file_path(&self) -> String {
        format!("archive:/{cab}/{cab}", cab = self.cab)
    }
}

fn make_bundle_compatible (mut bundle: Bundle, output_file: Option<PathBuf>, dry_run: bool) {
    // Hardcode this for testing
    let char_standard = Dependency {
        path: r"StreamingAssets\aa\Switch\fe_assets_customrp\shaders\chara\charastandard.shader.bundle".to_string(),
        game_node: DependencyNode { // the game's version
            cab: "CAB-8b98d58e992699f07f87c0943f678979".to_string(),
            path_id: -3637526271425215770
        },
        custom_bundle_node: DependencyNode { // our fake version
            cab: "CAB-237730efbe63b97e4798d3f981576779".to_string(),
            path_id: 6103793082863834008
        }
    };

    let dependecies_to_fix = vec![char_standard];

    let cab = bundle.get_cab().unwrap().to_string();
    let mutbundle: Option<&mut BundleFile> = bundle.get_mut(cab.as_str());

    if let Some(bundle_file) = mutbundle {
        match bundle_file {
            BundleFile::Assets(asset_file) => {
                asset_file.externals.iter_mut().for_each(|external| {
                    // find a dependency that matches this external's 
                    let matching_dependency = dependecies_to_fix.iter().find(|dependency| {
                        dependency.custom_bundle_node.get_file_path() == external.path.to_string()
                    });
                    if let Some(dependency) = matching_dependency {
                        println!("Found a matching dependency: {:#?}", dependency);
                        external.path = dependency.game_node.get_file_path().into();
                    } else {
                        println!("No matching dependency found for: {:#?}", external);
                    }
                });
                asset_file.assets.iter_mut().for_each(|asset| {
                    match asset {
                        Asset::Material(material) => {
                            let shader = &mut material.shader;
                            // Find the dependency that matches this shader
                            let matching_dependency = dependecies_to_fix.iter().find(|dependency| {
                                dependency.custom_bundle_node.path_id == shader.path_id
                            });
                            if let Some(match_found) = matching_dependency {
                                println!("Found a matching shader dependency: {:#?}", match_found);
                                shader.path_id = match_found.game_node.path_id;
                            } else {
                                println!("No matching shader dependency found for: {:#?}", shader);
                            }
                        },     
                        _ => {},
                    }
                });
            },
            _ => {}
        }
    }
    if dry_run {
        println!("Dry run, not saving");
    } else {
        if let Some(output_file) = output_file {
            bundle.save(output_file).expect("Could not save bundle for some reason...");
        } else {
            println!("No output file specified, saving to output.bundle");
            bundle.save("output.bundle").expect("Could not save bundle for some reason...");
        }
    }
}