use {
    iced::{
        widget::{pick_list, slider, tooltip::Position},
        Length,
    },
    itertools::Itertools,
};
use crate::application::Property;
use {
    crate::{
        application::{filter_servers::PropertyFilterSwitch, game_mode::GameModes, Filter, FilterMessage, Message},
        icons,
        ui::{buttons::svg_button, widgets::tooltip},
    },
    iced::{
        widget::{checkbox, column, horizontal_space, row, text, text_input},
        Element,
    },
};
use crate::application::ServersCounts;

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
        .available_countries()
        .fold(column![].spacing(4), |column, country| {
            let label = format!("{} ({})", country.name(), counts.countries.get(country).unwrap_or(&0));

            column.push(checkbox(label, filter.country.is_checked(country), |checked| {
                Message::Filter(FilterMessage::CountryChecked(country.clone(), checked))
            }))
        })
        .push(checkbox("No country", filter.country.accept_no_country(), |checked| {
            Message::Filter(FilterMessage::NoCountryChecked(checked))
        }))
        .into()
}

pub fn bookmark_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    checkbox(format!("Bookmarks ({})", counts.bookmarks), filter.bookmarked_only, |checked| {
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

pub fn game_modes_filter<'l>(filter: &'l Filter, game_modes: &'l GameModes, counts: &'l ServersCounts) -> Element<'l, Message> {
    filter
        .game_modes
        .game_modes()
        .filter_map(|(id, enabled)| game_modes.get(&id).map(|mode| (id, mode, enabled)))
        .sorted_by(|(_, l, _), (_, r, _)| l.title.cmp(&r.title))
        .fold(column![].spacing(4), |column, (id, mode, enabled)| {
            let label = format!("{} ({})", mode.title, counts.game_modes.get(id).unwrap_or(&0));
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

fn property_switch<'l>(
    label: String,
    property: PropertyFilterSwitch,
    f: impl Fn(PropertyFilterSwitch) -> Message + 'l,
) -> Element<'l, Message> {
    let selector = pick_list(PROPERTY_FILTER_VALUES.as_slice(), Some(property), f)
        .text_size(16)
        .padding([2, 4])
        .width(80);

    row![text(label), horizontal_space(Length::Fill), selector].spacing(8).into()
}

pub fn server_properties_filter<'l>(filter: &'l Filter, counts: &'l ServersCounts) -> Element<'l, Message> {
    column![
        property_switch(
            format!("Valve secured ({})", counts.properties.get(&Property::VacSecured).unwrap_or(&0)),
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
            format!("No respawn time ({})", counts.properties.get(&Property::NoRespawnTime).unwrap_or(&0)),
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