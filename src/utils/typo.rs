use iced::font::{Family, Font, Stretch, Style, Weight};
use iced::widget::text::{self, Ellipsis, Text};

static REGULAR: &[u8] = include_bytes!("../../resources/fonts/Inter-VariableFont_slnt,wght.ttf");
static REGULAR_IT: &[u8] = include_bytes!("../../resources/fonts/Inter-italic.var.ttf");
const REGULAR_NAME: &str = "Inter";

static MONO: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-VariableFont_wght.ttf");
static MONO_IT: &[u8] =
    include_bytes!("../../resources/fonts/JetBrainsMono-Italic-VariableFont_wght.ttf");
const MONO_NAME: &str = "JetBrains Mono";

static FRAUNCES: &[u8] =
    include_bytes!("../../resources/fonts/Fraunces-VariableFont_SOFT,WONK,opsz,wght.ttf");
static FRAUNCES_IT: &[u8] =
    include_bytes!("../../resources/fonts/Fraunces-Italic-VariableFont_SOFT,WONK,opsz,wght.ttf");
const FRAUNCES_NAME: &str = "Fraunces";

static SUBTITLES: &[u8] = include_bytes!("../../resources/fonts/Roboto-VariableFont_wdth,wght.ttf");
pub const DEFAULT_SUBTITLE_FONT_NAME: &str = "Roboto";

pub const RATIO: f32 = 1.125;
pub const H1: f32 = H2 * RATIO;
pub const H2: f32 = H3 * RATIO;
pub const H3: f32 = H4 * RATIO;
pub const H4: f32 = H5 * RATIO;
pub const H5: f32 = H6 * RATIO;
pub const H6: f32 = P * RATIO;
pub const P: f32 = 16.0;
pub const H7: f32 = P / RATIO;
pub const H8: f32 = H7 / RATIO;

#[rustfmt::skip]
pub fn typo_fonts() -> Vec<std::borrow::Cow<'static, [u8]>> {
    [
        REGULAR.into(),
        REGULAR_IT.into(),
        MONO.into(),
        MONO_IT.into(),
        FRAUNCES.into(),
        FRAUNCES_IT.into(),
        SUBTITLES.into(),
    ]
    .to_vec()
}

pub fn display_font() -> Font {
    Font {
        family: Family::Name(FRAUNCES_NAME),
        weight: Weight::Semibold,
        ..Default::default()
    }
}

pub fn mono_font() -> Font {
    Font {
        family: Family::Name(MONO_NAME),
        weight: Weight::Normal,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn mono_bold_font() -> Font {
    Font {
        family: Family::Name(MONO_NAME),
        weight: Weight::Semibold,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn regular_font() -> Font {
    Font {
        family: Family::Name(REGULAR_NAME),
        weight: Weight::Normal,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn italic_font() -> Font {
    Font {
        family: Family::Name(REGULAR_NAME),
        weight: Weight::Normal,
        style: Style::Italic,
        stretch: Stretch::Normal,
    }
}

pub fn bold_italic_font() -> Font {
    Font {
        family: Family::Name(REGULAR_NAME),
        weight: Weight::Semibold,
        style: Style::Italic,
        stretch: Stretch::Normal,
    }
}

pub fn bold_font() -> Font {
    Font {
        family: Family::Name(REGULAR_NAME),
        weight: Weight::Semibold,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn medium_font() -> Font {
    Font {
        family: Family::Name(REGULAR_NAME),
        weight: Weight::Medium,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn mono<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    Text::new(text)
        .size(P)
        .font(mono_font())
        .ellipsis(Ellipsis::End)
}

pub fn display<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    Text::new(text)
        .size(H2)
        .font(display_font())
        .ellipsis(Ellipsis::End)
}

pub fn mono_bold<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    Text::new(text)
        .size(P)
        .font(mono_bold_font())
        .ellipsis(Ellipsis::End)
}

pub fn sized_bold<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text)
        .size(size)
        .font(bold_font())
        .ellipsis(Ellipsis::End)
}

pub fn sized_medium<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text)
        .size(size)
        .font(medium_font())
        .ellipsis(Ellipsis::End)
}

pub fn sized_italic<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text)
        .size(size)
        .font(italic_font())
        .ellipsis(Ellipsis::End)
}

pub fn sized_regular<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text)
        .size(size)
        .font(regular_font())
        .ellipsis(Ellipsis::End)
}

pub fn regular<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_regular(text, P)
}

pub fn bold<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, P)
}

pub fn medium<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_medium(text, P)
}

pub fn italic<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_italic(text, P)
}

pub fn h1<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H1)
}

pub fn h2<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H2)
}

pub fn h3<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H3)
}

pub fn h4<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H4)
}

pub fn h5<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H5)
}

pub fn h6<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H6)
}

pub fn h7<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H7)
}

pub fn h8<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    sized_bold(text, H8)
}
