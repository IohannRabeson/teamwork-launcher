use {
    crate::application::{
        country_filter::CountryFilter, game_mode_filter::GameModeFilter, text_filter::TextFilter, Bookmarks, PromisedValue,
        Server,
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
    pub vac_secured_only: bool,
    pub with_rtd_only: bool,
    pub with_all_talk_only: bool,
    pub with_no_respawn_time_only: bool,
    pub with_random_crits: bool,
    pub exclude_password: bool,
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
            vac_secured_only: true,
            with_rtd_only: false,
            with_all_talk_only: false,
            with_no_respawn_time_only: false,
            with_random_crits: false,
            exclude_password: false,
        }
    }
}

impl Filter {
    pub fn accept(&self, server: &Server, bookmarks: &Bookmarks) -> bool {
        self.filter_by_bookmark(server, bookmarks)
            && self.filter_by_text(&server)
            && self.filter_by_countries(&server)
            && self.filter_by_ping(&server)
            && self.filter_by_game_mode(&server)
            && self.filter_by_vac(&server)
            && self.filter_by_properties(&server)
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
    fn filter_by_vac(&self, server: &Server) -> bool {
        !self.vac_secured_only || server.vac_secured
    }
    fn filter_by_properties(&self, server: &Server) -> bool {
        (!self.with_all_talk_only || server.has_all_talk)
        && (!self.with_random_crits || server.has_random_crits)
        && (!self.with_rtd_only || server.has_rtd)
        && (!self.with_no_respawn_time_only || server.has_no_respawn_time)
        && (!self.exclude_password || !server.need_password)
    }
}
