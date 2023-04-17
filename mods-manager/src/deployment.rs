use {
    crate::{
        fetch_package, package::PackageEntryKind, source::Source, FetchError, Install, ModName, OpenModDirectoryError,
        PackageEntry,
    },
    std::path::{Path, PathBuf},
    tempdir::TempDir,
};

#[derive(thiserror::Error, Debug)]
pub enum InstallError {
    #[error(transparent)]
    IoError(#[from] std::io::Error),
    #[error(transparent)]
    FetchPackageFailed(#[from] FetchError),
    #[error("Mod '{0}' not found")]
    HudNotFound(ModName),
    #[error(transparent)]
    FailedToOpenHud(#[from] OpenModDirectoryError),
    #[error(transparent)]
    FailedToMoveDirectory(#[from] fs_extra::error::Error),
}

pub async fn install(source: Source, name: ModName, mods_directory: PathBuf) -> Install {
    match install_impl(source, name, mods_directory).await {
        Ok(package) => Install::installed_now(package),
        Err(error) => Install::failed(error),
    }
}

async fn install_impl(source: Source, name: ModName, mods_directory: PathBuf) -> Result<PackageEntry, InstallError> {
    if !mods_directory.exists() {
        std::fs::create_dir_all(&mods_directory)?;
    }

    let directory = TempDir::new(&format!("install_{}", name))?;
    let package = fetch_package(source, directory.path()).await?;
    let source_hud_entry = package.find_mod(&name).ok_or(InstallError::HudNotFound(name.clone()))?;
    let source_name = source_hud_entry.path.file_name().expect("source file name");
    let destination_path = mods_directory.join(source_name);

    match source_hud_entry.kind {
        PackageEntryKind::Directory => {
            let copy_options = fs_extra::dir::CopyOptions::new().copy_inside(true);

            fs_extra::dir::move_dir(&source_hud_entry.path, &destination_path, &copy_options)?;
        }
        PackageEntryKind::VpkFile => {
            let copy_options = fs_extra::file::CopyOptions::new().overwrite(true);

            fs_extra::file::copy(&source_hud_entry.path, &destination_path, &copy_options)?;
        }
    };

    Ok(PackageEntry::from_path(&destination_path).expect("scan mod"))
}

pub async fn uninstall(mod_path: &Path, mods_directory: PathBuf) -> Result<(), std::io::Error> {
    assert!(mod_path.starts_with(&mods_directory));

    if mod_path.is_dir() {
        return tokio::fs::remove_dir_all(mod_path).await;
    }

    if mod_path.is_file() {
        return tokio::fs::remove_file(mod_path).await;
    }

    panic!("Unsupported HUD type");
}

#[cfg(all(test, feature = "flaky-tests"))]
mod slow_tests {
    use {
        super::install,
        crate::{ModName, Source},
        tempdir::TempDir,
    };

    #[tokio::test]
    async fn test_install_zip() {
        let source = Source::DownloadUrl("https://github.com/n0kk/ahud/archive/refs/heads/master.zip".into());
        let directory = TempDir::new("test_install_zip").unwrap();
        let install = install(source, ModName::new("ahud-master"), directory.path().to_path_buf()).await;
        let entry = install.as_installed().unwrap().0;

        assert_eq!(ModName::new("ahud-master"), entry.name);
    }

    #[tokio::test]
    async fn test_install_7z() {
        let source = Source::DownloadUrl("https://www.dropbox.com/s/cwwmppnn3nn68av/3HUD.7z?dl=1".into());
        let directory = TempDir::new("test_install_7z").unwrap();
        let install = install(source, ModName::new("3HUD"), directory.path().to_path_buf()).await;
        let entry = install.as_installed().unwrap().0;

        assert_eq!(ModName::new("3HUD"), entry.name);
    }

    #[tokio::test]
    async fn test_install_vpk() {
        let source = Source::DownloadUrl("https://gamebanana.com/dl/945012".into());
        let directory = TempDir::new("test_install_vpk").unwrap();
        let install = install(source, ModName::new("minhud_plus"), directory.path().to_path_buf()).await;
        let entry = install.as_installed().unwrap().0;

        assert_eq!(ModName::new("minhud_plus"), entry.name);
        assert!(directory.path().join("minhud_plus.vpk").exists());
    }
}
