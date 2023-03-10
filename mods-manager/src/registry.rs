use {
    crate::{source::Source, ModName, PackageEntry},
    chrono::{DateTime, Utc},
    enum_as_inner::EnumAsInner,
    serde::{Deserialize, Serialize},
    std::collections::BTreeMap,
};

#[derive(Serialize, Deserialize, Default, Clone, Debug)]
pub struct Registry {
    info: BTreeMap<ModName, ModInfo>,
}

impl Registry {
    pub fn new() -> Self {
        Registry::default()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ModInfo> {
        self.info.values()
    }

    pub fn add(&mut self, name: ModName, source: Source) {
        if self.info.contains_key(&name) {
            return;
        }

        self.info.insert(
            name.clone(),
            ModInfo {
                name,
                source,
                install: Install::None,
            },
        );
    }

    pub fn remove(&mut self, name: &ModName) -> Option<ModInfo> {
        self.info.remove(name)
    }

    pub fn get(&self, name: &ModName) -> Option<&ModInfo> {
        self.info.get(name)
    }

    pub fn get_installed(&self) -> Option<&ModInfo> {
        self.info
            .values()
            .find(|info| matches!(info.install, Install::Installed { .. }))
    }

    pub fn set_install(&mut self, name: &ModName, install: Install) {
        if let Some(info) = self.info.get_mut(name) {
            info.install = install;
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ModInfo {
    pub name: ModName,
    pub source: Source,
    pub install: Install,
}

#[derive(Clone, Debug, EnumAsInner, Serialize, Deserialize)]
pub enum Install {
    None,
    Installed { package: PackageEntry, when: DateTime<Utc> },
    Failed { error: String },
}

impl Install {
    pub fn installed_now(package: PackageEntry) -> Self {
        Self::Installed {
            package,
            when: Utc::now(),
        }
    }

    pub fn failed(error: impl ToString) -> Self {
        Self::Failed {
            error: error.to_string(),
        }
    }
}
