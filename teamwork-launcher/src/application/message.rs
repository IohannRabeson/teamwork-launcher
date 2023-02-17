use iced::widget::pane_grid;

use {
    crate::application::{
        filter_servers::PropertyFilterSwitch, game_mode::GameModeId, geolocation, map::MapName, ping,
        servers_source::SourceKey, Country, FetchServersEvent, IpPort, Server,
    },
    iced::{futures::channel::mpsc::UnboundedSender, widget::image},
    std::{net::Ipv4Addr, sync::Arc, time::Duration},
};

#[derive(Debug, Clone)]
pub enum FetchServersMessage {
    FetchServersStart,
    FetchServersFinish,
    FetchServersError(Arc<teamwork::Error>),
    NewServers(Vec<Server>),
}

#[derive(Debug, Clone)]
pub enum CountryServiceMessage {
    Started(UnboundedSender<Ipv4Addr>),
    CountryFound(Ipv4Addr, Country),
    Error(geolocation::Error),
}

#[derive(Debug, Clone)]
pub enum PingServiceMessage {
    Started(UnboundedSender<Ipv4Addr>),
    Answer(Ipv4Addr, Duration),
    Error(Ipv4Addr, ping::Error),
}

#[derive(Debug, Clone)]
pub enum ThumbnailMessage {
    Started(UnboundedSender<MapName>),
    Thumbnail(MapName, image::Handle),
    Error(MapName, Arc<teamwork::Error>),
}

#[derive(Debug, Clone)]
pub enum FilterMessage {
    CountryChecked(Country, bool),
    NoCountryChecked(bool),
    TextChanged(String),
    BookmarkedOnlyChecked(bool),
    IgnoreCaseChanged(bool),
    IgnoreAccentChanged(bool),
    MaxPingChanged(u32),
    AcceptPingTimeoutChanged(bool),
    GameModeChecked(GameModeId, bool),
    CountryFilterEnabled(bool),
    GameModeFilterEnabled(bool),
    VacSecuredChanged(PropertyFilterSwitch),
    RtdChanged(PropertyFilterSwitch),
    AllTalkChanged(PropertyFilterSwitch),
    NoRespawnTimeChanged(PropertyFilterSwitch),
    PasswordChanged(PropertyFilterSwitch),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    TeamworkApiKeyChanged(String),
    SteamExecutableChanged(String),
    SourceEnabled(SourceKey, bool),
    QuitWhenLaunchChecked(bool),
    QuitWhenCopyChecked(bool),
}

#[derive(Debug, Clone)]
pub enum PaneMessage {
    Resized(pane_grid::ResizeEvent),
}

#[derive(Debug, Clone)]
pub enum GameModesMessage {
    GameModes(Vec<teamwork::GameMode>),
    Error(Arc<teamwork::Error>),
}

#[derive(Debug, Clone)]
pub enum KeyboardMessage {
    ShiftPressed,
    ShiftReleased,
}

#[derive(Debug, Clone)]
pub enum Message {
    Servers(FetchServersMessage),
    Country(CountryServiceMessage),
    Ping(PingServiceMessage),
    Thumbnail(ThumbnailMessage),
    Filter(FilterMessage),
    Settings(SettingsMessage),
    Pane(PaneMessage),
    GameModes(GameModesMessage),
    Keyboard(KeyboardMessage),
    RefreshServers,
    ShowSettings,
    ShowServer(IpPort),
    LaunchGame(IpPort),
    CopyConnectionString(IpPort),
    Bookmarked(IpPort, bool),
    CopyToClipboard(String),
    Back,
}

impl From<FetchServersEvent> for Message {
    fn from(value: FetchServersEvent) -> Self {
        match value {
            FetchServersEvent::Start => Message::Servers(FetchServersMessage::FetchServersStart),
            FetchServersEvent::Finish => Message::Servers(FetchServersMessage::FetchServersFinish),
            FetchServersEvent::Servers(servers) => Message::Servers(FetchServersMessage::NewServers(servers)),
            FetchServersEvent::Error(error) => Message::Servers(FetchServersMessage::FetchServersError(error)),
        }
    }
}

impl From<CountryServiceMessage> for Message {
    fn from(value: CountryServiceMessage) -> Self {
        Message::Country(value)
    }
}

impl From<PingServiceMessage> for Message {
    fn from(message: PingServiceMessage) -> Self {
        Message::Ping(message)
    }
}

impl From<ThumbnailMessage> for Message {
    fn from(value: ThumbnailMessage) -> Self {
        Message::Thumbnail(value)
    }
}

impl From<pane_grid::ResizeEvent> for Message {
    fn from(value: pane_grid::ResizeEvent) -> Self {
        Message::Pane(PaneMessage::Resized(value))
    }
}

impl From<GameModesMessage> for Message {
    fn from(value: GameModesMessage) -> Self {
        Message::GameModes(value)
    }
}

impl From<KeyboardMessage> for Message {
    fn from(value: KeyboardMessage) -> Self {
        Message::Keyboard(value)
    }
}
