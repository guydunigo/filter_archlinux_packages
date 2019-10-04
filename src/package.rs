use std::fmt::Debug;
use std::path::Path;

use regex::Regex;
use version_compare::Version;

const PARSE_PKG_NAME_REGEX: &str = r"(.*)-([^-]+)-(\d+)-x86_64.pkg.tar.xz";
//const PARSE_PKG_NAME_REGEX: &str = r"(.*)-x86_64.pkg.tar.xz";

#[derive(Debug)]
pub struct Package<'a> {
    pub name: &'a str,
    pub pkgver_semver: Version<'a>,
    pub pkgver_tail: &'a str,
    pub pkgrel: usize,
    pub path: &'a Path,
}

impl<'a> Package<'a> {
    pub fn from_path(path: &'a Path) -> Result<Self, String> {
        let file_name = if let Some(file_name) = path.file_name() {
            file_name.to_str().unwrap()
        } else {
            return Err("Empty path or root directory".to_string());
        };

        println!("{}", file_name);

        lazy_static! {
            static ref RE: Regex =
                Regex::new(PARSE_PKG_NAME_REGEX).expect("Bad PARSE_PKG_NAME_REGEX");
        }

        let captures = if let Some(captures) = RE.captures(file_name) {
            captures
        } else {
            return Err(format!(
                "Couldn't parse pkgname, pkgver, and arch package version from file name: `{}`",
                file_name
            ));
        };

        // Needs to do this jump to get direct access to &str
        let name = captures.get(1).unwrap().as_str();
        let version_str = captures.get(2).unwrap().as_str();
        let release_str = &captures[3];

        let pkgrel: usize = if let Ok(pkgrel) = release_str.parse() {
            pkgrel
        } else {
            return Err(format!(
                "Couldn't parse arch package version from string: `{}`",
                &captures[3]
            ));
        };

        let pkgver_opt = (0..=version_str.len())
            .rev()
            .map(|i| {
                let semver = &version_str[..i];
                let tail = &version_str[i..];
                println!("{} {}", semver, tail);

                if let Some(pkgver_semver) = Version::from(semver) {
                    Some((pkgver_semver, tail))
                } else {
                    None
                }
            })
            .skip_while(|pkgver_opt| pkgver_opt.is_none())
            .map(|pkgver_opt| pkgver_opt.unwrap())
            .next();
        let (pkgver_semver, pkgver_tail) = if let Some(pkgver) = pkgver_opt {
            pkgver
        } else {
            return Err(format!(
                "Couldn't parse pkgver from pkgver: `{}`",
                version_str
            ));
        };

        Ok(Package {
            name,
            path,
            pkgver_semver,
            pkgver_tail,
            pkgrel,
        })
    }
}
