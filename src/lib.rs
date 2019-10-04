extern crate regex;
extern crate version_compare;
#[macro_use]
extern crate lazy_static;

use std::path::Path;

mod package;
use package::Package;

const TEST_NAME: &str = "/mnt/arch/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const TEST_NAME_2: &str = "/mnt/archlinux/linux-5.3.1.arch1-1-x86_64.pkg.tar.xz";
const TEST_NAME_3: &str = "/mnt/archlinux/zeitgeist-1.0+1+g1bcc8585-1-x86_64.pkg.tar.xz";

pub fn filter_archlinux_packages<P: AsRef<Path>>(_path: P) {
    {
        let file_path = Path::new(TEST_NAME);
        let pkg = Package::from_path(file_path);
        println!("{:?}", pkg);
    }
    {
        let file_path = Path::new(TEST_NAME_2);
        let pkg = Package::from_path(file_path);
        println!("{:?}", pkg);
    }
    {
        let file_path = Path::new(TEST_NAME_3);
        let pkg = Package::from_path(file_path);
        println!("{:?}", pkg);
    }
}
