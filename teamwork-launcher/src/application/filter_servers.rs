use {
    crate::application::{
        country_filter::CountryFilter, game_mode_filter::GameModeFilter, properties_filter::PropertyFilterSwitch,
        text_filter::TextFilter, Bookmarks, PromisedValue, Server,
    },
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize)]
pub struct Filter {
    pub text: TextFilter,
    pub country: CountryFilter,
    pub game_modes: GameModeFilter,
    pub bookmarked_only: bool,
    pub max_ping: u32,
    pub accept_ping_timeout: bool,
    pub vac_secured: PropertyFilterSwitch,
    pub rtd: PropertyFilterSwitch,
    pub all_talk: PropertyFilterSwitch,
    pub no_respawn_time: PropertyFilterSwitch,
    pub random_crits: PropertyFilterSwitch,
    pub password: PropertyFilterSwitch,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            text: TextFilter::default(),
            country: CountryFilter::default(),
            game_modes: GameModeFilter::default(),
            bookmarked_only: false,
            max_ping: 50,
            accept_ping_timeout: true,
            vac_secured: PropertyFilterSwitch::With,
            rtd: PropertyFilterSwitch::Ignore,
            all_talk: PropertyFilterSwitch::Ignore,
            no_respawn_time: PropertyFilterSwitch::Ignore,
            random_crits: PropertyFilterSwitch::Ignore,
            password: PropertyFilterSwitch::Ignore,
        }
    }
}

impl Filter {
    pub fn accept(&self, server: &Server, bookmarks: &Bookmarks) -> bool {
        self.filter_by_bookmark(server, bookmarks)
            && self.filter_by_text(server)
            && self.filter_by_countries(server)
            && self.filter_by_ping(server)
            && self.filter_by_game_mode(server)
            && self.filter_by_properties(server)
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
    fn filter_by_properties(&self, server: &Server) -> bool {
        self.all_talk.accept(|s| s.has_all_talk, server)
            && self.vac_secured.accept(|s| s.vac_secured, server)
            && self.rtd.accept(|s| s.has_rtd, server)
            && self.no_respawn_time.accept(|s| s.has_no_respawn_time, server)
            && self.password.accept(|s| s.need_password, server)
    }
}
