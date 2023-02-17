use {
    iced::widget::{slider, tooltip::Position},
    itertools::Itertools,
    std::collections::{btree_map, BTreeMap},
};

use {
    crate::{
        application::{game_mode::GameModes, Filter, FilterMessage, Message, Server},
        icons,
        ui::{buttons::svg_button, widgets::tooltip},
    },
    iced::{
        widget::{checkbox, column, row, text, text_input},
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
    .spacing(4)
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
            let check_box = checkbox(&label, *enabled, |checked| {
                Message::Filter(FilterMessage::GameModeChecked(id.clone(), checked))
            });

            column.push(tooltip(check_box, &mode.description, Position::Bottom))
        })
        .into()
}

/// Count each element.
/// For example with this collection `[3, 3, 3, 2, 2, 1]`
/// The result will be: `3 -> 3, 2 -> 2, 1 -> 1`
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

pub fn server_properties_filter(filter: &Filter) -> Element<Message> {
    column![
        checkbox("With RTD", filter.with_rtd_only, |checked|Message::Filter(FilterMessage::WithRtdOnlyChanged(checked))),
        checkbox("With all talk", filter.with_all_talk_only, |checked|Message::Filter(FilterMessage::WithAllTalkOnlyChanged(checked))),
        checkbox("With no respawn time", filter.with_no_respawn_time_only, |checked|Message::Filter(FilterMessage::WithNoRespawnTimeOnlyChanged(checked))),
        checkbox("Without password", filter.exclude_password, |checked|Message::Filter(FilterMessage::ExcludePasswordChanged(checked))),
        checkbox("With VAC", filter.vac_secured_only, |checked|Message::Filter(FilterMessage::VacSecuredOnlyChanged(checked))),
    ].spacing(4).into()
}

#[cfg(test)]
mod tests {
    use crate::ui::filter::histogram;

    #[test]
    fn test_histogram() {
        let numbers = vec![3, 3, 3, 2, 2, 1];
        let h = histogram(numbers.iter());

        assert_eq!(h.get(&3), Some(&3));
        assert_eq!(h.get(&2), Some(&2));
        assert_eq!(h.get(&1), Some(&1));
        assert_eq!(h.get(&0), None);
    }
}
