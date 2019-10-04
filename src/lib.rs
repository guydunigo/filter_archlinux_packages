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

const _TEST_NAME: &str = "/mnt/arch/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_2: &str = "/mnt/archlinux/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_3: &str = "/mnt/archlinux/zeitgeist-1.0+1+g1bcc8585-1-x86_64.pkg.tar.xz";

pub fn remove_old_archlinux_packages<P: AsRef<Path>>(containing_dir_path: P) -> io::Result<()> {
    let old_files = list_old_archlinux_packages(containing_dir_path)?;
    remove_files(old_files)
}

/// Returns a list of all archlinux packages in `containing_dir_path` if there is a newer version
/// also present.
/// `containing_dir_path` should be a path to a **directory**.
fn list_old_archlinux_packages<P: AsRef<Path>>(containing_dir_path: P) -> io::Result<Vec<PathBuf>> {
    let mut pkgs = HashMap::new();
    let mut old_pkgs = Vec::new();

    for entry in read_dir(containing_dir_path)? {
        let entry_path = entry?.path();
        if !entry_path.is_file() {
            continue;
        }

        let entry_name = entry_path
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        let pkg = match Package::from_path(entry_path) {
            Ok(pkg) => pkg,
            Err(_) => {
                ignore(&entry_name);
                continue;
            }
        };

        if pkgs.contains_key(&pkg.name) {
            let existing_pkg = pkgs.get(&pkg.name).unwrap();
            match Package::compare_versions(&pkg, &existing_pkg) {
                Ordering::Greater => {
                    eprintln!(
                        "=====> Keeping ver `{}` over `{}`.",
                        pkg.pkgver, existing_pkg.pkgver
                    );
                    let existing_pkg = pkgs.insert(pkg.name.clone(), pkg).unwrap();
                    old_pkgs.push(existing_pkg.path);
                }
                Ordering::Less => {
                    eprintln!(
                        "=====> Keeping ver `{}` over `{}`.",
                        existing_pkg.pkgver, pkg.pkgver
                    );
                    old_pkgs.push(pkg.path);
                }
                Ordering::Equal => ignore(pkg.path.file_name().unwrap().to_str().unwrap()),
            }
        } else {
            pkgs.insert(pkg.name.clone(), pkg);
        }
    }

    Ok(old_pkgs)
}

fn ignore(path: &str) {
    eprintln!("... ignoring `{}`", path);
}

pub fn remove_files<P: AsRef<Path>>(files: Vec<P>) -> io::Result<()> {
    println!("{} files to remove...", files.len());
    for file in files.iter() {
        println!(
            "Removing file: `{}`",
            file.as_ref().file_name().unwrap().to_str().unwrap()
        );
        //remove_file(file)?;
    }
    Ok(())
}
