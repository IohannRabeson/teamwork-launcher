use {
    crate::{
        application::{servers_source::ServersSource, Message, UserSettings},
        ui::SettingsMessage,
    },
    iced::{
        widget::{button, checkbox, column, container, horizontal_space, row, text, text_input, vertical_space},
        Element, Length,
    },
};

pub fn view<'l>(settings: &'l UserSettings, sources: &'l [ServersSource]) -> Element<'l, Message> {
    column![
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
        )
    ]
    .padding(8)
    .spacing(4)
    .into()
}

fn section_title<'a, Message>(label: &str) -> Element<'a, Message> {
    text(label).size(25).into()
}

/// Compose a field by creating a label and an element.
fn field<'a>(
    label: Option<&str>,
    description: Option<&str>,
    field: impl Into<Element<'a, Message>>,
) -> Element<'a, Message> {
    let mut content = match label {
        Some(label) => column![section_title(label), vertical_space(Length::Units(4)),],
        None => column![],
    };

    if let Some(description) = description {
        content = content.push(container(text(description).size(16)).padding(8))
    }

    content.push(container(field).padding(8).width(Length::Fill)).into()
}
