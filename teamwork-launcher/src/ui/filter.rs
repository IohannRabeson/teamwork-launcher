use {
    crate::{
        application::{
            Filter,
            FilterMessage,
            game_mode::GameModes,
            Message, properties_filter::PropertyFilterSwitch, Property, sort_servers::{SortCriterion, SortDirection},
        },
        icons,
        ui::{buttons::svg_button, widgets::tooltip},
    },
    iced::{
        Element,
        Length, widget::{vertical_space, checkbox, column, horizontal_space, pick_list, row, slider, text, text_input, tooltip::Position},
    },
    itertools::Itertools,
};
use crate::application::servers_counts::ServersCounts;

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

pub fn text_filter_options(filter: &Filter) -> Element<Message> {
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

pub fn country_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    filter
        .country
        .dictionary.iter()
        .map(|(country, checked)|(country, checked, *counts.countries.get(country).unwrap_or(&0)))
        .fold(column![].spacing(4), |column, (country, checked, count)| {
            let label = format!("{} ({})", country.name(), count);

            column.push(checkbox(label, checked, |checked| {
                Message::Filter(FilterMessage::CountryChecked(country.clone(), checked))
            }))
        })
        .push(vertical_space(Length::Fixed(8.0)))
        .push(checkbox("No country", filter.country.no_countries, |checked| {
            Message::Filter(FilterMessage::NoCountryChecked(checked))
        }))
        .into()
}

pub fn bookmark_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    checkbox(
        format!("Bookmarks ({})", counts.bookmarks),
        filter.bookmarked_only,
        |checked| Message::Filter(FilterMessage::BookmarkedOnlyChecked(checked)),
    )
    .into()
}

const MAX_PING: u32 = 250;
const MIN_PING: u32 = 5;

pub fn ping_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    column![
        row![
            text("Max:"),
            slider(MIN_PING..=MAX_PING, filter.max_ping, |value| Message::Filter(
                FilterMessage::MaxPingChanged(value)
            )),
            text(format!("{}ms", filter.max_ping))
        ]
        .spacing(8),
        checkbox(
            format!("Timeouts ({})", counts.timeouts),
            filter.accept_ping_timeout,
            |checked| Message::Filter(FilterMessage::AcceptPingTimeoutChanged(checked))
        )
    ]
    .spacing(4)
    .into()
}

pub fn game_modes_filter<'l>(
    filter: &'l Filter,
    game_modes: &'l GameModes,
    counts: &'l ServersCounts,
) -> Element<'l, Message> {
    filter
        .game_modes
        .game_modes()
        .filter_map(|(id, enabled)| game_modes.get(id).map(|mode| (id, mode, enabled)))
        .sorted_by(|(_, l, _), (_, r, _)| l.title.cmp(&r.title))
        .filter_map(|(id, mode, enabled)| {
            let count = *counts.game_modes.get(id).unwrap_or(&0);

            if count == 0 {
                return None
            }

            Some((id, mode, enabled, count))
        })
        .fold(column![].spacing(4), |column, (id, mode, enabled, count)| {
            let label = format!("{} ({})", mode.title, count);
            let check_box = checkbox(&label, *enabled, |checked| {
                Message::Filter(FilterMessage::GameModeChecked(id.clone(), checked))
            });

            column.push(tooltip(check_box, &mode.description, Position::Bottom))
        })
        .into()
}

const PROPERTY_FILTER_VALUES: [PropertyFilterSwitch; 3] = [
    PropertyFilterSwitch::With,
    PropertyFilterSwitch::Without,
    PropertyFilterSwitch::Ignore,
];
const PICK_LIST_WIDTH: Length = Length::Fixed(90.0);

fn property_switch<'l>(
    label: String,
    property: PropertyFilterSwitch,
    f: impl Fn(PropertyFilterSwitch) -> Message + 'l,
) -> Element<'l, Message> {
    let selector = pick_list(PROPERTY_FILTER_VALUES.as_slice(), Some(property), f)
        .text_size(16)
        .padding([2, 4])
        .width(PICK_LIST_WIDTH);

    row![text(label), horizontal_space(Length::Fill), selector].spacing(8).into()
}

