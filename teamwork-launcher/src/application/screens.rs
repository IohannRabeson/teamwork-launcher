use {crate::application::IpPort, iced::widget::text_input};

pub enum Screens {
    Main,
    Server(ServerView),
    Mods,
    AddMod(AddModView),
    Settings,
}

pub struct ServerView {
    pub ip_port: IpPort,
}

impl ServerView {
    pub fn new(ip_port: IpPort) -> Self {
        Self { ip_port }
    }
}

pub enum PaneId {
    Servers,
    Filters,
}

pub struct PaneView {
    pub id: PaneId,
}

impl PaneView {
    pub fn new(id: PaneId) -> Self {
        Self { id }
    }
}

pub struct AddModView {
    pub download_url: String,
    pub is_form_valid: bool,
    pub error: Option<String>,
    pub download_url_text_input: text_input::Id,
    pub scanning: bool,
}

impl Default for AddModView {
    fn default() -> Self {
        Self {
            download_url: String::new(),
            is_form_valid: false,
            error: None,
            download_url_text_input: text_input::Id::unique(),
            scanning: false,
        }
    }
}
