use iced::pure::widget::Svg;
use iced::pure::{button, Element};
///! Note for myself about component creation.
/// The type called State is dedicated to the animation of the component.
/// Remember Iced is based on Elm and this means each time the widget is drawn
/// the value it displays are passed as parameter, the widget does not store the changing value
/// it only stores what will be displayed.
use iced::svg;
use iced_lazy::pure::{self, Component};

use crate::styles::Palette;
use crate::ApplicationIcons;

pub struct FavoriteButton<'l, Message> {
    icon_on: svg::Handle,
    icon_off: svg::Handle,
    on_toggle: Box<dyn Fn(bool) -> Message + 'l>,
    toggled: bool,
    palette: &'l Palette,
}

impl<'l, Message> FavoriteButton<'l, Message> {
    pub fn new<F>(icons: &ApplicationIcons, palette: &'l Palette, toggled: bool, f: F) -> Self
    where
        F: 'l + Fn(bool) -> Message,
    {
        Self {
            icon_on: icons.favorite_on.clone(),
            icon_off: icons.favorite_off.clone(),
            on_toggle: Box::new(f),
            toggled,
            palette,
        }
    }
}

#[derive(Debug, Clone)]
pub enum Event {
    Pressed,
}

#[derive(Default, Clone)]
pub struct FavoriteButtonState {
    pub toggled: bool,
}

struct ButtonStyleSheet<'l> {
    palette: &'l Palette,
}

impl<'l> ButtonStyleSheet<'l> {
    pub fn new(palette: &'l Palette) -> Self {
        Self { palette }
    }
}

impl<'l> iced::pure::widget::button::StyleSheet for ButtonStyleSheet<'l> {
    fn active(&self) -> iced::button::Style {
        iced::button::Style {
            shadow_offset: iced::Vector::new(0f32, 0f32),
            background: None,
            border_radius: 6f32,
            border_width: 1f32,
            border_color: iced::Color::TRANSPARENT,
            text_color: self.palette.card_foreground.clone(),
        }
    }
}

impl<'l, Message, Renderer: 'l> Component<Message, Renderer> for FavoriteButton<'l, Message>
where
    Renderer: iced_native::text::Renderer + iced_native::svg::Renderer,
{
    type State = ();
    type Event = Event;

    fn update(&mut self, _state: &mut Self::State, event: Self::Event) -> Option<Message> {
        match event {
            Event::Pressed => {
                self.toggled = !self.toggled;

                let message = (self.on_toggle)(self.toggled);

                return Some(message);
            }
        }
    }

    fn view(&self, _state: &Self::State) -> iced_pure::Element<'l, Self::Event, Renderer> {
        let icon = match self.toggled {
            true => self.icon_on.clone(),
            false => self.icon_off.clone(),
        };

        button(Svg::new(icon))
            .on_press(Event::Pressed)
            .style(ButtonStyleSheet::new(&self.palette))
            .into()
    }
}

impl<'l, Message> From<FavoriteButton<'l, Message>> for Element<'l, Message>
where
    Message: 'l,
{
    fn from(button: FavoriteButton<'l, Message>) -> Self {
        pure::component(button)
    }
}
