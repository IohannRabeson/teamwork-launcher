pub use iced::widget::{image::Handle as ImageHandle, svg::Handle as SvgHandle};
use {
    iced::{Color, Theme},
    include_dir::{include_dir, Dir},
    nom::AsBytes,
    std::{collections::BTreeMap, rc::Rc},
};

pub struct Icons {
    storage: Rc<IconsStorage>,
}

/// This folder is part of a Git submodule.
/// It contains all the SVG files for country flags.
/// Mind to update it sometimes.
/// Also, it's not that huge, only ~3.3Mo for the whole directory.
static FLAGS_SVG_ICONS: Dir<'_> = include_dir!("teamwork-launcher/src/icons/flag-icons/flags/1x1");

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
                no_image: ImageHandle::from_memory(include_bytes!("no-image.png").as_bytes()),
                flags: load_flags_icons(),
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
    pub fn no_image(&self) -> ImageHandle {
        self.storage.no_image.clone()
    }
    pub fn flag(&self, country_code: &str) -> Option<SvgHandle> {
        self.storage.flags.get(&country_code.to_lowercase()).cloned()
    }
}

fn load_flags_icons() -> BTreeMap<String, SvgHandle> {
    FLAGS_SVG_ICONS
        .files()
        .filter_map(|entry| {
            entry.path().file_stem().map(|file_stem| {
                let key = file_stem.to_string_lossy().to_string().to_lowercase();
                let svg = SvgHandle::from_memory(entry.contents());

                (key, svg)
            })
        })
        .collect()
}

struct IconsStorage {
    clear: SvgHandle,
    copy: SvgHandle,
    favorite_border: SvgHandle,
    favorite: SvgHandle,
    refresh: SvgHandle,
    settings: SvgHandle,
    back: SvgHandle,
    no_image: ImageHandle,
    flags: BTreeMap<String, SvgHandle>,
}

/// Load and color a SVG image.
/// This hack to color the SVG will be properly fixed with https://github.com/iced-rs/iced/pull/1541.
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
