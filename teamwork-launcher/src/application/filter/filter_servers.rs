use {
    crate::application::{
        filter::{
            country_filter::CountryFilter,
            game_mode_filter::GameModeFilter,
            map_filter::MapFilter,
            ping_filter::PingFilter,
            player_filter::PlayerFilter,
            properties_filter::PropertyFilterSwitch,
            provider_filter::ProviderFilter,
            sort_servers::{SortCriterion, SortDirection},
            text_filter::TextFilter,
        },
        Bookmarks, Server,
    },
    serde::{Deserialize, Serialize},
};

#[derive(Serialize, Deserialize)]
pub struct Filter {
    pub text: TextFilter,
    pub country: CountryFilter,
    pub game_modes: GameModeFilter,
    pub players: PlayerFilter,
    pub maps: MapFilter,
    pub providers: ProviderFilter,
    pub bookmarked_only: bool,
    pub ping: PingFilter,
    pub vac_secured: PropertyFilterSwitch,
    pub rtd: PropertyFilterSwitch,
    pub all_talk: PropertyFilterSwitch,
    pub no_respawn_time: PropertyFilterSwitch,
    pub random_crits: PropertyFilterSwitch,
    pub password: PropertyFilterSwitch,
    pub sort_criterion: SortCriterion,
    pub sort_direction: SortDirection,
}

impl Default for Filter {
    fn default() -> Self {
        Filter {
            text: TextFilter::default(),
            country: CountryFilter::default(),
            game_modes: GameModeFilter::default(),
            players: PlayerFilter::default(),
            maps: MapFilter::default(),
            providers: ProviderFilter::default(),
            bookmarked_only: false,
            ping: PingFilter::default(),
            vac_secured: PropertyFilterSwitch::With,
            rtd: PropertyFilterSwitch::Ignore,
            all_talk: PropertyFilterSwitch::Ignore,
            no_respawn_time: PropertyFilterSwitch::Ignore,
            random_crits: PropertyFilterSwitch::Ignore,
            password: PropertyFilterSwitch::Ignore,
            sort_criterion: SortCriterion::Ip,
            sort_direction: SortDirection::Ascending,
        }
    }
}

impl Filter {
    pub fn accept(&self, server: &Server, bookmarks: &Bookmarks) -> bool {
        self.filter_by_bookmark(server, bookmarks)
            && self.filter_by_text(server)
            && self.filter_by_player(server)
            && self.filter_by_countries(server)
            && self.filter_by_ping(server)
            && self.filter_by_game_mode(server)
            && self.filter_by_properties(server)
            && self.filter_by_maps(server)
            && self.filter_by_providers(server)
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
        self.ping.accept(server)
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
            && self.random_crits.accept(|s| s.has_random_crits, server)
    }
    fn filter_by_player(&self, server: &Server) -> bool {
        self.players.accept(server)
    }
    fn filter_by_maps(&self, server: &Server) -> bool {
        if !self.maps.enabled {
            return true;
        }

        self.maps.dictionary.is_checked(&server.map)
    }
    fn filter_by_providers(&self, server: &Server) -> bool {
        self.providers.accept(server)
    }
}
