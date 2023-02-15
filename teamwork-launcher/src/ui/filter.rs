use {
    crate::{
        application::{Filter, FilterMessage, Message},
        icons,
        ui::buttons::svg_button,
    },
    iced::{
        widget::{button, checkbox, column, row, text_input},
        Element,
    },
};

pub fn text_filter(filter: &Filter) -> Element<Message> {
    row![
        text_input("Filter", filter.text.text(), |text| {
            Message::Filter(FilterMessage::TextChanged(text))
        }),
        svg_button(icons::CLEAR_ICON.clone(), 20).on_press(Message::Filter(FilterMessage::TextChanged(String::new()))),
    ]
    .spacing(4)
    .into()
}

pub fn country_filter(filter: &Filter) -> Element<Message> {
    filter
        .country
        .available_countries()
        .fold(column![].spacing(4), |column, country| {
            column.push(checkbox(country.name(), filter.country.is_checked(country), |checked| {
                Message::Filter(FilterMessage::CountryChecked(country.clone(), checked))
            }))
        })
        .push(checkbox("No country", filter.country.accept_no_country(), |checked| {
            Message::Filter(FilterMessage::NoCountryChecked(checked))
        }))
        .into()
}
