extern crate regex;
extern crate version_compare;
#[macro_use]
extern crate lazy_static;
extern crate chrono;

use std::cmp::Ordering;
use std::collections::HashMap;
use std::fs::remove_file;
use std::fs::{metadata, read_dir};
use std::io;
use std::path::{Path, PathBuf};
use std::str::FromStr;

mod package;
use package::{Package, Packages};
mod options;
pub use options::{AutoConfirmLevel, Options};

// TODO: loglevel
const DEBUG_VERSIONS_COMPARISON: bool = false;

const _TEST_NAME: &str = "/mnt/archlinux/linux-5.3.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_2: &str = "/mnt/archlinux/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const _TEST_NAME_3: &str = "/mnt/archlinux/zeitgeist-1.0+1+g1bcc8585-1-x86_64.pkg.tar.xz";

pub fn remove_old_archlinux_packages(opts: Options) -> io::Result<()> {
    let (old_pkgs, ignored_files) = list_old_archlinux_packages(&opts)?;

    if opts.dry_run {
        list_removed_files(&old_pkgs);
        list_ignored_files(&ignored_files);
    } else {
        let input = if opts.auto_confirm_level.is_at_least_removal() {
            list_removed_files(&old_pkgs);
            list_ignored_files(&ignored_files);

            println!("\n------------");
            println!("Are you agreeing to these removals ? Type `y` and press enter if you do.");
            let mut input = String::new();
            if let Err(err) = io::stdin().read_line(&mut input) {
                // TODO: not panic ?
                panic!(
                    "EEE Can't read from input to ask anything to the user: {}",
                    err
                );
            }

            input == "y\n"
        } else {
            true
        };

        if input {
            remove_files(old_pkgs)?;

            // It has'n been shown before
            if !opts.auto_confirm_level.is_at_least_removal() {
                list_ignored_files(&ignored_files);
            }
        } else {
            println!("\n------------");
            println!("Abording : Not removing any file.");
        }
    }

    Ok(())
}

