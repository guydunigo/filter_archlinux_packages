extern crate filter_archlinux_packages;

// use std::env::Args;
use std::env::current_dir;

use filter_archlinux_packages::filter_archlinux_packages;

fn main() {
    let cur_dir = current_dir().unwrap();
    filter_archlinux_packages(cur_dir);
}
