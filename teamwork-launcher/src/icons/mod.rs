pub use iced::widget::{image::Handle as ImageHandle, svg::Handle as SvgHandle};
use {
    iced::{Color, Theme},
    include_dir::{include_dir, Dir},
    nom::AsBytes,
    std::{collections::BTreeMap, rc::Rc},
};

/// This folder is part of a Git submodule.
/// It contains all the SVG files for country flags.
/// Mind to update it sometimes.
/// Also, it's not that huge, only ~3.3Mo for the whole directory.
static FLAGS_SVG_ICONS: Dir<'_> = include_dir!("teamwork-launcher/src/icons/flag-icons/flags/1x1");

use lazy_static::lazy_static;

lazy_static! {
    pub static ref PLAY_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("box-arrow-in-right.svg").as_bytes());
    pub static ref CLEAR_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("clear.svg").as_bytes());
    pub static ref COPY_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("copy.svg").as_bytes());
    pub static ref FAVORITE_UNCHECKED_ICON: SvgHandle =
        SvgHandle::from_memory(include_bytes!("favorite_border.svg").as_bytes());
    pub static ref FAVORITE_CHECKED_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("favorite.svg").as_bytes());
    pub static ref REFRESH_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("refresh.svg").as_bytes());
    pub static ref SETTINGS_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("settings.svg").as_bytes());
    pub static ref BACK_ICON: SvgHandle = SvgHandle::from_memory(include_bytes!("back.svg").as_bytes());
    pub static ref NO_IMAGE: ImageHandle = ImageHandle::from_memory(include_bytes!("no-image.png").as_bytes());
    pub static ref FLAGS: BTreeMap<String, SvgHandle> = FLAGS_SVG_ICONS
        .files()
        .filter_map(|entry| {
            entry.path().file_stem().map(|file_stem| {
                let key = file_stem.to_string_lossy().to_string().to_lowercase();
                let svg = SvgHandle::from_memory(entry.contents());

                (key, svg)
            })
        })
        .collect();
}

pub fn flag(country_code: &str) -> Option<SvgHandle> {
    FLAGS.get(&country_code.to_lowercase()).cloned()
}
