use std::cmp::Ordering;
use std::fmt::Debug;
use std::iter::{once, Iterator};
use std::ops::{Deref, DerefMut};
use std::path::PathBuf;

#[cfg(feature = "regex")]
use regex::Regex;
use version_compare::{Cmp, Version};

#[derive(Debug)]
pub enum PackageParseError {
    NoPackageName,
    EmptyPathOrRoot,
    CouldntParsePkgver(String),
}

#[derive(Debug)]
pub struct Package<'a> {
    pub path: &'a PathBuf,
    pub name: &'a str,
    pub pkgverstr: &'a str,
    pub pkgver: Version<'a>,
}

impl<'a> Package<'a> {
    pub fn from_path(path: &'a PathBuf) -> Result<Self, (PackageParseError, PathBuf)> {
        match path.file_name().map(|file_name| {
            extract_name_version(file_name.to_str().unwrap()).map(|(n, v)| (n, v, Version::from(v)))
        }) {
            Some(Ok((name, pkgverstr, Some(pkgver)))) => Ok(Package {
                name,
                path,
                pkgverstr,
                pkgver,
            }),
            Some(Ok((_, pkgverstr, None))) => Err((
                PackageParseError::CouldntParsePkgver(pkgverstr.to_string()),
                path.clone(),
            )),
            Some(Err((e, _))) => Err((e, path.clone())),
            None => Err((PackageParseError::EmptyPathOrRoot, path.clone())),
        }
    }

    pub fn compare_versions(a: &Package, b: &Package) -> Ordering {
        match Version::compare(&a.pkgver, &b.pkgver) {
            Cmp::Eq => {
                // TODO: log_lvl
                eprintln!(
                    "WWW package `{}` : versions `{}` and `{}` seems to be the same.",
                    a.name, a.pkgver, b.pkgver
                );
                Ordering::Equal
            }
            Cmp::Ge | Cmp::Gt => Ordering::Greater,
            Cmp::Le | Cmp::Lt => Ordering::Less,
            Cmp::Ne => {
                eprintln!("WWW package `{}` : versions `{}` and `{}` seems to be different, but we can't compare them.",
                    a.name,
                    a.pkgver,
                    b.pkgver
                );
                Ordering::Equal
            }
        }
    }
}

/// Contains a package and its possible ambiguities, the first one having the priority over the
/// rest.
/// If two of the rest can be compared, it's not taken into account.
///
/// We use a value and an array for it to be easier to handle and might save some memory as most
/// packages won't even fill the vec.
pub struct Packages<'a>(Package<'a>, Vec<Package<'a>>);

impl<'a> Packages<'a> {
    pub fn new(p: Package<'a>) -> Self {
        Packages(p, Vec::with_capacity(0))
    }

    pub fn get_name(&self) -> &str {
        self.0.name
    }

    pub fn has_ambs(&self) -> bool {
        !self.1.is_empty()
    }

    pub fn add_ambiguity(&mut self, p: Package<'a>) {
        self.1.push(p);
    }

    /// Iterate over the current package and its ambiguities.
    pub fn into_iter(self) -> impl Iterator<Item = Package<'a>> {
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

impl<'a> Deref for Packages<'a> {
    type Target = Package<'a>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a> DerefMut for Packages<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// `{name}-{soft_version-pkg_version}-{arch}.pkg.tar.{compress_algo}`
#[cfg(feature = "regex")]
const PARSE_PKG_NAME_REGEX: &str = r"(.*)-([^-]+-[^-]+)-[^-]+.pkg.tar.*";

/// Returns (name, pkgver) based on the provided file_name.
///
/// Example :
/// ```
/// // TODO: can't run because private...
/// // use remove_old_arch_pkgs::package::extract_name_version;
/// // assert_eq!(extract_name_version("acpi-1.7-3-x86_64.pkg.tar.zst"), Ok(("acpi", "1.7-3")))
/// ```
#[cfg(feature = "regex")]
fn extract_name_version(file_name: &str) -> Result<(&str, &str), (PackageParseError, String)> {
    // filename.split
    lazy_static! {
        static ref RE: Regex = Regex::new(PARSE_PKG_NAME_REGEX).expect("Bad PARSE_PKG_NAME_REGEX");
    }

    let captures = if let Some(captures) = RE.captures(file_name) {
        captures
    } else {
        return Err((PackageParseError::NoPackageName, file_name.to_string()));
    };

    // Needs to do this jump to get direct access to &str
    let name = captures.get(1).unwrap().as_str();
    let pkgver = captures.get(2).unwrap().as_str();

    Ok((name, pkgver))
}

#[cfg(not(feature = "regex"))]
fn extract_name_version(file_name: &str) -> Result<(&str, &str), (PackageParseError, String)> {
    if file_name.split('-').count() <= 3 {
        return Err((PackageParseError::NoPackageName, file_name.to_string()));
    }

    let mut f_iter = file_name.split('-');
    let name = f_iter.next().unwrap();
    // TODO : better way to extract slice to file_name without allocating new vec/string ?
    let name_len = name.len();
    let pkgver_0_len = f_iter.next().unwrap().len();
    let pkgver_1_len = f_iter.next().unwrap().len();
    let pkgver = &file_name[(name_len + 1)..(name_len + pkgver_0_len + pkgver_1_len + 2)];

    // Checking extension in `.pkg.tar.{algo}` :
    let rest = f_iter.next().unwrap();
    if rest.split('.').count() != 4 {
        return Err((PackageParseError::NoPackageName, file_name.to_string()));
    }
    let mut rest_iter = rest.split('.').skip(1);
    // let _arch = rest_iter_iter.next().unwrap();
    let pkg = rest_iter.next().unwrap();
    let tar = rest_iter.next().unwrap();
    if !(pkg == "pkg" && tar == "tar") {
        return Err((PackageParseError::NoPackageName, file_name.to_string()));
    }
    // let _algo = f_iter.next().unwrap();

    Ok((name, pkgver))
}
