#![allow(dead_code)]
use iced::{
    Background, Shadow, Vector, border, border::Border, color, color::Color, theme, widget,
};

use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

mod variants {
    use super::*;

    pub static ABYSS: LazyLock<Schema> = LazyLock::new(|| abyss());

    pub static BLACK: LazyLock<Schema> = LazyLock::new(|| black());

    pub static EMBER: LazyLock<Schema> = LazyLock::new(|| ember());

    pub static LUXURY: LazyLock<Schema> = LazyLock::new(|| luxury());

    pub static FANTASY: LazyLock<Schema> = LazyLock::new(|| fantasy());

    pub static BUMBLEBEE: LazyLock<Schema> = LazyLock::new(|| bumblebee());

    pub static AUTUMN: LazyLock<Schema> = LazyLock::new(|| autumn());

    fn abyss() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(0, 30, 41),
                    text: color!(255, 214, 167),
                },
                base: Pair {
                    color: color!(0, 17, 29),
                    text: color!(255, 214, 167),
                },
                strong: Pair {
                    color: color!(0, 6, 17),
                    text: color!(255, 214, 167),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(189, 255, 0),
                text: color!(66, 118, 0),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(206, 190, 244),
                text: color!(86, 71, 117),
            }),
            accent: Triplet::generate(Pair {
                color: color!(80, 80, 80),
                text: color!(248, 248, 248),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(0, 56, 67),
                text: color!(255, 214, 167),
            }),
            info: Triplet::generate(Pair {
                color: color!(0, 186, 254),
                text: color!(4, 46, 73),
            }),
            success: Triplet::generate(Pair {
                color: color!(1, 223, 114),
                text: color!(2, 45, 20),
            }),
            warning: Triplet::generate(Pair {
                color: color!(255, 191, 0),
                text: color!(133, 66, 0),
            }),
            danger: Triplet::generate(Pair {
                color: color!(240, 78, 79),
                text: color!(105, 0, 0),
            }),
            radii: Radii {
                boxes: 8.0,
                fields: 8.0,
                selectors: 4.0,
            },
            is_dark: true,
        }
    }

    fn black() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(20, 20, 20),
                    text: color!(214, 214, 214),
                },
                base: Pair {
                    color: color!(0, 0, 0),
                    text: color!(214, 214, 214),
                },
                strong: Pair {
                    color: color!(0, 0, 0),
                    text: color!(214, 214, 214),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(58, 58, 58),
                text: color!(255, 255, 255),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(58, 58, 58),
                text: color!(255, 255, 255),
            }),
            accent: Triplet::generate(Pair {
                color: color!(58, 58, 58),
                text: color!(255, 255, 255),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(58, 58, 58),
                text: color!(255, 255, 255),
            }),
            info: Triplet::generate(Pair {
                color: color!(0, 0, 255),
                text: color!(198, 219, 255),
            }),
            success: Triplet::generate(Pair {
                color: color!(2, 128, 2),
                text: color!(211, 230, 208),
            }),
            warning: Triplet::generate(Pair {
                color: color!(255, 255, 0),
                text: color!(22, 22, 0),
            }),
            danger: Triplet::generate(Pair {
                color: color!(255, 3, 1),
                text: color!(22, 0, 0),
            }),
            radii: Radii {
                boxes: 4.0,
                fields: 4.0,
                selectors: 4.0,
            },
            is_dark: true,
        }
    }

    fn ember() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(27, 27, 27),
                    text: color!(214, 214, 214),
                },
                base: Pair {
                    color: color!(0, 0, 0),
                    text: color!(214, 214, 214),
                },
                strong: Pair {
                    color: color!(11, 11, 11),
                    text: color!(214, 214, 214),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(255, 103, 0),
                text: color!(19, 22, 22),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(60, 90, 120),
                text: color!(255, 255, 255),
            }),
            accent: Triplet::generate(Pair {
                color: color!(253, 207, 43),
                text: color!(0, 0, 0),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(58, 58, 58),
                text: color!(255, 255, 255),
            }),
            info: Triplet::generate(Pair {
                color: color!(25, 58, 183),
                text: color!(198, 219, 255),
            }),
            success: Triplet::generate(Pair {
                color: color!(2, 128, 2),
                text: color!(211, 230, 208),
            }),
            warning: Triplet::generate(Pair {
                color: color!(255, 167, 0),
                text: color!(22, 22, 0),
            }),
            danger: Triplet::generate(Pair {
                color: color!(191, 0, 4),
                text: color!(22, 0, 0),
            }),
            radii: Radii {
                boxes: 4.0,
                fields: 4.0,
                selectors: 4.0,
            },
            is_dark: true,
        }
    }

    fn luxury() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(30, 29, 31),
                    text: color!(220, 165, 77),
                },
                base: Pair {
                    color: color!(9, 9, 11),
                    text: color!(220, 165, 77),
                },
                strong: Pair {
                    color: color!(23, 22, 24),
                    text: color!(220, 165, 77),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(255, 255, 255),
                text: color!(22, 22, 22),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(21, 39, 71),
                text: color!(203, 208, 215),
            }),
            accent: Triplet::generate(Pair {
                color: color!(81, 52, 72),
                text: color!(218, 211, 215),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(51, 24, 0),
                text: color!(255, 231, 164),
            }),
            info: Triplet::generate(Pair {
                color: color!(103, 198, 255),
                text: color!(4, 14, 22),
            }),
            success: Triplet::generate(Pair {
                color: color!(135, 208, 58),
                text: color!(6, 16, 1),
            }),
            warning: Triplet::generate(Pair {
                color: color!(226, 213, 99),
                text: color!(18, 16, 3),
            }),
            danger: Triplet::generate(Pair {
                color: color!(255, 111, 111),
                text: color!(22, 4, 4),
            }),
            radii: Radii {
                boxes: 16.0,
                fields: 8.0,
                selectors: 16.0,
            },
            is_dark: true,
        }
    }

    fn fantasy() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(255, 255, 255),
                    text: color!(31, 41, 55),
                },
                base: Pair {
                    color: color!(232, 232, 232),
                    text: color!(31, 41, 55),
                },
                strong: Pair {
                    color: color!(209, 209, 209),
                    text: color!(31, 41, 55),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(109, 0, 118),
                text: color!(227, 206, 228),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(0, 117, 194),
                text: color!(207, 228, 244),
            }),
            accent: Triplet::generate(Pair {
                color: color!(255, 134, 0),
                text: color!(24, 6, 0),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(31, 41, 55),
                text: color!(205, 208, 211),
            }),
            info: Triplet::generate(Pair {
                color: color!(0, 181, 255),
                text: color!(0, 0, 0),
            }),
            success: Triplet::generate(Pair {
                color: color!(0, 169, 110),
                text: color!(0, 0, 0),
            }),
            warning: Triplet::generate(Pair {
                color: color!(255, 190, 0),
                text: color!(0, 0, 0),
            }),
            danger: Triplet::generate(Pair {
                color: color!(255, 88, 97),
                text: color!(0, 0, 0),
            }),
            radii: Radii {
                boxes: 16.0,
                fields: 8.0,
                selectors: 16.0,
            },
            is_dark: false,
        }
    }

    fn bumblebee() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(255, 255, 255),
                    text: color!(22, 22, 22),
                },
                base: Pair {
                    color: color!(245, 245, 245),
                    text: color!(22, 22, 22),
                },
                strong: Pair {
                    color: color!(228, 228, 228),
                    text: color!(22, 22, 22),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(253, 199, 0),
                text: color!(115, 62, 10),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(255, 137, 4),
                text: color!(124, 40, 8),
            }),
            accent: Triplet::generate(Pair {
                color: color!(0, 0, 0),
                text: color!(255, 255, 255),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(67, 63, 58),
                text: color!(230, 228, 227),
            }),
            info: Triplet::generate(Pair {
                color: color!(0, 186, 254),
                text: color!(1, 74, 112),
            }),
            success: Triplet::generate(Pair {
                color: color!(0, 211, 144),
                text: color!(0, 76, 57),
            }),
            warning: Triplet::generate(Pair {
                color: color!(252, 183, 0),
                text: color!(121, 50, 5),
            }),
            danger: Triplet::generate(Pair {
                color: color!(255, 98, 102),
                text: color!(255, 98, 102),
            }),
            radii: Radii {
                boxes: 16.0,
                fields: 8.0,
                selectors: 16.0,
            },
            is_dark: false,
        }
    }

    fn autumn() -> Schema {
        Schema {
            background: Triplet {
                weak: Pair {
                    color: color!(241, 241, 241),
                    text: color!(20, 20, 20),
                },
                base: Pair {
                    color: color!(219, 219, 219),
                    text: color!(20, 20, 20),
                },
                strong: Pair {
                    color: color!(197, 197, 197),
                    text: color!(20, 20, 20),
                },
            },
            primary: Triplet::generate(Pair {
                color: color!(140, 3, 39),
                text: color!(237, 208, 208),
            }),
            secondary: Triplet::generate(Pair {
                color: color!(216, 82, 81),
                text: color!(17, 2, 2),
            }),
            accent: Triplet::generate(Pair {
                color: color!(213, 155, 107),
                text: color!(16, 9, 4),
            }),
            neutral: Triplet::generate(Pair {
                color: color!(130, 106, 92),
                text: color!(229, 224, 221),
            }),
            info: Triplet::generate(Pair {
                color: color!(68, 173, 187),
                text: color!(2, 11, 13),
            }),
            success: Triplet::generate(Pair {
                color: color!(73, 147, 128),
                text: color!(2, 8, 6),
            }),
            warning: Triplet::generate(Pair {
                color: color!(233, 127, 22),
                text: color!(19, 6, 0),
            }),
            danger: Triplet::generate(Pair {
                color: color!(255, 212, 209),
                text: color!(212, 0, 20),
            }),
            radii: Radii {
                boxes: 16.0,
                fields: 8.0,
                selectors: 16.0,
            },
            is_dark: false,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Pair {
    pub color: Color,
    pub text: Color,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Triplet {
    pub weak: Pair,
    pub base: Pair,
    pub strong: Pair,
}

impl Triplet {
    fn generate(base: Pair) -> Self {
        use theme::palette::{darken, lighten};

        Self {
            weak: Pair {
                color: lighten(base.color, 0.025),
                text: base.text,
            },
            base,
            strong: Pair {
                color: darken(base.color, 0.025),
                text: base.text,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Schema {
    pub background: Triplet,
    pub primary: Triplet,
    pub secondary: Triplet,
    pub accent: Triplet,
    pub neutral: Triplet,
    pub info: Triplet,
    pub success: Triplet,
    pub warning: Triplet,
    pub danger: Triplet,
    pub is_dark: bool,
    pub radii: Radii,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Radii {
    /// Cards, modals, alerts, Containers
    boxes: f32,
    /// Buttons, inputs, tabs
    fields: f32,
    /// Checkbox, toggle, badge
    selectors: f32,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub enum Theme {
    #[default]
    Autumn,
    Luxury,
    Ember,
    Bumblebee,
    Fantasy,
    Black,
    Abyss,
    Custom(Arc<Custom>),
}

impl Theme {
    pub const ALL: &[Self] = &[
        Self::Abyss,
        Self::Black,
        Self::Fantasy,
        Self::Bumblebee,
        Self::Ember,
        Self::Luxury,
        Self::Autumn,
    ];

    pub fn schema(&self) -> &Schema {
        match self {
            Self::Abyss => &variants::ABYSS,
            Self::Black => &variants::BLACK,
            Self::Fantasy => &variants::FANTASY,
            Self::Bumblebee => &variants::BUMBLEBEE,
            Self::Ember => &variants::EMBER,
            Self::Luxury => &variants::LUXURY,
            Self::Autumn => &variants::AUTUMN,
            Self::Custom(custom) => &custom.schema,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Self::Abyss => "Abyss",
            Self::Black => "Black",
            Self::Fantasy => "Fantasy",
            Self::Bumblebee => "Bumblebee",
            Self::Ember => "Ember",
            Self::Luxury => "Luxury",
            Self::Autumn => "Autumn",
            Self::Custom(custom) => &custom.name,
        }
    }

    pub fn from_custom(custom: Custom) -> Self {
        custom.into()
    }
}

impl theme::Base for Theme {
    fn default(preference: theme::Mode) -> Self {
        match preference {
            theme::Mode::Light => Self::Bumblebee,
            _ => Self::Ember,
        }
    }

    fn mode(&self) -> theme::Mode {
        if self.schema().is_dark {
            theme::Mode::Dark
        } else {
            theme::Mode::Light
        }
    }

    fn base(&self) -> theme::Style {
        let base = self.schema().background.base;

        theme::Style {
            background_color: base.color,
            text_color: base.text,
        }
    }

    fn seed(&self) -> Option<theme::palette::Seed> {
        let schema = self.schema();

        Some(theme::palette::Seed {
            background: schema.background.base.color,
            text: schema.background.base.text,
            primary: schema.primary.base.color,
            success: schema.primary.base.color,
            warning: schema.warning.base.color,
            danger: schema.danger.base.color,
        })
    }

    fn name(&self) -> &str {
        self.name()
    }
}

impl std::fmt::Display for Theme {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name())
    }
}

#[derive(Debug, PartialEq)]
pub struct Custom {
    pub name: Cow<'static, str>,
    pub schema: Schema,
}

impl From<Custom> for Theme {
    fn from(value: Custom) -> Self {
        Theme::Custom(Arc::new(value))
    }
}

pub mod container {
    use super::*;
    use widget::container::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(transparent)
        }

        fn style(&self, class: &Self::Class<'_>) -> Style {
            class(self)
        }
    }

    fn style(radii: &Radii, pair: Pair) -> Style {
        Style {
            background: Some(pair.color.into()),
            text_color: Some(pair.text),
            border: border::rounded(radii.boxes),
            ..Style::default()
        }
    }

    /// A transparent [`Container`].
    pub fn transparent<Theme>(_theme: &Theme) -> Style {
        Style::default()
    }

    /// A [`Container`] with the given [`Background`].
    pub fn background(background: impl Into<Background>) -> Style {
        Style::default().background(background)
    }

    /// A rounded [`Container`] with a weak background.
    pub fn rounded_box(theme: &Theme) -> Style {
        let schema = theme.schema();
        let pair = schema.background.weak;

        Style {
            background: Some(pair.color.into()),
            text_color: Some(pair.text),
            border: border::rounded(schema.radii.boxes),
            ..Style::default()
        }
    }

    /// A bordered [`Container`] with a background.
    pub fn bordered_box(theme: &Theme) -> Style {
        let schema = theme.schema();
        let weak = schema.background.weak;
        let border = schema.background.base;

        Style {
            background: Some(weak.color.into()),
            text_color: Some(weak.text),
            border: Border {
                width: 1.,
                radius: schema.radii.boxes.into(),
                color: border.color,
            },
            ..Style::default()
        }
    }

    /// A [`Container`] with a dark background and white text.
    pub fn dark(theme: &Theme) -> Style {
        style(
            &theme.schema().radii,
            Pair {
                color: color!(0x111111),
                text: Color::WHITE,
            },
        )
    }

    /// A [`Container`] with a primary background color.
    pub fn primary(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.primary.base)
    }

    /// A [`Container`] with a secondary background color.
    pub fn secondary(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.secondary.base)
    }

    /// A [`Container`] with a accent background color.
    pub fn accent(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.accent.base)
    }

    /// A [`Container`] with a neutral background color.
    pub fn neutral(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.neutral.base)
    }

    /// A [`Container`] with a info background color.
    pub fn info(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.info.base)
    }

    /// A [`Container`] with a success background color.
    pub fn success(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.success.base)
    }

    /// A [`Container`] with a warning background color.
    pub fn warning(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.warning.base)
    }

    /// A [`Container`] with a danger background color.
    pub fn danger(theme: &Theme) -> Style {
        let schema = theme.schema();
        style(&schema.radii, schema.danger.base)
    }
}

