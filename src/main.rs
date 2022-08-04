use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process::exit;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the jdeps file to parse
    #[clap(long, value_parser)]
    path: String,

    #[clap(short, long, value_parser)]
    package: String,
}

#[derive(Debug, Serialize)]
struct Package {
    child_of: HashSet<String>,
    parent_of: HashSet<String>,
}

type Packages = HashMap<String, Package>;

fn main() {
    let args = Args::parse();

    let file = File::open(args.path).unwrap();
    let reader = BufReader::new(file);
    let mut packages = Packages::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let mut dep_line = line.trim().split("->");
        let package_name = dep_line.next().unwrap().trim().to_string();

        if package_name.ends_with(".jar") {
            continue;
        }

        let raw_child = dep_line.next().unwrap().trim();

        let mut child: &str = "";
        for (i, c) in raw_child.chars().enumerate() {
            if c.is_whitespace() {
                // first whitespace means we're done with the dep_name
                child = raw_child[..i].trim();
                break;
            }
        }

        {
            let maybe_package = packages.get_mut(&package_name);

            match maybe_package {
                None => {
                    let mut new_parent = HashSet::new();
                    new_parent.insert(child.to_string());
                    packages.insert(
                        package_name.clone(),
                        Package {
                            child_of: HashSet::new(),
                            parent_of: new_parent,
                        },
                    );
                }
                Some(package) => {
                    package.parent_of.insert(child.to_string());
                }
            }
        }

        {
            let maybe_child_package = packages.get_mut(child);

            match maybe_child_package {
                None => {
                    let mut new_child = HashSet::new();
                    new_child.insert(package_name);
                    packages.insert(
                        child.to_string(),
                        Package {
                            child_of: new_child,
                            parent_of: HashSet::new(),
                        },
                    );
                }
                Some(package) => {
                    package.child_of.insert(package_name.clone());
                }
            }
        }
    }
    // let results_file = io::stdout();
    // serde_json::to_writer_pretty(&results_file, &packages).unwrap();
    println!("PACKAGES: {:#?}", packages);
}
