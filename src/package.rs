use std::cmp::Ordering;
use std::fmt::Debug;
use std::path::PathBuf;

use regex::Regex;
use version_compare::{CompOp, VersionCompare};

const PARSE_PKG_NAME_REGEX: &str = r"(.*)-([^-]+-[^-]+)-[^-]+.pkg.tar.xz";

#[derive(Debug)]
pub enum PackageParseError {
    NoPackageName,
    EmptyPathOrRoot,
}

#[derive(Debug)]
pub struct Package {
    pub name: String,
    pub pkgver: String,
    pub path: PathBuf,
}

impl Package {
    pub fn from_path(path: PathBuf) -> Result<Self, (PackageParseError, PathBuf)> {
        let file_name = if let Some(file_name) = path.file_name() {
            file_name.to_str().unwrap()
        } else {
            return Err((PackageParseError::EmptyPathOrRoot, path));
        };

        lazy_static! {
            static ref RE: Regex =
                Regex::new(PARSE_PKG_NAME_REGEX).expect("Bad PARSE_PKG_NAME_REGEX");
        }

        let captures = if let Some(captures) = RE.captures(file_name) {
            captures
        } else {
            return Err((PackageParseError::NoPackageName, path));
        };

        // Needs to do this jump to get direct access to &str
        let name = captures.get(1).unwrap().as_str().to_string();
        let pkgver = captures.get(2).unwrap().as_str().to_string();

        Ok(Package { name, path, pkgver })
    }

    pub fn compare_versions(a: &Package, b: &Package) -> Ordering {
        match VersionCompare::compare(&a.pkgver, &b.pkgver).unwrap_or(CompOp::Ne) {
            CompOp::Eq => {
                eprintln!(
                    "WWW package `{}` : versions `{}` and `{}` seems to be the same.",
                    a.name, a.pkgver, b.pkgver
                );
                Ordering::Equal
            }
            CompOp::Ge | CompOp::Gt => Ordering::Greater,
            CompOp::Le | CompOp::Lt => Ordering::Less,
            CompOp::Ne => {
                eprintln!("WWW package `{}` : versions `{}` and `{}` seems to be different, but we can't compare them.",
                    a.name,
                    a.pkgver,
                    b.pkgver
                );
                Ordering::Equal
            }
        }
    }
    /*
    fn parse_pkgver(pkgver: &str) -> Result<(Version, &str), String> {
        if let Some(semver) = Version::from(pkgver) {
            Ok((semver, ""))
        } else {
            Err(format!("Couldn't parse pkgver: `{}`", pkgver))
        }
    }
    */
}
