mod details {
    use nom::{
        bytes::complete::{tag, take_while_m_n},
        combinator::map_res,
        sequence::tuple,
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

fn compute_color_component(value: f32) -> u8 {
    (value * u8::MAX as f32) as u8
}

pub fn color_to_str(color: &iced::Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        compute_color_component(color.r),
        compute_color_component(color.g),
        compute_color_component(color.b)
    )
}
