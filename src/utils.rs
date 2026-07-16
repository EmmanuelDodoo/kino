use iced::animation::{Animation, Easing};
use std::path::{Path, PathBuf};

use crate::config::SubtitleDescription;
use core::variants;
pub use length::InterpolableLength;
pub mod icons;
pub mod typo;
use crate::Element;
use crate::theme::{self, Theme};

pub fn toggler<'a, Message, Theme: iced::widget::toggler::Catalog>(
    is_checked: bool,
) -> widgets::toggler::Toggler<'a, Message, Theme> {
    widgets::toggler::Toggler::new(is_checked).size(typo::H6)
}

/// Returns an empty [`iced::Element`].
pub fn empty<'a, Message: 'a>() -> Element<'a, Message> {
    iced::widget::Space::new().width(0).height(0).into()
}

pub fn save_btn<'a, Message: 'a + Clone>() -> iced::widget::Button<'a, Message, Theme> {
    iced::widget::button(typo::medium("Save")).style(theme::styles::button::primary)
}

pub fn delete_btn<'a, Message: 'a + Clone>(
    label: &'a str,
) -> iced::widget::Button<'a, Message, Theme> {
    use iced::widget::{button, row, text};

    button(
        row!(
            icons::icon(icons::DELETE).size(typo::P),
            typo::sized_medium(label, typo::H7)
        )
        .spacing(10.0)
        .align_y(iced::alignment::Vertical::Center),
    )
    .padding([6, 12])
    .style(theme::styles::button::danger)
}

pub fn cancel_btn<'a, Message: 'a + Clone>() -> iced::widget::Button<'a, Message, Theme> {
    iced::widget::button(typo::medium("Cancel")).style(theme::styles::button::secondary)
}

pub fn tooltip<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
    label: impl iced::widget::text::IntoFragment<'a>,
    position: iced::widget::tooltip::Position,
) -> iced::widget::Tooltip<'a, Message, Theme> {
    use iced::Length;
    use iced::Shadow;
    use iced::widget::{container, tooltip};

    tooltip(
        content,
        container(typo::sized_medium(label, typo::H8))
            .clip(true)
            .width(Length::Fit.max(350))
            .height(Length::Fit.max(100))
            .padding([5, 10])
            .style(|theme: &Theme| {
                let schema = theme.schema();
                let color = schema.neutral.strong.color;
                let default = theme::styles::container::nw(theme);
                let shadow = Shadow {
                    color,
                    blur_radius: schema.radii.boxes,
                    offset: [0.0, 0.0].into(),
                };

                container::Style { shadow, ..default }
            }),
        position,
    )
    .delay(iced::time::Duration::from_millis(500))
    .gap(2.0)
}

pub fn path_container<'a, Message>(
    path: impl iced::widget::text::IntoFragment<'a>,
    text_size: f32,
    rtl: bool,
) -> iced::widget::Container<'a, Message, Theme> {
    let path = widgets::marquee(path)
        .size(text_size)
        .font(typo::mono_font())
        .direction(rtl);

    iced::widget::container(path)
        .width(iced::Length::Fit.max(250))
        .style(|theme: &Theme| {
            let color = theme.schema().secondary.weak.color;
            let default = theme::styles::container::transparent(theme);
            let border = default.border.color(color).width(1.0);

            iced::widget::container::Style { border, ..default }
        })
        .padding([2, 6])
}

pub fn modal_container<'a, Message: 'a>(
    content: impl Into<Element<'a, Message>>,
) -> iced::widget::Container<'a, Message, Theme> {
    use iced::alignment::{Horizontal, Vertical};
    use iced::widget::container;

    container(content)
        .padding([8, 12])
        .style(theme::styles::container::bb)
        .clip(true)
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
}

pub fn picklist_handle(size: f32) -> iced::widget::pick_list::Handle<iced::Font> {
    use iced::widget::{pick_list, text};

    let up = pick_list::Icon {
        font: icons::FONT,
        code_point: icons::CHEV_UP,
        size: Some(size.into()),
        line_height: text::LineHeight::Relative(1.0),
        shaping: text::Shaping::Basic,
    };

    let down = pick_list::Icon {
        font: icons::FONT,
        code_point: icons::CHEV_DOWN,
        size: Some(size.into()),
        line_height: text::LineHeight::Relative(1.0),
        shaping: text::Shaping::Basic,
    };

    pick_list::Handle::Dynamic {
        closed: down,
        open: up,
    }
}

pub fn input_actions<'a, Message: 'a + Clone>(
    increase: Message,
    decrease: Message,
) -> Element<'a, Message> {
    let incr = iced::widget::button(icons::icon(icons::CHEV_UP).size(12))
        .padding([2, 2])
        .style(theme::button::subtle_inv)
        .on_press(increase);
    let decr = iced::widget::button(icons::icon(icons::CHEV_DOWN).size(12))
        .padding([2, 2])
        .style(theme::button::subtle_inv)
        .on_press(decrease);

    iced::widget::column!(incr, decr).spacing(2.0).into()
}

