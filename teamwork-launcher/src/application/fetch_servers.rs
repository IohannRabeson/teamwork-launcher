use {
    crate::application::{server::Server, servers_source::SourceKey},
    async_stream::stream,
    iced::futures::{stream::FuturesUnordered, Stream, StreamExt},
    std::sync::Arc,
    teamwork::{Error, UrlWithKey},
};

#[derive(Debug, Clone)]
pub enum FetchServersEvent {
    Start,
    Finish,
    Servers(Vec<Server>),
    Error(Arc<Error>),
}

async fn get_servers(client: &teamwork::Client, url: UrlWithKey, key: SourceKey) -> Result<Vec<Server>, Error> {
    let mut servers: Vec<Server> = client
        .get_servers(url)
        .await?
        .into_iter()
        .map(std::convert::Into::<Server>::into)
        .collect();

    for server in servers.iter_mut() {
        server.source_key = Some(key.clone());
    }

    Ok(servers)
}

pub fn fetch_servers(urls: Vec<(SourceKey, UrlWithKey)>) -> impl Stream<Item = FetchServersEvent> {
    const SERVERS_CHUNK_SIZE: usize = 50;

    stream! {
        yield FetchServersEvent::Start;
        let client = teamwork::Client::default();
        let mut request_servers = FuturesUnordered::from_iter(urls.into_iter().map(|(source_key, url)| get_servers(&client, url, source_key.clone())));

        while let Some(result) = request_servers.next().await {
            match result {
                Ok(servers) => {
                    let servers: Vec<Server> = servers.into_iter().collect();

                    for chunk in servers.as_slice().chunks(SERVERS_CHUNK_SIZE) {
                        yield FetchServersEvent::Servers(chunk.to_vec())
                    }
                }
                Err(error) => {
                    yield FetchServersEvent::Error(Arc::new(error))
                }
            }
        }

        yield FetchServersEvent::Finish;
    }
}

#[cfg(test)]
mod tests {
    use {
        super::FetchServersEvent,
        crate::application::servers_source::SourceKey,
        iced::futures::{pin_mut, StreamExt},
        teamwork::UrlWithKey,
    };

    #[tokio::test]
    async fn smoke_test_fetch_servers() {
        let api_key = std::env::var("TEST_TEAMWORK_API_KEY").unwrap();
        let stream = super::fetch_servers(vec![(
            SourceKey::new("test"),
            UrlWithKey::new("https://teamwork.tf/api/v1/quickplay/payload/servers", &api_key),
        )]);

        pin_mut!(stream);

        let mut results = Vec::new();

        while let Some(event) = stream.next().await {
            if let FetchServersEvent::Servers(servers) = event {
                results.extend(servers.into_iter());
            }
        }

        assert!(results.len() > 0);

        assert_eq!(Some(SourceKey::new("test")), results[0].source_key);
    }
}
