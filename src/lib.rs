extern crate regex;
extern crate version_compare;
#[macro_use]
extern crate lazy_static;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::read_dir;
use std::io;
use std::path::Path;

mod package;
use package::Package;

const _TEST_NAME: &str = "/mnt/arch/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_2: &str = "/mnt/archlinux/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_3: &str = "/mnt/archlinux/zeitgeist-1.0+1+g1bcc8585-1-x86_64.pkg.tar.xz";

/// Returns a list of all archlinux packages in `containing_dir_path` if there is a newer version
/// also present.
/// `containing_dir_path` should be a path to a **directory**.
pub fn list_old_archlinux_packages<P: AsRef<Path>>(containing_dir_path: P) -> io::Result<Vec<P>> {
    let mut pkgs = HashMap::new();
    let mut old_pkgs = Vec::new();

    if !containing_dir_path.as_ref().is_dir() {
        panic!(
            "`{}` is not a directory.",
            containing_dir_path.as_ref().to_str().unwrap()
        );
    } else {
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
                    eprintln!(".. ignoring `{}`", entry_name);
                    continue;
                }
            };

            if pkgs.contains_key(&pkg.name) {
                let existing_pkg = pkgs.get(&pkg.name).unwrap();
                match Package::compare_versions(&pkg, &existing_pkg) {
                    Ordering::Greater => {
                        let existing_pkg = pkgs.insert(pkg.name.clone(), pkg).unwrap();
                        old_pkgs.push(existing_pkg);
                    }
                    Ordering::Less => {
                        old_pkgs.push(pkg);
                    }
                    Ordering::Equal => eprintln!(
                        "-> `{}` not deleted",
                        pkg.path.file_name().unwrap().to_str().unwrap()
                    ),
                }
            } else {
                pkgs.insert(pkg.name.clone(), pkg);
            }
        }
    }

    Ok(Vec::new())
}
