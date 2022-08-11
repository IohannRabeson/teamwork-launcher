use iced::widget::svg::Handle;
use xmltree::Element;

use crate::colors::color_to_str;

pub fn load_svg(bytes: &[u8], color: &iced::Color, error_message: &str) -> Handle {
    let mut svg_document = Element::parse(bytes).expect(error_message);

    svg_document.attributes.insert("fill".to_string(), color_to_str(color));

    let mut buffer: Vec<u8> = Vec::new();

    svg_document.write(&mut buffer).expect(error_message);

    Handle::from_memory(buffer)
}
