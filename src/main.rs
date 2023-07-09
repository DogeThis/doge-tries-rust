use std::path::PathBuf;

use clap::{Parser};

use astra_formats::{Bundle, BundleFile, Asset, UString};

use std::collections::HashMap;

use serde_derive::Deserialize;
use toml;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(long)]
    /// toml file with dependencies to patch
    dependencies: PathBuf,

    #[arg(short, long)]
    /// your custom unity bundle you want to patch (this file will be read but not changed)
    target_bundle_path: PathBuf,

    #[arg(short, long)]
    /// output path for the patched bundle
    output_path: Option<PathBuf>,
    
    #[clap(long, short, action)]
    /// dry run, don't save the bundle, just for testing purposes to see if the dependencies are found
    dry_run: bool,
}

fn main() {
    let cli = Cli::parse();
    let bundle = Bundle::load(cli.target_bundle_path);

    let dependencies = std::fs::read_to_string(cli.dependencies).expect("Could not read dependencies file");
    let dependencies = toml::from_str::<DependenciesVec>(dependencies.as_str()).expect("Could not parse dependencies file");

    println!("Dependencies understood: {:#?}", dependencies);

    match bundle {
        Ok(bundle) => make_bundle_compatible(bundle, cli.output_path, cli.dry_run, dependencies.dependencies),
        Err(err) => println!("Error: {}", err),
    }
}

#[derive(Debug, Deserialize)]
struct DependenciesVec {
    dependencies: Vec<Dependency>
}

#[derive(Debug)]
#[derive(Deserialize)]
struct Dependency {
    path: String,
    game_node: DependencyNode,
    custom_bundle_node: DependencyNode,
}

#[derive(Debug)]
#[derive(Deserialize)]
struct DependencyNode {
    cab: String,
    path_id: i64
}

impl DependencyNode {
    fn get_file_path(&self) -> String {
        format!("archive:/{cab}/{cab}", cab = self.cab)
    }
}

fn make_bundle_compatible (mut bundle: Bundle, output_file: Option<PathBuf>, dry_run: bool, dependecies_to_fix: Vec<Dependency>) {
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
                           for element in material.saved_properties.text_envs.iter_mut() {
                            let matching_dependency = dependecies_to_fix.iter().find(|dependency| {
                                element.1.texture.path_id == dependency.custom_bundle_node.path_id
                            });
                            if let Some(match_found) = matching_dependency {
                                println!("Found a matching texture dependency: {:#?}", match_found);
                                element.1.texture.path_id = match_found.game_node.path_id;
                            } else {
                                println!("No matching texture dependency found for: {:#?}", element.1.texture);
                            }
                        }
                        },
                        _ => {}
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