use {
    crate::application::{country_filter::CountryFilter, text_filter::TextFilter, Country, PromisedValue, Server},
    std::collections::BTreeSet,
};

pub struct Filter {
    pub text: TextFilter,
    pub country: CountryFilter,
}

impl Filter {
    pub fn new() -> Self {
        Self {
            text: TextFilter::default(),
            country: CountryFilter::new(),
        }
    }

    pub fn accept(&self, server: &Server) -> bool {
        self.filter_by_text(&server) && self.filter_by_countries(&server)
    }

    fn filter_by_countries(&self, server: &Server) -> bool {
        self.country.accept(&server.country)
    }

    fn filter_by_text(&self, server: &Server) -> bool {
        self.text.accept(&server.name)
    }
}
