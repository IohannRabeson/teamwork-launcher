use iced::{
    pure::{button, text, widget::{Column, Row}, scrollable, Application}, 
    Settings, Command, Length
};
use iced::pure::{text_input, column};
use server_info::{ServerInfo, parse_server_infos};
use thiserror::Error;
use std::sync::Arc;

mod server_info;

#[derive(Error, Debug, Clone)]
enum Error {
    #[error("HTTP error: {0}")]
    Http(#[from] Arc<reqwest::Error>),
    #[error("UI error: {0}")]
    Ui(#[from] Arc<iced::Error>)
}

struct MyApplication 
{
    server_infos: Vec<ServerInfo>,
    error_message: Option<String>,
    filter: String,
}

#[derive(Debug, Clone)]
enum Messages
{
    UpdateServers,
    ServersInfoResponse(Result<Vec<ServerInfo>, Error>),
    FilterChanged(String),
    ClearFilter,
    Connect(std::net::Ipv4Addr, u16),
}

impl MyApplication {
    async fn request_servers_infos(address: &str) -> Result<Vec<ServerInfo>, Error> {
        let html = reqwest::get(address)
            .await
            .map_err(|e| Error::Http(Arc::new(e)))?
            .text()
            .await
            .map_err(|e| Error::Http(Arc::new(e)))?
            ;
        
        Ok(parse_server_infos(&html))
    }

    fn make_server(server_info: &ServerInfo) -> iced::pure::Element<Messages> {
        let element = Column::new()
            .push(text(&server_info.name))
            .push(text(server_info.ip.to_string()))
            .push(text(format!("{} / {} players", server_info.current_players_count, server_info.max_players_count)))
            ;

        let button = button(text("Connect"))
            .on_press(Messages::Connect(server_info.ip, server_info.port))
            ;

        Row::new()
            .push(column().push(button).align_items(iced::Alignment::Center))
            .push(element.width(Length::Fill))
            .width(Length::Fill)
            .height(Length::Shrink)
            .into()
    }

    fn make_servers_list<'a>(&self, server_infos: &'a [ServerInfo], filter: &str) -> iced::pure::Element<'a, Messages> {
        let mut column: Column<Messages> = Column::new();

        for server_element in server_infos
            .iter()
            .filter(|server_info| filter.is_empty() || Self::accept_server(*server_info, filter))
            .map(MyApplication::make_server)
        {
            column = column.push(server_element);
        }

        scrollable(column.width(Length::Fill)).into()
    }

    fn accept_server(server_info: &ServerInfo, filter: &str) -> bool {
        server_info.name.as_str().to_lowercase().contains(&filter)
    }

    fn launch_game(&self, ip: &std::net::Ipv4Addr, port: u16) {
        use std::process::Command;

        Command::new(r"C:\Program Files (x86)\Steam\Steam.exe")
            .args(["-applaunch", "440", "+connect", &format!("{}:{}", ip, port)])
            .output()
            .expect("failed to execute process");
    }
}

const SKIAL_URL: &str = "https://www.skial.com/api/servers.php";

impl Application for MyApplication {
    type Message = Messages;
    type Executor = iced::executor::Default;
    type Flags = ();

    fn new(_flags: Self::Flags) -> (Self, Command<Self::Message>) {
        (Self {
            server_infos: Vec::new(),
            error_message: None,
            filter: String::new(),
        }, Command::perform(MyApplication::request_servers_infos(SKIAL_URL), Messages::ServersInfoResponse))
    }

    fn title(&self) -> String {
        "TF2 launcher".to_string()
    }

    fn update(&mut self, message: Self::Message) -> iced::Command<Self::Message> {
        match message {
            Messages::ServersInfoResponse(response) => {
                match response {
                    Ok(servers_info) => {
                        self.server_infos = servers_info;
                        self.error_message = None;
                    },
                    Err(error_message) => self.error_message = Some(error_message.to_string()),
                };
            },
            Messages::UpdateServers => return Command::perform(MyApplication::request_servers_infos(SKIAL_URL), Messages::ServersInfoResponse),
            Messages::FilterChanged(filter) => self.filter = filter.to_lowercase(),
            Messages::ClearFilter => self.filter.clear(),
            Messages::Connect(ip, port) => self.launch_game(&ip, port),
        }
        
        Command::none()
    }

    fn view(&self) -> iced::pure::Element<Messages> {
        let filter = text_input("Filter...", &self.filter, Messages::FilterChanged);
        let row: Row<Messages> = Row::new()
            .push(filter)
            .push(button("X").on_press(Messages::ClearFilter))
            .push(button("Refresh").on_press(Messages::UpdateServers))
            ;
        let column: Column<Messages> = Column::new()
            .push(row)
            .push(self.make_servers_list(&self.server_infos, &self.filter))
            .push(text(self.error_message.as_ref().unwrap_or(&String::new())))
            .width(Length::Fill)
            .padding(3)
            ;

        column.into()
    }
}

fn main() -> Result<(), Error> {
    MyApplication::run(Settings::default()).map_err(|e| Error::Ui(Arc::new(e)))
}
