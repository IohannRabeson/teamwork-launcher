use {
    crate::{
        application::{
            message::{AddViewMessage, ListViewMessage, ModsMessage},
            Message,
        },
        ui::{color, styles::BoxContainerStyle, DEFAULT_SPACING},
    },
    iced::{
        theme,
        widget::{button, column, container, row, scrollable, text, vertical_space, Container},
        Alignment, Background, Color, Element, Length, Theme,
    },
    iced_aw::Spinner,
    mods_manager::{Install, ModInfo, ModName, Registry},
};
use crate::ui::widgets::tooltip;

pub fn view<'a>(registry: &'a Registry, selected_mod: Option<&'a ModName>, is_loading: bool) -> Element<'a, Message> {
    row![
        mod_list(registry, selected_mod)
            .width(Length::FillPortion(4))
            .height(Length::Fill),
        action_list(registry, selected_mod, is_loading)
            .width(Length::Fill)
            .height(Length::Fill)
    ]
    .spacing(DEFAULT_SPACING)
    .padding(DEFAULT_SPACING)
    .into()
}

fn action_list<'a>(registry: &'a Registry, selected_mod: Option<&'a ModName>, is_loading: bool) -> Container<'a, Message> {
    if is_loading {
        return container(Spinner::new())
            .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
            .center_x()
            .center_y()
            .width(Length::Fill)
            .height(Length::Fill);
    }

    let mut content = column![];

    if let Some(selected_mod) = selected_mod {
        match registry.get(selected_mod) {
            None => {}
            Some(info) => match info.install {
                Install::None => {
                    content =
                        content.push(button("Install").on_press(Message::Mods(ModsMessage::Install(info.name.clone()))));
                }
                Install::Installed { .. } => {
                    content =
                        content.push(button("Uninstall").on_press(Message::Mods(ModsMessage::Uninstall(info.name.clone()))));
                }
                Install::Failed { .. } => {
                    content =
                        content.push(button("Install").on_press(Message::Mods(ModsMessage::Install(info.name.clone()))));
                }
            },
        }

        content = content.push(
            button("Remove").on_press(Message::Mods(ModsMessage::ListView(ListViewMessage::RemoveMod(
                selected_mod.clone(),
            )))),
        );
    }

    content = content.push(vertical_space(Length::Fill));
    content = content.push(
        button(text("Add mod").size(36))
            .padding(16)
            .on_press(Message::Mods(ModsMessage::AddView(AddViewMessage::Show)))
            .style(theme::Button::Positive),
    );

    container(
        content
            .spacing(DEFAULT_SPACING)
            .align_items(Alignment::Center)
            .width(Length::Fill),
    )
    .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
    .padding(DEFAULT_SPACING)
    .width(Length::Fill)
}

fn mod_list<'a>(registry: &'a Registry, selected_mod: Option<&'a ModName>) -> Container<'a, Message> {
    container(scrollable(
        registry.iter().fold(column![].spacing(DEFAULT_SPACING), |c, info| {
            c.push(mod_info_view(info, selected_mod == Some(&info.name)))
        }),
    ))
    .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
    .padding(DEFAULT_SPACING)
}

struct UnselectedInfoView;
struct SelectedInfoView;

impl button::StyleSheet for SelectedInfoView {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: Some(Background::Color(style.palette().primary)),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut appearance = self.active(style);

        appearance.background = Some(Background::Color(color::brighter(style.palette().primary)));

        appearance
    }
}

impl button::StyleSheet for UnselectedInfoView {
    type Style = Theme;

    fn active(&self, style: &Self::Style) -> button::Appearance {
        button::Appearance {
            shadow_offset: Default::default(),
            background: Some(Background::Color(color::brighter(style.palette().background))),
            border_radius: 0.0,
            border_width: 0.0,
            border_color: Color::TRANSPARENT,
            text_color: style.palette().text,
        }
    }

    fn hovered(&self, style: &Self::Style) -> button::Appearance {
        let mut appearance = self.active(style);

        appearance.background = Some(Background::Color(color::brighter_by(style.palette().background, 0.2)));

        appearance
    }
}

struct InstalledBadge;

impl container::StyleSheet for InstalledBadge {
    type Style = Theme;

    fn appearance(&self, style: &Self::Style) -> container::Appearance {
        container::Appearance {
            background: Some(Background::Color(style.palette().success)),
            ..Default::default()
        }
    }
}

fn installed_badge<'a>() -> Element<'a, Message> {
    container(text("Installed").size(16))
        .style(theme::Container::Custom(Box::new(InstalledBadge {})))
        .padding(2)
        .into()
}

fn error_badge<'a>(error: &str) -> Element<'a, Message> {
    let content = container(text("Failed").size(16))
        .style(theme::Container::Custom(Box::new(InstalledBadge {})))
        .padding(2);

    tooltip(content, error, iced::widget::tooltip::Position::Bottom)
        .into()
}

fn mod_info_view(info: &ModInfo, is_selected: bool) -> Element<Message> {
    let content_row = match &info.install {
        Install::Installed { .. } => {row![installed_badge(), text(&info.name)] },
        Install::Failed { error } => { row![error_badge(error), text(&info.name)]}
        _ => row![text(&info.name)],
    }.spacing(DEFAULT_SPACING);
    let mut button = button(content_row)
        .on_press(Message::Mods(ModsMessage::ListView(ListViewMessage::ModClicked(
            info.name.clone(),
        ))))
        .width(Length::Fill)
        .style(theme::Button::Custom(match is_selected {
            true => Box::new(SelectedInfoView {}),
            false => Box::new(UnselectedInfoView {}),
        }));

    if !is_selected {
        button = button.style(theme::Button::Custom(Box::new(UnselectedInfoView {})));
    }

    button.into()
}
