use {
    crate::{
        application::Messages,
        servers_provider::ServersProvider,
        settings::UserSettings,
        sources::SourceKey,
        ui::{VISUAL_SPACING_BIG, VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL},
    },
    iced::{
        widget::{button, checkbox, column, container, scrollable, text, text_input, vertical_space, Container},
        Element, Length,
    },
};

pub fn settings_view<'a>(settings: &'a UserSettings, servers_provider: &'a ServersProvider) -> Element<'a, Messages> {
    scrollable(
        column![
            field(
                Some("Game executable path:"),
                None,
                text_input("Game executable path", &settings.game_executable_path(), |text| {
                    let mut new_settings = settings.clone();

                    new_settings.set_game_executable_path::<&str>(&text);

                    Messages::SettingsChanged(new_settings)
                })
            ),
            field(
                Some("Teamwork.tf API key:"),
                None,
                text_input("Key", &settings.teamwork_api_key(), |text| {
                    let mut new_settings = settings.clone();

                    new_settings.set_teamwork_api_key::<&str>(&text);

                    Messages::SettingsChanged(new_settings)
                })
                .password()
            ),
            field(
                Some("Server sources:"),
                Some(
                    "For each source the Teamwork API will be queried. \
                     Remember the count of query per minutes is limited."
                ),
                sources_list_view(settings.source_filter(servers_provider))
            ),
            field(
                Some("Auto refresh favorite servers:"),
                Some("If enabled, the favorites servers data will be refreshed every 5 minutes."),
                checkbox("Auto refresh", settings.auto_refresh_favorite(), |checked| {
                    let mut new_settings = settings.clone();

                    new_settings.set_auto_refresh_favorite(checked);

                    Messages::SettingsChanged(new_settings)
                })
            ),
            section_title("Auto quit:"),
            column![
                field(
                    None,
                    Some("If enabled, the launcher quits when the game starts."),
                    checkbox("Quit when the game is started", settings.quit_on_launch(), |checked| {
                        let mut new_settings = settings.clone();

                        new_settings.set_quit_on_launch(checked);

                        Messages::SettingsChanged(new_settings)
                    })
                ),
                field(
                    None,
                    Some("If enabled, the launcher quits when the connection string is copied to the clipboard."),
                    checkbox("Quit when connection string is copied", settings.quit_on_copy(), |checked| {
                        let mut new_settings = settings.clone();

                        new_settings.set_quit_on_copy(checked);

                        Messages::SettingsChanged(new_settings)
                    })
                ),
                field(
                    Some("Log and configuration directory: "),
                    Some("Currently editing settings is limited. For full control, you can edit the JSON file user_settings.json in the configuration directory."),
                    button("Open location").on_press(Messages::OpenConfigurationDirectory(
                        crate::directories::get_configuration_directory()
                    ))
                ),
            ]
        ]
        .padding(VISUAL_SPACING_BIG)
        .spacing(VISUAL_SPACING_SMALL),
    )
    .into()
}

fn section_title<'a>(label: &str) -> Element<'a, Messages> {
    text(label).size(25).into()
}

/// Compose a field by creating a label and an element.
fn field<'a>(
    label: Option<&str>,
    description: Option<&str>,
    field: impl Into<Element<'a, Messages>>,
) -> Element<'a, Messages> {
    let mut content = match label {
        Some(label) => column![section_title(label), vertical_space(Length::Units(VISUAL_SPACING_SMALL)),],
        None => column![],
    };

    if let Some(description) = description {
        content = content.push(container(text(description).size(16)).padding(VISUAL_SPACING_MEDIUM))
    }

    content
        .push(container(field).padding(VISUAL_SPACING_MEDIUM).width(Length::Fill))
        .into()
}

/// The `sources` parameter is a vector of tuple containing:
///  - the displayable name of the source
///  - the key of the source
///  - a boolean specifying if the checkbox is checked or not
fn sources_list_view<'a>(sources: Vec<(String, SourceKey, bool)>) -> Container<'a, Messages> {
    container(sources.into_iter().fold(
        column![].width(Length::Fill).spacing(VISUAL_SPACING_SMALL),
        |column, (name, key, checked)| {
            column.push(checkbox(name, checked, move |c| {
                Messages::SourceFilterClicked(key.clone(), c)
            }))
        },
    ))
    .width(Length::Fill)
}
