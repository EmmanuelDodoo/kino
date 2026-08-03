use super::{Pair, Palette, Radii, Theme, Triplet, Variant};
use iced::{
    Background, Color, Shadow, Vector,
    border::{self, Border},
    color, widget,
};

const SLATE: Color = Color::from_rgb8(144, 161, 185);
#[allow(dead_code)]
const SLATE2: Color = Color::from_rgb8(202, 213, 226);

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

    fn styled(theme: &Theme, palette: Palette, variant: Variant) -> Style {
        let schema = theme.schema();
        let pair = palette.triplet(schema).variant(variant);

        Style {
            background: Some(pair.color.into()),
            text_color: Some(pair.text),
            border: border::rounded(schema.radii.boxes),
            ..Style::default()
        }
    }

    pub fn sw(theme: &Theme) -> Style {
        styled(theme, Palette::Secondary, Variant::Weak)
    }

    pub fn sb(theme: &Theme) -> Style {
        styled(theme, Palette::Secondary, Variant::Base)
    }

    pub fn ss(theme: &Theme) -> Style {
        styled(theme, Palette::Secondary, Variant::Strong)
    }

    pub fn pw(theme: &Theme) -> Style {
        styled(theme, Palette::Primary, Variant::Weak)
    }

    pub fn pb(theme: &Theme) -> Style {
        styled(theme, Palette::Primary, Variant::Base)
    }

    pub fn ps(theme: &Theme) -> Style {
        styled(theme, Palette::Primary, Variant::Strong)
    }

    pub fn bw(theme: &Theme) -> Style {
        styled(theme, Palette::Background, Variant::Weak)
    }

    pub fn bb(theme: &Theme) -> Style {
        styled(theme, Palette::Background, Variant::Base)
    }

    pub fn bs(theme: &Theme) -> Style {
        styled(theme, Palette::Background, Variant::Strong)
    }

    pub fn anw(theme: &Theme) -> Style {
        styled(theme, Palette::Accent, Variant::Weak)
    }

    pub fn anb(theme: &Theme) -> Style {
        styled(theme, Palette::Accent, Variant::Base)
    }

    pub fn ans(theme: &Theme) -> Style {
        styled(theme, Palette::Accent, Variant::Strong)
    }

    pub fn nw(theme: &Theme) -> Style {
        styled(theme, Palette::Neutral, Variant::Weak)
    }

    pub fn nb(theme: &Theme) -> Style {
        styled(theme, Palette::Neutral, Variant::Base)
    }

    pub fn ns(theme: &Theme) -> Style {
        styled(theme, Palette::Neutral, Variant::Strong)
    }

    pub fn iw(theme: &Theme) -> Style {
        styled(theme, Palette::Info, Variant::Weak)
    }

    pub fn ib(theme: &Theme) -> Style {
        styled(theme, Palette::Info, Variant::Base)
    }

    pub fn is(theme: &Theme) -> Style {
        styled(theme, Palette::Info, Variant::Strong)
    }

    pub fn suw(theme: &Theme) -> Style {
        styled(theme, Palette::Success, Variant::Weak)
    }

    pub fn sub(theme: &Theme) -> Style {
        styled(theme, Palette::Success, Variant::Base)
    }

    pub fn sus(theme: &Theme) -> Style {
        styled(theme, Palette::Success, Variant::Strong)
    }

    pub fn ww(theme: &Theme) -> Style {
        styled(theme, Palette::Warning, Variant::Weak)
    }

    pub fn wb(theme: &Theme) -> Style {
        styled(theme, Palette::Warning, Variant::Base)
    }

    pub fn ws(theme: &Theme) -> Style {
        styled(theme, Palette::Warning, Variant::Strong)
    }

    pub fn dw(theme: &Theme) -> Style {
        styled(theme, Palette::Danger, Variant::Weak)
    }

    pub fn db(theme: &Theme) -> Style {
        styled(theme, Palette::Danger, Variant::Base)
    }

    pub fn ds(theme: &Theme) -> Style {
        styled(theme, Palette::Danger, Variant::Strong)
    }

    pub fn bordered(theme: &Theme) -> Style {
        let schema = theme.schema();

        Style {
            background: Some(schema.background.base.color.into()),
            text_color: Some(schema.background.base.text),
            border: border::rounded(schema.radii.boxes)
                .width(1.0)
                .color(schema.background.weak.color),
            ..Style::default()
        }
    }

    pub fn text(theme: &Theme) -> Style {
        let schema = theme.schema();
        let text_color = SLATE;

        Style {
            text_color: Some(text_color),
            background: None,
            border: border::rounded(schema.radii.boxes),
            ..Default::default()
        }
    }

    pub fn text_pb(theme: &Theme) -> Style {
        let schema = theme.schema();
        let text_color = schema.primary.base.color;

        Style {
            text_color: Some(text_color),
            background: None,
            border: border::rounded(schema.radii.boxes),
            ..Default::default()
        }
    }

    pub fn text_ps(theme: &Theme) -> Style {
        let schema = theme.schema();
        let text_color = schema.primary.strong.color;

        Style {
            text_color: Some(text_color),
            background: None,
            border: border::rounded(schema.radii.boxes),
            ..Default::default()
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
        Style {
            background: Some(color!(0x111111).into()),
            text_color: Some(Color::WHITE),
            border: border::rounded(theme.schema().radii.boxes),
            ..Style::default()
        }
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
            color: Some(pair.color),
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

    pub fn ps(theme: &Theme) -> Style {
        style(theme.schema().primary.strong)
    }

    pub fn pw(theme: &Theme) -> Style {
        style(theme.schema().primary.weak)
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
        styled(theme, Palette::Background, status)
    }

    /// A primary button; denoting a main action.
    pub fn primary(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Primary, status)
    }

    /// A secondary button; denoting a main action.
    pub fn secondary(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Secondary, status)
    }

    /// A accent button; denoting a main action.
    pub fn accent(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Accent, status)
    }

    /// A neutral button; denoting a main action.
    pub fn neutral(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Neutral, status)
    }

    /// A info button; denoting a main action.
    pub fn info(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Info, status)
    }

    /// A success button; denoting a main action.
    pub fn success(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Success, status)
    }

    /// A warning button; denoting a main action.
    pub fn warning(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Warning, status)
    }

    /// A danger button; denoting a main action.
    pub fn danger(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Danger, status)
    }

    /// A text button
    pub fn text(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().background.base.text, status)
    }

    pub fn text_hover(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let base = text_base(theme, schema.background.base.text, status);

        match status {
            Status::Hovered => {
                let pair = schema.neutral.base;
                Style {
                    background: Some(pair.color.scale_alpha(0.5).into()),
                    text_color: pair.text,
                    ..base
                }
            }
            _ => base,
        }
    }

    pub fn text_white(theme: &Theme, status: Status) -> Style {
        text_base(theme, Color::WHITE, status)
    }

    pub fn text_danger(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().danger.base.color, status)
    }

    pub fn text_primary(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().primary.base.color, status)
    }

    pub fn text_background(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().background.base.color, status)
    }

    pub fn text_secondary(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().secondary.base.color, status)
    }

    pub fn text_accent(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().accent.base.color, status)
    }

    pub fn text_neutral(theme: &Theme, status: Status) -> Style {
        text_base(theme, theme.schema().neutral.base.color, status)
    }

    pub fn text_slate(theme: &Theme, status: Status) -> Style {
        text_base(theme, SLATE, status)
    }

    pub fn subtle(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let pair = schema.background.weak;

        let base = Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            ..base(theme)
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

    pub fn subtle_2(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let pair = schema.background.weak;

        let light = iced::theme::palette::lighten(pair.color, 0.025);

        let base = Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            ..base(theme)
        };

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(schema.background.weak.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(light)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtle_inv(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        let pair = schema.background.base;
        let base = Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            ..base(theme)
        };

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(schema.background.weak.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(schema.background.strong.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn weak_primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let pair = Pair {
            color: schema.background.weak.color,
            text: schema.primary.strong.color,
        };

        let base = Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            ..base(theme)
        };

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(schema.neutral.base.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(schema.background.strong.color.into()),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtle_primary(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();
        let palette = schema.primary;
        let pair = palette.weak;

        let base = Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            ..base(theme)
        };

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(pair.color.scale_alpha(0.75))),

                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(palette.strong.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    fn styled(theme: &Theme, palette: Palette, status: Status) -> Style {
        let schema = theme.schema();
        let triplet = palette.triplet(schema);

        let style = |pair: Pair| Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            ..base(theme)
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

    fn base(theme: &Theme) -> Style {
        Style {
            border: border::rounded(theme.schema().radii.fields),
            ..Style::default()
        }
    }

    fn text_base(theme: &Theme, color: Color, status: Status) -> Style {
        let base = Style {
            text_color: color,
            ..base(theme)
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: color.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
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
            let border = if invalid && matches!(status, text_input::Status::Focused { .. }) {
                default.border.color(danger)
            } else {
                default.border
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
                radius: schema.radii.boxes.into(),
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
            color: schema.neutral.strong.color,
            radius: 0.0.into(),
            fill_mode: FillMode::Full,
            snap: true,
        }
    }

    /// A [`Rule`] styling using the weak background color.
    pub fn weak(theme: &Theme) -> Style {
        let schema = theme.schema();

        Style {
            color: schema.neutral.weak.color,
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
            Box::new(container::nb)
        }
    }

    impl From<&Theme> for Style {
        fn from(value: &Theme) -> Self {
            let schema = value.schema();

            Self {
                font: iced::Font::default(),
                inline_code_padding: iced::padding::horizontal(4).vertical(2),
                inline_code_highlight: Highlight {
                    background: schema.primary.weak.color.scale_alpha(0.5).into(),
                    border: iced::border::rounded(schema.radii.boxes),
                },
                inline_code_color: schema.background.base.text,
                inline_code_font: iced::Font::MONOSPACE,
                code_block_font: iced::Font::MONOSPACE,
                link_color: schema.primary.base.color,
            }
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
        styled(theme, Palette::Primary, status)
    }

    pub fn primary_inv(theme: &Theme, status: Status) -> Style {
        styled_inv(theme, Palette::Primary, status)
    }

    /// The secondary style of a [`Toggler`].
    pub fn secondary(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Secondary, status)
    }

    /// The accent style of a [`Toggler`].
    pub fn accent(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Accent, status)
    }

    /// The neutral style of a [`Toggler`].
    pub fn neutral(theme: &Theme, status: Status) -> Style {
        styled(theme, Palette::Neutral, status)
    }

    fn styled_inv(theme: &Theme, palette: Palette, status: Status) -> Style {
        let schema = theme.schema();

        let base = palette.triplet(schema);
        let background = schema.background;

        // fn styled(base: Triplet, background: Triplet, radii: Radii, status: Status) -> Style {
        let bg = match status {
            Status::Active { is_toggled } | Status::Hovered { is_toggled } => {
                if is_toggled {
                    base.base.color
                } else {
                    background.base.color
                }
            }
            Status::Disabled { .. } => background.strong.color,
        };

        let foreground = match status {
            Status::Active { is_toggled } => {
                if is_toggled {
                    base.base.text
                } else {
                    background.weak.color
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
            border_radius: Some(schema.radii.selectors.into()),
            padding_ratio: 0.1,
        }
    }

    fn styled(theme: &Theme, palette: Palette, status: Status) -> Style {
        let schema = theme.schema();

        let base = palette.triplet(schema);
        let background = schema.background;

        // fn styled(base: Triplet, background: Triplet, radii: Radii, status: Status) -> Style {
        let bg = match status {
            Status::Active { is_toggled } | Status::Hovered { is_toggled } => {
                if is_toggled {
                    base.base.color
                } else {
                    background.weak.color
                }
            }
            Status::Disabled { .. } => background.strong.color,
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
                    background.strong.color
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
            border_radius: Some(schema.radii.selectors.into()),
            padding_ratio: 0.1,
        }
    }
}

pub mod modal {
    use super::*;
    use widgets::modal::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>) -> Style {
            class(self)
        }
    }

    pub fn default(_theme: &Theme) -> Style {
        Style {
            blur: Background::Color(Color::BLACK.scale_alpha(0.5)),
        }
    }
}

pub mod pagination {
    use super::*;
    use widgets::pagination::*;

    impl Catalog for Theme {
        type Class<'a> = StyleFn<'a, Self>;

        fn default<'a>() -> Self::Class<'a> {
            Box::new(default)
        }

        fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
            class(self, status)
        }
    }

    /// The default styling of a [`Pagination`].
    pub fn default(theme: &Theme, status: Status) -> Style {
        let schema = theme.schema();

        let pair = match status {
            Status::Active => schema.primary.weak,
            Status::Idle => schema.background.base,
            Status::Hovered => schema.background.weak,
        };

        Style {
            background: Some(Background::Color(pair.color)),
            text: Some(pair.text),
            border: border::rounded(schema.radii.fields),
        }
    }
}

pub mod toast {
    use super::*;
    use widgets::toast::*;

    impl Catalog for Theme {
        fn toast_status<'a>(status: Status) -> <Self as widget::container::Catalog>::Class<'a> {
            match status {
                Status::Info => Box::new(container::pb),
                Status::Success => Box::new(container::sub),
                Status::Warn => Box::new(container::wb),
                Status::Error => Box::new(container::db),
            }
        }

        fn container_rounded<'a>() -> <Self as widget::container::Catalog>::Class<'a> {
            Box::new(|theme: &Theme| {
                let default = container::rounded_box(theme);
                let border = default
                    .border
                    .rounded(theme.schema().radii.boxes)
                    .width(0.5)
                    .color(default.text_color.unwrap_or_default());

                widget::container::Style { border, ..default }
            })
        }

        fn button_text<'a>() -> <Self as widget::button::Catalog>::Class<'a> {
            Box::new(button::text)
        }
    }
}

pub mod throbber {
    use super::*;
    use widgets::throbber;

    pub mod linear {
        use super::*;
        use throbber::linear::*;

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

            Style {
                track_color: schema.primary.base.color.scale_alpha(0.3),
                bar_color: schema.primary.base.color,
            }
        }
    }

    pub mod circular {
        use super::*;
        use throbber::circular::*;

        impl Catalog for Theme {
            type Class<'a> = StyleFn<'a, Self>;

            fn default<'a>() -> Self::Class<'a> {
                Box::new(default)
            }

            fn style(&self, class: &Self::Class<'_>) -> Style {
                class(self)
            }
        }

        /// The default style of a [`Circular`].
        pub fn default(theme: &Theme) -> Style {
            let schema = theme.schema();

            Style {
                background: None,
                track_color: schema.primary.base.color.scale_alpha(0.3),
                bar_color: schema.primary.base.color,
            }
        }
    }
}