pub mod text {
    use super::*;
    use widget::text::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Theme>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(|_theme| Style::default())
        }

        fn style(&self, item: &Self::Class<'_>) -> Style {
            item(self)
        }
    }

    /// The default text styling; color is inherited.
    pub fn default(_theme: &Theme) -> Style {
        Style { color: None }
    }

    fn style(pair: Pair) -> Style {
        Style {
            color: Some(pair.text),
        }
    }

    /// Text with the default base color.
    pub fn base(theme: &Theme) -> Style {
        style(theme.schema().background.base)
    }

    /// Text with the primary color.
    pub fn primary(theme: &Theme) -> Style {
        style(theme.schema().primary.base)
    }

    /// Text with the secondary color.
    pub fn secondary(theme: &Theme) -> Style {
        style(theme.schema().secondary.base)
    }

    /// Text with the accent color.
    pub fn accent(theme: &Theme) -> Style {
        style(theme.schema().accent.base)
    }

    /// Text with the neutral color.
    pub fn neutral(theme: &Theme) -> Style {
        style(theme.schema().neutral.base)
    }

    /// Text with the info color.
    pub fn info(theme: &Theme) -> Style {
        style(theme.schema().info.base)
    }

    /// Text with the success color.
    pub fn success(theme: &Theme) -> Style {
        style(theme.schema().success.base)
    }

    /// Text with the warning color.
    pub fn warning(theme: &Theme) -> Style {
        style(theme.schema().warning.base)
    }

    /// Text with the danger color.
    pub fn danger(theme: &Theme) -> Style {
        style(theme.schema().danger.base)
    }
}

