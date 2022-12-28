use iced::{Element, theme, widget::tooltip};

use super::{VISUAL_SPACING_MEDIUM, styles};

pub fn tooltip<'a, Message, Renderer>(
    content: impl Into<Element<'a, Message, Renderer>>,
    tooltip: impl ToString) 
{
    tooltips(
        content,
        tooltip,
        tooltip::Position::Bottom,
    )
    .gap(VISUAL_SPACING_MEDIUM)
    .style(theme::Container::Custom(Box::new(styles::ToolTip::default())))
}