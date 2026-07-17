//! Show a circular progress indicator.
use iced::advanced::layout;
use iced::advanced::renderer;
use iced::advanced::widget::tree::{self, Tree};
use iced::advanced::{self, Layout, Shell, Widget};
use iced::animation::{Animation, Easing};
use iced::mouse;
use iced::time::Instant;
use iced::widget::canvas::{self, path::Arc};
use iced::window;
use iced::{Background, Color, Element, Event, Length, Radians, Rectangle, Renderer, Size, Vector};

use std::time::Duration;

pub fn circular<'a, Theme: Catalog>() -> Circular<'a, Theme> {
    Circular::new()
}

pub struct Circular<'a, Theme>
where
    Theme: Catalog,
{
    radius: f32,
    bar_height: f32,
    easing: Easing,
    cycle_duration: Duration,
    rotation_duration: Duration,
    class: Theme::Class<'a>,
}

impl<'a, Theme> Circular<'a, Theme>
where
    Theme: Catalog,
{
    /// Creates a new [`Circular`] with the given content.
    pub fn new() -> Self {
        Circular {
            radius: 40.0,
            bar_height: 4.0,
            easing: Easing::EaseInOut,
            cycle_duration: Duration::from_millis(2000),
            rotation_duration: Duration::from_millis(2000),
            class: Theme::default(),
        }
    }

    /// Sets the radius of the [`Circular`].
    pub fn radius(mut self, radius: f32) -> Self {
        self.radius = radius;
        self
    }

    /// Sets the bar height of the [`Circular`].
    pub fn bar_height(mut self, bar_height: f32) -> Self {
        self.bar_height = bar_height;
        self
    }

    /// Sets the style variant of this [`Circular`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the easing of this [`Circular`].
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the cycle duration of this [`Circular`].
    pub fn cycle_duration(mut self, duration: Duration) -> Self {
        self.cycle_duration = duration;
        self
    }

    /// Sets the base rotation duration of this [`Circular`]. This is the duration that a full
    /// rotation would take if the cycle rotation were set to 0.0 (no expanding or contracting)
    pub fn rotation_duration(mut self, duration: Duration) -> Self {
        self.rotation_duration = duration;
        self
    }
}

impl<'a, Theme> Default for Circular<'a, Theme>
where
    Theme: Catalog,
{
    fn default() -> Self {
        Self::new()
    }
}

struct State {
    rotation: Animation<f32>,
    cycle: Animation<f32>,
    cache: canvas::Cache,
    cycle_duration: Duration,
    cycle_easing: Easing,

    start: Radians,
    end: Radians,
    direction: bool,
    growth: Radians,
    length: Radians,

    rot_count: u32,
}

impl State {
    const START: f32 = 0.0;
    const END: f32 = 0.25;

    const CYCLE_DELAY: Duration = Duration::from_millis(500);
    const CYCLE_END: f32 = 1.05;

    fn new(rotation_duration: Duration, cycle_duration: Duration, easing: Easing) -> Self {
        let now = Instant::now();
        let length = Radians::PI * (Self::END - Self::START);

        Self {
            cycle: Animation::new(0.0)
                .duration(cycle_duration)
                .easing(easing)
                .delay(Self::CYCLE_DELAY)
                .go(Self::CYCLE_END, now),
            rotation: Animation::new(0.0)
                .duration(rotation_duration)
                .repeat_forever()
                .easing(Easing::Linear)
                .go(2.0, now),
            cache: canvas::Cache::default(),

            start: Radians::PI * Self::START,
            end: Radians::PI * Self::END,

            direction: true,
            cycle_duration,
            cycle_easing: easing,
            growth: Radians::PI * (1.5 + Self::END) - length,
            length,

            rot_count: 0,
        }
    }

