use {
    crate::application::GameModesMessage,
    async_stream::stream,
    iced::{subscription, Color, Subscription},
    serde::{Deserialize, Serialize},
    std::{
        collections::BTreeMap,
        fmt::{Display, Formatter},
        sync::Arc,
    },
};

#[derive(Clone, Debug, Hash, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct GameModeId(String);

impl GameModeId {
    pub fn new(id: impl ToString) -> Self {
        Self(id.to_string())
    }
}

impl Display for GameModeId {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

#[derive(Debug, Clone)]
pub struct GameMode {
    pub id: String,
    pub title: String,
    pub description: String,
    pub color: Option<Color>,
}

impl From<teamwork::GameMode> for GameMode {
    fn from(value: teamwork::GameMode) -> Self {
        Self {
            id: value.id,
            title: value.title,
            description: value.description,
            color: value.color.map(|c| iced::Color::from_rgb8(c.r, c.g, c.b)),
        }
    }
}

pub struct GameModes {
    game_modes_info: BTreeMap<GameModeId, GameMode>,
}

impl GameModes {
    pub fn new() -> Self {
        Self {
            game_modes_info: BTreeMap::new(),
        }
    }

    pub fn reset(&mut self, modes: &[teamwork::GameMode]) {
        self.game_modes_info = modes
            .into_iter()
            .map(|mode| (GameModeId::new(mode.id.clone()), mode.clone().into()))
            .collect();
    }

    pub fn get(&self, id: &GameModeId) -> Option<&GameMode> {
        self.game_modes_info.get(id)
    }
}

pub fn subscription(id: u64, teamwork_api_key: &str) -> Subscription<GameModesMessage> {
    let teamwork_api_key = teamwork_api_key.to_string();
    let s = stream! {
        let client = teamwork::Client::default();

        match client.get_game_modes(&teamwork_api_key).await {
             Ok(game_modes) => {
                 yield GameModesMessage::GameModes(game_modes);
             }
             Err(error) => {
                 yield GameModesMessage::Error(Arc::new(error));
             }
        }
    };

    subscription::run(id, s)
}
