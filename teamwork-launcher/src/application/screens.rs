use iced::widget::text_input;
use {
    crate::application::IpPort,
    iced_native::widget::{pane_grid, pane_grid::Axis},
};

pub enum Screens {
    Main(MainView),
    Server(ServerView),
    Mods,
    AddMod(AddModView),
    Settings,
}

pub struct MainView {
    pub panes: pane_grid::State<PaneView>,
}

impl MainView {
    pub fn new(pane_ratio: f32) -> Self {
        let (mut panes, servers_pane) = pane_grid::State::new(PaneView::new(PaneId::Servers));

        if let Some((_filter_pane, split)) = panes.split(Axis::Vertical, &servers_pane, PaneView::new(PaneId::Filters)) {
            panes.resize(&split, pane_ratio);
        }

        Self { panes }
    }
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
    fn new(id: PaneId) -> Self {
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