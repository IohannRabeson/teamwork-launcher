use {
    crate::application::{map::MapName, message::ScreenshotsMessage, Message, PromisedValue},
    iced::{widget::image, Command},
};

pub struct Screenshots {
    screenshots: PromisedValue<Vec<image::Handle>>,
    current: usize,
}

impl Screenshots {
    pub fn new() -> Self {
        Self {
            screenshots: PromisedValue::None,
            current: 0,
        }
    }

    pub fn set(&mut self, screenshots: PromisedValue<Vec<image::Handle>>) {
        self.screenshots = screenshots;
        self.current = 0;
    }

    pub fn count(&self) -> usize {
        match &self.screenshots {
            PromisedValue::Ready(screenshots) => screenshots.len(),
            _ => 0,
        }
    }

    pub fn can_move_next(&self) -> bool {
        matches!(&self.screenshots, PromisedValue::Ready(screenshots) if self.current + 1 < screenshots.len())
    }

    pub fn can_move_previous(&self) -> bool {
        matches!(&self.screenshots, PromisedValue::Ready(_) if self.current > 0)
    }

    pub fn next(&mut self) {
        if let PromisedValue::Ready(screenshots) = &self.screenshots {
            if self.current + 1 < screenshots.len() {
                self.current += 1;
            }
        }
    }

    pub fn previous(&mut self) {
        if let PromisedValue::Ready(_screenshots) = &self.screenshots {
            if self.current > 0 {
                self.current -= 1;
            }
        }
    }

    pub fn current_index(&self) -> usize {
        self.current
    }

    pub fn current(&self) -> PromisedValue<&image::Handle> {
        match &self.screenshots {
            PromisedValue::Ready(screenshots) if !screenshots.is_empty() => {
                let index = std::cmp::min(self.current, screenshots.len() - 1);

                PromisedValue::Ready(&screenshots[index])
            }
            PromisedValue::Ready(_screenshots) => PromisedValue::None,
            PromisedValue::Loading => PromisedValue::Loading,
            PromisedValue::None => PromisedValue::None,
        }
    }
}

pub fn fetch_screenshot(map_name: MapName, api_key: String) -> Command<Message> {
    Command::perform(
        async move {
            let client = teamwork::Client::default();

            client
                .get_map_screenshots(&api_key, map_name.as_str(), image::Handle::from_memory)
                .await
        },
        |result| match result {
            Ok(screenshots) => Message::Screenshots(ScreenshotsMessage::Screenshots(screenshots)),
            Err(error) => Message::Screenshots(ScreenshotsMessage::Error(error.into())),
        },
    )
}
