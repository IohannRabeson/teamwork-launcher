use crate::ui::DEFAULT_SPACING;
use mods_manager::Source;
use iced::widget::{button, column, container, horizontal_space, row, text, text_input};
use iced::{Alignment, Element, Length};
use iced_aw::floating_element::Anchor;
use iced_aw::native::FloatingElement;
use iced_aw::Spinner;
use crate::application::Message;
use crate::application::message::AddViewMessage::ScanPackageToAdd;
use crate::application::message::ModsMessage;
use crate::application::screens::AddModView;
use crate::application::message::AddViewMessage;

pub fn view(context: &AddModView) -> Element<Message> {
    match context.scanning {
        true => container(
            Spinner::new()
                .circle_radius(4.0)
                .width(Length::Fixed(64.0))
                .height(Length::Fixed(64.0)),
        )
        .width(Length::Fill)
        .height(Length::Fill)
        .center_x()
        .center_y()
        .into(),
        false => {
            let mut add_button = button("Add HUB!");
            let scan_package_message = Message::Mods(ModsMessage::AddView(ScanPackageToAdd(Source::DownloadUrl(
                context.download_url.clone(),
            ))));
            if context.is_form_valid {
                add_button = add_button.on_press(scan_package_message.clone());
            }

            let input = row![
                horizontal_space(Length::Fill),
                text_input("Enter a download url", &context.download_url, |text| Message::Mods(
                    ModsMessage::AddView(AddViewMessage::DownloadUrlChanged(text))
                ))
                .id(context.download_url_text_input.clone())
                .width(Length::FillPortion(3))
                .on_submit(scan_package_message.clone()),
                horizontal_space(Length::Fill)
            ];

            let mut main_column = column![input].align_items(Alignment::Center).spacing(DEFAULT_SPACING);

            if let Some(error) = context.error.as_ref() {
                main_column = main_column.push(text(error))
            }

            main_column = main_column.push(add_button);

            let content = container(main_column).height(Length::Fill).center_y();

            FloatingElement::new(content, || button("X").on_press(Message::Back).into())
                .anchor(Anchor::NorthEast)
                .into()
        }
    }
}
