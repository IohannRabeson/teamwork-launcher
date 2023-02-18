use {
    crate::{application::SettingsError, APPLICATION_NAME},
    serde::{de::DeserializeOwned, Serialize},
    std::path::{Path, PathBuf},
};

pub fn write_file(settings: &impl Serialize, file_path: impl AsRef<Path>) -> Result<(), SettingsError> {
    use std::{io::Write, sync::Arc};

    let json = serde_json::to_string(settings).map_err(|e| SettingsError::Json(Arc::new(e)))?;
    let mut file = std::fs::File::create(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    file.write_all(json.as_bytes()).map_err(|e| SettingsError::Io(Arc::new(e)))?;
    Ok(())
}

pub fn read_file<S>(file_path: impl AsRef<Path>) -> Result<S, SettingsError>
where
    S: DeserializeOwned + Default,
{
    use std::sync::Arc;

    let file = std::fs::File::open(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    serde_json::from_reader(file).map_err(|e| SettingsError::Json(Arc::new(e)))
}

pub fn get_configuration_directory() -> PathBuf {
    platform_dirs::AppDirs::new(APPLICATION_NAME.into(), false)
        .map(|dirs| dirs.config_dir)
        .expect("config directory path")
}
