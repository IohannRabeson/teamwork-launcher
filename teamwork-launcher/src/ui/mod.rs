use {
    crate::application::{
        filter::{
            properties_filter::PropertyFilterSwitch,
            sort_servers::{SortCriterion, SortDirection},
        },
        SettingsMessage,
    },
    iced::Length,
};

pub mod buttons;
pub mod filter;
mod form;
pub mod header;
pub mod main;
pub mod server_details;
pub mod settings;
pub mod styles;
pub mod widgets;
pub mod mods_view;
pub mod add_mod_view;
pub mod color;

const PICK_LIST_WIDTH: Length = Length::Fixed(120.0);
const DEFAULT_SPACING: f32 = 8.0;

const PROPERTY_FILTER_VALUES: [PropertyFilterSwitch; 3] = [
    PropertyFilterSwitch::With,
    PropertyFilterSwitch::Without,
    PropertyFilterSwitch::Ignore,
];

/// List of criterion exposed by the UI
pub(crate) const AVAILABLE_CRITERION: [SortCriterion; 7] = [
    SortCriterion::Ip,
    SortCriterion::Name,
    SortCriterion::Country,
    SortCriterion::Ping,
    SortCriterion::Players,
    SortCriterion::PlayerSlots,
    SortCriterion::FreePlayerSlots,
];

/// List of criterion exposed by the UI
pub(crate) const AVAILABLE_DIRECTIONS: [SortDirection; 2] = [SortDirection::Ascending, SortDirection::Descending];