pub mod button {
    use super::*;
    use widget::button::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Theme>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(primary)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// A background button; denoting a main action.
    pub fn background(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.background;
        styled(schema.radii, triplet, status)
    }

    /// A primary button; denoting a main action.
    pub fn primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.primary;
        styled(schema.radii, triplet, status)
    }

    /// A secondary button; denoting a main action.
    pub fn secondary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.secondary;
        styled(schema.radii, triplet, status)
    }

    /// A accent button; denoting a main action.
    pub fn accent(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.accent;
        styled(schema.radii, triplet, status)
    }

    /// A neutral button; denoting a main action.
    pub fn neutral(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.neutral;
        styled(schema.radii, triplet, status)
    }

    /// A info button; denoting a main action.
    pub fn info(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.info;
        styled(schema.radii, triplet, status)
    }

    /// A success button; denoting a main action.
    pub fn success(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.success;
        styled(schema.radii, triplet, status)
    }

    /// A warning button; denoting a main action.
    pub fn warning(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.warning;
        styled(schema.radii, triplet, status)
    }

    /// A danger button; denoting a main action.
    pub fn danger(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = schema.danger;
        styled(schema.radii, triplet, status)
    }

    /// A text button
    pub fn text(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let base = Style {
            text_color: schema.background.base.text,
            border: border::rounded(schema.radii.fields),
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: schema.background.base.text.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtle(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let pair = schema.background.weak;

        let base = Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            border: border::rounded(schema.radii.fields),
            ..Style::default()
        };

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(schema.background.strong.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(schema.background.base.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    fn styled(radii: Radii, triplet: Triplet, status: Status) -> Style {
        let style = |pair: Pair| Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            border: border::rounded(radii.fields),
            ..Style::default()
        };

        match status {
            Status::Active => style(triplet.base),
            Status::Hovered => style(triplet.strong),
            Status::Pressed => style(triplet.weak),
            Status::Disabled => disabled(style(triplet.base)),
        }
    }

    fn disabled(style: Style) -> Style {
        Style {
            background: style
                .background
                .map(|background| background.scale_alpha(0.5)),
            text_color: style.text_color.scale_alpha(0.8),
            ..style
        }
    }
}

pub mod checkbox {
    use super::*;
    use widget::checkbox::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(primary)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// A primary checkbox; denoting a main toggle.
    pub fn primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.primary, schema.background, status)
    }

    /// A secondary checkbox; denoting a main toggle.
    pub fn secondary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.secondary, schema.background, status)
    }

    /// A accent checkbox; denoting a main toggle.
    pub fn accent(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.accent, schema.background, status)
    }

    /// A neutral checkbox; denoting a main toggle.
    pub fn neutral(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.neutral, schema.background, status)
    }

    /// A info checkbox; denoting a main toggle.
    pub fn info(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.info, schema.background, status)
    }

    /// A success checkbox; denoting a main toggle.
    pub fn success(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.success, schema.background, status)
    }

    /// A warning checkbox; denoting a main toggle.
    pub fn warning(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.warning, schema.background, status)
    }

    /// A danger checkbox; denoting a main toggle.
    pub fn danger(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        styled(schema.radii, schema.danger, schema.background, status)
    }

    fn styled(radii: Radii, base: Triplet, background: Triplet, status: Status) -> Style {
        let style = |border: Color, pair: Pair| Style {
            background: Background::Color(pair.color),
            icon_color: pair.text,
            border: Border {
                radius: radii.selectors.into(),
                width: 1.0,
                color: border,
            },
            text_color: None,
        };

        let disabled = |style: Style| Style {
            background: style.background.scale_alpha(0.5),
            icon_color: style.icon_color.scale_alpha(0.8),
            border: style.border.color(style.border.color.scale_alpha(0.5)),
            text_color: style.text_color.map(|color| color.scale_alpha(0.8)),
        };

        match status {
            Status::Active { is_checked } => {
                let border = base.base.color;
                let base = if is_checked {
                    base.base
                } else {
                    background.base
                };

                style(border, base)
            }
            Status::Hovered { is_checked } => {
                let border = base.weak.color;
                let base = if is_checked {
                    base.strong
                } else {
                    background.weak
                };

                style(border, base)
            }
            Status::Disabled { is_checked } => {
                let border = base.base.color;
                let base = if is_checked {
                    base.base
                } else {
                    background.base
                };

                disabled(style(border, base))
            }
        }
    }
}

