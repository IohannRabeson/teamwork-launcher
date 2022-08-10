mod details {
    use nom::{
        bytes::complete::{tag, take_while_m_n}, combinator::{map, map_res}, multi::separated_list0, sequence::{separated_pair, tuple},
        IResult,
    };

    fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
        u8::from_str_radix(input, 16)
    }

    fn is_hex_digit(c: char) -> bool {
        c.is_digit(16)
    }

    fn hex_primary(input: &str) -> IResult<&str, u8> {
        map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
    }

    pub fn hex_color(input: &str) -> IResult<&str, iced::Color> {
        let (input, _) = tag("#")(input)?;
        let (input, (red, green, blue)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

        Ok((input, iced::Color::from_rgb8(red, green, blue)))
    }
}

pub fn parse_color(text: &str) -> iced::Color {
    details::hex_color(text).unwrap().1
}
