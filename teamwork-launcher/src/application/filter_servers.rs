use {
    crate::application::{
        country_filter::CountryFilter, game_mode_filter::GameModeFilter, text_filter::TextFilter, Bookmarks, Country,
        PromisedValue, Server,
    },
    serde::{Deserialize, Serialize},
    std::{collections::BTreeSet, time::Duration},
};

#[derive(Serialize, Deserialize)]
pub struct Filter {
    pub text: TextFilter,
    pub country: CountryFilter,
    pub game_modes: GameModeFilter,
    pub bookmarked_only: bool,
    pub max_ping: u32,
    pub accept_ping_timeout: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            text: TextFilter::default(),
            country: CountryFilter::default(),
            game_modes: GameModeFilter::new(),
            bookmarked_only: false,
            max_ping: 50,
            accept_ping_timeout: true,
        }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            text: TextFilter::default(),
            country: CountryFilter::new(),
            game_modes: GameModeFilter::new(),
            bookmarked_only: false,
            max_ping: 500,
            accept_ping_timeout: false,
        }
    }

    pub fn accept(&self, server: &Server, bookmarks: &Bookmarks) -> bool {
        self.filter_by_bookmark(server, bookmarks)
            && self.filter_by_text(&server)
            && self.filter_by_countries(&server)
            && self.filter_by_ping(&server)
            && self.filter_by_game_mode(&server)
    }

    fn filter_by_countries(&self, server: &Server) -> bool {
        self.country.accept(&server.country)
    }
    fn filter_by_text(&self, server: &Server) -> bool {
        self.text.accept(&server.name)
    }
    fn filter_by_bookmark(&self, server: &Server, bookmarks: &Bookmarks) -> bool {
        !self.bookmarked_only || bookmarks.is_bookmarked(&server.ip_port)
    }
    fn filter_by_ping(&self, server: &Server) -> bool {
        match server.ping {
            PromisedValue::Ready(ping) => ping.as_millis() <= self.max_ping as u128,
            PromisedValue::Loading => true,
            PromisedValue::None => self.accept_ping_timeout,
        }
    }
    fn filter_by_game_mode(&self, server: &Server) -> bool {
        self.game_modes.accept(server)
    }
}
