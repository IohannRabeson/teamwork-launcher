use {
    super::widgets::{ping, region, thumbnail},
    crate::{
        application::{game_mode::GameModes, IpPort, Message, Server},
        ui::{styles::BoxContainerStyle, widgets},
    },
    iced::{
        theme,
        widget::{column, container, horizontal_space, row, text, vertical_space},
        Element, Length,
    },
};

fn yes_no(value: bool) -> &'static str {
    match value {
        true => "Yes",
        false => "No",
    }
}

fn content<'l>(servers: &'l [Server], game_modes: &'l GameModes, ip_port: &'l IpPort) -> Element<'l, Message> {
    let server = servers.iter().find(|s| &s.ip_port == ip_port).expect("find server");
    let mut details_column = column![
        text(&server.name).size(28),
        row![text("Ping:"), ping(&server.ping)].spacing(4),
        region(server, 20, 0),
        text(&format!(
            "Players: {} / {}",
            server.current_players_count, server.max_players_count
        )),
        text(format!("Map: {}", server.map)),
    ]
    .padding(4)
    .spacing(4);

    if let Some(map) = &server.next_map {
        details_column = details_column.push(text(format!("Next map: {}", map)));
    }
    let details_column = details_column.push(row![text("Game modes:"), widgets::game_modes(game_modes, &server.game_modes)].spacing(4));
    let details_column = details_column.push(text(format!("Valve secure: {}", yes_no(server.vac_secured))));
    let details_column = details_column.push(text(format!("Role the dice: {}", yes_no(server.has_rtd))));
    let details_column = details_column.push(text(format!("Password protected: {}", yes_no(server.need_password))));
    let details_column = details_column.push(text(format!("Has \"all talk\" command: {}", yes_no(server.has_all_talk))));
    let details_column = details_column.push(text(format!("No respawn time: {}", yes_no(server.has_no_respawn_time))));

    column![
        row![
            container(thumbnail(server, Length::Fixed(500.0), Length::Fixed(250.0))).padding(16).center_y().center_x(),
            details_column,
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
