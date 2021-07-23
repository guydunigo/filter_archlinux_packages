use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter::{once, Iterator};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

use regex::Regex;
use version_compare::{CompOp, VersionCompare};

// TODO: do this manually and remove regexes completely
const PARSE_PKG_NAME_REGEX: &str = r"(.*)-([^-]+-[^-]+)-[^-]+.pkg.tar.*";

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

        match extract_name_version(file_name) {
            Ok((name, pkgver)) => Ok(Package { name, path, pkgver }),
            Err((e, _)) => Err((e, path)),
        }
    }

    pub fn compare_versions(a: &Package, b: &Package) -> Ordering {
        match VersionCompare::compare(&a.pkgver, &b.pkgver).unwrap_or(CompOp::Ne) {
            CompOp::Eq => {
                // TODO: log_lvl
                /*
                eprintln!(
                    "WWW package `{}` : versions `{}` and `{}` seems to be the same.",
                    a.name, a.pkgver, b.pkgver
                );
                */
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

/// Contains a package and its possible ambiguities, the first one having the priority over the
/// rest.
/// If two of the rest can be compared, it's not taken into account.
///
/// We use a value and an array for it to be easier to handle and might save some memory as most
/// packages won't even fill the vec.
pub struct Packages(Package, Vec<Package>);

impl Packages {
    pub fn new(p: Package) -> Self {
        Packages(p, Vec::with_capacity(0))
    }

    pub fn get_name(&self) -> &str {
        &self.0.name[..]
    }

    pub fn has_ambs(&self) -> bool {
        !self.1.is_empty()
    }

    pub fn add_ambiguity(&mut self, p: Package) {
        self.1.push(p);
    }

    pub fn into_iter(self) -> impl Iterator<Item = Package> {
        let Packages(p, pkgs) = self;
        once(p).chain(pkgs.into_iter())
    }

    /*
    pub fn into_vec(self) -> Vec<Package> {
        let Packages(p, mut pkgs) = self;
        pkgs.push(p);
        pkgs
    }
    */
}

impl Deref for Packages {
    type Target = Package;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Packages {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

fn extract_name_version(file_name: &str) -> Result<(String, String), (PackageParseError, String)> {
    // filename.spli
    lazy_static! {
        static ref RE: Regex = Regex::new(PARSE_PKG_NAME_REGEX).expect("Bad PARSE_PKG_NAME_REGEX");
    }

    let captures = if let Some(captures) = RE.captures(file_name) {
        captures
    } else {
        return Err((PackageParseError::NoPackageName, file_name.to_string()));
    };

    // Needs to do this jump to get direct access to &str
    let name = captures.get(1).unwrap().as_str().to_string();
    let pkgver = captures.get(2).unwrap().as_str().to_string();

    Ok((name, pkgver))
}
