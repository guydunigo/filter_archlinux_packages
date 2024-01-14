use std::{env::current_dir, fmt, path::PathBuf};

/// Options for the program
#[derive(Debug, Clone)]
pub struct Options {
    /// Path in which to remove packages
    pub dir: PathBuf,
    /// Autoconfirm level : how often is the user asked for confirmation
    pub auto_confirm_level: AutoConfirmLevel,
    pub dry_run: bool,
}

impl Default for Options {
    fn default() -> Self {
        Options {
            dir: current_dir().unwrap(),
            auto_confirm_level: Default::default(),
            dry_run: false,
        }
    }
}

/// How much interraction from the user is needed.
#[derive(Copy, Clone, Debug, PartialEq, Eq, Default)]
pub enum AutoConfirmLevel {
    /// Ask nothing (be careful)
    Nothing,
    /// Ask only at the end before removing any files
    Removal,
    /// Ask when there are ambiguities between two or more versions (the version library can't say they are equal)
    #[default]
    Ambiguities,
    /// Ask for every choice (in case you're not sure the version library compared successfuly)
    Everything,
}

impl AutoConfirmLevel {
    pub fn is_nothing(&self) -> bool {
        *self == AutoConfirmLevel::Nothing
    }

    pub fn is_at_least_removal(&self) -> bool {
        *self == AutoConfirmLevel::Removal || self.is_at_least_ambiguities()
    }

    pub fn is_at_least_ambiguities(&self) -> bool {
        *self == AutoConfirmLevel::Ambiguities || self.is_everything()
    }

    pub fn is_everything(&self) -> bool {
        *self == AutoConfirmLevel::Everything
    }
}

impl fmt::Display for AutoConfirmLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        use AutoConfirmLevel::*;

        match self {
            Nothing => write!(f, "{:?} (never ask for any confirmation, be careful)", self),
            Removal => write!(f, "{:?} (only ask at the end before any removal)", self),
            Ambiguities => write!(f, "{:?} (ask when there are ambiguities when the version compare library considers two version equals)", self),
            Everything => write!(f, "{:?} (ask for every package to be removed, in case you don't trust the version compare library)", self),
        }
    }
}
