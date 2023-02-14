use {
    super::{VISUAL_SPACING_MEDIUM, VISUAL_SPACING_SMALL},
    crate::{application::Messages, fonts, settings::UserSettings},
    iced::widget::{checkbox, column, Column},
};

pub fn advanced_filter_view(settings: &UserSettings) -> Column<Messages> {
    let filter = settings.server_filter();

    column![
        checkbox("With players only", filter.minimum_players_count > 0, |checked| {
            let mut settings = settings.clone();

            settings.set_minimum_players_count(match checked {
                true => 1u8,
                false => 0u8,
            });

            Messages::SettingsChanged(settings)
        })
        .text_size(fonts::TEXT_FONT_SIZE),
        checkbox("Online only", filter.with_valid_ping, |checked| {
            let mut settings = settings.clone();

            settings.set_online_server_only(checked);

            Messages::SettingsChanged(settings)
        })
        .text_size(fonts::TEXT_FONT_SIZE)
    ]
    .padding(VISUAL_SPACING_MEDIUM)
    .spacing(VISUAL_SPACING_SMALL)
}
