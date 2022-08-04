use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process::exit;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Name of the jdeps file to parse
    #[clap(short, long, value_parser)]
    jdeps_path: String,

    /// Name of the package root you want to see results for
    #[clap(short, long, value_parser)]
    package: String,

    /// Only include packages that contain this value. E.g. -i io.wisesystems
    /// will only show packages that contain io.wisesystems in the result
    #[clap(short, long, value_parser)]
    include: Option<String>,
}

#[derive(Debug, Serialize)]
struct Package {
    child_of: HashSet<String>,
    parent_of: HashSet<String>,
}

#[derive(Debug, Serialize)]
struct ExpandedPackage {
    circular_with: HashSet<String>,
    name: String,
    deps: Vec<ExpandedPackage>,
}

type Packages = HashMap<String, Package>;

fn main() {
    let args = Args::parse();

    let file = File::open(args.jdeps_path).unwrap();
    let reader = BufReader::new(file);
    let mut packages = Packages::new();

    for line in reader.lines() {
        let line = line.unwrap();
        let mut dep_line = line.trim().split("->");
        let package_name = dep_line.next().unwrap().trim().to_string();

        match &args.include {
            None => {}
            Some(include) => {
                if !package_name.contains(include) {
                    continue;
                }
            }
        }

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

        match &args.include {
            None => {}
            Some(include) => {
                if !child.contains(include) {
                    continue;
                }
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

    // println!("PACKAGES: {:#?}", packages);
    let results_file = io::stdout();
    let expanded_package = expand_package(&args.package, &packages);
    // serde_json::to_writer_pretty(&results_file, &packages).unwrap();
    serde_json::to_writer_pretty(&results_file, &expanded_package).unwrap();
    // println!("EXPANDED PACKAGE: {:#?}", expanded_package);
}

fn expand_package(package_name: &String, packages: &Packages) -> ExpandedPackage {
    let package = match packages.get(package_name) {
        None => panic!("Invalid package provided {}", package_name),
        Some(p) => p,
    };

    let circ_deps_iter: HashSet<_> = package.parent_of.intersection(&package.child_of).collect();
    let mut circular_deps: HashSet<String> = HashSet::new();
    for dep in circ_deps_iter {
        // eprintln!(
        //     "PACKAGE {} HAS CIRCULAR DEPENDENCY WITH {}",
        //     package_name, dep
        // );
        circular_deps.insert(dep.clone());
    }

    let mut expanded_package = ExpandedPackage {
        circular_with: circular_deps.clone(),
        name: package_name.clone(),
        deps: vec![],
    };

    for dep in package.parent_of.iter() {
        if !circular_deps.contains(dep) {
            let expanded_dep = expand_package(dep, packages);
            expanded_package.deps.push(expanded_dep);
        }
    }

    expanded_package
}
