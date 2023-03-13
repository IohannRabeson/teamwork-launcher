use {
    crate::application::{
        message::{AddViewMessage, ListViewMessage, ModsMessage},
        screens::{AddModView, Screens},
        Message, TeamworkLauncher,
    },
    iced::widget::text_input,
    iced_native::Command,
    mods_manager::{Install, Source},
    reqwest::Url,
};
use crate::application::notifications::NotificationKind;

impl TeamworkLauncher {
    pub(crate) fn process_mods_message(&mut self, message: ModsMessage) -> Command<Message> {
        match message {
            ModsMessage::AddView(message) => {
                return self.process_add_view_message(message);
            }
            ModsMessage::ListView(message) => {
                return self.process_list_view_message(message);
            }
            ModsMessage::AddMods(source, mod_names) => {
                for mod_name in mod_names.into_iter() {
                    self.mods_registry.add(mod_name, source.clone());
                }

                if let Some(Screens::AddMod(_context)) = self.views.current() {
                    self.views.pop();
                }
            }
            ModsMessage::Error(title, error) => {
                println!("{}: {}", title, error);
                self.is_loading_mods = false;
                if let Some(Screens::AddMod(context)) = self.views.current_mut() {
                    context.error = Some(error);
                    context.scanning = false;
                }
            }
            ModsMessage::Install(mod_name) => {
                if let Some(info) = self.mods_registry.get(&mod_name) {
                    if let Some(mods_directory) = self.paths.get_mods_directory() {
                        assert!(!matches!(info.install, Install::Installed { .. }));

                        self.is_loading_mods = true;

                        return commands::install_mod(info.source.clone(), mod_name, mods_directory);
                    }
                }
            }
            ModsMessage::Uninstall(mod_name) => {
                if let Some(info) = self.mods_registry.get(&mod_name) {
                    if let Some(mods_directory) = self.paths.get_mods_directory() {
                        assert!(matches!(info.install, Install::Installed { .. }));
                        self.is_loading_mods = true;
                        return commands::uninstall_mod(info, mods_directory);
                    }
                }
            }
            ModsMessage::InstallationFinished(mod_name, install) => {
                self.mods_registry.set_install(&mod_name, install);
                self.is_loading_mods = false;
            }
            ModsMessage::UninstallationFinished(mod_name) => {
                self.mods_registry.set_install(&mod_name, Install::None);
                self.is_loading_mods = false;
            }
            ModsMessage::FoundInstalledMods(packages) => {
                for package_entry in packages {
                    let mut install_to_set = None;

                    if let Some(info) = self.mods_registry.get(&package_entry.name) {
                        if let Install::Installed { package, .. } = &info.install {
                            if package_entry.path != package.path {
                                install_to_set = Some((info.name.clone(), package_entry.clone()));
                            }
                        }
                    } else {
                        self.mods_registry.add(package_entry.name.clone(), Source::None);
                        install_to_set = Some((package_entry.name.clone(), package_entry.clone()));
                    }

                    if let Some(install_to_set) = install_to_set {
                        self.mods_registry
                            .set_install(&install_to_set.0, Install::installed_now(install_to_set.1));
                    }
                }
            }
            ModsMessage::OpenInstallDirectory(mod_name) => {
                if let Some(mod_info) = self.mods_registry.get(&mod_name) {
                    if let Install::Installed { package, .. } = &mod_info.install {
                        if let Err(error) = open::that(&package.path) {
                            self.push_notification(error, NotificationKind::Error);
                        }
                    }
                }
            }
        }

        Command::none()
    }

    fn process_add_view_message(&mut self, message: AddViewMessage) -> Command<Message> {
        match message {
            AddViewMessage::Show => {
                let context = AddModView::default();
                let focus_command = text_input::focus(context.download_url_text_input.clone());

                self.views.push(Screens::AddMod(context));

                return focus_command;
            }
            AddViewMessage::DownloadUrlChanged(url) => {
                if let Some(Screens::AddMod(context)) = self.views.current_mut() {
                    context.download_url = url.clone();
                    context.is_form_valid = if !url.is_empty() {
                        match Url::parse(&url) {
                            Ok(_) => true,
                            Err(error) => {
                                context.error = Some(format!("Invalid URL: {}", error));
                                false
                            }
                        }
                    } else {
                        context.error = None;
                        false
                    };
                }
            }
            AddViewMessage::ScanPackageToAdd(source) => {
                if let Some(Screens::AddMod(context)) = self.views.current_mut() {
                    context.error = None;
                    context.scanning = true;
                    return commands::scan_package(source);
                }
            }
        }

        Command::none()
    }