pub mod text_input {
    use super::*;
    use widget::text_input::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    pub fn required(invalid: bool) -> impl Fn(&Theme, Status) -> Style {
        move |theme: &Theme, status| {
            let danger = theme.schema().danger.strong.color;
            let default = default(theme, status);
            let border = default.border;
            let border = if invalid && matches!(status, text_input::Status::Focused { .. }) {
                border.color(danger)
            } else {
                border
            };

            text_input::Style { border, ..default }
        }
    }

    /// The default style of a [`TextInput`].
    pub fn default(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        let active = Style {
            background: Background::Color(schema.background.base.color),
            border: Border {
                radius: schema.radii.fields.into(),
                width: 1.0,
                color: schema.accent.base.color,
            },
            icon: schema.background.weak.text,
            placeholder: schema.neutral.weak.color,
            value: schema.background.base.text,
            selection: schema.primary.weak.color.scale_alpha(0.5),
        };

        match status {
            Status::Active => active,
            Status::Hovered => Style {
                border: Border {
                    color: schema.background.base.text,
                    ..active.border
                },
                ..active
            },
            Status::Focused { .. } => Style {
                border: Border {
                    color: schema.primary.strong.color,
                    ..active.border
                },
                ..active
            },
            Status::Disabled => Style {
                background: Background::Color(schema.background.weak.color),
                value: schema.neutral.weak.text.scale_alpha(0.8),
                placeholder: schema.neutral.weak.color.scale_alpha(0.8),
                ..active
            },
        }
    }
}

