use {
    crate::{
        application::{servers_source::ServersSource, user_settings::LauncherTheme, Message, UserSettings},
        ui::{styles::BoxContainerStyle, SettingsMessage},
    },
    iced::{
        theme,
        widget::{checkbox, column, container, pick_list, scrollable, text, text_input},
        Element, Length,
    },
};

const THEMES: [LauncherTheme; 2] = [LauncherTheme::Blue, LauncherTheme::Red];

pub fn view<'l>(settings: &'l UserSettings, sources: &'l [ServersSource]) -> Element<'l, Message> {
    let teamwork_api_key_field: Element<'l, Message> = match settings.is_teamwork_api_key_from_env() {
        true => text("API key specified as environment variable").into(),
        false => {
            text_input("Put your Teamwork.tf API key here", &settings.teamwork_api_key(), |text| Message::Settings(
                SettingsMessage::TeamworkApiKeyChanged(text)
            ))
            .password().into()
        }
    };

    scrollable(
        column![
            field(
                Some("Teamwork.tf API key"),
                None,
                teamwork_api_key_field
            ),
            field(
                Some("Steam executable file path"),
                None,
                text_input("Put Steam executable file path here", &settings.steam_executable_path, |text| {
                    Message::Settings(SettingsMessage::SteamExecutableChanged(text))
                })
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
                        "Quit when the connection string is copied to the clipboard",
                        settings.quit_on_copy,
                        |checked| Message::Settings(SettingsMessage::QuitWhenCopyChecked(checked))
                    ),
                ]
                .spacing(4)
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
        .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
        .into()
}
