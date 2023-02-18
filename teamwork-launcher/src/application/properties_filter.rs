use std::fmt::{Display, Formatter};
use serde::{Deserialize, Serialize};
use crate::application::Server;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, Ord, PartialOrd, PartialEq, Eq)]
pub enum PropertyFilterSwitch {
    With,
    Without,
    Ignore,
}

impl Display for PropertyFilterSwitch {
    fn fmt(&self, f: &mut Formatter) -> std::fmt::Result {
        match self {
            PropertyFilterSwitch::With => {
                write!(f, "Require")
            }
            PropertyFilterSwitch::Without => {
                write!(f, "Reject")
            }
            PropertyFilterSwitch::Ignore => {
                write!(f, "Ignore")
            }
        }
    }
}

impl PropertyFilterSwitch {
    pub fn accept(&self, f: impl Fn(&Server) -> bool, server: &Server) -> bool {
        match self {
            PropertyFilterSwitch::With => (f)(server),
            PropertyFilterSwitch::Without => !(f)(server),
            PropertyFilterSwitch::Ignore => true,
        }
    }
}
