use {
    crate::{
        application::Messages,
        fonts,
        ui::{styles, VISUAL_SPACING_MEDIUM},
    },
    iced::{
        theme,
        widget::{button, column, text},
        Color, Element, Length,
    },
    std::collections::VecDeque,
};

#[derive(Debug, Clone)]
pub struct Announce {
    pub title: String,
    pub message: String,
    pub background_color: Color,
    pub text_color: Color,
}

impl Eq for Announce {}

impl PartialEq for Announce {
    fn eq(&self, other: &Self) -> bool {
        self.title.eq(&other.title) && self.message.eq(&other.message)
    }
}

impl Announce {
    pub fn error(title: impl ToString, message: impl ToString) -> Self {
        Announce::new(title, message, Color::from_rgb8(255, 255, 255), Color::from_rgb8(159, 49, 47))
    }

    pub fn warning(title: impl ToString, message: impl ToString) -> Self {
        Announce::new(
            title,
            message,
            Color::from_rgb8(255, 255, 255),
            Color::from_rgb8(240, 129, 73),
        )
    }

    pub fn new(title: impl ToString, message: impl ToString, text_color: Color, background_color: Color) -> Self {
        Self {
            title: title.to_string(),
            message: message.to_string(),
            background_color,
            text_color,
        }
    }

    /// Show an announce.
    ///
    /// An announce display a title and a message.
    /// When you click anywhere on it it's discarded.
    pub fn view(&self) -> Element<Messages> {
        button(
            column![text(&self.title).size(24), text(&self.message).size(fonts::TEXT_FONT_SIZE)]
                .spacing(VISUAL_SPACING_MEDIUM),
        )
        .padding(VISUAL_SPACING_MEDIUM)
        .width(Length::Fill)
        .style(theme::Button::Custom(Box::new(styles::Announce::new(
            self.text_color,
            self.background_color,
        ))))
        .on_press(Messages::DiscardCurrentAnnounce)
        .into()
    }
}

#[derive(Default)]
pub struct AnnounceQueue {
    queue: VecDeque<Announce>,
}

impl AnnounceQueue {
    /// Enqueue a new announce on the display queue.
    /// The announce is only added if it's not already present in the queue.
    pub fn push(&mut self, announce: Announce) {
        if !self.queue.contains(&announce) {
            self.queue.push_front(announce);
        }
    }

    pub fn pop(&mut self) {
        self.queue.pop_back();
    }

    pub fn current(&self) -> Option<&Announce> {
        self.queue.back()
    }
}
