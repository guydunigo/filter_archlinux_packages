extern crate remove_old_arch_pkgs;

use std::env::args;
use std::path::PathBuf;
use std::process::exit;

use remove_old_arch_pkgs::{remove_old_archlinux_packages, AutoConfirmLevel, Options};

const EXIT_UNKNOWN_OPT: i32 = 1;
const EXIT_NOT_A_DIR: i32 = 2;
const EXIT_IO_ERROR: i32 = 3;

fn main() {
    let mut opts = Options::default();
    let mut dir_given = false;

    for arg in args().skip(1) {
        match &arg[..] {
            "-h" => println!("Help"),
            "-d" => opts.dry_run = true,
            "-0" => opts.auto_confirm_level = AutoConfirmLevel::Nothing,
            "-1" => opts.auto_confirm_level = AutoConfirmLevel::Removal,
            "-2" => opts.auto_confirm_level = AutoConfirmLevel::Ambiguities,
            "-3" => opts.auto_confirm_level = AutoConfirmLevel::Everything,
            dir if dir.starts_with('-') => {
                eprintln!("Error: unknown option `{}`.", dir);
                exit(EXIT_UNKNOWN_OPT);
            }
            dir => {
                opts.dir = PathBuf::from(&dir);

                if !opts.dir.is_dir() {
                    eprintln!(
                        "Error: provided argument `{}` is not a directory or can't be found.",
                        dir
                    );
                    exit(EXIT_NOT_A_DIR);
                }

                dir_given = true;
            }
        }
    }

    if !dir_given {
        eprintln!("No folder was provided, using current working directory...");
    } else {
        eprintln!("Cleaning directory : {}", opts.dir.to_string_lossy());
    }
    eprintln!("Selected confirm level : {}...", opts.auto_confirm_level);
    if opts.dry_run {
        eprintln!("Dry run enabled, nothing will be deleted.");
    }

    if let Err(err) = remove_old_archlinux_packages(opts) {
        eprintln!("An io error occurred : `{}`", err);
        exit(EXIT_IO_ERROR);
    }
}
