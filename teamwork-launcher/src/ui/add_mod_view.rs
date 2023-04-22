use {
    crate::{
        application::{
            message::{AddViewMessage, AddViewMessage::ScanPackageToAdd, ModsMessage},
            screens::AddModView,
            Message,
        },
        ui::DEFAULT_SPACING,
    },
    iced::{
        theme,
        widget::{button, column, container, horizontal_space, row, text, text_input},
        Alignment, Element, Length,
    },
    iced_aw::{floating_element::Anchor, native::FloatingElement, Spinner},
    mods_manager::Source,
};

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
            let mut add_button = button("Add mod!").style(theme::Button::Positive);
            let scan_package_message = Message::Mods(ModsMessage::AddView(ScanPackageToAdd(Source::DownloadUrl(
                context.download_url.clone(),
            ))));
            if context.is_form_valid {
                add_button = add_button.on_press(scan_package_message.clone());
            }

            let input = row![
                horizontal_space(Length::Fill),
                text_input("Enter a download url", &context.download_url)
                    .on_input(|text| Message::Mods(ModsMessage::AddView(AddViewMessage::DownloadUrlChanged(text))))
                    .id(context.download_url_text_input.clone())
                    .width(Length::FillPortion(3))
                    .on_submit(scan_package_message),
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
