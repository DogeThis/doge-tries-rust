use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

use astra_formats::{Asset, AssetFile, Bundle, BundleFile, UString};

use std::collections::HashMap;

use serde_derive::Deserialize;
use toml;

use walkdir::WalkDir;

#[derive(Subcommand)]
enum Commands {
    /// The old migration method we used to use with a toml file
    MigrateOld(MigrateOldArgs),
    Crawl(CrawlArgs),
}

#[derive(Args)]
struct CrawlArgs {
    /// Target path of the which will be crawled, contains an aa folder
    #[arg(short, long)]
    target_path: PathBuf,
}

#[derive(Args)]
struct MigrateOldArgs {
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

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

fn main() {
    let cli = Cli::parse();
    match &cli.command {
        Commands::MigrateOld(args) => {
            let bundle = Bundle::load(&args.target_bundle_path);

            let dependencies = std::fs::read_to_string(&args.dependencies)
                .expect("Could not read dependencies file");
            let dependencies = toml::from_str::<DependenciesVec>(dependencies.as_str())
                .expect("Could not parse dependencies file");

            println!("Dependencies understood: {:#?}", dependencies);

            match bundle {
                Ok(bundle) => make_bundle_compatible(
                    bundle,
                    &args.output_path,
                    args.dry_run,
                    dependencies.dependencies,
                ),
                Err(err) => println!("Error: {}", err),
            }
        }
        Commands::Crawl(args) => {
            println!("Crawling: {:#?}", args.target_path);
            let aa_path = args.target_path.join("aa");
            let aa_path_exists = std::path::Path::new(&aa_path).exists();
            if !aa_path_exists {
                println!("aa folder does not exist, exiting");
                return;
            }
            let mut dependencies: Vec<DependencyNode> = Vec::new();

            for entry in WalkDir::new(&aa_path) {
                let path = entry.unwrap();
                // if the path ends in .bundle, let's try to do something with it
                let name = path.file_name();
                let name = name.to_str().unwrap();
                if name.ends_with(".bundle") {
                    // println!("Found bundle: {}", name);
                    let bundle = Bundle::load(path.path());
                    match bundle {
                        Ok(bundle) => {
                            // println!("Bundle loaded: {:#?}", bundle.get_cab());
                            bundle
                                .files()
                                .into_iter()
                                .for_each(|(name, bundle_file)| -> () {
                                    match bundle_file {
                                        BundleFile::Assets(hello) => {
                                            hello.assets.iter().for_each(|asset| match asset {
                                                // Asset::Material(material) => {
                                                //     println!("Material: {:#?}", material);
                                                //     material
                                                //         .saved_properties
                                                //         .text_envs
                                                //         .iter()
                                                //         .for_each(|(key, value)| {
                                                //             println!(
                                                //                 "Key: {:#?}, Value: {:#?}",
                                                //                 key, value
                                                //             );
                                                //         });
                                                // }
                                                Asset::Bundle(asset_bundle) => {
                                                    println!("Bundle: {:#?}", asset_bundle);
                                                    // asset_bundle.

                                                    let cab = bundle
                                                        .get_cab()
                                                        .unwrap()
                                                        .to_string()
                                                        .into();
                                                    let mut path_id = 0;
                                                    asset_bundle
                                                        .container_map
                                                        .items
                                                        .iter()
                                                        .for_each(|(key, value)| {
                                                            path_id = value.asset.path_id;
                                                        });
                                                    let dependency =
                                                        DependencyNode { cab, path_id };
                                                    dependencies.push(dependency);
                                                }
                                                _ => {}
                                            })
                                        }
                                        _ => {}
                                    }
                                })
                        }
                        Err(err) => println!("Error: {}", err),
                    }
                }
            }
            println!("Found dependencies: {:#?}", dependencies);
        }
    }
}

#[derive(Debug, Deserialize)]
struct DependenciesVec {
    dependencies: Vec<Dependency>,
}

#[derive(Debug, Deserialize)]
struct Dependency {
    path: String,
    game_node: DependencyNode,
    custom_bundle_node: DependencyNode,
}

#[derive(Debug, Deserialize)]
struct DependencyNode {
    cab: String,
    path_id: i64,
}

impl DependencyNode {
    fn get_file_path(&self) -> String {
        format!("archive:/{cab}/{cab}", cab = self.cab)
    }
}

fn make_bundle_compatible(
    mut bundle: Bundle,
    output_file: &Option<PathBuf>,
    dry_run: bool,
    dependecies_to_fix: Vec<Dependency>,
) {
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
                            let matching_dependency =
                                dependecies_to_fix.iter().find(|dependency| {
                                    dependency.custom_bundle_node.path_id == shader.path_id
                                });
                            if let Some(match_found) = matching_dependency {
                                println!("Found a matching shader dependency: {:#?}", match_found);
                                shader.path_id = match_found.game_node.path_id;
                            } else {
                                println!("No matching shader dependency found for: {:#?}", shader);
                            }
                            for element in material.saved_properties.text_envs.iter_mut() {
                                let matching_dependency =
                                    dependecies_to_fix.iter().find(|dependency| {
                                        element.1.texture.path_id
                                            == dependency.custom_bundle_node.path_id
                                    });
                                if let Some(match_found) = matching_dependency {
                                    println!(
                                        "Found a matching texture dependency: {:#?}",
                                        match_found
                                    );
                                    element.1.texture.path_id = match_found.game_node.path_id;
                                } else {
                                    println!(
                                        "No matching texture dependency found for: {:#?}",
                                        element.1.texture
                                    );
                                }
                            }
                        }
                        _ => {}
                    }
                });
            }
            _ => {}
        }
    }
    if dry_run {
        println!("Dry run, not saving");
    } else {
        if let Some(output_file) = output_file {
            bundle
                .save(output_file)
                .expect("Could not save bundle for some reason...");
        } else {
            println!("No output file specified, saving to output.bundle");
            bundle
                .save("output.bundle")
                .expect("Could not save bundle for some reason...");
        }
    }
}
