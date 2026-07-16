use crate::config::{de_color, se_color};
use iced::{animation::Interpolable, color, color::Color, theme};
use serde::{Deserialize, Serialize, de, ser};
use std::{
    borrow::Cow,
    sync::{Arc, LazyLock},
};

pub mod styles;
mod variants;

pub use styles::*;
use variants::*;

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

impl Interpolable for Pair {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        Self {
            color: self.color.interpolated(other.color, ratio),
            text: self.text.interpolated(other.color, ratio),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct Triplet {
    pub weak: Pair,
    pub base: Pair,
    pub strong: Pair,
}

impl Triplet {
    pub fn generate(base: Pair) -> Self {
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

impl Interpolable for Triplet {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        Self {
            weak: self.weak.interpolated(other.weak, ratio),
            base: self.base.interpolated(other.base, ratio),
            strong: self.strong.interpolated(other.strong, ratio),
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

impl Interpolable for Schema {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        let is_dark = if ratio >= 0.5 {
            other.is_dark
        } else {
            self.is_dark
        };

        Self {
            background: self.background.interpolated(other.background, ratio),
            primary: self.primary.interpolated(other.primary, ratio),
            secondary: self.secondary.interpolated(other.secondary, ratio),
            accent: self.accent.interpolated(other.accent, ratio),
            neutral: self.neutral.interpolated(other.neutral, ratio),
            info: self.info.interpolated(other.info, ratio),
            success: self.success.interpolated(other.success, ratio),
            warning: self.warning.interpolated(other.warning, ratio),
            danger: self.danger.interpolated(other.danger, ratio),
            is_dark,
            radii: self.radii.interpolated(other.radii, ratio),
        }
    }
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

impl Interpolable for Radii {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        Self {
            boxes: self.boxes.interpolated(other.boxes, ratio),
            fields: self.fields.interpolated(other.fields, ratio),
            selectors: self.selectors.interpolated(other.selectors, ratio),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub enum Theme {
    Autumn,
    Luxury,
    Ember,
    Winter,
    Fantasy,
    Black,
    #[default]
    Abyss,
    #[serde(
        serialize_with = "se_custom",
        deserialize_with = "de_custom",
        rename = "custom"
    )]
    Custom(Arc<Custom>),
}

impl Theme {
    pub const DEFAULTS: &[Self] = &[
        Self::Abyss,
        Self::Autumn,
        Self::Black,
        Self::Ember,
        Self::Fantasy,
        Self::Luxury,
        Self::Winter,
    ];

    pub fn schema(&self) -> &Schema {
        match self {
            Self::Abyss => &variants::ABYSS,
            Self::Black => &variants::BLACK,
            Self::Fantasy => &variants::FANTASY,
            Self::Winter => &variants::WINTER,
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
            Self::Winter => "Winter",
            Self::Ember => "Ember",
            Self::Luxury => "Luxury",
            Self::Autumn => "Autumn",
            Self::Custom(custom) => &custom.name,
        }
    }

    pub fn from_custom(custom: Custom) -> Self {
        custom.into()
    }

    pub fn is_custom(&self) -> bool {
        matches!(self, Self::Custom(_))
    }

    pub fn all(&self) -> Vec<Self> {
        if self.is_custom() {
            let mut values = vec![self.clone()];

            values.extend_from_slice(Self::DEFAULTS);

            values
        } else {
            Self::DEFAULTS.to_vec()
        }
    }
}

impl theme::Base for Theme {
    fn default(preference: theme::Mode) -> Self {
        match preference {
            theme::Mode::Light => Self::Winter,
            theme::Mode::Dark => Self::Ember,
            theme::Mode::None => Self::Abyss,
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

impl Interpolable for Theme {
    fn interpolated(&self, other: Self, ratio: f32) -> Self {
        let schema = self.schema().interpolated(*other.schema(), ratio);

        Self::Custom(Arc::new(Custom {
            name: Cow::Borrowed("Custom Animated Theme"),
            schema,
        }))
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Custom {
    pub name: Cow<'static, str>,
    #[serde(flatten)]
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
