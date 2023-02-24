use {
    crate::application::message::NotificationMessage,
    iced::Subscription,
    std::time::{Duration, Instant},
};

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum NotificationKind {
    /// A feedback to the user, usually those notifications have an expiry.
    Feedback,
    /// Show an error to the user. Usually those notifications must be clicked to be discarded.
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
                } else {
                    self.pending.push(notification);
                }
            }
        }
    }

    pub fn clear_current(&mut self) {
        self.current = self.take_next_pending();
    }

    pub fn update(&mut self, now: Instant) {
        if let Some((notification, instant)) = &self.current {
            if let Some(duration) = notification.expiry {
                if now - *instant >= duration {
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

#[cfg(test)]
mod tests_take_pending {
    use crate::application::notifications::{Notification, NotificationKind, Notifications};

    #[test]
    fn test_take_empty() {
        let mut notifications = Notifications::new();

        notifications.push(Notification::new("test", None, NotificationKind::Feedback));

        assert_eq!(notifications.take_next_pending(), None);
    }

    #[test]
    fn test_take_one() {
        let mut notifications = Notifications::new();
        let notification = Notification::new("test2", None, NotificationKind::Feedback);

        notifications.push(Notification::new("test", None, NotificationKind::Feedback));
        notifications.push(notification.clone());

        assert_eq!(
            notifications.take_next_pending().as_ref().map(|(n, _)| n),
            Some(&notification)
        );
        assert_eq!(notifications.take_next_pending(), None);
    }

    #[test]
    fn test_take_two() {
        let mut notifications = Notifications::new();
        let notification = Notification::new("test2", None, NotificationKind::Feedback);
        let notification2 = Notification::new("test3", None, NotificationKind::Feedback);

        notifications.push(Notification::new("test", None, NotificationKind::Feedback));
        notifications.push(notification.clone());
        notifications.push(notification2.clone());

        assert_eq!(
            notifications.take_next_pending().as_ref().map(|(n, _)| n),
            Some(&notification)
        );
        assert_eq!(
            notifications.take_next_pending().as_ref().map(|(n, _)| n),
            Some(&notification2)
        );
        assert_eq!(notifications.take_next_pending(), None);
    }
}

#[cfg(test)]
mod tests {
    use {
        crate::application::notifications::{Notification, NotificationKind, Notifications},
        std::time::{Duration, Instant},
    };

    #[test]
    fn test_clear_current() {
        let mut notifications = Notifications::new();
        let notification = Notification::new("test", None, NotificationKind::Feedback);

        notifications.push(notification.clone());
        notifications.clear_current();

        assert_eq!(notifications.current(), None);
    }

    #[test]
    fn test_update() {
        let mut notifications = Notifications::new();
        let notification0 = Notification::new("test0", Some(Duration::from_secs(1)), NotificationKind::Feedback);
        let notification1 = Notification::new("test1", None, NotificationKind::Feedback);

        notifications.push(notification0.clone());
        notifications.push(notification1.clone());
        notifications.update(Instant::now() + Duration::from_millis(1));
        assert_eq!(notifications.current(), Some(&notification0));
        notifications.update(Instant::now() + Duration::from_secs(10));
        assert_eq!(notifications.current(), Some(&notification1));
        notifications.update(Instant::now() + Duration::from_secs(100));
        assert_eq!(notifications.current(), Some(&notification1));
    }
}
