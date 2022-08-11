use nom::{bytes::complete::tag, combinator::map, multi::separated_list0, sequence::separated_pair, IResult};
use select::{
    document::Document,
    predicate::{Attr, Name, Predicate},
};
use std::net::Ipv4Addr;

#[derive(Debug, Clone)]
pub struct ServerInfo {
    pub name: String,
    pub max_players_count: u8,
    pub current_players_count: u8,
    pub map: String,
    pub ip: std::net::Ipv4Addr,
    pub port: u16,
}

impl Default for ServerInfo {
    fn default() -> Self {
        Self {
            name: Default::default(),
            max_players_count: Default::default(),
            current_players_count: Default::default(),
            map: Default::default(),
            ip: std::net::Ipv4Addr::UNSPECIFIED,
            port: Default::default(),
        }
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

pub fn parse_server_infos(html: &str) -> Vec<ServerInfo> {
    let mut infos = Vec::new();
    let document = Document::from(html);
    let columns_name = document
        .find(Attr("id", "servers").descendant(Name("th")))
        .map(|n| n.text())
        .collect::<Vec<String>>();
    let name_column = columns_name.iter().position(|n| n == "Name").expect("Column Name");
    let players_column = columns_name
        .iter()
        .position(|n| n == "Players")
        .expect("Column Players");
    let slots_column = columns_name.iter().position(|n| n == "Slots").expect("Column Slots");
    let map_column = columns_name.iter().position(|n| n == "Map").expect("Column Map");
    let address_column = columns_name
        .iter()
        .position(|n| n == "Address")
        .expect("Column Address");

    for tr_node in document.find(Name("tbody").descendant(Name("tr"))) {
        let cells = tr_node.find(Name("td")).map(|n| n.text()).collect::<Vec<String>>();
        let mut server_info = ServerInfo::default();
        let address_and_port = &cells[address_column];
        let (_, (address, port)) = parse_ip_and_port(address_and_port).unwrap();

        server_info.name = cells[name_column].clone();
        server_info.map = cells[map_column].clone();
        server_info.max_players_count = u8::from_str_radix(&cells[slots_column], 10).unwrap();
        server_info.current_players_count = u8::from_str_radix(&cells[players_column], 10).unwrap();
        server_info.ip = address;
        server_info.port = port;
        infos.push(server_info);
    }

    infos
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_parse_server_infos() {
        let html_input = include_str!("tests/skial.html");
        let server_infos = super::parse_server_infos(html_input);

        assert_eq!(47, server_infos.len());
    }
}
