use {
    crate::application::message::NotificationMessage,
    iced::Subscription,
    std::time::{Duration, Instant},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NotificationKind {
    Feedback,
    Error,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Notification {
    pub text: String,
    pub expiry: Option<Duration>,
    pub kind: NotificationKind,
    pub multiplier: usize,
}

impl Notification {
    pub fn new(text: impl ToString, expiry: Option<Duration>, kind: NotificationKind) -> Self {
        Self {
            text: text.to_string(),
            expiry,
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
    /// Queue of pending notifications.
    /// The new element are push to the back, and the front elements are consumed.
    pending: Vec<Notification>,
}

impl Notifications {
    pub fn new() -> Self {
        Self {
            current: None,
            pending: Vec::new(),
        }
    }

    pub fn current(&self) -> Option<&Notification> {
        self.current.as_ref().map(|(notification, _)| notification)
    }

    /// Push a new notification
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
                } else if current.expiry.is_none() {
                    self.pending.push(notification);
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
            if let Some(duration) = notification.expiry {
                if Instant::now() - *instant >= duration {
                    self.current = self.take_next_pending();
                }
            }
        }
    }

    pub fn subscription(&self) -> Subscription<NotificationMessage> {
        const REFRESH_RATE: Duration = Duration::from_millis(20);

        iced::time::every(REFRESH_RATE).map(|_| NotificationMessage::Update)
    }

    fn take_next_pending(&mut self) -> Option<(Notification, Instant)> {
        if self.pending.is_empty() {
            None
        } else {
            Some((self.pending.remove(0), Instant::now()))
        }
    }
}
