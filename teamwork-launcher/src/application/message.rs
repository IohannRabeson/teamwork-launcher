use {
    crate::application::{
        blacklist::ImportBlacklistError,
        filter::{
            properties_filter::PropertyFilterSwitch,
            sort_servers::{SortCriterion, SortDirection},
        },
        game_mode::GameModeId,
        geolocation,
        map::MapName,
        ping,
        servers_source::SourceKey,
        user_settings::LauncherTheme,
        Country, FetchServersEvent, IpPort, Server,
    },
    iced::{
        futures::channel::mpsc::UnboundedSender,
        widget::{image, pane_grid, scrollable::RelativeOffset},
    },
    mods_manager::{Install, ModName, PackageEntry, Source},
    std::{net::Ipv4Addr, path::PathBuf, sync::Arc, time::Duration},
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
pub enum NotificationMessage {
    Update,
    Clear,
}

#[derive(Debug, Clone)]
pub enum ThumbnailMessage {
    Started(UnboundedSender<MapName>),
    Thumbnail(MapName, Option<image::Handle>),
    Error(MapName, Arc<teamwork::Error>),
    Wait,
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
    PingFilterEnabled(bool),
    GameModeChecked(GameModeId, bool),
    CountryFilterEnabled(bool),
    GameModeFilterEnabled(bool),
    VacSecuredChanged(PropertyFilterSwitch),
    RtdChanged(PropertyFilterSwitch),
    AllTalkChanged(PropertyFilterSwitch),
    NoRespawnTimeChanged(PropertyFilterSwitch),
    PasswordChanged(PropertyFilterSwitch),
    RandomCritsChanged(PropertyFilterSwitch),
    SortCriterionChanged(SortCriterion),
    SortDirectionChanged(SortDirection),
    MinimumPlayersChanged(u8),
    MinimumFreeSlotsChanged(u8),
    PlayerFilterEnabled(bool),
    MapChecked(MapName, bool),
    MapFilterEnabled(bool),
    ProviderChecked(String, bool),
    ProviderFilterEnabled(bool),
    MapNameFilterChanged(String),
}

#[derive(Debug, Clone)]
pub enum SettingsMessage {
    TeamworkApiKeyChanged(String),
    SteamExecutableChanged(String),
    SourceEnabled(SourceKey, bool),
    QuitWhenLaunchChecked(bool),
    QuitWhenCopyChecked(bool),
    WindowMoved { x: i32, y: i32 },
    WindowResized { width: u32, height: u32 },
    ThemeChanged(LauncherTheme),
    OpenDirectory(PathBuf),
    MaxCacheSizeChanged(u64),
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
pub enum ScreenshotsMessage {
    Screenshots(Vec<image::Handle>),
    Next,
    Previous,
    Error(Arc<teamwork::Error>),
}

#[derive(Clone, Debug)]
pub enum AddViewMessage {
    Show,
    DownloadUrlChanged(String),
    ScanPackageToAdd(Source),
}

#[derive(Clone, Debug)]
pub enum ListViewMessage {
    ModClicked(ModName),
    RemoveMod(ModName),
}

#[derive(Clone, Debug)]
pub enum ModsMessage {
    AddView(AddViewMessage),
    ListView(ListViewMessage),
    AddMods(Source, Vec<ModName>),
    Install(ModName),
    Uninstall(ModName),
    OpenInstallDirectory(ModName),
    InstallationFinished(ModName, Install),
    UninstallationFinished(ModName),
    FoundInstalledMods(Vec<PackageEntry>),
    Error(String, String),
}

impl ModsMessage {
    pub fn error(title: impl ToString, message: impl ToString) -> Self {
        Self::Error(title.to_string(), message.to_string())
    }
}

#[derive(Debug, Clone)]
pub enum BlacklistMessage {
    Add(String),
    Extend(Vec<String>),
    Remove(usize),
    RemoveAll,
    Import,
    ImportFailed(ImportBlacklistError),
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
    Notification(NotificationMessage),
    Screenshots(ScreenshotsMessage),
    Blacklist(BlacklistMessage),
    Mods(ModsMessage),
    RefreshServers,
    ShowSettings,
    ShowServer(IpPort, MapName),
    ShowMods,
    LaunchGame(IpPort),
    CopyConnectionString(IpPort),
    Bookmarked(IpPort, bool),
    CopyToClipboard(String),
    ServerListScroll(RelativeOffset),

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

impl From<NotificationMessage> for Message {
    fn from(message: NotificationMessage) -> Self {
        Message::Notification(message)
    }
}
