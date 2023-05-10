use std::collections::BTreeSet;
use {
    crate::application::{IpPort, Server},
    nom::Finish,
    rfd::AsyncFileDialog,
    serde::{Deserialize, Serialize},
    std::{
        fmt::{Display, Formatter},
        net::Ipv4Addr,
        path::PathBuf,
    },
    tokio::io::{AsyncBufReadExt, BufReader},
};

#[derive(Eq, PartialEq, Debug, Serialize, Deserialize, Clone)]
pub enum BlacklistEntry {
    Text(String),
    Ip(Ipv4Addr),
    IpPort(IpPort),
}

impl BlacklistEntry {
    pub fn parse(input: &str) -> BlacklistEntry {
        // It's safe to unwrap as the parsing should never fail (in the worst case it's
        // a BlacklistEntry::Text.
        parsing::parse_entry(input).finish().ok().map(|(_, output)| output).unwrap()
    }

    pub fn accept(&self, server: &Server) -> bool {
        match self {
            BlacklistEntry::Text(text) => !server.name.contains(text),
            BlacklistEntry::Ip(ip) => server.ip_port.ip() != ip,
            BlacklistEntry::IpPort(ip_port) => &server.ip_port != ip_port,
        }
    }
}

impl Display for BlacklistEntry {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            BlacklistEntry::Text(text) => {
                write!(f, "{}", text)
            }
            BlacklistEntry::Ip(ip) => {
                write!(f, "{}", ip)
            }
            BlacklistEntry::IpPort(ip_port) => {
                write!(f, "{}", ip_port)
            }
        }
    }
}

mod parsing {
    use {
        crate::application::{blacklist::BlacklistEntry, IpPort},
        nom::{
            branch::alt,
            bytes::complete::tag,
            character::complete::digit1,
            combinator::{map, map_res},
            sequence::tuple,
            IResult,
        },
        std::net::Ipv4Addr,
    };

    fn parse_u8(input: &str) -> IResult<&str, u8> {
        let mut parser = map_res(digit1, str::parse);

        parser(input)
    }

    fn parse_u16(input: &str) -> IResult<&str, u16> {
        let mut parser = map_res(digit1, str::parse);

        parser(input)
    }

    fn parse_ip_impl(input: &str) -> IResult<&str, Ipv4Addr> {
        let mut parser = map(
            tuple((parse_u8, tag("."), parse_u8, tag("."), parse_u8, tag("."), parse_u8)),
            |(a, _, b, _, c, _, d)| Ipv4Addr::new(a, b, c, d),
        );

        parser(input)
    }

    fn parse_ip(input: &str) -> IResult<&str, BlacklistEntry> {
        let mut parser = map(parse_ip_impl, |ip| BlacklistEntry::Ip(ip));

        parser(input)
    }

    fn parse_ip_port(input: &str) -> IResult<&str, BlacklistEntry> {
        let mut parser = map(tuple((parse_ip_impl, tag(":"), parse_u16)), |(ip, _sep, port)| {
            BlacklistEntry::IpPort(IpPort::new(ip, port))
        });

        parser(input)
    }

    fn parse_text(input: &str) -> IResult<&str, BlacklistEntry> {
        Ok(("", BlacklistEntry::Text(input.to_string())))
    }

    pub fn parse_entry(input: &str) -> IResult<&str, BlacklistEntry> {
        let mut parser = alt((parse_ip_port, parse_ip, parse_text));

        parser(input)
    }
}

#[derive(Serialize, Deserialize, Default)]
pub struct Blacklist {
    entries: Vec<BlacklistEntry>,
}

impl Blacklist {
    pub fn push(&mut self, entry: BlacklistEntry) {
        if !self.entries.contains(&entry) {
            self.entries.push(entry);
        }
    }

    pub fn remove(&mut self, index: usize) {
        self.entries.remove(index);
    }
    pub fn clear(&mut self) {
        self.entries.clear();
    }
    pub fn accept(&self, server: &Server) -> bool {
        for entry in &self.entries {
            if !entry.accept(server) {
                return false;
            }
        }

        true
    }

    pub fn index_of(&self, entry: &BlacklistEntry) -> Option<usize> {
        self.entries.iter().position(|e| e == entry)
    }

