use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, prelude::*, BufReader};
use std::process::exit;
use std::time::Instant;

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
    child_of: Vec<String>,
    parent_of: Vec<String>,
}

#[derive(Debug, Serialize, Clone)]
struct ExpandedPackage {
    circular_with: Vec<String>,
    name: String,
    #[serde(skip_serializing)]
    children: Vec<String>,
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
                    let mut new_parent = Vec::new();
                    new_parent.push(child.to_string());
                    packages.insert(
                        package_name.clone(),
                        Package {
                            child_of: Vec::new(),
                            parent_of: new_parent,
                        },
                    );
                }
                Some(package) => {
                    package.parent_of.push(child.to_string());
                }
            }
        }

        {
            let maybe_child_package = packages.get_mut(child);

            match maybe_child_package {
                None => {
                    let mut new_child = Vec::new();
                    new_child.push(package_name);
                    packages.insert(
                        child.to_string(),
                        Package {
                            child_of: new_child,
                            parent_of: Vec::new(),
                        },
                    );
                }
                Some(package) => {
                    package.child_of.push(package_name.clone());
                }
            }
        }
    }

    // println!("PACKAGES: {:#?}", packages);
    let results_file = io::stdout();
    // let expanded_package = expand_package(&args.package, &packages);
    let expanded_package = expand_package_iter(&args.package, &packages);
    eprintln!("EXPANDED PACKAGE");
    // serde_json::to_writer_pretty(&results_file, &packages).unwrap();
    serde_json::to_writer_pretty(&results_file, &expanded_package).unwrap();
    // println!("EXPANDED PACKAGE: {:#?}", expanded_package);
}

struct Stack<T> {
    stack: Vec<T>,
}

impl<T> Stack<T> {
    fn new() -> Self {
        Stack { stack: Vec::new() }
    }
    fn length(&self) -> usize {
        self.stack.len()
    }
    fn pop(&mut self) -> Option<T> {
        self.stack.pop()
    }
    fn push(&mut self, item: T) {
        self.stack.push(item)
    }
    fn is_empty(&self) -> bool {
        self.stack.is_empty()
    }
    fn peek(&self) -> Option<&T> {
        self.stack.last()
    }
}

fn expand_package_iter(package_name: &String, packages: &Packages) -> ExpandedPackage {
    // returns an expanded package

    let mut fully_expanded_packages: HashMap<String, ExpandedPackage> = HashMap::new();
    let root_package = match packages.get(package_name) {
        None => panic!("Invalid package provided {}", package_name),
        Some(p) => p,
    };
    let mut stack: Stack<ExpandedPackage> = Stack::new();
    let root_circular_with = get_circular_deps(root_package);
    let mut children = vec![];
    for child in root_package.parent_of.iter() {
        if !root_circular_with.contains(child) {
            children.push(child.clone());
        }
    }
    let expanded_root_package = ExpandedPackage {
        circular_with: root_circular_with.clone(),
        name: package_name.clone(),
        children,
        deps: vec![],
    };
    // let final_answer: ExpandedPackage = ExpandedPackage {
    //     circular_with: root_circular_with,
    //     name: package_name.clone(),
    //     children: vec![],
    //     deps: vec![],
    // };

    stack.push(expanded_root_package);
    let mut count = 0;
    loop {
        let now = Instant::now();
        count += 1;
        if stack.is_empty() {
            // final_answer = current_expanded_package;
            break;
        }
        let mut current_expanded_package = stack.pop().unwrap();

        let package_name = &current_expanded_package.name;

        if current_expanded_package.children.len() == 0 {
            // all the children are expanded, now add them to deps
            let package = match packages.get(package_name) {
                None => panic!("Invalid package provided {}", package_name),
                Some(p) => p,
            };

            for dep_name in package.parent_of.iter() {
                let now = Instant::now();
                match fully_expanded_packages.get(dep_name) {
                    Some(dep_expanded_package) => {
                        current_expanded_package
                            .deps
                            .push(dep_expanded_package.clone());
                    }
                    None => {
                        // this means we likely hit one of the circular deps
                    }
                }
                let elapsed = now.elapsed();
                eprintln!("TIME FOR CLONING DEP {:.2?}", elapsed);
            }

            let now = Instant::now();
            fully_expanded_packages.insert(package_name.clone(), current_expanded_package.clone());
            let elapsed = now.elapsed();
            eprintln!("TIME FOR CLONING PACKAGE {:.2?}", elapsed);
            continue;
        }

        // we know there's at least 1 dep
        let dep_name = current_expanded_package.children.pop().unwrap();
        let dep_package = match packages.get(&dep_name) {
            None => panic!("Invalid package provided {}", dep_name),
            Some(p) => p,
        };
        let dep_circ_with = get_circular_deps(dep_package);
        let mut dep_children = vec![];
        for dep_child in dep_package.parent_of.iter() {
            if !dep_circ_with.contains(dep_child) {
                dep_children.push(dep_child.clone());
            }
        }
        let next_expanded_package = ExpandedPackage {
            circular_with: dep_circ_with,
            name: dep_name,
            children: dep_children,
            deps: vec![],
        };
        // now we add the current package back to the stack, then add the next
        // package to the stack
        stack.push(current_expanded_package);
        stack.push(next_expanded_package);
        let elapsed = now.elapsed();
        eprintln!("TIME FULL ITER {:.2?}", elapsed);
    }

    fully_expanded_packages.get(package_name).unwrap().clone()
}

fn get_circular_deps(package: &Package) -> Vec<String> {
    // let circ_deps_iter: HashSet<_> = package.parent_of.intersection(&package.child_of).collect();
    let mut circ_deps = vec![];
    for child_dep in &package.parent_of {
        if package.child_of.contains(child_dep) {
            circ_deps.push(child_dep.clone());
        }
    }
    circ_deps
}

fn expand_package(package_name: &String, packages: &Packages) -> ExpandedPackage {
    let package = match packages.get(package_name) {
        None => panic!("Invalid package provided {}", package_name),
        Some(p) => p,
    };

    // let circ_deps_iter: HashSet<_> = package.parent_of.intersection(&package.child_of).collect();
    // let mut circular_deps: Vec<String> = Vec::new();
    // for dep in circ_deps_iter {
    //     circular_deps.push(dep.clone());
    // }
    let circular_deps = get_circular_deps(package);

    let mut expanded_package = ExpandedPackage {
        circular_with: circular_deps.clone(),
        name: package_name.clone(),
        children: vec![],
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
