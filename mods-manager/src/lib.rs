mod deployment;
mod package;
mod registry;
mod source;

pub use {
    deployment::{install, uninstall, InstallError},
    package::{ModName, OpenModDirectoryError, OpenPackageError, Package, PackageEntry, ScanPackageError},
    registry::{Install, ModInfo, Registry},
    reqwest::Url,
    source::{fetch_package, FetchError, Source},
};

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    pub fn get_resource_path(relative_path: impl AsRef<Path>) -> PathBuf {
        let mut p = PathBuf::from(env!("CARGO_MANIFEST_DIR"));

        p.push("resources");
        p.push("tests");
        p.push(relative_path);
        p
    }
}