use {
    crate::application::{map::MapName, message::ThumbnailMessage},
    iced::{
        futures::{
            channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
            SinkExt,
        },
        subscription,
        widget::image,
        Subscription,
    },
    std::{
        collections::{btree_map::Entry, BTreeMap},
        sync::Arc,
        time::Duration,
    },
};

enum State {
    Starting { api_key: String },
    Ready(Context),
    Wait(Duration, Context),
}

struct Context {
    requests_sender: UnboundedSender<MapName>,
    requests_receiver: UnboundedReceiver<MapName>,
    client: teamwork::Client,
    teamwork_api_key: String,
    cache: BTreeMap<MapName, Option<image::Handle>>,
}

pub fn subscription(id: u64, api_key: &str) -> Subscription<ThumbnailMessage> {
    const SECONDS_TO_WAIT_IF_TOO_MANY_ATTEMPTS: u64 = 60;
    const SECONDS_TO_WAIT_ON_ERROR: u64 = 5;

    subscription::unfold(
        id,
        State::Starting {
            api_key: api_key.to_string(),
        },
        |state| async move {
            match state {
                State::Wait(duration, context) => {
                    tokio::time::sleep(duration).await;
                    (None, State::Ready(context))
                }
                State::Starting { api_key } => {
                    let (sender, receiver) = unbounded();
                    let context = Context {
                        requests_sender: sender.clone(),
                        requests_receiver: receiver,
                        teamwork_api_key: api_key,
                        client: teamwork::Client::default(),
                        cache: BTreeMap::new(),
                    };

                    (Some(ThumbnailMessage::Started(sender)), State::Ready(context))
                }
                State::Ready(mut context) => {
                    use iced::futures::StreamExt;

                    let map_name = context.requests_receiver.select_next_some().await;

                    match context.cache.entry(map_name.clone()) {
                        Entry::Vacant(vacant) => {
                            match context
                                .client
                                .get_map_thumbnail(&context.teamwork_api_key, map_name.as_str(), image::Handle::from_memory)
                                .await
                            {
                                Ok(thumbnail) => {
                                    vacant.insert(thumbnail.clone());
                                    (Some(ThumbnailMessage::Thumbnail(map_name, thumbnail)), State::Ready(context))
                                }
                                Err(teamwork::Error::TooManyAttempts) => {
                                    context.requests_sender.send(map_name.clone()).await.unwrap();

                                    (
                                        Some(ThumbnailMessage::Error(map_name, Arc::new(teamwork::Error::TooManyAttempts))),
                                        State::Wait(Duration::from_secs(SECONDS_TO_WAIT_IF_TOO_MANY_ATTEMPTS), context),
                                    )
                                }
                                Err(error) => {
                                    context.requests_sender.send(map_name.clone()).await.unwrap();

                                    (
                                        Some(ThumbnailMessage::Error(map_name, Arc::new(error))),
                                        State::Wait(Duration::from_secs(SECONDS_TO_WAIT_ON_ERROR), context),
                                    )
                                },
                            }
                        }
                        Entry::Occupied(occupied) => (
                            Some(ThumbnailMessage::Thumbnail(map_name, occupied.get().clone())),
                            State::Ready(context),
                        ),
                    }
                }
            }
        },
    )
}
