use {
    crate::application::server::Server,
    async_stream::stream,
    futures::stream::FuturesUnordered,
    iced::futures::{Stream, StreamExt},
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

pub fn fetch_servers(urls: Vec<UrlWithKey>) -> impl Stream<Item = FetchServersEvent> {
    const SERVERS_CHUNK_SIZE: usize = 50;

    stream! {
        yield FetchServersEvent::Start;
        let client = teamwork::Client::default();
        let mut request_servers = FuturesUnordered::from_iter(urls.into_iter().map(|url|client.get_servers(url)));

        while let Some(result) = request_servers.next().await {
            match result {
                Ok(servers) => {
                    let servers: Vec<Server> = servers.into_iter().map(create_server).collect();

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

fn create_server(server: teamwork::Server) -> Server {
    server.into()
}

#[cfg(test)]
mod tests {
    use {
        super::FetchServersEvent,
        iced::futures::{pin_mut, StreamExt},
        teamwork::UrlWithKey,
    };

    #[tokio::test]
    async fn smoke_test_fetch_servers() {
        let api_key = std::env::var("TEST_TEAMWORK_API_KEY").unwrap();
        let stream = super::fetch_servers(vec![UrlWithKey::new(
            "https://teamwork.tf/api/v1/quickplay/payload/servers",
            &api_key,
        )]);

        pin_mut!(stream);

        let mut results = Vec::new();

        while let Some(event) = stream.next().await {
            if let FetchServersEvent::Servers(servers) = event {
                results.extend(servers.into_iter());
            }
        }

        assert!(results.len() > 0);
    }
}