/// Returns a list of all archlinux packages in `dir` if there is a newer version
/// also present.
/// `dir` should be a path to an existing **directory**.
/// Returns : `(old_pkgs, ignored_files)` where:
///     - `old_pkgs` are the packages that have a newer version
///     - `ignored_files` are the files ignored because of ambiguous version number or non-package
fn list_old_archlinux_packages(opts: &Options) -> io::Result<(Vec<PathBuf>, Vec<PathBuf>)> {
    // TODO: assert dir.is_dir(), already done in read_dir ?
    let mut old_pkgs = Vec::new();
    let mut new_pkgs: HashMap<String, Packages> = HashMap::new();
    let mut ignored_files = Vec::new();
    let mut sig_files = Vec::new();

    // TODO: extract the function(s)
    // for entry in read_dir(&opts.dir)? {
    for entry in read_dir(&opts.dir)? {
        let entry_path = entry?.path();
        if !entry_path.is_file() || ignored_files.contains(&entry_path) {
            continue;
        }

        if entry_path.extension().map_or(false, |s| s == "sig") {
            sig_files.push(entry_path);
            continue;
        }

        let pkg = match Package::from_path(entry_path) {
            Ok(pkg) => pkg,
            Err((_, entry_path)) => {
                ignored_files.push(entry_path);
                continue;
            }
        };

        if let Some(existing_pkg) = new_pkgs.get_mut(&pkg.name) {
            if pkg.path == existing_pkg.path {
                continue;
            }

            match Package::compare_versions(&pkg, existing_pkg) {
                Ordering::Greater => {
                    if DEBUG_VERSIONS_COMPARISON {
                        eprintln!(
                            "=====> Keeping ver `{}` over `{}`.",
                            pkg.pkgver, existing_pkg.pkgver
                        );
                    }

                    // TODO: ideally add them back to this loop as long as there are any for better
                    // handling
                    let existing_pkg = new_pkgs
                        .insert(pkg.name.clone(), Packages::new(pkg))
                        .unwrap();
                    let pkg = new_pkgs.get_mut(&existing_pkg.name).unwrap();
                    for p in existing_pkg.into_iter() {
                        match Package::compare_versions(&p, pkg) {
                            Ordering::Less => old_pkgs.push(p.path),
                            Ordering::Greater => {
                                eprintln!("WWW Ambiguous package from older version is seen with greater version than the newer one has.");
                                pkg.add_ambiguity(p);
                            }
                            Ordering::Equal => pkg.add_ambiguity(p),
                        }
                    }
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
                Ordering::Equal => existing_pkg.add_ambiguity(pkg),
            }
        } else {
            new_pkgs.insert(pkg.name.clone(), Packages::new(pkg));
        }
    }

    let mut single_new_pkgs = Vec::with_capacity(new_pkgs.len());
    println!("\n------------");
    println!("Handling ambiguous versions...\n");
    for p in new_pkgs.into_values() {
        if !p.has_ambs() {
            single_new_pkgs.push(p.into_iter().next().unwrap());
        } else {
            let name = p.get_name().to_string();
            let mut ambs: Vec<_> = p.into_iter().collect();
            // index 0 should always exist
            println!("package `{}` has {} ambiguities :", name, ambs.len());
            // We get the "biggest" string on top.
            ambs.sort_by(|a, b| b.pkgver.cmp(&a.pkgver));
            ambs.iter().enumerate().rev().for_each(|(i, p)| {
                let date: chrono::DateTime<chrono::Local> =
                    chrono::DateTime::from(metadata(&p.path).unwrap().created().unwrap());
                println!("{:2}.\t{}\t(created {})", i, p.pkgver, date.to_rfc2822())
            });

            let number_opt = if !opts.auto_confirm_level.is_at_least_ambiguities() {
                // TODO: also the possibility to select all 0
                /*
                println!("> selecting index 0");
                Some(0)
                */
                println!("> keeping all");
                None
            } else {
                loop {
                    println!("> The index corresponding to the version to keep (default 0), or `i` to ignore :");
                    let mut input = String::new();
                    if let Err(err) = io::stdin().read_line(&mut input) {
                        // TODO: not panic ?
                        panic!(
                            "EEE Can't read from input to ask anything to the user: {}",
                            err
                        );
                    }
                    // We remove the line feed.
                    input.truncate(input.len() - 1);

                    if input.is_empty() {
                        break Some(0);
                    } else if input == "i" {
                        break None;
                    } else {
                        match usize::from_str(&input[..]) {
                            Err(err) => {
                                eprintln!(
                                    "WWW Can't parse input `{}` into number : {}",
                                    input, err
                                );
                            }
                            Ok(number) => {
                                if number < ambs.len() {
                                    break Some(number);
                                } else {
                                    eprintln!("WWW parsed number {} from `{}` is too high, please provide a number between 0 and {}.", number, input, ambs.len());
                                }
                            }
                        }
                    }
                }
            };

            if let Some(number) = number_opt {
                ambs.drain(..).enumerate().for_each(|(i, p)| {
                    if i == number {
                        single_new_pkgs.push(p)
                    } else {
                        old_pkgs.push(p.path)
                    }
                });
            } else {
                ignored_files.extend(ambs.drain(..).map(|p| p.path));
            }
        }
    }

    // If a sig file corresponds to an old package, we remove it as well, and if it doesn't
    // correpsond to a package to keep, we ignore it.
    for sig_path in sig_files.drain(..) {
        if old_pkgs.iter().any(|p| p.eq(&sig_path.with_extension(""))) {
            old_pkgs.push(sig_path);
        } else if !single_new_pkgs
            .iter()
            .map(|p| &p.path)
            .any(|p| p.eq(&sig_path.with_extension("")))
        {
            ignored_files.push(sig_path);
        }
    }

    // Ideally I might not sort them here as it is purely aesthetical, but for such a simple prog,
    // it's okay.
    old_pkgs.sort();
    ignored_files.sort();

    Ok((old_pkgs, ignored_files))
}

fn list_removed_files(files: &[PathBuf]) {
    println!("\n------------");
    println!("{} files about to be removed...\n", files.len());
    files
        .iter()
        .map(|path| path.to_str().unwrap())
        .for_each(|path| println!("{}", path));
}

fn list_ignored_files(ignored_files: &[PathBuf]) {
    println!("\n------------");
    println!("{} files ignored...\n", ignored_files.len());
    ignored_files
        .iter()
        .map(|path| path.to_str().unwrap())
        .for_each(|path| println!("{}", path));
}

fn remove_files<P: AsRef<Path>>(files: Vec<P>) -> io::Result<()> {
    println!("\n------------");
    println!("Actually removing {} files...\n", files.len());
    for file in files.iter() {
        println!("{}", file.as_ref().file_name().unwrap().to_str().unwrap());
        remove_file(file)?;
    }
    Ok(())
}
