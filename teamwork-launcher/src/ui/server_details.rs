use {
    crate::{
        application::{game_mode::GameModes, IpPort, Message, Server},
        ui::{styles::BoxContainerStyle, widgets},
    },
    iced::{
        Element,
        Length,
        theme, widget::{column, container, horizontal_space, row, text, vertical_space},
    },
    super::widgets::{ping, region, thumbnail},
};
use crate::ui::form::Form;

fn yes_no<'l>(value: bool) -> Element<'l, Message> {
    match value {
        true => text("Yes"),
        false => text("No"),
    }.into()
}

fn content<'l>(servers: &'l [Server], game_modes: &'l GameModes, ip_port: &'l IpPort) -> Element<'l, Message> {
    let server = servers.iter().find(|s| &s.ip_port == ip_port).expect("find server");
    let form = Form::new()
        .push("Ping:", |server: &Server|ping(&server.ping))
        .push("Region:", |server: &Server|region(&server.country, 20, 0))
        .push("Players:", |server: &Server|text(&format!("{} / {}", server.current_players_count, server.max_players_count)).into())
        .push("Map:", |server: &Server|text(server.map.as_str()).into())
        .push_if(server.next_map.is_some(), "Next map:", |server: &Server|text(server.next_map.as_ref().unwrap().as_str()).into())
        .push("Game modes:", |server: &Server|widgets::game_modes(game_modes, &server.game_modes))
        .push("Valve secure:", |server: &Server|yes_no(server.vac_secured))
        .push("Password protected:", |server: &Server|yes_no(server.need_password))
        .push("Role the dice:", |server: &Server|yes_no(server.has_rtd))
        .push("All talk:", |server: &Server|yes_no(server.has_all_talk))
        .push("No respawn time:", |server: &Server|yes_no(server.has_no_respawn_time))
        .push("Random crits:", |server: &Server|yes_no(server.has_random_crits))
        ;

    column![
        row![
            container(thumbnail(server, Length::Fixed(500.0), Length::Fixed(250.0))).padding(16).center_y().center_x(),
            column![
                text(&server.name).size(28),
                form.view(server)
            ],
            horizontal_space(Length::Fill),
        ],
        vertical_space(Length::Fill)
    ]
    .into()
}

pub fn view<'l>(servers: &'l [Server], game_modes: &'l GameModes, ip_port: &'l IpPort) -> Element<'l, Message> {
    let content =
        container(content(servers, game_modes, ip_port))
            .style(theme::Container::Custom(Box::new(BoxContainerStyle {})));

    container(content).width(Length::Fill).height(Length::Fill).padding(16).into()
}
