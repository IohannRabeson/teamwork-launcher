use async_trait::async_trait;
use nom::{bytes::complete::tag, combinator::map, multi::separated_list0, sequence::separated_pair, IResult};
use select::{
    document::Document,
    predicate::{Attr, Name, Predicate},
};
use std::net::Ipv4Addr;

use crate::servers::{GetServersInfosError, Server, Source};

#[derive(Default)]
pub struct SkialSource;

const SKIAL_URL: &str = "https://www.skial.com/api/servers.php";

#[async_trait]
impl Source for SkialSource {
    fn display_name(&self) -> String {
        "Skial".to_string()
    }

    async fn get_servers_infos(&self) -> Result<Vec<Server>, GetServersInfosError> {
        let html = reqwest::get(SKIAL_URL)
            .await
            .map_err(|e| GetServersInfosError {
                source_name: self.display_name(),
                message: e.to_string(),
            })?
            .text()
            .await
            .map_err(|e| GetServersInfosError {
                source_name: self.display_name(),
                message: e.to_string(),
            })?;

        parse_server_infos(&html).map_err(|e| GetServersInfosError {
            source_name: self.display_name(),
            message: e.to_string(),
        })
    }
}

fn from_dec(input: &str) -> IResult<&str, u8> {
    use nom::character::complete::u8;

    u8(input)
}

fn parse_ip(input: &str) -> IResult<&str, Ipv4Addr> {
    let parser = separated_list0(tag("."), from_dec);

    map(parser, |numbers: Vec<u8>| -> Ipv4Addr {
        Ipv4Addr::new(numbers[0], numbers[1], numbers[2], numbers[3])
    })(input)
}

fn parse_port(input: &str) -> IResult<&str, u16> {
    let parser = nom::character::complete::u16;

    parser(input)
}

fn parse_ip_and_port(input: &str) -> IResult<&str, (Ipv4Addr, u16)> {
    let separator = tag(":");
    let mut parser = separated_pair(parse_ip, separator, parse_port);

    parser(input)
}

#[derive(Debug, thiserror::Error)]
pub enum ParseServerInfoError {
    #[error("Missing column '{column_name}' in table")]
    MissingColumn { column_name: String },
}

fn parse_server_infos(html: &str) -> Result<Vec<Server>, ParseServerInfoError> {
    let mut infos = Vec::new();
    let document = Document::from(html);
    let columns_name = document
        .find(Attr("id", "servers").descendant(Name("th")))
        .map(|n| n.text())
        .collect::<Vec<String>>();
    let name_column = find_column_index(&columns_name, "Name")?;
    let players_column = find_column_index(&columns_name, "Players")?;
    let slots_column = find_column_index(&columns_name, "Slots")?;
    let map_column = find_column_index(&columns_name, "Map")?;
    let address_column = find_column_index(&columns_name, "Address")?;

    for tr_node in document.find(Name("tbody").descendant(Name("tr"))) {
        let cells = tr_node.find(Name("td")).map(|n| n.text()).collect::<Vec<String>>();
        let mut server_info = Server::default();
        let address_and_port = &cells[address_column];
        let (_, (address, port)) = parse_ip_and_port(address_and_port).unwrap();

        server_info.name = cells[name_column].clone();
        server_info.map = cells[map_column].clone();
        server_info.max_players_count = cells[slots_column].parse::<u8>().unwrap();
        server_info.current_players_count = cells[players_column].parse::<u8>().unwrap();
        server_info.ip = address;
        server_info.port = port;
        infos.push(server_info);
    }

    Ok(infos)
}

fn find_column_index(columns_names: &[String], name: &str) -> Result<usize, ParseServerInfoError> {
    columns_names
        .iter()
        .position(|n| n == name)
        .ok_or(ParseServerInfoError::MissingColumn {
            column_name: name.to_string(),
        })
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_server_infos() {
        let html_input = include_str!("tests/skial.html");
        let server_infos = super::parse_server_infos(html_input).unwrap();

        assert_eq!(47, server_infos.len());
    }
}
