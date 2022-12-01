use nom::{
    bytes::complete::{tag, take_while_m_n},
    combinator::map_res,
    sequence::tuple,
    IResult,
};

use crate::models::Color;

fn from_hex(input: &str) -> Result<u8, std::num::ParseIntError> {
    u8::from_str_radix(input, 16)
}

fn is_hex_digit(c: char) -> bool {
    c.is_ascii_hexdigit()
}

fn hex_primary(input: &str) -> IResult<&str, u8> {
    map_res(take_while_m_n(2, 2, is_hex_digit), from_hex)(input)
}

pub(crate) fn hex_color(input: &str) -> IResult<&str, Color> {
    let (input, (r, g, b)) = tuple((hex_primary, hex_primary, hex_primary))(input)?;

    Ok((input, Color { r, g, b }))
}

pub(crate) fn hex_color_prefix(input: &str) -> IResult<&str, Color> {
    let (input, _) = tag("#")(input)?;

    hex_color(input)
}

pub(crate) fn compute_color_component(value: f32) -> u8 {
    (value * u8::MAX as f32) as u8
}