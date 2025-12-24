use iced::font::{Family, Font, Stretch, Style, Weight};
use iced::widget::text::{self, Text};

static INTER: &[u8] = include_bytes!("../../resources/fonts/Inter-Regular.ttf");
const INTER_NAME: &str = "Inter";

static INTER_IT: &[u8] = include_bytes!("../../resources/fonts/Inter-Italic.ttf");
const INTER_NAME_IT: &str = "Inter-IT";

static INTER_SB: &[u8] = include_bytes!("../../resources/fonts/Inter-SemiBold.ttf");
const INTER_NAME_SB: &str = "Inter-SB";

static INTER_MD: &[u8] = include_bytes!("../../resources/fonts/Inter-Medium.ttf");
const INTER_NAME_MD: &str = "Inter-MD";

static INTER_SB_IT: &[u8] = include_bytes!("../../resources/fonts/Inter-SemiBoldItalic.ttf");
const INTER_NAME_SB_IT: &str = "Inter-SB-IT";

static JB: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-Regular.ttf");
const JB_NAME: &str = "JetBrains Mono";

static JB_SB: &[u8] = include_bytes!("../../resources/fonts/JetBrainsMono-SemiBold.ttf");
const JB_NAME_SB: &str = "JetBrains-SB";

static FRAUNCES: &[u8] = include_bytes!("../../resources/fonts/Fraunces-SemiBold.ttf");
const FRAUNCES_NAME: &str = "Fraunces-SB";

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
        INTER.into(),
        INTER_SB.into(),
        INTER_MD.into(),
        INTER_IT.into(),
        INTER_SB_IT.into(),
        JB.into(),
        JB_SB.into(),
        FRAUNCES.into(),
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
        family: Family::Name(JB_NAME),
        weight: Weight::Normal,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn mono_bold_font() -> Font {
    Font {
        family: Family::Name(JB_NAME_SB),
        weight: Weight::Semibold,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn regular_font() -> Font {
    Font {
        family: Family::Name(INTER_NAME),
        weight: Weight::Normal,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn italic_font() -> Font {
    Font {
        family: Family::Name(INTER_NAME_IT),
        weight: Weight::Normal,
        style: Style::Italic,
        stretch: Stretch::Normal,
    }
}

pub fn bold_italic_font() -> Font {
    Font {
        family: Family::Name(INTER_NAME_SB_IT),
        weight: Weight::Semibold,
        style: Style::Italic,
        stretch: Stretch::Normal,
    }
}

pub fn bold_font() -> Font {
    Font {
        family: Family::Name(INTER_NAME_SB),
        weight: Weight::Semibold,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn medium_font() -> Font {
    Font {
        family: Family::Name(INTER_NAME_MD),
        weight: Weight::Medium,
        style: Style::Normal,
        stretch: Stretch::Normal,
    }
}

pub fn mono<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    Text::new(text).size(P).font(mono_font())
}

pub fn display<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    Text::new(text).size(H2).font(display_font())
}

pub fn mono_bold<'a>(text: impl text::IntoFragment<'a>) -> Text<'a> {
    Text::new(text).size(P).font(mono_bold_font())
}

pub fn sized_bold<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text).size(size).font(bold_font())
}

pub fn sized_medium<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text).size(size).font(medium_font())
}

pub fn sized_italic<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text).size(size).font(italic_font())
}

pub fn sized_regular<'a>(text: impl text::IntoFragment<'a>, size: f32) -> Text<'a> {
    Text::new(text).size(size).font(regular_font())
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
