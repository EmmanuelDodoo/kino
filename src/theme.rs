use iced::{color, color::Color, theme};
use serde::{Deserialize, Serialize, de, ser};

use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

mod widget;

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

fn se_color<S: ser::Serializer>(color: &Color, s: S) -> Result<S::Ok, S::Error> {
    use ser::Serializer;
    let color = color.to_string();

    s.serialize_str(&color)
}

fn de_color<'de, D: de::Deserializer<'de>>(d: D) -> Result<Color, D::Error> {
    use de::{Deserializer, Visitor};

    struct ColorVisitor;

    impl<'de> Visitor<'de> for ColorVisitor {
        type Value = Color;

        fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
            write!(formatter, "a valid Color string")
        }

        fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
        where
            E: de::Error,
        {
            use de::Error;

            v.parse::<Color>().map_err(|error| de::Error::custom(error))
        }
    }

    d.deserialize_str(ColorVisitor)
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Pair {
    #[serde(serialize_with = "se_color", deserialize_with = "de_color")]
    pub color: Color,
    #[serde(serialize_with = "se_color", deserialize_with = "de_color")]
    pub text: Color,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Radii {
    /// Cards, modals, alerts, Containers
    pub boxes: f32,
    /// Buttons, inputs, tabs
    pub fields: f32,
    /// Checkbox, toggle, badge
    pub selectors: f32,
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Theme {
    #[default]
    Autumn,
    Luxury,
    Ember,
    Bumblebee,
    Fantasy,
    Black,
    Abyss,
    #[serde(serialize_with = "se_custom", deserialize_with = "de_custom")]
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Custom {
    pub name: Cow<'static, str>,
    pub schema: Schema,
}

impl From<Custom> for Theme {
    fn from(value: Custom) -> Self {
        Theme::Custom(Arc::new(value))
    }
}

fn se_custom<S: ser::Serializer>(field: &Arc<Custom>, s: S) -> Result<S::Ok, S::Error> {
    let custom = field.as_ref();

    custom.serialize(s)
}

fn de_custom<'de, D: de::Deserializer<'de>>(d: D) -> Result<Arc<Custom>, D::Error> {
    let custom = Arc::new(Custom::deserialize(d)?);

    Ok(custom)
}
