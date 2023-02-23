use {
    crate::application::message::NotificationMessage,
    iced::Subscription,
    std::time::{Duration, Instant},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NotificationKind {
    Info,
    Error,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Notification {
    pub text: String,
    pub duration: Option<Duration>,
    pub kind: NotificationKind,
    pub multiplier: usize,
}

impl Notification {
    pub fn new(text: impl ToString, duration: Option<Duration>, kind: NotificationKind) -> Self {
        Self {
            text: text.to_string(),
            duration,
            kind,
            multiplier: 1,
        }
    }

    fn can_combine(&self, notification: &Notification) -> bool {
        self.text == notification.text && self.kind == notification.kind
    }
}

pub struct Notifications {
    current: Option<(Notification, Instant)>,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            current: None,
        }
    }

    pub fn current(&self) -> Option<&Notification> {
        self.current.as_ref().map(|(notification, _)| notification)
    }

    pub fn push(&mut self, notification: Notification) {
        let now = Instant::now();

        match self.current.as_mut() {
            None => {
                self.current = Some((notification, now));
            }
            Some((current, started)) => {
                if current.can_combine(&notification) {
                    *started = now;
                    current.multiplier += 1;
                } else {
                    self.current = Some((notification, now));
                }
            }
        }
    }

    pub fn clear_current(&mut self) {
        self.current = None;
    }

    pub fn update(&mut self) {
        if let Some((notification, instant)) = &self.current {
            if let Some(duration) = notification.duration {
                if Instant::now() - *instant >= duration {
                    self.current = None;
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<NotificationMessage> {
        iced::time::every(std::time::Duration::from_millis(20)).map(|_| NotificationMessage::Update)
    }
}