pub fn server_properties_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    column![
        property_switch(
            format!(
                "Valve secured ({})",
                counts.properties.get(&Property::VacSecured).unwrap_or(&0)
            ),
            filter.vac_secured,
            |checked| Message::Filter(FilterMessage::VacSecuredChanged(checked))
        ),
        property_switch(
            format!("Roll the dice ({})", counts.properties.get(&Property::Rtd).unwrap_or(&0)),
            filter.rtd,
            |checked| Message::Filter(FilterMessage::RtdChanged(checked))
        ),
        property_switch(
            format!("All talk ({})", counts.properties.get(&Property::AllTalk).unwrap_or(&0)),
            filter.all_talk,
            |checked| Message::Filter(FilterMessage::AllTalkChanged(checked))
        ),
        property_switch(
            format!(
                "No respawn time ({})",
                counts.properties.get(&Property::NoRespawnTime).unwrap_or(&0)
            ),
            filter.no_respawn_time,
            |checked| Message::Filter(FilterMessage::NoRespawnTimeChanged(checked))
        ),
        property_switch(
            format!("Password ({})", counts.properties.get(&Property::Password).unwrap_or(&0)),
            filter.password,
            |checked| Message::Filter(FilterMessage::PasswordChanged(checked))
        ),
    ]
    .spacing(4)
    .into()
}

pub fn players_filter(filter: &Filter) -> Element<Message> {
    column![
        row![
            text("Minimum players:"),
            slider(0..=filter.players.maximum_players, filter.players.minimum_players, |value| {
                Message::Filter(FilterMessage::MinimumPlayersChanged(value))
            }),
            text(filter.players.minimum_players.to_string())
        ]
        .spacing(8),
        row![
            text("Minimum free slots:"),
            slider(
                0..=filter.players.maximum_free_slots,
                filter.players.minimum_free_slots,
                |value| Message::Filter(FilterMessage::MinimumFreeSlotsChanged(value))
            ),
            text(filter.players.minimum_free_slots.to_string())
        ]
        .spacing(8)
    ]
    .spacing(4)
    .into()
}

const AVAILABLE_CRITERION: [SortCriterion; 6] = [
    SortCriterion::Ip,
    SortCriterion::Name,
    SortCriterion::Country,
    SortCriterion::Ping,
    SortCriterion::Players,
    SortCriterion::PlayerSlots,
];

const AVAILABLE_DIRECTIONS: [SortDirection; 2] = [SortDirection::Ascending, SortDirection::Descending];

pub fn server_sort(filter: &Filter) -> Element<Message> {
    column![
        row![
            text("Criterion:"),
            horizontal_space(Length::Fill),
            pick_list(&AVAILABLE_CRITERION[..], Some(filter.sort_criterion), |value| {
                Message::Filter(FilterMessage::SortCriterionChanged(value))
            })
            .text_size(16)
            .padding([2, 4])
            .width(PICK_LIST_WIDTH),
        ]
        .spacing(4),
        row![
            text("Direction:"),
            horizontal_space(Length::Fill),
            pick_list(&AVAILABLE_DIRECTIONS[..], Some(filter.sort_direction), |value| {
                Message::Filter(FilterMessage::SortDirectionChanged(value))
            })
            .text_size(16)
            .padding([2, 4])
            .width(PICK_LIST_WIDTH),
        ]
    ]
    .spacing(4)
    .into()
}

pub fn maps_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    filter.maps.dictionary.iter()
        .filter_map(|(name, enabled)|{
            let count = *counts.maps.get(name).unwrap_or(&0);

            if count == 0 {
                return None
            }

            Some((name, enabled, count))
        })
        .fold(column![].spacing(4), |column, (name, enabled, count)|{
            let label = format!("{} ({})", name.as_str(), count);

            column.push(checkbox(label, enabled, move |checked|Message::Filter(FilterMessage::MapChecked(name.clone(), checked))))
        }).into()
}