pub fn trim_path(path: &Path, components: usize) -> String {
    let path = path
        .components()
        .rev()
        .take(components)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<PathBuf>();
    let path = path.display().to_string();
    path.strip_prefix(r"\\?\").unwrap_or(&path).to_owned()
}

pub fn draw_subtitles<'a, Message: 'a>(
    subtitles: &'a str,
    description: &'a SubtitleDescription,
) -> Element<'a, Message> {
    use iced::alignment::Vertical;
    use iced::font::{Family, Font, Weight};
    use iced::widget::container;

    let SubtitleDescription {
        size,
        color,
        font,
        background_color,
    } = description;

    let font = Font::with_family(font.as_str());

    let content = typo::regular(subtitles)
        .size((*size).max(5))
        .color(*color)
        .font(font)
        .align_y(Vertical::Center);

    let subtitles = container(content)
        .align_y(Vertical::Center)
        .padding([6, 6])
        .style(move |theme| {
            let base = theme::styles::container::transparent(theme);

            container::Style {
                background: Some((*background_color).into()),
                text_color: None,
                ..base
            }
        });

    subtitles.into()
}

fn convert_color_str(input: &str) -> Option<u32> {
    if input.is_empty() {
        return None;
    }

    let input = input.trim();

    let color = if input.contains(",") {
        let values = input
            .trim_start_matches("rgb(")
            .trim_end_matches(")")
            .split(",")
            .enumerate()
            .filter_map(|(idx, value)| {
                if idx != 3 {
                    value.trim().parse::<u8>().ok()
                } else {
                    match value.trim().parse::<u8>().ok() {
                        Some(alpha) => Some(alpha),
                        None => {
                            let alpha = value.trim().parse::<f32>().ok()?;
                            let alpha = alpha.max(0.0);

                            Some((255.0 * alpha).trunc() as u8)
                        }
                    }
                }
            })
            .collect::<Vec<u8>>();

        if values.len() < 3 || values.len() > 4 {
            return None;
        }

        let r = *values.first()? as u32;
        let g = *values.get(1)? as u32;
        let b = *values.get(2)? as u32;
        let a = values.get(3).copied().unwrap_or(255) as u32;

        (r << 24) | (g << 16) | (b << 8) | a
    } else if input.contains("#") {
        u32::from_str_radix(input.trim_start_matches("#"), 16).ok()?
    } else {
        u32::from_str_radix(input.trim(), 16).ok()?
    };

    Some(color)
}

fn u32_to_rgba(color: u32) -> iced::Color {
    let r = (color & 0xff000000) >> 24;
    let g = (color & 0x00ff0000) >> 16;
    let b = (color & 0x0000ff00) >> 8;
    let a = color & 0xff;

    let a = (a as f32) / 255.0;

    iced::color!(r as u8, g as u8, b as u8, a)
}

/// Duration as a String in format `00:00:00`
pub fn duration_string(duration: u64) -> String {
    format!(
        "{:02}:{:02}:{:02}",
        duration / 3600,
        (duration % 3600) / 60,
        (duration % 3600) % 60,
    )
}

pub fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_millis()
}

#[derive(Debug, Clone)]
pub struct Scroll {
    pub id: iced::widget::Id,
    pub offset: iced::widget::operation::AbsoluteOffset,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            id: iced::widget::Id::unique(),
            offset: iced::widget::operation::AbsoluteOffset::default(),
        }
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug)]
pub struct FontState {
    pub state: widgets::font_selection::State,
    pub selected: Option<iced::font::Family>,
}

impl FontState {
    pub fn new(fonts: Vec<iced::font::Family>, default: &String) -> Self {
        let selected = fonts
            .iter()
            .find(|family| family.to_string() == *default)
            .copied();
        let state = widgets::font_selection::State::with_selection(fonts, selected);

        Self { state, selected }
    }
}

mod length {
    use iced::animation::Interpolable;

    #[derive(Debug, Clone, Copy)]
    pub struct InterpolableLength(pub iced::Length);

    impl InterpolableLength {
        // todo: FillPortion(0) takes the entire length. Otherwise this would ideally
        // be set to it
        pub const FILL_ZERO: Self = Self(iced::Length::FillPortion(1));
        pub const FIXED_ZERO: Self = Self(iced::Length::Fixed(0.0));
    }

    impl Interpolable for InterpolableLength {
        fn interpolated(&self, other: Self, ratio: f32) -> Self {
            match (self.0, other.0) {
                (iced::Length::Fixed(x), iced::Length::Fixed(y)) => {
                    let value = x.interpolated(y, ratio);
                    Self(iced::Length::Fixed(value))
                }
                (iced::Length::FillPortion(x), iced::Length::FillPortion(y)) => {
                    let value = ((x as f32).interpolated(y as f32, ratio)).round();

                    Self(iced::Length::FillPortion(value as u16))
                }
                _ => other,
            }
        }
    }

    impl From<iced::Length> for InterpolableLength {
        fn from(value: iced::Length) -> Self {
            Self(value)
        }
    }

    impl From<InterpolableLength> for iced::Length {
        fn from(value: InterpolableLength) -> Self {
            value.0
        }
    }

    impl From<i8> for InterpolableLength {
        fn from(value: i8) -> Self {
            Self((value as f32).into())
        }
    }

    impl From<u16> for InterpolableLength {
        fn from(value: u16) -> Self {
            Self((value as f32).into())
        }
    }
    impl From<i16> for InterpolableLength {
        fn from(value: i16) -> Self {
            Self((value as f32).into())
        }
    }

    impl From<f32> for InterpolableLength {
        fn from(value: f32) -> Self {
            Self((value).into())
        }
    }
}
