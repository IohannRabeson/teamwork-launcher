use std::path::{Path, PathBuf};
use iced_native::image::Data;
use log::{error, trace};

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
        collections::BTreeMap,
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
                    };

                    (Some(ThumbnailMessage::Started(sender)), State::Ready(context))
                }
                State::Ready(mut context) => {
                    use iced::futures::StreamExt;

                    let map_name = context.requests_receiver.select_next_some().await;

                    match context
                        .client
                        .get_map_thumbnail(&context.teamwork_api_key, map_name.as_str(), image::Handle::from_memory)
                        .await
                    {
                        Ok(thumbnail) => {
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
                        }
                    }
                }
            }
        },
    )
}

#[derive(thiserror::Error, Debug)]
pub enum ThumbnailCacheError {
    #[error("Invalid directory path '{0}'")]
    InvalidDirectoryPath(PathBuf),

    #[error("I/O failed: {0}")]
    IoFailure(#[from] std::io::Error),
}

pub struct ThumbnailCache {
    directory_path: PathBuf,
    cache: BTreeMap<MapName, image::Handle>,
}

impl ThumbnailCache {
    pub fn new(directory_path: impl AsRef<Path>) -> Self {
        Self {
            directory_path: directory_path.as_ref().to_path_buf(),
            cache: BTreeMap::new(),
        }
    }

    pub fn get(&self, map_name: &MapName) -> Option<image::Handle> {
        self.cache.get(map_name).cloned()
    }

    pub fn insert(&mut self, map_name: MapName, image: image::Handle) {
        self.cache.insert(map_name, image);
    }

    pub fn load(&mut self) -> Result<(), ThumbnailCacheError> {
        if !self.directory_path.is_dir() {
            return Err(ThumbnailCacheError::InvalidDirectoryPath(self.directory_path.clone()))
        }

        for entry in std::fs::read_dir(&self.directory_path)? {
            if let Ok(entry) = entry {
                if let Ok(metadata) = entry.metadata() {
                    if metadata.is_file() {
                        let map_name = MapName::new(entry.path().file_name().expect("file name").to_string_lossy());

                        trace!("image: {}", entry.path().display());

                        let bytes = std::fs::read(entry.path())?;

                        self.insert(map_name, image::Handle::from_memory(bytes));
                    }
                }
            }
        }

        trace!("Loaded {} images", self.cache.len());

        Ok(())
    }

    pub fn write(&self, max_bytes: u64) -> Result<(), ThumbnailCacheError> {
        std::fs::create_dir_all(&self.directory_path)?;
        Self::clear_cache(&self.directory_path)?;

        if !self.directory_path.is_dir() {
            return Err(ThumbnailCacheError::InvalidDirectoryPath(self.directory_path.clone()))
        }

        let mut current_bytes = 0u64;

        for (map_name, handle) in self.cache.iter() {
            if max_bytes > 0 && current_bytes >= max_bytes {
                break
            }

            if let Data::Bytes(bytes) = handle.data() {
                let file_path = self.directory_path.join(map_name.as_str());

                current_bytes += bytes.len() as u64;
                std::fs::write(file_path, bytes)?;
            }
        }

        trace!("Written {} images", self.cache.len());

        Ok(())
    }

    fn clear_cache(directory: &Path) -> Result<(), std::io::Error> {
        for entry in std::fs::read_dir(directory)? {
            if let Ok(entry) = entry {
                if let Err(error) = std::fs::remove_file(entry.path()) {
                    error!("Failed to delete '{}': {}", entry.path().display(), error);
                }
            }
        }

        Ok(())
    }
}