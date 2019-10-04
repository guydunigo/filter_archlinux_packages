extern crate filter_archlinux_packages;

use std::env::args;
use std::env::current_dir;
use std::path::PathBuf;
use std::process::exit;

use filter_archlinux_packages::list_old_archlinux_packages;

fn main() {
    let dir = if let Some(dir) = args().skip(1).next() {
        PathBuf::from(&dir)
    } else {
        eprintln!("No folder was provided, using current working directory...");
        current_dir().unwrap()
    };

    if dir.is_dir() {
        list_old_archlinux_packages(dir).unwrap();
    } else {
        eprintln!(
            "Error: provided argument `{}` is not a directory.",
            dir.to_string_lossy()
        );
        exit(1);
    }
}
