//! Show a linear progress indicator.
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Layout, Shell, Widget};
use iced::animation::{Animation, Easing};
use iced::mouse;
use iced::time::Instant;
use iced::window;
use iced::{Background, Color, Element, Event, Length, Rectangle, Size};

use std::time::Duration;

pub fn linear<'a, Theme: Catalog>() -> Linear<'a, Theme> {
    Linear::new()
}

pub struct Linear<'a, Theme>
where
    Theme: Catalog,
{
    width: Length,
    height: Length,
    class: Theme::Class<'a>,
    easing: Easing,
    cycle_duration: Duration,
}

impl<'a, Theme> Linear<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a new [`Linear`] with the given content.
    pub fn new() -> Self {
        Linear {
            width: Length::Fixed(100.0),
            height: Length::Fixed(4.0),
            class: Theme::default(),
            easing: Easing::EaseInOut,
            cycle_duration: Duration::from_millis(2500),
        }
    }

    /// Sets the width of the [`Linear`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Linear`].
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the style variant of this [`Linear`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the motion easing of this [`Linear`].
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the cycle duration of this [`Linear`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }
}

impl<'a, Theme> Default for Linear<'a, Theme>
where
    Theme: Catalog,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State {
    animation: Animation<f32>,
}

impl State {
    fn new(duration: Duration, easing: Easing) -> Self {
        State {
            animation: Animation::new(0.0)
                .duration(duration)
                .repeat_forever()
                .easing(easing)
                .go(2.0, Instant::now()),
        }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Linear<'a, Theme>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: advanced::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.cycle_duration, self.easing))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: self.height,
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.width, self.height)
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        _layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            if state.animation.is_animating(*now) {
                shell.request_redraw();
            }
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();
        let style = theme.style(&self.class);
        let state = tree.state.downcast_ref::<State>();

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: bounds.y,
                    width: bounds.width,
                    height: bounds.height,
                },
                ..renderer::Quad::default()
            },
            Background::Color(style.track_color),
        );

        let factor = state
            .animation
            .interpolate_with(std::convert::identity, Instant::now());

        let x_factor = (1.0 - factor).min(0.0);
        let x_diff = x_factor * bounds.width;

        let width_factor = if x_factor == 0.0 {
            factor.min(1.0)
        } else {
            (2.0 - factor).max(0.0)
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x - x_diff,
                    y: bounds.y,
                    width: width_factor * bounds.width,
                    height: bounds.height,
                },
                ..renderer::Quad::default()
            },
            Background::Color(style.bar_color),
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Linear<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: iced::advanced::Renderer + 'a,
{
    fn from(linear: Linear<'a, Theme>) -> Self {
        Self::new(linear)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
}

impl Default for Style {
    fn default() -> Self {
        Self {
            track_color: Color::TRANSPARENT,
            bar_color: Color::BLACK,
        }
    }
}

/// The theme catalog of a [`Linear`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Linear`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>) -> Style {
        class(self)
    }
}

pub fn default(theme: &iced::Theme) -> Style {
    let palette = theme.palette();

    Style {
        track_color: palette.background.weak.color,
        bar_color: palette.primary.base.color,
    }
}
