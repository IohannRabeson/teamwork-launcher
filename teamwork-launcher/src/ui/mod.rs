use crate::application::SettingsMessage;

pub mod buttons;
pub mod filter;
pub mod header;
pub mod main;
pub mod settings;
pub mod styles;
pub mod widgets;
pub mod server {
    use {
        super::widgets::{ping, region, thumbnail},
        crate::{
            application::{game_mode::GameModes, IpPort, Message, Server},
            ui::widgets,
        },
        iced::{
            widget::{column, horizontal_space, row, text},
            Alignment, Element, Length,
        },
    };

    fn yes_no(value: bool) -> &'static str {
        match value {
            true => "Yes",
            false => "No",
        }
    }

    pub fn view<'l>(servers: &'l [Server], game_modes: &'l GameModes, ip_port: &'l IpPort) -> Element<'l, Message> {
        let server = servers.iter().find(|s| &s.ip_port == ip_port).expect("find server");
        let mut c = column![
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
            c = c.push(text(format!("Next map: {}", map)));
        }
        let c = c.push(row![text("Game modes:"), widgets::game_modes(game_modes, &server.game_modes)].spacing(4));
        let c = c.push(text(format!("Valve secure: {}", yes_no(server.vac_secured))));
        let c = c.push(text(format!("Role the dice: {}", yes_no(server.has_rtd))));
        let c = c.push(text(format!("Password protected: {}", yes_no(server.need_password))));
        let c = c.push(text(format!("Has \"all talk\" command: {}", yes_no(server.has_all_talk))));
        let c = c.push(text(format!("No respawn time: {}", yes_no(server.has_no_respawn_time))));

        column![
            text(&server.name).size(28),
            row![
                thumbnail(server, Length::Fixed(500.0), Length::Fixed(250.0)),
                c,
                horizontal_space(Length::Fill),
            ]
            .align_items(Alignment::Start)
        ]
        .spacing(4)
        .padding(4)
        .into()
    }
}
