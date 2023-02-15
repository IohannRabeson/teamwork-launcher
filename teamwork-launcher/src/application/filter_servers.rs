use std::time::Duration;
use {
    crate::application::{
        country_filter::CountryFilter, text_filter::TextFilter, Bookmarks, Country, PromisedValue, Server,
    },
    serde::{Deserialize, Serialize},
    std::collections::BTreeSet,
};

#[derive(Serialize, Deserialize)]
pub struct Filter {
    pub text: TextFilter,
    pub country: CountryFilter,
    pub bookmarked_only: bool,
    pub max_ping: u32,
    pub accept_ping_timeout: bool,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            text: TextFilter::default(),
            country: CountryFilter::default(),
            bookmarked_only: false,
            max_ping: 150,
            accept_ping_timeout: true,
        }
    }
}

impl Filter {
    pub fn new() -> Self {
        Self {
            text: TextFilter::default(),
            country: CountryFilter::new(),
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
            PromisedValue::Ready(ping) => { ping.as_millis() <= self.max_ping as u128 }
            PromisedValue::Loading => { true }
            PromisedValue::None => { self.accept_ping_timeout }
        }
    }
}