pub mod scrollable {
    use super::*;
    use widget::scrollable::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// The default style of a [`Scrollable`].
    pub fn default(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        let scrollbar = Rail {
            background: Some(schema.background.weak.color.into()),
            border: border::rounded(schema.radii.boxes),
            scroller: Scroller {
                background: schema.neutral.base.color.into(),
                border: border::rounded(schema.radii.selectors),
            },
        };

        let auto_scroll = AutoScroll {
            background: schema.background.base.color.scale_alpha(0.9).into(),
            border: border::rounded(u32::MAX)
                .width(1)
                .color(schema.background.base.text.scale_alpha(0.8)),
            shadow: Shadow {
                color: Color::BLACK.scale_alpha(0.7),
                offset: Vector::ZERO,
                blur_radius: 2.0,
            },
            icon: schema.background.base.text.scale_alpha(0.8),
        };

        match status {
            Status::Active { .. } => Style {
                container: widget::container::Style::default(),
                vertical_rail: scrollbar,
                horizontal_rail: scrollbar,
                gap: None,
                auto_scroll,
            },
            Status::Hovered {
                is_horizontal_scrollbar_hovered,
                is_vertical_scrollbar_hovered,
                ..
            } => {
                let hovered_scrollbar = Rail {
                    scroller: Scroller {
                        background: schema.primary.base.color.into(),
                        ..scrollbar.scroller
                    },
                    ..scrollbar
                };

                Style {
                    container: widget::container::Style::default(),
                    vertical_rail: if is_vertical_scrollbar_hovered {
                        hovered_scrollbar
                    } else {
                        scrollbar
                    },
                    horizontal_rail: if is_horizontal_scrollbar_hovered {
                        hovered_scrollbar
                    } else {
                        scrollbar
                    },
                    gap: None,
                    auto_scroll,
                }
            }
            Status::Dragged {
                is_horizontal_scrollbar_dragged,
                is_vertical_scrollbar_dragged,
                ..
            } => {
                let dragged_scrollbar = Rail {
                    scroller: Scroller {
                        background: schema.primary.strong.color.into(),
                        ..scrollbar.scroller
                    },
                    ..scrollbar
                };

                Style {
                    container: widget::container::Style::default(),
                    vertical_rail: if is_vertical_scrollbar_dragged {
                        dragged_scrollbar
                    } else {
                        scrollbar
                    },
                    horizontal_rail: if is_horizontal_scrollbar_dragged {
                        dragged_scrollbar
                    } else {
                        scrollbar
                    },
                    gap: None,
                    auto_scroll,
                }
            }
        }
    }
}

