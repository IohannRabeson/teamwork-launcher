use crate::application::message::ThumbnailMessage;
use {
    //futures::StreamExt,
    iced::{
        futures::channel::mpsc::{unbounded, UnboundedReceiver, UnboundedSender},
        subscription,
        widget::image,
        Subscription,
    },
    std::{
        collections::{btree_map::Entry, BTreeMap},
        sync::Arc,
    },
};

enum State {
    Starting { api_key: String },
    Ready(Context),
}

struct Context {
    requests_receiver: UnboundedReceiver<String>,
    client: teamwork::Client,
    teamwork_api_key: String,
    cache: BTreeMap<String, image::Handle>,
}

pub fn subscription(api_key: &str) -> Subscription<ThumbnailMessage> {
    subscription::unfold(
        (),
        State::Starting {
            api_key: api_key.to_string(),
        },
        |state| async move {
            match state {
                State::Starting { api_key } => {
                    let (sender, receiver) = unbounded();
                    let context = Context {
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
                                .get_map_thumbnail(&context.teamwork_api_key, &map_name, image::Handle::from_memory)
                                .await
                            {
                                Ok(thumbnail) => {
                                    vacant.insert(thumbnail.clone());
                                    (Some(ThumbnailMessage::Thumbnail(map_name, thumbnail)), State::Ready(context))
                                }
                                Err(error) => (
                                    Some(ThumbnailMessage::Error(map_name, Arc::new(error))),
                                    State::Ready(context),
                                ),
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
