use serde::Serialize;
use std::path::Path;
use serde::de::DeserializeOwned;
use crate::application::SettingsError;

pub fn write_file(settings: &impl Serialize, file_path: impl AsRef<Path>) -> Result<(), SettingsError> {
    use std::io::Write;
    use std::sync::Arc;

    let json = serde_json::to_string(settings).map_err(|e| SettingsError::Json(Arc::new(e)))?;
    let mut file = std::fs::File::create(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    file.write_all(json.as_bytes()).map_err(|e| SettingsError::Io(Arc::new(e)))?;
    Ok(())
}

pub fn read_file<'de, S>(file_path: impl AsRef<Path>) -> Result<S, SettingsError>
where
    S: DeserializeOwned + Default,
{
    use std::sync::Arc;

    let file = std::fs::File::open(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    Ok(serde_json::from_reader(file).map_err(|e| SettingsError::Json(Arc::new(e)))?)
}
