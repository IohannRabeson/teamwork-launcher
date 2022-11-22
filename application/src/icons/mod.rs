use std::rc::Rc;

pub use iced::widget::svg::Handle as SvgHandle;
use iced::{Color, Theme};

pub struct Icons {
    storage: Rc<IconsStorage>,
}

impl Icons {
    pub fn new(theme: &Theme) -> Self {
        let light_color = &theme.palette().background;

        Self {
            storage: Rc::new(IconsStorage {
                clear: load_svg(include_bytes!("clear.svg"), light_color, "clear.svg"),
                copy: load_svg(include_bytes!("copy.svg"), light_color, "copy.svg"),
                favorite_border: load_svg(include_bytes!("favorite_border.svg"), light_color, "favorite_border.svg"),
                favorite: load_svg(include_bytes!("favorite.svg"), light_color, "favorite.svg"),
                refresh: load_svg(include_bytes!("refresh.svg"), light_color, "refresh.svg"),
                settings: load_svg(include_bytes!("settings.svg"), light_color, "settings.svg"),
                back: load_svg(include_bytes!("back.svg"), light_color, "back.svg"),
            }),
        }
    }

    pub fn clear(&self) -> SvgHandle {
        self.storage.clear.clone()
    }
    pub fn copy(&self) -> SvgHandle {
        self.storage.copy.clone()
    }
    pub fn favorite_border(&self) -> SvgHandle {
        self.storage.favorite_border.clone()
    }
    pub fn favorite(&self) -> SvgHandle {
        self.storage.favorite.clone()
    }
    pub fn refresh(&self) -> SvgHandle {
        self.storage.refresh.clone()
    }
    pub fn settings(&self) -> SvgHandle {
        self.storage.settings.clone()
    }
    pub fn back(&self) -> SvgHandle {
        self.storage.back.clone()
    }
}

struct IconsStorage {
    clear: SvgHandle,
    copy: SvgHandle,
    favorite_border: SvgHandle,
    favorite: SvgHandle,
    refresh: SvgHandle,
    settings: SvgHandle,
    back: SvgHandle,
}

fn load_svg(bytes: &[u8], color: &Color, error_message: &str) -> SvgHandle {
    use xmltree::Element;

    let mut svg_document = Element::parse(bytes).expect(error_message);

    svg_document.attributes.insert("fill".to_string(), color_to_str(color));

    let mut buffer: Vec<u8> = Vec::new();

    svg_document.write(&mut buffer).expect(error_message);

    SvgHandle::from_memory(buffer)
}

fn compute_color_component(value: f32) -> u8 {
    (value * u8::MAX as f32) as u8
}

fn color_to_str(color: &iced::Color) -> String {
    format!(
        "#{:02x}{:02x}{:02x}",
        compute_color_component(color.r),
        compute_color_component(color.g),
        compute_color_component(color.b)
    )
}
