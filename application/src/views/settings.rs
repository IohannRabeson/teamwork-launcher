use std::sync::Arc;

use crate::application::Messages;

use async_rwlock::RwLock;
use iced::{
    widget::{column, text, text_input, vertical_space},
    Element, Length,
};

use crate::settings::UserSettings;

pub fn settings_view<'a>(settings: Arc<RwLock<UserSettings>>) -> Element<'a, Messages> {
    let self_settings = settings.clone();

    column![
        text("Settings").font(crate::fonts::TF2_SECONDARY).size(32),
        field(
            "Game executable path:",
            text_input(
                "Game executable path",
                &settings.try_read().unwrap().game_executable_path.as_os_str().to_string_lossy(),
                move |text| {
                    let settings = self_settings.try_read().unwrap();
                    let mut settings = settings.clone();

                    settings.game_executable_path = text.into();

                    Messages::ModifySettings(settings)
                }
            )
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
