use iced::{Background, Border, Color, Theme, border, color, theme, widget};

const SLATE: Color = Color::from_rgb8(144, 161, 185);
const SLATE2: Color = Color::from_rgb8(202, 213, 226);

pub mod container {
    use super::*;
    use widget::container::{self, Style, primary, secondary};

    fn style(pair: theme::palette::Pair) -> Style {
        Style {
            background: Some(pair.color.into()),
            text_color: Some(pair.text),
            ..Style::default()
        }
    }

    pub fn sw(theme: &Theme) -> Style {
        style(theme.extended_palette().secondary.weak)
    }

    pub fn sb(theme: &Theme) -> Style {
        style(theme.extended_palette().secondary.base)
    }

    pub fn ss(theme: &Theme) -> Style {
        style(theme.extended_palette().secondary.strong)
    }

    pub fn pw(theme: &Theme) -> Style {
        style(theme.extended_palette().primary.weak)
    }

    pub fn pb(theme: &Theme) -> Style {
        style(theme.extended_palette().primary.base)
    }

    pub fn ps(theme: &Theme) -> Style {
        style(theme.extended_palette().primary.strong)
    }

    pub fn bw(theme: &Theme) -> Style {
        style(theme.extended_palette().background.weaker)
    }

    pub fn bw2(theme: &Theme) -> Style {
        style(theme.extended_palette().background.weakest)
    }

    pub fn bw3(theme: &Theme) -> Style {
        style(theme.extended_palette().background.weak)
    }

    pub fn bb(theme: &Theme) -> Style {
        style(theme.extended_palette().background.base)
    }

    pub fn bs(theme: &Theme) -> Style {
        style(theme.extended_palette().background.strong)
    }

    pub fn dark(theme: &Theme) -> Style {
        container::dark(theme)
    }

    pub fn bordered(theme: &Theme) -> Style {
        let palette = theme.extended_palette();

        Style {
            background: Some(palette.background.base.color.into()),
            text_color: Some(palette.background.base.text),
            border: Border::default()
                .width(1.0)
                .color(palette.background.weak.color),
            ..Style::default()
        }
    }

    pub fn transparent(theme: &Theme) -> Style {
        container::transparent(theme)
    }

    pub fn text(_theme: &Theme) -> Style {
        let text_color = SLATE;

        Style {
            text_color: Some(text_color),
            background: None,
            ..Default::default()
        }
    }
}

pub mod button {
    use super::*;

    use widget::button::{self, Status, Style};

    fn styled(pair: theme::palette::Pair) -> Style {
        Style {
            background: Some(Background::Color(pair.color)),
            text_color: pair.text,
            border: border::rounded(2),
            ..Style::default()
        }
    }

    fn disabled(style: Style) -> Style {
        Style {
            background: style
                .background
                .map(|background| background.scale_alpha(0.5)),
            text_color: style.text_color.scale_alpha(0.5),
            ..style
        }
    }

    pub fn primary(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.primary.base);

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                background: Some(Background::Color(palette.primary.strong.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn secondary(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.secondary.base);

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                background: Some(Background::Color(palette.secondary.strong.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn background(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.background.base);

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(palette.background.weak.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn danger(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.danger.base);

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                background: Some(Background::Color(palette.danger.strong.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();

        let base = Style {
            text_color: palette.background.base.text,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: palette.background.base.text.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text_white(_theme: &Theme, status: Status) -> Style {
        let base = Style {
            text_color: Color::WHITE,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: Color::WHITE.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text_danger(theme: &Theme, status: Status) -> Style {
        let danger = theme.extended_palette().danger;

        let base = theme.extended_palette().danger.base;
        let text_color = base.color;

        let base = Style {
            text_color,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: danger.weak.color,
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text_primary(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();

        let base = Style {
            text_color: palette.primary.base.color,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: palette.primary.base.color.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text_background(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();

        let base = Style {
            text_color: palette.background.base.color,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: palette.background.base.color.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text_secondary(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();

        let base = Style {
            text_color: palette.secondary.base.color,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: palette.secondary.base.color.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn text_slate(_theme: &Theme, status: Status) -> Style {
        let base = Style {
            text_color: SLATE,
            ..Style::default()
        };

        match status {
            Status::Active | Status::Pressed => base,
            Status::Hovered => Style {
                text_color: SLATE.scale_alpha(0.8),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtle(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.background.weak);

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(palette.background.neutral.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtler(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.background.weaker);

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(palette.background.weak.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtlest(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let base = styled(palette.background.weakest);

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(palette.background.strong.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(palette.background.weaker.color)),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn subtle_primary(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();

        let pair = theme::palette::Pair {
            color: palette.primary.strong.color.scale_alpha(0.5),
            text: palette.primary.strong.text,
        };

        let base = styled(pair);

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(palette.primary.strong.color)),

                ..base
            },
            Status::Hovered => Style {
                background: Some(Background::Color(
                    palette.primary.strong.color.scale_alpha(0.75),
                )),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }

    pub fn background_primary(theme: &Theme, status: Status) -> Style {
        let palette = theme.extended_palette();
        let pair = theme::palette::Pair {
            color: palette.background.weaker.color,
            text: palette.primary.strong.color,
        };

        let base = styled(pair);

        match status {
            Status::Active => base,
            Status::Pressed => Style {
                background: Some(Background::Color(palette.background.strongest.color)),
                ..base
            },
            Status::Hovered => Style {
                background: Some(palette.background.strong.color.into()),
                ..base
            },
            Status::Disabled => disabled(base),
        }
    }
}
