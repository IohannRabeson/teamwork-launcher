use iced::{
    theme,
    widget::{container, slider, toggler},
    Length, Theme,
};

use {
    crate::{
        application::{Filter, FilterMessage, Message},
        icons,
        ui::buttons::svg_button,
    },
    iced::{
        widget::{button, checkbox, column, row, text, text_input},
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

pub fn advanced_text_filter(filter: &Filter) -> Element<Message> {
    column![
        checkbox("Ignore case", filter.text.ignore_case, |checked| {
            Message::Filter(FilterMessage::IgnoreCaseChanged(checked))
        }),
        checkbox("Ignore accents", filter.text.ignore_accents, |checked| {
            Message::Filter(FilterMessage::IgnoreAccentChanged(checked))
        })
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

pub fn bookmark_filter(filter: &Filter) -> Element<Message> {
    checkbox("Bookmarks only", filter.bookmarked_only, |checked| {
        Message::Filter(FilterMessage::BookmarkedOnlyChecked(checked))
    })
    .into()
}

const MAX_PING: u32 = 250;
const MIN_PING: u32 = 5;

pub fn ping_filter(filter: &Filter) -> Element<Message> {
    column![
        row![
            slider(MIN_PING..=MAX_PING, filter.max_ping, |value| Message::Filter(
                FilterMessage::MaxPingChanged(value)
            )),
            text(format!("{}ms", filter.max_ping))
        ]
        .spacing(8),
        checkbox("Timeouts", filter.accept_ping_timeout, |checked| Message::Filter(
            FilterMessage::AcceptPingTimeoutChanged(checked)
        ))
    ]
    .into()
}