pub mod menu {
    use super::*;
    use iced::overlay::menu::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> <Self as Catalog>::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &<Self as Catalog>::Class<'_>) -> Style {
            class(self)
        }
    }

    /// The default style of a [`Menu`].
    pub fn default(theme: &Theme) -> Style {
        let schema = theme.schema();

        Style {
            background: schema.background.weak.color.into(),
            border: Border {
                width: 1.0,
                radius: schema.radii.selectors.into(),
                color: schema.background.strong.color,
            },
            text_color: schema.background.weak.text,
            selected_text_color: schema.primary.strong.text,
            selected_background: schema.primary.strong.color.into(),
            shadow: Shadow::default(),
        }
    }
}

pub mod combo_box {
    use super::*;
    use widget::combo_box::*;

    impl Catalog for Theme {}
}

pub mod float {
    use super::*;
    use widget::float::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(|_| Style::default())
        }

        fn style(&self, class: &Self::Class<'_>) -> Style {
            class(self)
        }
    }
}

pub mod rule {
    use super::*;
    use widget::rule::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>) -> Style {
            class(self)
        }
    }

    /// The default styling of a [`Rule`].
    pub fn default(theme: &Theme) -> Style {
        let schema = theme.schema();

        Style {
            color: schema.background.strong.color,
            radius: 0.0.into(),
            fill_mode: FillMode::Full,
            snap: true,
        }
    }

    /// A [`Rule`] styling using the weak background color.
    pub fn weak(theme: &Theme) -> Style {
        let schema = theme.schema();

        Style {
            color: schema.background.weak.color,
            radius: 0.0.into(),
            fill_mode: FillMode::Full,
            snap: true,
        }
    }
}

