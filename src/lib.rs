extern crate regex;
extern crate version_compare;
#[macro_use]
extern crate lazy_static;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::read_dir;
use std::fs::remove_file;
use std::io;
use std::path::{Path, PathBuf};

mod package;
use package::Package;

const DEBUG_VERSIONS_COMPARISON: bool = false;

const _TEST_NAME: &str = "/mnt/arch/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_2: &str = "/mnt/archlinux/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_3: &str = "/mnt/archlinux/zeitgeist-1.0+1+g1bcc8585-1-x86_64.pkg.tar.xz";

pub fn remove_old_archlinux_packages<P: AsRef<Path>>(containing_dir_path: P) -> io::Result<()> {
    let (old_pkgs, ignored_files) = list_old_archlinux_packages(&containing_dir_path)?;
    remove_files(old_pkgs)?;
    list_ignored_files(&ignored_files);
    Ok(())
}

/// Returns a list of all archlinux packages in `containing_dir_path` if there is a newer version
/// also present.
/// `containing_dir_path` should be a path to a **directory**.
/// Returns : `(old_pkgs, ignored_files)` where:
///     - `old_pkgs` are the packages that have a newer version
///     - `ignored_files` are the files ignored because of ambiguous version number or non-package
fn list_old_archlinux_packages<P: AsRef<Path>>(
    containing_dir_path: &P,
) -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    let mut old_pkgs = Vec::new();
    let mut new_pkgs: HashMap<String, Package> = HashMap::new();
    let mut ignored_files = Vec::new();

    for entry in read_dir(containing_dir_path)? {
        let entry_path = entry?.path();
        if !entry_path.is_file() {
            continue;
        } else if ignored_files.contains(&entry_path) {
            continue;
        }

        let pkg = match Package::from_path(entry_path) {
            Ok(pkg) => pkg,
            Err((_, entry_path)) => {
                ignored_files.push(entry_path);
                continue;
            }
        };

        if new_pkgs.contains_key(&pkg.name) {
            let existing_pkg = new_pkgs.get(&pkg.name).unwrap();

            if pkg.path == existing_pkg.path {
                continue;
            }

            match Package::compare_versions(&pkg, &existing_pkg) {
                Ordering::Greater => {
                    if DEBUG_VERSIONS_COMPARISON {
                        eprintln!(
                            "=====> Keeping ver `{}` over `{}`.",
                            pkg.pkgver, existing_pkg.pkgver
                        );
                    }
                    let existing_pkg = new_pkgs.insert(pkg.name.clone(), pkg).unwrap();
                    old_pkgs.push(existing_pkg.path);
                }
                Ordering::Less => {
                    if DEBUG_VERSIONS_COMPARISON {
                        eprintln!(
                            "=====> Keeping ver `{}` over `{}`.",
                            existing_pkg.pkgver, pkg.pkgver
                        );
                    }
                    old_pkgs.push(pkg.path);
                }
                Ordering::Equal => ignored_files.push(pkg.path),
            }
        } else {
            new_pkgs.insert(pkg.name.clone(), pkg);
        }
    }

    Ok((old_pkgs, ignored_files))
}

fn list_ignored_files(ignored_files: &Vec<PathBuf>) {
    println!("{} files ignored...", ignored_files.len());
    ignored_files
        .iter()
        .map(|path| path.to_str().unwrap())
        .for_each(|path| eprintln!("... ignoring `{}`", path));
}

fn remove_files<P: AsRef<Path>>(files: Vec<P>) -> io::Result<()> {
    println!("{} files to remove...", files.len());
    for file in files.iter() {
        println!(
            "... removing `{}`",
            file.as_ref().file_name().unwrap().to_str().unwrap()
        );
        //remove_file(file)?;
    }
    Ok(())
}
