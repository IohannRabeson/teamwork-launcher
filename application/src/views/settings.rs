use {
    crate::application::Messages,
    iced::{
        widget::{column, text, text_input, vertical_space},
        Element, Length,
    },
};

use crate::settings::UserSettings;

pub fn settings_view<'a>(settings: &'a UserSettings) -> Element<'a, Messages> {
    column![
        text("Settings").font(crate::fonts::TF2_SECONDARY).size(32),
        field(
            "Game executable path:",
            text_input("Game executable path", &settings.game_executable_path(), |text| {
                let new_settings = settings.clone();

                new_settings.set_game_executable_path::<&str>(&text);

                Messages::ModifySettings(new_settings)
            })
            .into()
        )
    ]
    .padding(12)
    .into()
}

/// Compose a field by creating a label and an element.
fn field<'a>(label: &str, field: Element<'a, Messages>) -> Element<'a, Messages> {
    column![text(label), vertical_space(Length::Units(4)), field,]
        .padding(4)
        .into()
}
