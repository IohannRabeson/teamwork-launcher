use {
    crate::{
        application::{message::BlacklistMessage, Message},
        icons,
        ui::{self, buttons::svg_button},
    },
    iced::widget::{button, column, row, text, text_input},
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
}

#[derive(Debug, Clone)]
pub enum Event {
    EditNewTerm(String),
    Add,
    Remove(usize),
}

impl<'l> Component<Message, iced::Renderer> for Blacklist<'l> {
    type State = String;
    type Event = Event;

    fn update(&mut self, state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::EditNewTerm(text) => {
                *state = text;
                None
            }
            Event::Add => {
                let message = Message::Blacklist(BlacklistMessage::Add(state.clone()));

                state.clear();

                Some(message)
            }
            Event::Remove(index) => Some(Message::Blacklist(BlacklistMessage::Remove(index))),
        }
    }

    fn view(&self, state: &Self::State) -> Element<'_, Self::Event, iced::Renderer> {
        column![
            row![
                text_input("", state).on_input(Event::EditNewTerm).on_submit(Event::Add),
                button("+").on_press(Event::Add)
            ]
            .spacing(ui::DEFAULT_SPACING),
            self.blacklist.iter().enumerate().fold(column![], |col, (index, term)| {
                col.push(
                    row![
                        text(&term),
                        svg_button(icons::CLEAR_ICON.clone(), 10).on_press(Event::Remove(index))
                    ]
                    .spacing(ui::DEFAULT_SPACING),
                )
            })
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