    fn process_list_view_message(&mut self, message: ListViewMessage) -> Command<Message> {
        match message {
            ListViewMessage::ModClicked(mod_name) => {
                self.selected_mod = Some(mod_name);
            }
            ListViewMessage::RemoveMod(mod_name) => {
                if let Some(selected_mod) = self.selected_mod.as_ref() {
                    if selected_mod == &mod_name {
                        self.selected_mod = None;
                    }
                }

                if let Some(info) = self.mods_registry.remove(&mod_name) {
                    if let Some(mods_directory) = self.paths.get_mods_directory() {
                        return commands::uninstall_mod(&info, mods_directory);
                    }
                }
            }
        }
        Command::none()
    }
}

pub mod commands {
    use {
        crate::application::{message::ModsMessage, Message},
        iced::Command,
        mods_manager::{fetch_package, install, uninstall, FetchError, Install, ModInfo, ModName, PackageEntry, Source},
        std::path::{Path, PathBuf},
        tempdir::TempDir,
    };

    #[derive(thiserror::Error, Debug)]
    enum ScanPackageError {
        #[error(transparent)]
        FetchPackageFailed(#[from] FetchError),

        #[error("Failed to create a temporary directory: {0}")]
        FailedToCreateTempDirectory(std::io::Error),
    }

    pub fn scan_package(source: Source) -> Command<Message> {
        let source_for_future = source.clone();

        Command::perform(
            async move { get_mod_names(source_for_future).await },
            move |result| match result {
                Err(error) => Message::Mods(ModsMessage::error("Failed to scan package", error)),
                Ok(mod_names) => Message::Mods(ModsMessage::AddMods(source, mod_names)),
            },
        )
    }

    async fn get_mod_names(source: Source) -> Result<Vec<ModName>, ScanPackageError> {
        let temp_directory = TempDir::new("fetch_package_name").map_err(ScanPackageError::FailedToCreateTempDirectory)?;
        let package = fetch_package(source.clone(), temp_directory.path()).await?;

        Ok(package.mod_names().cloned().collect())
    }

    /// Search for installs of mods in a directory.
    fn search_mod_install(mods_directory: &Path) -> Vec<PackageEntry> {
        let mut directories = Vec::new();

        for entry in std::fs::read_dir(mods_directory).into_iter().flatten().flatten() {
            if let Ok(entry) = PackageEntry::from_path(entry.path()) {
                directories.push(entry);
            }
        }

        directories
    }

    pub fn scan_mods_directory(mods_directory: Option<PathBuf>) -> Command<Message> {
        match mods_directory {
            Some(mods_directory) => Command::perform(async move { search_mod_install(&mods_directory) }, |mods| {
                Message::Mods(ModsMessage::FoundInstalledMods(mods))
            }),
            None => Command::none(),
        }
    }

    pub fn install_mod(source: Source, name: ModName, mods_directory: PathBuf) -> Command<Message> {
        let mod_name = name.clone();

        Command::perform(
            async move { install(source, mod_name, mods_directory).await },
            move |result| Message::Mods(ModsMessage::InstallationFinished(name, result)),
        )
    }

    pub fn uninstall_mod(mod_info: &ModInfo, mods_directory: PathBuf) -> Command<Message> {
        if let Install::Installed { package, .. } = &mod_info.install {
            let mod_name = mod_info.name.clone();
            let mod_path = package.path.clone();

            Command::perform(
                async move { uninstall(&mod_path, mods_directory).await },
                move |result| match result {
                    Ok(()) => Message::Mods(ModsMessage::UninstallationFinished(mod_name)),
                    Err(error) => {
                        Message::Mods(ModsMessage::error(format!("Failed to uninstall mod '{0}'", mod_name), error))
                    }
                },
            )
        } else {
            Command::none()
        }
    }
}
