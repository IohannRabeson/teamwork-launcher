use {
    iced::{
        theme,
        widget::{container, slider, toggler, Column},
        Length, Theme,
    },
    itertools::Itertools,
    std::collections::{btree_map, BTreeMap},
};

use {
    crate::{
        application::{
            game_mode::{GameModeId, GameModes},
            Filter, FilterMessage, Message, Server,
        },
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

pub fn country_filter<'l>(filter: &'l Filter, servers: &'l [Server]) -> Element<'l, Message> {
    let counts = histogram(servers.iter().filter_map(|server| server.country.get()));

    filter
        .country
        .available_countries()
        .fold(column![].spacing(4), |column, country| {
            let label = format!("{} ({})", country.name(), counts.get(country).unwrap_or(&0));

            column.push(checkbox(label, filter.country.is_checked(country), |checked| {
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

pub fn game_modes_filter<'l>(filter: &'l Filter, game_modes: &'l GameModes, servers: &'l [Server]) -> Element<'l, Message> {
    let counts = histogram(servers.iter().map(|server| &server.game_modes).flatten());

    filter
        .game_modes
        .game_modes()
        .filter_map(|(id, enabled)| game_modes.get(&id).map(|mode| (id, mode, enabled)))
        .sorted_by(|(_, l, _), (_, r, _)| l.title.cmp(&r.title))
        .fold(column![].spacing(4), |column, (id, mode, enabled)| {
            let label = format!("{} ({})", mode.title, counts.get(id).unwrap_or(&0));

            column.push(checkbox(&label, *enabled, |checked| {
                Message::Filter(FilterMessage::GameModeChecked(id.clone(), checked))
            }))
        })
        .into()
}

fn histogram<'l, T: Ord>(values: impl Iterator<Item = &'l T> + 'l) -> BTreeMap<&'l T, usize> {
    values.fold(BTreeMap::new(), |mut count, value| {
        match count.entry(value) {
            btree_map::Entry::Vacant(vacant) => {
                vacant.insert(1usize);
            }
            btree_map::Entry::Occupied(mut occupied) => {
                *occupied.get_mut() += 1;
            }
        }

        count
    })
}
