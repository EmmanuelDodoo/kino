use iced::{color, color::Color, theme};
use serde::{Deserialize, Serialize, de, ser};

use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

pub mod styles;
mod variants;

pub use styles::{
    button, checkbox, combo_box, container, float, markdown, menu, modal, pagination, pick_list,
    progress_bar, radio, rule, scrollable, slider, table, text, text_editor, text_input, throbber,
    toast, toggler,
};
use variants::*;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Variant {
    Weak,
    Base,
    Strong,
}

impl Variant {
    pub fn pair(self, trip: &Triplet) -> Pair {
        match self {
            Self::Weak => trip.weak,
            Self::Base => trip.base,
            Self::Strong => trip.strong,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum Palette {
    Background,
    Primary,
    Secondary,
    Accent,
    Neutral,
    Info,
    Success,
    Warning,
    Danger,
}

impl Palette {
    pub fn triplet(self, schema: &Schema) -> Triplet {
        match self {
            Self::Background => schema.background,
            Self::Primary => schema.primary,
            Self::Secondary => schema.secondary,
            Self::Accent => schema.accent,
            Self::Neutral => schema.neutral,
            Self::Info => schema.info,
            Self::Success => schema.success,
            Self::Warning => schema.warning,
            Self::Danger => schema.danger,
        }
    }
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

    pub fn variant(&self, variant: Variant) -> Pair {
        variant.pair(self)
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
    Autumn,
    Luxury,
    Ember,
    Bumblebee,
    Fantasy,
    Black,
    #[default]
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
