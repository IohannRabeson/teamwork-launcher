use {async_std::sync::Mutex, async_trait::async_trait};

use {
    crate::models::Country,
    std::{
        collections::{
            btree_map::Entry::{Occupied, Vacant},
            BTreeMap,
        },
        net::Ipv4Addr,
        sync::Arc,
    },
};

#[derive(thiserror::Error, Debug)]
#[error("Failed to geolocalize IP '{ip}': {message}")]
pub struct Error {
    pub ip: String,
    pub message: String,
}

impl Error {
    pub fn new(ip: String, message: &impl ToString) -> Self {
        Self {
            ip,
            message: message.to_string(),
        }
    }
}

pub type IpGeolocationService = IpGeolocator<country_is::CountryIsService>;

/// This trait must be implemented to create a geolocalization service.
#[async_trait]
pub trait Service {
    async fn locate(&self, ip: Ipv4Addr) -> Result<Country, Error>;
}

struct Inner<S: Service> {
    service: S,
    cache: BTreeMap<Ipv4Addr, Country>,
}

impl<S: Service> Inner<S> {
    fn new(service: S) -> Self {
        Self {
            service,
            cache: BTreeMap::new(),
        }
    }

    async fn locate(&mut self, ip: Ipv4Addr) -> Result<Country, Error> {
        match self.cache.entry(ip) {
            Vacant(vacant) => match self.service.locate(ip).await {
                Ok(country) => Ok(vacant.insert(country).clone()),
                Err(error) => Err(Error::new(ip.to_string(), &error)),
            },
            Occupied(occupied) => Ok(occupied.get().clone()),
        }
    }
}

#[derive(Clone)]
pub struct IpGeolocator<S: Service + Clone + Default> {
    inner: Arc<Mutex<Inner<S>>>,
}

impl<S: Service + Clone + Default> Default for IpGeolocator<S> {
    fn default() -> Self {
        Self {
            inner: Arc::new(Mutex::new(Inner::new(S::default()))),
        }
    }
}

impl<S> IpGeolocator<S>
where
    S: Service + Clone + Default,
{
    pub async fn locate(&self, ip: Ipv4Addr) -> Result<Country, Error> {
        self.inner.lock().await.locate(ip).await
    }
}

mod country_is {
    use std::net::Ipv4Addr;

    use {
        crate::models::Country,
        async_trait::async_trait,
        serde::{Deserialize, Serialize},
    };

    use super::{Error, Service};

    #[derive(Clone)]
    pub struct CountryIsService {
        reqwest_client: reqwest::Client,
    }

    #[derive(Serialize, Deserialize)]
    struct CountryIsResponse {
        ip: String,
        country: String,
    }

    const COUNTYIS_API_URL: &str = "https://api.country.is";

    impl Default for CountryIsService {
        fn default() -> Self {
            Self {
                reqwest_client: Default::default(),
            }
        }
    }

    #[async_trait]
    impl Service for CountryIsService {
        async fn locate(&self, ip: Ipv4Addr) -> Result<Country, Error> {
            let url = format!("{}/{}", COUNTYIS_API_URL, ip.to_string());
            let ip = ip.to_string();
            let reqwest_client = self.reqwest_client.clone();
            let raw_text = reqwest_client
                .get(url)
                .send()
                .await
                .map_err(|error| Error::new(ip.clone(), &error))?
                .text()
                .await
                .map_err(|error| Error::new(ip.clone(), &error))?;

            let response: CountryIsResponse =
                serde_json::from_str(&raw_text).map_err(|error| Error::new(ip.clone(), &error))?;

            Ok(Country::new(&response.country))
        }
    }
}
