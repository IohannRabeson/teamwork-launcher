use {
    crate::application::Server,
    serde::{Deserialize, Serialize},
    std::{
        cmp::Ordering,
        fmt::{Display, Formatter},
    },
};

#[derive(Serialize, Deserialize, Clone, Copy, Eq, PartialEq, Debug)]
pub enum SortCriterion {
    Ip,
    Name,
    Country,
    Ping,
    PlayerSlots,
    Players,
    FreePlayerSlots,
    Map,
}

impl Display for SortCriterion {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SortCriterion::Name => {
                write!(f, "Name")
            }
            SortCriterion::Ip => {
                write!(f, "Ip")
            }
            SortCriterion::Country => {
                write!(f, "Country")
            }
            SortCriterion::Ping => {
                write!(f, "Ping")
            }
            SortCriterion::PlayerSlots => {
                write!(f, "Player slots")
            }
            SortCriterion::Players => {
                write!(f, "Players")
            }
            SortCriterion::FreePlayerSlots => {
                write!(f, "Free slots")
            }
            SortCriterion::Map => {
                write!(f, "Map")
            }
        }
    }
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Eq, PartialEq)]
pub enum SortDirection {
    Ascending,
    Descending,
}

impl Display for SortDirection {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            SortDirection::Ascending => {
                write!(f, "Ascending")
            }
            SortDirection::Descending => {
                write!(f, "Descending")
            }
        }
    }
}

pub fn sort_servers(criterion: SortCriterion, left: &Server, right: &Server) -> Ordering {
    match criterion {
        SortCriterion::Name => left.name.cmp(&right.name),
        SortCriterion::Ip => left.ip_port.cmp(&right.ip_port),
        SortCriterion::Country => left.country.cmp(&right.country),
        SortCriterion::Ping => left.ping.cmp(&right.ping),
        SortCriterion::PlayerSlots => left.max_players_count.cmp(&right.max_players_count),
        SortCriterion::Players => left.current_players_count.cmp(&right.current_players_count),
        SortCriterion::FreePlayerSlots => left.free_slots().cmp(&right.free_slots()),
        SortCriterion::Map => left.map.cmp(&right.map),
    }
}
