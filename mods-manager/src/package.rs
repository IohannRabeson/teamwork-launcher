//! Model and functions to manipulate installation packages.
//! Usually, an installation package contains one file info.vdf, but it can contain
//! more than one if the package contains multiple HUDs.

use {
    serde::{Deserialize, Serialize},
    std::{
        fmt::{Display, Formatter},
        path::{Path, PathBuf},
    },
};

const INFO_VDF_FILE_NAME: &str = "info.vdf";
const VALVE_PACKAGE_FILE_EXTENSION: &str = "vpk";

#[derive(Ord, PartialOrd, Eq, PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct ModName(String);

impl ModName {
    pub fn new(name: impl ToString) -> Self {
        Self(name.to_string())
    }
}

impl Display for ModName {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
/// A HUD in a package.
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PackageEntry {
    /// The path to the directory, relative to the package root directory.
    pub path: PathBuf,

    /// The name of the HUD.
    /// This is the unique identifier of the HUD.
    pub name: ModName,

    /// The kind of entry.
    pub kind: PackageEntryKind,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum PackageEntryKind {
    Directory,
    VpkFile,
}

impl PackageEntry {
    pub fn from_path(path: impl AsRef<Path>) -> Result<Self, OpenModDirectoryError> {
        let path = path.as_ref();

        if path.is_dir() && path.join(INFO_VDF_FILE_NAME).is_file() {
            Self::directory(path)
        } else if path.is_file() && path.extension().and_then(|e| e.to_str()) == Some(VALVE_PACKAGE_FILE_EXTENSION) {
            Self::vpk_file(path)
        } else {
            Err(OpenModDirectoryError::UnsupportedType)
        }
    }

    fn directory(directory_path: impl AsRef<Path>) -> Result<Self, OpenModDirectoryError> {
        let path = directory_path.as_ref().to_path_buf();
        assert!(path.is_dir());
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .ok_or(OpenModDirectoryError::FailedToFindHudName)?;

        Ok(Self {
            path: path.clone(),
            name: ModName::new(name),
            kind: PackageEntryKind::Directory,
        })
    }

    fn vpk_file(file_path: impl AsRef<Path>) -> Result<Self, OpenModDirectoryError> {
        let path = file_path.as_ref().to_path_buf();
        assert!(path.is_file());
        let name = path
            .file_stem()
            .and_then(|name| name.to_str())
            .ok_or(OpenModDirectoryError::FailedToFindHudName)?;

        Ok(Self {
            path: path.clone(),
            name: ModName::new(name),
            kind: PackageEntryKind::VpkFile,
        })
    }
}

/// A package that contains 0 - n [`PackageEntry`].
pub struct Package {
    pub root_directory: PathBuf,
    pub entries: Vec<PackageEntry>,
}

impl Package {
    pub fn open(root_directory: impl Into<PathBuf>) -> Result<Self, OpenPackageError> {
        let root_directory = root_directory.into();

        Ok(Self {
            root_directory: root_directory.clone(),
            entries: Self::scan(&root_directory)?,
        })
    }

    pub fn mod_names(&self) -> impl Iterator<Item = &ModName> {
        self.entries.iter().map(|directory| &directory.name)
    }

    pub fn find_mod(&self, name: &ModName) -> Option<&PackageEntry> {
        self.entries.iter().find(|entry| &entry.name == name)
    }

    fn scan(root_directory: &Path) -> Result<Vec<PackageEntry>, ScanPackageError> {
        let mut hud_directories = Vec::new();

        for entry in walkdir::WalkDir::new(root_directory).into_iter().flatten() {
            if let Ok(package_entry) = PackageEntry::from_path(entry.path()) {
                hud_directories.push(package_entry);
            }
        }

        Ok(hud_directories)
    }
}

#[derive(thiserror::Error, Debug)]
pub enum OpenModDirectoryError {
    #[error("Failed to find HUD's name in info.vdf")]
    FailedToFindHudName,

    #[error("Failed to read .vdf file: {1}")]
    FailedToReadVdfFile(PathBuf, std::io::Error),

    #[error("Unsupported type")]
    UnsupportedType,
}

#[derive(thiserror::Error, Debug)]
pub enum OpenPackageError {
    #[error(transparent)]
    ScanFailed(#[from] ScanPackageError),
}

#[derive(thiserror::Error, Debug)]
pub enum ScanPackageError {
    #[error("Can't read directory '{0}': {1}")]
    CantReadDirectory(PathBuf, std::io::Error),
    #[error(transparent)]
    CantOpenHudDirectory(#[from] OpenModDirectoryError),
}

#[cfg(test)]
mod slow_tests {
    use {
        crate::package::{ModName, Package, INFO_VDF_FILE_NAME},
        std::path::Path,
        tempdir::TempDir,
    };

    fn create_vdf_file(name: &str, directory: &Path) {
        let mut content = format!("\"{}\"\n", name);
        content.push_str("{\n    \"ui_version\"    \"3\"\n}");
        std::fs::write(directory.join(INFO_VDF_FILE_NAME), content).unwrap();
    }

    #[test]
    fn test_open_package_one_vdf() {
        let package_dir = TempDir::new("test_open_package_one_vdf").unwrap();
        create_vdf_file("test", package_dir.path());

        let package = Package::open(package_dir.path()).unwrap();

        assert_eq!(1, package.entries.len());
    }

    /// Notice the name of the HUD is specified by the name of the directory
    /// that contains the file info.vdf. The reason I do not use the file info.vdf is
    /// because real HUDs often do not care about this file and I noticed error is several huds.
    #[test]
    fn test_open_package_multiple_vdf() {
        let package_dir = TempDir::new("test_open_package_one_vdf").unwrap();
        let d0 = package_dir.path().join("d0");
        let d1 = package_dir.path().join("d1");
        std::fs::create_dir_all(&d0).unwrap();
        std::fs::create_dir_all(&d1).unwrap();
        create_vdf_file("test0", &d0);
        create_vdf_file("test1", &d1);

        let package = Package::open(package_dir.path()).unwrap();

        assert_eq!(2, package.entries.len());
        assert_eq!(ModName("d0".into()), package.entries[0].name);
        assert_eq!(ModName("d1".into()), package.entries[1].name);
    }
}
