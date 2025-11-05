#![allow(dead_code)]
use iced::{
    Font, alignment, font,
    widget::{Button, Text, text},
};

use std::sync::LazyLock;

use super::typo::*;

static ICONS: &[u8] = include_bytes!("../../assets/kino-icons.ttf");
pub const NAME: &str = "kino-icons";
pub const FONT: Font = Font::with_name(NAME);

pub const LOGO: char = '\u{e80b}';
pub const SEARCH: char = '\u{e80c}';

pub const BACK: char = '\u{e80d}';
pub const FORWARD: char = '\u{e80e}';

pub const LOOP: char = '\u{e80f}';
pub const UNLOOP: char = '\u{e80f}';

pub const CHEV_UP: char = '\u{e812}';
pub const CHEV_DOWN: char = '\u{e811}';
pub const CHEV_LEFT: char = '\u{e810}';
pub const CHEV_RIGHT: char = '\u{e813}';
pub const CANCEL: char = '\u{e814}';
pub const EQUALS: char = '\u{e815}';

pub const FILTER: char = '\u{e816}';
pub const SORT: char = '\u{e817}';
pub const GRID: char = '\u{e818}';
pub const LIST: char = '\u{e825}';
pub const RAND: char = '\u{e826}';

pub const ADD_COLLECTION: char = '\u{e807}';
// pub const REM_COLLECTION: char = '\u{e23c}';
// pub const IN_COLLECTION: char = '\u{e524}';
pub const ADD: char = '\u{e819}';
pub const BOOKMARK: char = '\u{e807}';
pub const LIBRARY: char = '\u{e828}';
// pub const LIBRARY: char = '\u{e83d}';

pub const COLLECTION_ICON: char = '\u{e82d}';
pub const POPCORN: char = '\u{e829}';
pub const FILM: char = '\u{e82a}';
pub const TODO: char = '\u{e842}';
pub const SWORD: char = '\u{e843}';
pub const HISTORY: char = '\u{e847}';
pub const GHOST: char = '\u{e84a}';
pub const ALIEN: char = '\u{e84b}';
pub const CROWN: char = '\u{e84c}';
pub const MASKS: char = '\u{e84d}';

pub const VIDEO_CONFIG: char = '\u{e82c}';

pub const SUBTITLES: char = '\u{e82e}';
pub const SUBTITLES_ON: char = '\u{e82e}';
pub const SUBTITLES_OFF: char = '\u{e82f}';

pub const VOLUME: char = '\u{e830}';
pub const MUTE: char = '\u{e831}';

pub const PREVIOUS_VIDEO: char = '\u{e81d}';
pub const NEXT_VIDEO: char = '\u{e81c}';
pub const SEEK_BACK: char = '\u{e81f}';
pub const SEEK_FRONT: char = '\u{e81e}';

pub const PLAY: char = '\u{e808}';
pub const PAUSE: char = '\u{e820}';
pub const REPLAY: char = '\u{e81a}';
pub const REFRESH: char = '\u{e827}';
pub const LOADING: char = '\u{e845}';

pub const FAVORITE: char = '\u{e821}';
pub const UNFAVORITE: char = '\u{e81b}';
pub const COMMENT: char = '\u{e83a}';
pub const MAXIMIZE: char = '\u{e823}';
pub const MINIMIZE: char = '\u{e822}';

pub const HOME: char = '\u{e824}';
pub const SHOW: char = '\u{e83e}';
pub const MOVIE: char = '\u{e83f}';
pub const SETTINGS: char = '\u{e840}';

pub const STAR: char = '\u{e809}';
pub const UNSTAR: char = '\u{e841}';
pub const HALF_STAR: char = '\u{e82b}';

pub const PROGRESS_10: char = '\u{e803}';
pub const PROGRESS_20: char = '\u{e801}';
pub const PROGRESS_40: char = '\u{e806}';
pub const PROGRESS_60: char = '\u{e804}';
pub const PROGRESS_80: char = '\u{e802}';
pub const PROGRESS_90: char = '\u{e805}';
pub const PROGRESS_100: char = '\u{f111}';

pub const HOURGLASS: char = '\u{e834}';
pub const ALARM: char = '\u{e835}';
pub const CLOCK: char = '\u{e836}';
pub const CALENDAR: char = '\u{e837}';
pub const NUMBER: char = '\u{e838}';
pub const EYE: char = '\u{e833}';
pub const HIDE: char = '\u{e832}';

pub const ELLIPSIS_VER: char = '\u{e800}';
pub const ELLIPSIS_HOR: char = '\u{e839}';

pub const PIN: char = '\u{e844}';
pub const EDIT: char = '\u{e846}';

const LOADING_SVG: &[u8] = "<svg xmlns='http://www.w3.org/2000/svg' width='32' height='32' viewBox='0 0 24 24'><!-- Icon from Material Design Icons by Pictogrammers - https://github.com/Templarian/MaterialDesign/blob/master/LICENSE --><path fill='currentColor' d='M12 4V2A10 10 0 0 0 2 12h2a8 8 0 0 1 8-8'/></svg>".as_bytes();

pub const LOADING_SVG_HANDLE: LazyLock<iced::widget::svg::Handle> =
    LazyLock::new(|| iced::widget::svg::Handle::from_memory(LOADING_SVG));

pub fn load_fonts() -> iced::Task<Result<(), font::Error>> {
    font::load(ICONS)
}

fn icon_maker<'a>(unicode: char, name: &'static str) -> Text<'a> {
    let fnt: Font = Font::with_name(name);
    text(unicode.to_string())
        .font(fnt)
        .align_x(alignment::Horizontal::Center)
        .line_height(1.0)
        .size(P)
}

pub fn icon<'a>(unicode: char) -> Text<'a> {
    icon_maker(unicode, NAME)
}

/// Returns a text button
pub fn text_button<'a, Message>(unicode: char) -> Button<'a, Message> {
    use iced::widget::{button, button::text};

    button(icon(unicode)).style(text)
}

pub fn sized_button<'a, Message>(
    unicode: char,
    size: impl Into<iced::Pixels>,
) -> Button<'a, Message> {
    use iced::widget::{button, button::text};

    button(icon(unicode).size(size)).style(text)
}

pub fn alt<'a>(unicode: u32) -> Text<'a> {
    let unicode = char::from_u32(unicode).unwrap();
    icon_maker(unicode, NAME)
}
