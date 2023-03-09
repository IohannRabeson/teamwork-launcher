use {
    crate::application::IpPort,
    iced_native::widget::{pane_grid, pane_grid::Axis},
};

pub enum Screens {
    Main(MainView),
    Server(ServerView),
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
