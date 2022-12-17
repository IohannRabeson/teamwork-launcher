use {
    crate::{
        application::Messages,
        settings::UserSettings,
        ui::{servers::sources_list_view, VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL},
    },
    iced::{
        widget::{checkbox, column, container, scrollable, text, text_input, vertical_space},
        Element, Length,
    },
};

pub fn settings_view(settings: &UserSettings) -> Element<Messages> {
    column![scrollable(
        column![
            field(
                "Game executable path:",
                None,
                text_input("Game executable path", &settings.game_executable_path(), |text| {
                    let mut new_settings = settings.clone();

                    new_settings.set_game_executable_path::<&str>(&text);

                    Messages::SettingsChanged(new_settings)
                })
            ),
            field(
                "Teamwork.tf API key:",
                None,
                text_input("Key", &settings.teamwork_api_key(), |text| {
                    let mut new_settings = settings.clone();

                    new_settings.set_teamwork_api_key::<&str>(&text);

                    Messages::SettingsChanged(new_settings)
                })
                .password()
            ),
            field(
                "Auto refresh favorite servers:",
                Some("If enabled, the favorites servers data will be refreshed every 5 minutes."),
                checkbox("Auto refresh", settings.auto_refresh_favorite(), |checked| {
                    let mut new_settings = settings.clone();

                    new_settings.set_auto_refresh_favorite(checked);

                    Messages::SettingsChanged(new_settings)
                })
            ),
            field(
                "Server sources:",
                Some(
                    "For each source the Teamwork API will be queried. Remember the count of query per minutes is limited."
                ),
                sources_list_view(settings.source_filter())
            ),
            field(
                "Quit when start game:",
                Some("When enabled, the launcher quits when the game starts."),
                checkbox("Quit when start game", settings.quit_on_launch(), |checked| {
                    let mut new_settings = settings.clone();

                    new_settings.set_quit_on_launch(checked);

                    Messages::SettingsChanged(new_settings)
                })
            )
        ]
        .padding(12)
        .spacing(VISUAL_SPACING_SMALL),
    )]
    .into()
}

/// Compose a field by creating a label and an element.
fn field<'a>(label: &str, description: Option<&str>, field: impl Into<Element<'a, Messages>>) -> Element<'a, Messages> {
    let mut content = column![text(label).size(25), vertical_space(Length::Units(VISUAL_SPACING_SMALL)),];

    if let Some(description) = description {
        content = content.push(container(text(description).size(16)).padding(VISUAL_SPACING_MEDIUM))
    }

    content
        .push(container(field).padding(VISUAL_SPACING_MEDIUM).width(Length::Fill))
        .padding(VISUAL_SPACING_MEDIUM)
        .into()
}