pub mod table {
    use super::*;
    use widget::table::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>) -> Style {
            class(self)
        }
    }

    pub fn default(theme: &Theme) -> Style {
        let schema = theme.schema();
        let separator = schema.neutral.strong.color.into();

        Style {
            separator_x: separator,
            separator_y: separator,
        }
    }
}

pub mod markdown {
    use super::*;
    use widget::markdown::*;

    impl Catalog for Theme {
        fn code_block<'a>() -> <Self as widget::container::Catalog>::Class<'a> {
            Box::new(container::neutral)
        }
    }
}

pub mod pick_list {
    use super::*;
    use widget::pick_list::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> <Self as Catalog>::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &<Self as Catalog>::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    pub fn default(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        let active = Style {
            text_color: schema.background.weak.text,
            background: schema.background.weak.color.into(),
            placeholder_color: schema.secondary.base.color,
            handle_color: schema.background.weak.text,
            border: Border {
                radius: schema.radii.selectors.into(),
                width: 1.0,
                color: schema.background.strong.color,
            },
        };

        match status {
            Status::Active => active,
            Status::Hovered | Status::Opened { .. } => Style {
                border: Border {
                    color: schema.primary.strong.color,
                    ..active.border
                },
                ..active
            },
            Status::Disabled => Style {
                text_color: schema.neutral.weak.text.scale_alpha(0.8),
                background: Background::Color(schema.neutral.weak.color),
                placeholder_color: schema.neutral.strong.text.scale_alpha(0.8),
                handle_color: schema.neutral.strong.text.scale_alpha(0.8),
                border: Border {
                    color: schema.background.weak.color,
                    ..active.border
                },
            },
        }
    }
}

pub mod progress_bar {
    use super::*;
    use widget::progress_bar::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(primary)
        }

        fn style(&self, class: &Self::Class<'_>) -> Style {
            class(self)
        }
    }

    /// The primary style of a [`ProgressBar`].
    pub fn primary(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.primary.base.color, schema.radii)
    }

    /// The secondary style of a [`ProgressBar`].
    pub fn secondary(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.secondary.base.color, schema.radii)
    }

    /// The accent style of a [`ProgressBar`].
    pub fn accent(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.accent.base.color, schema.radii)
    }

    /// The neutral style of a [`ProgressBar`].
    pub fn neutral(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.neutral.base.color, schema.radii)
    }

    /// The info style of a [`ProgressBar`].
    pub fn info(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.info.base.color, schema.radii)
    }

    /// The success style of a [`ProgressBar`].
    pub fn success(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.success.base.color, schema.radii)
    }

    /// The warning style of a [`ProgressBar`].
    pub fn warning(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.warning.base.color, schema.radii)
    }

    /// The danger style of a [`ProgressBar`].
    pub fn danger(theme: &Theme) -> Style {
        let schema = theme.schema();

        styled(schema.danger.base.color, schema.radii)
    }

    fn styled(bar: Color, radii: Radii) -> Style {
        Style {
            background: bar.scale_alpha(0.3).into(),
            bar: bar.into(),
            border: border::rounded(radii.boxes),
        }
    }
}

pub mod radio {
    use super::*;
    use widget::radio::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(primary)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    pub fn primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.primary, status)
    }

    pub fn secondary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.secondary, status)
    }

    pub fn accent(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.accent, status)
    }

    pub fn neutral(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.neutral, status)
    }

    pub fn info(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.info, status)
    }

    pub fn success(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.success, status)
    }

    pub fn warning(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.warning, status)
    }

    pub fn danger(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.danger, status)
    }

    fn styled(base: Triplet, status: Status) -> Style {
        let active = Style {
            background: Color::TRANSPARENT.into(),
            dot_color: base.strong.color,
            border_width: 1.0,
            border_color: base.strong.color,
            text_color: None,
        };

        match status {
            Status::Active { .. } => active,
            Status::Hovered { .. } => Style {
                dot_color: base.strong.color,
                background: base.weak.color.into(),
                ..active
            },
        }
    }
}

