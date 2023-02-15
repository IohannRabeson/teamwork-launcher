use {
    crate::application::{
        country_filter::CountryFilter, text_filter::TextFilter, Bookmarks, Country, PromisedValue, Server,
    },
    serde::{Deserialize, Serialize},
    std::collections::BTreeSet,
};

#[derive(Serialize, Deserialize, Default)]
pub struct Filter {
    pub text: TextFilter,
    pub country: CountryFilter,
    pub bookmarked_only: bool,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            text: TextFilter::default(),
            country: CountryFilter::new(),
            bookmarked_only: false,
        }
    }

    pub fn accept(&self, server: &Server, bookmarks: &Bookmarks) -> bool {
        self.filter_by_bookmark(server, bookmarks) && self.filter_by_text(&server) && self.filter_by_countries(&server)
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
}
