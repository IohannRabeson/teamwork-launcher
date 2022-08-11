use iced::widget::svg::Handle;
use std::sync::Arc;
use xmltree::Element;

use crate::{colors::color_to_str, Error};

pub fn load_svg(bytes: &[u8], color: &iced::Color) -> Result<Handle, Error> {
    let mut svg_document = Element::parse(bytes).map_err(|e| Error::Xml(Arc::new(e)))?;

    svg_document
        .attributes
        .insert("fill".to_string(), color_to_str(color));

    let mut buffer: Vec<u8> = Vec::new();

    svg_document
        .write(&mut buffer)
        .map_err(|e| Error::XmlTree(Arc::new(e)))?;

    Ok(Handle::from_memory(buffer))
}