pub mod slider {
    use super::*;
    use widget::slider::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(primary)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    pub fn primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(
            schema.primary,
            schema.background.strong.color,
            schema.radii,
            status,
        )
    }

    pub fn secondary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(
            schema.secondary,
            schema.background.strong.color,
            schema.radii,
            status,
        )
    }

    pub fn accent(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(
            schema.accent,
            schema.background.strong.color,
            schema.radii,
            status,
        )
    }

    pub fn neutral(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(
            schema.neutral,
            schema.background.strong.color,
            schema.radii,
            status,
        )
    }

    fn styled(
        base: Triplet,
        background: impl Into<Background>,
        radii: Radii,
        status: Status,
    ) -> Style {
        let color = match status {
            Status::Active => base.base.color,
            Status::Hovered => base.strong.color,
            Status::Dragged => base.weak.color,
        };

        Style {
            rail: Rail {
                backgrounds: (color.into(), background.into()),
                width: 4.0,
                border: Border {
                    radius: radii.boxes.into(),
                    width: 0.0,
                    color: Color::TRANSPARENT,
                },
            },
            handle: Handle {
                // shape: HandleShape::Circle { radius: radii.selectors },
                shape: HandleShape::Rectangle {
                    width: 12,
                    border_radius: radii.selectors.into(),
                },
                background: color.into(),
                border_color: Color::TRANSPARENT,
                border_width: 0.0,
            },
        }
    }
}

pub mod text_editor {
    use super::*;
    use widget::text_editor::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// The default style of a [`TextEditor`].
    pub fn default(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        let active = Style {
            background: Background::Color(schema.background.base.color),
            border: Border {
                radius: schema.radii.fields.into(),
                width: 1.0,
                color: schema.accent.base.color,
            },
            placeholder: schema.neutral.weak.color,
            value: schema.background.base.text,
            selection: schema.primary.weak.color.scale_alpha(0.5),
        };

        match status {
            Status::Active => active,
            Status::Hovered => Style {
                border: Border {
                    color: schema.background.base.text,
                    ..active.border
                },
                ..active
            },
            Status::Focused { .. } => Style {
                border: Border {
                    color: schema.primary.strong.color,
                    ..active.border
                },
                ..active
            },
            Status::Disabled => Style {
                background: Background::Color(schema.background.weak.color),
                value: schema.neutral.weak.text.scale_alpha(0.8),
                placeholder: schema.neutral.weak.color.scale_alpha(0.8),
                ..active
            },
        }
    }
}

pub mod toggler {
    use super::*;
    use widget::toggler::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(primary)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// The primary style of a [`Toggler`].
    pub fn primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.primary, schema.background, schema.radii, status)
    }

    /// The secondary style of a [`Toggler`].
    pub fn secondary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.secondary, schema.background, schema.radii, status)
    }

    /// The accent style of a [`Toggler`].
    pub fn accent(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.accent, schema.background, schema.radii, status)
    }

    /// The neutral style of a [`Toggler`].
    pub fn neutral(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        styled(schema.neutral, schema.background, schema.radii, status)
    }

    fn styled(base: Triplet, background: Triplet, radii: Radii, status: Status) -> Style {
        let bg = match status {
            Status::Active { is_toggled } | Status::Hovered { is_toggled } => {
                if is_toggled {
                    base.base.color
                } else {
                    background.strong.color
                }
            }
            Status::Disabled { .. } => background.weak.color,
        };

        let foreground = match status {
            Status::Active { is_toggled } => {
                if is_toggled {
                    base.base.text
                } else {
                    background.base.color
                }
            }
            Status::Hovered { is_toggled } => {
                if is_toggled {
                    Color {
                        a: 0.5,
                        ..base.base.text
                    }
                } else {
                    background.weak.color
                }
            }
            Status::Disabled { .. } => background.base.color.scale_alpha(0.5),
        };

        Style {
            background: bg.into(),
            foreground: foreground.into(),
            foreground_border_width: 0.0,
            foreground_border_color: Color::TRANSPARENT,
            background_border_width: 0.0,
            background_border_color: Color::TRANSPARENT,
            text_color: None,
            border_radius: Some(radii.selectors.into()),
            padding_ratio: 0.1,
        }
    }
}
