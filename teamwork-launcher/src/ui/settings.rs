use {
    crate::{
        application::{
            servers_source::ServersSource, user_settings::LauncherTheme, BlacklistMessage, Message, UserSettings,
        },
        icons,
        ui::{self, blacklist::Blacklist, buttons::svg_button, styles::BoxContainerStyle, SettingsMessage},
    },
    iced::{
        theme,
        widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input},
        Element, Length,
    },
    iced_aw::NumberInput,
    std::path::PathBuf,
};

const THEMES: [LauncherTheme; 2] = [LauncherTheme::Blue, LauncherTheme::Red];

pub fn view<'l>(
    settings: &'l UserSettings,
    sources: &'l [ServersSource],
    blacklist: &'l crate::Blacklist,
    configuration_directory_path: PathBuf,
) -> Element<'l, Message> {
    let teamwork_api_key_field: Element<'l, Message> = match settings.is_teamwork_api_key_from_env() {
        true => text("API key specified as environment variable").into(),
        false => text_input("Put your Teamwork.tf API key here", &settings.teamwork_api_key())
            .on_input(|text| Message::Settings(SettingsMessage::TeamworkApiKeyChanged(text)))
            .password()
            .into(),
    };

    scrollable(
        column![
            field(Some("Teamwork.tf API key"), None, teamwork_api_key_field),
            field(
                Some("Steam executable file path"),
                None,
                text_input("Put Steam executable file path here", &settings.steam_executable_path,)
                    .on_input(|text| { Message::Settings(SettingsMessage::SteamExecutableChanged(text)) })
            ),
            field(
                Some("Team"),
                None,
                pick_list(THEMES.as_slice(), Some(settings.theme), |value| Message::Settings(
                    SettingsMessage::ThemeChanged(value)
                )),
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
                        "Quit on connection string copied to clipboard",
                        settings.quit_on_copy,
                        |checked| Message::Settings(SettingsMessage::QuitWhenCopyChecked(checked))
                    ),
                ]
                .spacing(4)
            ),
            field(
                Some("Configuration directory"),
                None,
                row![
                    text(configuration_directory_path.display()),
                    svg_button(icons::FOLDER2_OPEN.clone(), 10).on_press(Message::Settings(SettingsMessage::OpenDirectory(
                        configuration_directory_path.to_path_buf()
                    )))
                ]
                .spacing(4),
            ),
            field(
                Some("Max cache size in MB"),
                None,
                NumberInput::new(settings.max_thumbnails_cache_size_mb, 50, |value| Message::Settings(
                    SettingsMessage::MaxCacheSizeChanged(value)
                )),
            ),
            field(
                Some("Servers blacklist"),
                Some(
                    "Servers can be blacklisted by name, or by IP.\n\
                You can enter a word, like \"fastpath\", that will be searched for in the server name.\n\
                It's also possible to specify an IP address like \"127.0.0.1\" or with the port \"127.0.0.1:1234\".\n\
                The import function expects a text file containing one address per line."
                ),
                column![
                    row![
                        button("Import file").on_press(Message::Blacklist(BlacklistMessage::Import)),
                        button("Clear blacklist").on_press(Message::Blacklist(BlacklistMessage::RemoveAll)),
                    ]
                    .spacing(ui::DEFAULT_SPACING),
                    Blacklist::new(blacklist)
                ]
                .spacing(ui::DEFAULT_SPACING),
            ),
        ]
        .padding(8)
        .spacing(8),
    )
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

    container(content.push(container(field).padding(8).width(Length::Fill)))
        .style(theme::Container::Custom(Box::new(BoxContainerStyle)))
        .into()
}
