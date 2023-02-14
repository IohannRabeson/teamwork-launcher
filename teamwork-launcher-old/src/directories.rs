use {
    crate::APPLICATION_NAME,
    log::{error, info},
    std::path::PathBuf,
};

pub fn create_configuration_directory_if_needed() {
    let application_directory_path = get_configuration_directory();

    if !application_directory_path.exists() {
        info!("Create directory '{}'", application_directory_path.to_string_lossy());

        if let Err(error) = std::fs::create_dir_all(&application_directory_path) {
            error!(
                "Unable to create application directory '{}': {}",
                application_directory_path.to_string_lossy(),
                error
            );
        }
    }
}

pub fn get_configuration_directory() -> PathBuf {
    platform_dirs::AppDirs::new(APPLICATION_NAME.into(), false)
        .map(|dirs| dirs.config_dir)
        .expect("config directory path")
}

pub fn get_log_output_file_path() -> PathBuf {
    let mut log_output_path = get_configuration_directory();

    log_output_path.push(format!("{}.log", APPLICATION_NAME));
    log_output_path
}

pub fn get_settings_file_path() -> PathBuf {
    let mut settings_file_path = get_configuration_directory();

    settings_file_path.push("user_settings.json");
    settings_file_path
}

pub fn get_providers_file_path() -> PathBuf {
    let mut settings_file_path = get_configuration_directory();

    settings_file_path.push("providers.json");
    settings_file_path
}
