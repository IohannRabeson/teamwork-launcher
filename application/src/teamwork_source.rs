use async_trait::async_trait;

use crate::{
    servers::{GetServersInfosError, Server, Source},
    settings::UserSettings,
};

#[derive(Default)]
pub struct TeamworkSource;

#[async_trait]
impl Source for TeamworkSource {
    fn display_name(&self) -> String {
        "Teamwork.tf".into()
    }

    async fn get_servers_infos(&self, settings: &UserSettings) -> Result<Vec<Server>, GetServersInfosError> {
        Ok(Vec::new())
    }
}
