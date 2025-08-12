#![allow(dead_code)]
use iced::{
    Font, alignment,
    widget::{Button, Text, text},
};

pub static LUCIDE_BYTES: &[u8] = include_bytes!("../../assets/lucide.ttf");
pub const NAME: &'static str = "lucide";
pub const FONT: Font = Font::with_name(NAME);

pub const BACK: char = '\u{e45c}';

pub const LOOP: char = '\u{e14a}';
pub const UNLOOP: char = '\u{E800}';

pub const ADD_COLLECTION: char = '\u{e23d}';
pub const REM_COLLECTION: char = '\u{e23c}';
pub const IN_COLLECTION: char = '\u{e524}';

pub const VIDEO_CONFIG: char = '\u{e29a}';

pub const SUBTITLES: char = '\u{e3a8}';
pub const SUBTITLES_ON: char = '\u{e3a8}';
pub const SUBTITLES_OFF: char = '\u{e5c6}';

pub const VOLUME: char = '\u{e1ab}';
pub const MUTE: char = '\u{e1ac}';

pub const PREVIOUS_VIDEO: char = '\u{e163}';
pub const NEXT_VIDEO: char = '\u{e164}';
pub const SEEK_BACK: char = '\u{e14b}';
pub const SEEK_FRONT: char = '\u{e0c1}';

pub const PLAY: char = '\u{e140}';
pub const PAUSE: char = '\u{e132}';
pub const REPLAY: char = '\u{e14c}';

pub const FAVORITE: char = '\u{e0f6}';
pub const COMMENT: char = '\u{e57a}';
pub const MAXIMIZE: char = '\u{e116}';
pub const MINIMIZE: char = '\u{e11e}';

fn icon_maker<'a>(unicode: char, name: &'static str) -> Text<'a> {
    let fnt: Font = Font::with_name(name);
    text(unicode.to_string())
        .font(fnt)
        .align_x(alignment::Horizontal::Center)
        .line_height(1.0)
        .size(20.0)
}

pub fn icon<'a>(unicode: char) -> Text<'a> {
    icon_maker(unicode, NAME)
}

/// Returns a text button
pub fn text_button<'a, Message>(unicode: char) -> Button<'a, Message> {
    use iced::widget::{button, button::text};

    button(icon(unicode)).style(text)
}

pub fn alt<'a>(unicode: u32) -> Text<'a> {
    let unicode = char::from_u32(unicode).unwrap();
    icon_maker(unicode, NAME)
}
