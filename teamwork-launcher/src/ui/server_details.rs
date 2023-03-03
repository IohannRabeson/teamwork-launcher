use iced_aw::Spinner;
use {
    super::widgets::{ping, ping_time, region},
    crate::{
        application::{
            game_mode::GameModes, map::MapName, message::ScreenshotsMessage, palettes, screenshots::Screenshots, IpPort,
            Message, PromisedValue, Server,
        },
        fonts, icons,
        ui::{buttons::svg_button, form::Form, styles::BoxContainerStyle, widgets},
    },
    iced::{
        theme,
        widget::{column, container, image, row, text, vertical_space, Image},
        Alignment, ContentFit, Element, Length,
    },
    iced_aw::{floating_element::Anchor, FloatingElement},
};

fn yes_no<'l>(value: bool) -> Element<'l, Message> {
    match value {
        true => text("Yes").style(*palettes::GREEN),
        false => text("No").style(*palettes::RED),
    }
    .into()
}

fn screenshot(screenshot: Option<&image::Handle>) -> Element<Message> {
    match screenshot {
        Some(screenshot) => Image::new(screenshot.clone())
            .content_fit(ContentFit::Contain)
            .width(Length::Fill)
            .height(Length::Fill)
            .into(),
        None => container(Image::new(icons::NO_IMAGE.clone()).content_fit(ContentFit::Contain))
            .width(Length::Fill)
            .height(Length::Fill)
            .center_x()
            .center_y()
            .into(),
    }
}

fn server_details_form<'l>(server: &'l Server, game_modes: &'l GameModes) -> Element<'l, Message> {
    Form::new()
        .spacing(4)
        .padding(4.into())
        .push("Ping:", |server: &Server| {
            row![ping(&server.ping), ping_time(&server.ping),].spacing(4).into()
        })
        .push("Region:", |server: &Server| {
            row![
                region(&server.country, 20, 0),
                // It's not possible to show the country as text as Form can't
                // handle properly the case where the label height is different than the field height and
                // it's very likely a country name will need 2 lines to be display. This will case a mislignement
                // between the labels column and the fields column.
                //
                // region_text(&server.country),
            ]
            .spacing(40)
            .into()
        })
        .push("Players:", |server: &Server| {
            text(&format!("{} / {}", server.current_players_count, server.max_players_count)).into()
        })
        .push("Map:", |server: &Server| text(server.map.as_str()).into())
        .push_if(server.next_map.is_some(), "Next map:", |server: &Server| {
            text(server.next_map.as_ref().unwrap().as_str()).into()
        })
        .push("Game modes:", |server: &Server| {
            widgets::game_modes(game_modes, &server.game_modes)
        })
        .push("Valve secure:", |server: &Server| yes_no(server.vac_secured))
        .push("Password protected:", |server: &Server| yes_no(server.need_password))
        .push("Role the dice:", |server: &Server| yes_no(server.has_rtd))
        .push("All talk:", |server: &Server| yes_no(server.has_all_talk))
        .push("No respawn time:", |server: &Server| yes_no(server.has_no_respawn_time))
        .push("Random crits:", |server: &Server| yes_no(server.has_random_crits))
        .view(server)
}

fn previous_button(screenshots: &Screenshots) -> Element<Message> {
    let mut button = svg_button(icons::ARROW_LEFT_SHORT.clone(), 20);

    if screenshots.can_move_previous() {
        button = button.on_press(Message::Screenshots(ScreenshotsMessage::Previous));
    }

    button.into()
}

fn next_button(screenshots: &Screenshots) -> Element<Message> {
    let mut button = svg_button(icons::ARROW_RIGHT_SHORT.clone(), 20);

    if screenshots.can_move_next() {
        button = button.on_press(Message::Screenshots(ScreenshotsMessage::Next));
    }

    button.into()
}

/// A screenshot with buttons to select the screenshot to display
fn screenshot_view<'l>(screenshots: &'l Screenshots, map_name: &'l MapName) -> Element<'l, Message> {
    match screenshots.current() {
        PromisedValue::Ready(image) => {
            let navigation_buttons = row![
                previous_button(screenshots),
                text(format!("{} / {}", screenshots.current_index() + 1, screenshots.count())),
                next_button(screenshots),
            ]
            .align_items(Alignment::Center)
            .spacing(4);

            column![
                FloatingElement::new(screenshot(Some(image)), || {
                    container(text(map_name.as_str()).font(fonts::TF2_SECONDARY).size(28))
                        .style(theme::Container::Custom(Box::new(BoxContainerStyle {})))
                        .padding(4)
                        .into()
                })
                .anchor(Anchor::NorthWest),
                container(navigation_buttons).width(Length::Fill).center_x(),
                vertical_space(Length::Shrink),
            ]
            .width(Length::FillPortion(2))
            .into()
        }
        PromisedValue::Loading => container(Spinner::new().width(Length::Fixed(64.0)).height(Length::Fixed(64.0)))
            .center_x()
            .center_y()
            .width(Length::FillPortion(2))
            .height(Length::Fill)
            .into(),
        PromisedValue::None => column![screenshot(None), vertical_space(Length::Shrink),]
            .width(Length::FillPortion(2))
            .into(),
    }
}

fn content<'l>(server: &'l Server, game_modes: &'l GameModes, screenshots: &'l Screenshots) -> Element<'l, Message> {
    row![
        screenshot_view(screenshots, &server.map),
        column![text(&server.name).size(28), server_details_form(server, game_modes)]
            .spacing(4)
            .width(Length::Fill),
    ]
    .spacing(4)
    .padding(4)
    .into()
}

pub fn view<'l>(
    servers: &'l [Server],
    game_modes: &'l GameModes,
    ip_port: &'l IpPort,
    screenshots: &'l Screenshots,
) -> Element<'l, Message> {
    let server = servers.iter().find(|s| &s.ip_port == ip_port).expect("find server");
    let content =
        container(content(server, game_modes, screenshots)).style(theme::Container::Custom(Box::new(BoxContainerStyle {})));

    container(content).width(Length::Fill).height(Length::Fill).padding(16).into()
}
