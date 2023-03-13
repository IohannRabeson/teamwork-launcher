use {
    crate::application::SettingsError,
    serde::{de::DeserializeOwned, Serialize},
    std::path::Path,
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

pub fn write_bin_file(settings: &impl Serialize, file_path: impl AsRef<Path>) -> Result<(), SettingsError> {
    use std::{io::Write, sync::Arc};

    let encoded: Vec<u8> = bincode::serialize(&settings).expect("serialize state");
    let mut file = std::fs::File::create(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    file.write_all(&encoded).map_err(|e| SettingsError::Io(Arc::new(e)))?;
    Ok(())
}

pub fn read_bin_file<S>(file_path: impl AsRef<Path>) -> Result<S, SettingsError>
where
    S: DeserializeOwned + Default,
{
    use std::sync::Arc;

    let encoded = std::fs::read(file_path).map_err(|e| SettingsError::Io(Arc::new(e)))?;

    bincode::deserialize(&encoded).map_err(|_| SettingsError::InvalidFileFormat)
}
