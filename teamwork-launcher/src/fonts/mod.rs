use iced::Font;

pub const TF2_SECONDARY: Font = Font::External {
    name: "TF2 Secondary",
    bytes: include_bytes!("TF2secondary.ttf"),
};
pub const TF2_BUILD: Font = Font::External {
    name: "TF2 build",
    bytes: include_bytes!("tf2build.ttf"),
};

pub const TITLE_FONT_SIZE: u16 = 44;
pub const VERSION_FONT_SIZE: u16 = 16;
pub const SUBTITLE_FONT_SIZE: u16 = 32;