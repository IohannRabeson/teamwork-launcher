use iced::{system, theme};
use {
    crate::{
        application::{servers_source::ServersSource, Message, UserSettings},
        ui::SettingsMessage,
    },
    iced::{
        widget::{checkbox, pick_list, column, container, text, text_input, scrollable},
        Element, Length,
    },
};
use crate::application::user_settings::LauncherTheme;
use crate::ui::styles::BoxContainerStyle;

const THEMES: [LauncherTheme; 2] = [
    LauncherTheme::Blue,
    LauncherTheme::Red,
];

pub fn view<'l>(settings: &'l UserSettings, sources: &'l [ServersSource], system_info: Option<&'l system::Information>) -> Element<'l, Message> {
    scrollable(column![
        field(
            Some("Teamwork.tf API key"),
            None,
            text_input("Teamwork.tf API key", &settings.teamwork_api_key, |text| Message::Settings(
                SettingsMessage::TeamworkApiKeyChanged(text)
            ))
            .password()
        ),
        field(
            Some("Steam executable path"),
            None,
            text_input("Steam executable path", &settings.steam_executable_path, |text| {
                Message::Settings(SettingsMessage::SteamExecutableChanged(text))
            })
        ),
        field(
            Some("Sources"),
            None,
            sources.iter().fold(column![].spacing(4), |column, source| {
                column.push(checkbox(source.display_name(), source.enabled(), |checked| {
                    Message::Settings(SettingsMessage::SourceEnabled(source.key().clone(), checked))
                }))
            })
        ),
        field(
            Some("Auto quit"),
            None,
            column![
                checkbox("Quit when the game is launched", settings.quit_on_launch, |checked| {
                    Message::Settings(SettingsMessage::QuitWhenLaunchChecked(checked))
                }),
                checkbox(
                    "Quit when the connection string is copied",
                    settings.quit_on_copy,
                    |checked| Message::Settings(SettingsMessage::QuitWhenCopyChecked(checked))
                ),
            ]
            .spacing(4)
        ),
        field(
            Some("System information"),
            None,
            text(format!("Memory usage: {}",
                match system_info.map(|info|info.memory_used) {
                    Some(memory_usage) => {
                        match memory_usage {
                            Some(memory_usage) => bytesize::ByteSize(memory_usage).to_string(),
                            None => String::from("unknown"),
                        }
                    },
                    None => String::from("Loading"),
                }
            ))
        ),
        field(
            Some("Team"),
            None,
            pick_list(THEMES.as_slice(), Some(settings.theme), |value|Message::Settings(SettingsMessage::ThemeChanged(value))),
        )
    ]
    .padding(8)
    .spacing(8))
        .height(Length::Fill)
    .into()
}

fn section_title(label: &str) -> Element<Message> {
    container(text(label).size(25)).padding(8).into()
}

/// Compose a field by creating a label and an element.
fn field<'a>(
    label: Option<&'a str>,
    description: Option<&'a str>,
    field: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = match label {
        Some(label) => column![section_title(label)],
        None => column![],
    };

    if let Some(description) = description {
        content = content.push(container(text(description).size(16)).padding(8))
    }

    container(content.push(container(field).padding(8).width(Length::Fill))).style(theme::Container::Custom(Box::new(BoxContainerStyle{}))).into()
}
