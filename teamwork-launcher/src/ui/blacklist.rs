use {
    crate::{
        application::{blacklist::BlacklistEntry, message::BlacklistMessage, Message},
        icons,
        ui::{self, buttons::svg_button},
    },
    iced::widget::{column, row, text, text_input},
    iced_lazy::Component,
    iced_native::Element,
};

pub struct Blacklist<'l> {
    blacklist: &'l crate::Blacklist,
}

impl<'l> Blacklist<'l> {
    pub fn new(blacklist: &'l crate::Blacklist) -> Self {
        Self { blacklist }
    }

    fn blacklist_view(&self) -> Element<'l, Event, iced::Renderer> {
        self.blacklist
            .iter()
            .enumerate()
            .fold(column![].spacing(2), |col, (index, term)| {
                col.push(
                    row![
                        text(&term),
                        svg_button(icons::CLEAR_ICON.clone(), 10).on_press(Event::Remove(index))
                    ]
                    .spacing(ui::DEFAULT_SPACING),
                )
            })
            .into()
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    EditNewTerm(String),
    Add,
    Remove(usize),
}

impl<'a> Component<Message, iced::Renderer> for Blacklist<'a> {
    type State = String;
    type Event = Event;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::EditNewTerm(text) => {
                *state = text;
                None
            }
            Event::Add => {
                let text = state.trim().to_string();

                state.clear();

                if text.is_empty() {
                    None
                } else {
                    Some(Message::Blacklist(BlacklistMessage::Add(BlacklistEntry::parse(&text))))
                }
            }
            Event::Remove(index) => Some(Message::Blacklist(BlacklistMessage::Remove(index))),
        }
    }

    fn view(&self, state: &Self::State) -> Element<'a, Self::Event, iced::Renderer> {
        column![
            row![
                text_input("Enter a word or an IP address", state)
                    .on_input(Event::EditNewTerm)
                    .on_submit(Event::Add),
                svg_button(icons::PLUS.clone(), 20).on_press(Event::Add)
            ]
            .spacing(ui::DEFAULT_SPACING),
            self.blacklist_view()
        ]
        .spacing(ui::DEFAULT_SPACING)
        .into()
    }
}

impl<'a> From<Blacklist<'a>> for Element<'a, Message, iced::Renderer> {
    fn from(blacklist: Blacklist<'a>) -> Self {
        iced_lazy::component(blacklist)
    }
}