    pub fn iter(&self) -> impl Iterator<Item = &BlacklistEntry> {
        self.entries.iter()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(thiserror::Error, Debug, Clone)]
pub enum ImportBlacklistError {
    #[error("Failed to import blacklist file {0}: {1}")]
    Io(PathBuf, String),
}

pub async fn import_blacklist() -> Result<Vec<BlacklistEntry>, ImportBlacklistError> {
    let file_handle = AsyncFileDialog::new().set_directory("/").pick_file().await;
    let mut terms = Vec::new();

    if let Some(file_handle) = file_handle {
        let file_content = tokio::fs::read_to_string(file_handle.path())
            .await
            .map_err(|error| ImportBlacklistError::Io(file_handle.path().to_path_buf(), error.to_string()))?;
        let mut reader = BufReader::new(file_content.as_bytes()).lines();

        while let Some(line) = reader
            .next_line()
            .await
            .map_err(|error| ImportBlacklistError::Io(file_handle.path().to_path_buf(), error.to_string()))?
        {
            if !line.trim().is_empty() {
                terms.push(BlacklistEntry::parse(&line));
            }
        }
    }

    Ok(terms)
}

#[cfg(test)]
mod tests {
    use crate::application::blacklist::Blacklist;
    use crate::application::Server;
    use {
        crate::application::{blacklist::BlacklistEntry, IpPort},
        std::net::Ipv4Addr,
        test_case::test_case,
    };

    #[test_case("hello", BlacklistEntry::Text("hello".to_string()))]
    #[test_case("123.45.67.89", BlacklistEntry::Ip(Ipv4Addr::new(123, 45, 67, 89)))]
    #[test_case(
    "123.45.67.89:321",
    BlacklistEntry::IpPort(IpPort::new(Ipv4Addr::new(123, 45, 67, 89), 321))
    )]
    fn test_entry_parse(input: &str, expected: BlacklistEntry) {
        assert_eq!(BlacklistEntry::parse(input), expected)
    }

    #[test]
    fn test_blacklist_by_ip() {
        let accepted_server = Server {
            name: "test 2".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            next_map: None,
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(4, 3, 2, 1), 1234),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![],
            provider: "".to_string(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        };
        let rejected_server = Server {
            name: "test".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            next_map: None,
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(1, 2, 3, 4), 1234),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![],
            provider: "".to_string(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        };
        let mut blacklist = Blacklist::default();

        blacklist.push(BlacklistEntry::Ip(Ipv4Addr::new(1, 2, 3, 4)));

        assert_eq!(false, blacklist.accept(&rejected_server));
        assert_eq!(true, blacklist.accept(&accepted_server));
    }

    #[test]
    fn test_blacklist_by_ip_port() {
        let accepted_server = Server {
            name: "test 2".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            next_map: None,
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(1, 2, 3, 4), 421),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![],
            provider: "".to_string(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        };
        let rejected_server = Server {
            name: "test".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            next_map: None,
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(1, 2, 3, 4), 1234),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![],
            provider: "".to_string(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        };
        let mut blacklist = Blacklist::default();

        blacklist.push(BlacklistEntry::IpPort(IpPort::new(Ipv4Addr::new(1, 2, 3, 4), 1234)));

        assert_eq!(false, blacklist.accept(&rejected_server));
        assert_eq!(true, blacklist.accept(&accepted_server));
    }

    #[test]
    fn test_blacklist_by_text_port() {
        let accepted_server = Server {
            name: "test 2".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            next_map: None,
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(1, 2, 3, 4), 421),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![],
            provider: "".to_string(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        };
        let rejected_server = Server {
            name: "test_reject".to_string(),
            max_players_count: 0,
            current_players_count: 0,
            map: Default::default(),
            next_map: None,
            map_thumbnail: Default::default(),
            ip_port: IpPort::new(Ipv4Addr::new(1, 2, 3, 4), 1234),
            country: Default::default(),
            ping: Default::default(),
            source_key: None,
            game_modes: vec![],
            provider: "".to_string(),
            vac_secured: false,
            has_rtd: false,
            has_no_respawn_time: false,
            has_all_talk: false,
            has_random_crits: false,
            need_password: false,
        };
        let mut blacklist = Blacklist::default();

        blacklist.push(BlacklistEntry::Text("reject".into()));

        assert_eq!(false, blacklist.accept(&rejected_server));
        assert_eq!(true, blacklist.accept(&accepted_server));
    }
}