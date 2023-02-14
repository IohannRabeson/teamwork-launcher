use {
    crate::{
        application::Message,
        ui::SettingsMessage,
    },
    iced::{
        Element,
        Length, widget::{button, column, horizontal_space, row, text_input},
    },
};
use crate::application::UserSettings;

pub fn view(settings: &UserSettings) -> Element<Message> {
    column![
        row![horizontal_space(Length::Fill), button("Close").on_press(Message::Back)],
        text_input("Teamwork.tf API key", &settings.teamwork_api_key, |text| Message::Settings(
            SettingsMessage::TeamworkApiKeyChanged(text)
        ))
        .password(),
        text_input("Steam executable path", &settings.steam_executable_path, |text| {
            Message::Settings(SettingsMessage::SteamExecutableChanged(text))
        })
    ]
    .into()
}