    fn frame(&mut self, now: Instant) {
        let factor = self.rotation.interpolate_with(std::convert::identity, now);
        let cycle = self.cycle.interpolate_with(std::convert::identity, now);

        let rotated_value = 2.0 * self.rot_count as f32;

        let extra = self.growth * self.rot_count as f32;
        self.start = Radians::PI * (Self::START + rotated_value + factor) + extra;

        if self.direction {
            if cycle > 1.0 {
                self.direction = !self.direction;
                self.cycle = Animation::new(1.0)
                    .duration(self.cycle_duration)
                    .easing(self.cycle_easing)
                    .delay(Self::CYCLE_DELAY)
                    .go(0.0, now);
                return;
            };

            self.end = self.start + self.length + cycle * self.growth;
        } else {
            self.start += (1.0 - cycle) * self.growth;
            self.end = (Radians::PI * (Self::END + rotated_value + factor))
                + (self.growth * (1 + self.rot_count) as f32);

            if cycle == 0.0 {
                self.rot_count += 1;
                self.direction = !self.direction;
                self.cycle = Animation::new(0.0)
                    .duration(self.cycle_duration)
                    .easing(self.cycle_easing)
                    .delay(Self::CYCLE_DELAY)
                    .go(Self::CYCLE_END, now);
            }
        }
    }
}

impl<'a, Message, Theme> Widget<Message, Theme, Renderer> for Circular<'a, Theme>
where
    Message: Clone,
    Theme: Catalog,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(
            self.rotation_duration,
            self.cycle_duration,
            self.easing,
        ))
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: Length::Fixed(self.radius),
            height: Length::Fixed(self.radius),
        }
    }

    fn layout(
        &mut self,
        _tree: &mut Tree,
        _renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        layout::atomic(limits, self.radius, self.radius)
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
            state.frame(*now);
            state.cache.clear();
            shell.request_redraw();
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
        use advanced::Renderer as _;

        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let custom_style = theme.style(&self.class);

        let geometry = state.cache.draw(renderer, bounds.size(), |frame| {
            let track_radius = frame.width() / 2.0 - self.bar_height;
            let track_path = canvas::Path::circle(frame.center(), track_radius);

            frame.stroke(
                &track_path,
                canvas::Stroke::default()
                    .with_color(custom_style.track_color)
                    .with_width(self.bar_height),
            );

            let mut builder = canvas::path::Builder::new();

            // let start = Radians::PI / -2.0;
            // let end = Radians::PI / -3.0;

            let start = state.start;
            let end = state.end;

            builder.arc(Arc {
                center: frame.center(),
                radius: track_radius,
                start_angle: start,
                end_angle: end,
            });

            let bar_path = builder.build();

            frame.stroke(
                &bar_path,
                canvas::Stroke::default()
                    .with_color(custom_style.bar_color)
                    .with_width(self.bar_height),
            );
        });

        renderer.with_translation(Vector::new(bounds.x, bounds.y), |renderer| {
            use iced::advanced::graphics::geometry::Renderer as _;

            renderer.draw_geometry(geometry);
        });
    }
}

impl<'a, Message, Theme> From<Circular<'a, Theme>> for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
{
    fn from(circular: Circular<'a, Theme>) -> Self {
        Self::new(circular)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Style {
    /// The [`Background`] of the progress indicator.
    pub background: Option<Background>,
    /// The track [`Color`] of the progress indicator.
    pub track_color: Color,
    /// The bar [`Color`] of the progress indicator.
    pub bar_color: Color,
}

impl std::default::Default for Style {
    fn default() -> Self {
        Self {
            background: None,
            track_color: Color::TRANSPARENT,
            bar_color: Color::BLACK,
        }
    }
}

/// The theme catalog of a [`Circular`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>) -> Style;
}

/// A styling function for a [`Circular`].
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

/// The default style of a [`Circular`].
pub fn default(theme: &iced::Theme) -> Style {
    let palette = theme.palette();

    Style {
        background: None,
        track_color: palette.background.weak.color,
        bar_color: palette.primary.base.color,
    }
}
