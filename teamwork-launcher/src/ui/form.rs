use iced::{
    widget::{column, row, text},
    Element, Length, Padding,
};

type FieldFn<'l, T, Message> = Box<dyn Fn(&'l T) -> Element<'l, Message> + 'l>;

pub struct Form<'l, T, Message: 'l> {
    fields: Vec<(String, FieldFn<'l, T, Message>)>,
    spacing: u16,
    padding: Padding,
}

impl<'l, T, Message: 'l> Form<'l, T, Message> {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            spacing: 0,
            padding: Padding::new(0.0),
        }
    }

    pub fn spacing(mut self, spacing: u16) -> Self {
        self.spacing = spacing;
        self
    }

    pub fn padding(mut self, padding: Padding) -> Self {
        self.padding = padding;
        self
    }

    pub fn push(mut self, label: impl ToString, field_fn: impl Fn(&'l T) -> Element<'l, Message> + 'l) -> Self {
        self.fields.push((label.to_string(), Box::new(field_fn)));
        self
    }

    pub fn push_if(
        mut self,
        condition: bool,
        label: impl ToString,
        field_fn: impl Fn(&'l T) -> Element<'l, Message> + 'l,
    ) -> Self {
        if condition {
            self = self.push(label, field_fn);
        }

        self
    }

    pub fn view(&self, context: &'l T) -> Element<'l, Message> {
        self.fields
            .iter()
            .fold(column![], |column, (label, field_fn)| {
                column.push(row![text(label).width(Length::Fixed(160.0)), (field_fn)(context)])
            })
            .into()
    }
}
