//! Modified version of [`iced::widget::toggler`] with animation support.

use iced::{
    Background, Border, Color, Element, Event, Length, Pixels, Rectangle, Size, Theme,
    advanced::{
        Layout, Shell, Widget, layout, renderer, text,
        widget::{self, Tree, tree},
    },
    alignment,
    animation::{Animation, Easing},
    border, mouse,
    time::{Duration, Instant},
    touch, window,
};

pub fn toggler<'a, Message, Theme, Renderer>(
    is_checked: bool,
) -> Toggler<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    Toggler::new(is_checked)
}

/// A modified version of an [`iced::widget::Toggler`] with animation support
pub struct Toggler<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    is_toggled: bool,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    label: Option<text::Fragment<'a>>,
    width: Length,
    size: f32,
    text_size: Option<Pixels>,
    text_line_height: text::LineHeight,
    text_alignment: text::Alignment,
    text_shaping: text::Shaping,
    text_wrapping: text::Wrapping,
    ellipsis: text::Ellipsis,
    spacing: f32,
    font: Option<Renderer::Font>,
    easing: Easing,
    duration: Duration,
    class: Theme::Class<'a>,
    last_status: Option<Status>,
}

impl<'a, Message, Theme, Renderer> Toggler<'a, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// The default size of a [`Toggler`].
    pub const DEFAULT_SIZE: f32 = 16.0;

    /// Creates a new [`Toggler`].
    ///
    /// It expects:
    ///   * a boolean describing whether the [`Toggler`] is checked or not
    ///   * An optional label for the [`Toggler`]
    ///   * a function that will be called when the [`Toggler`] is toggled. It
    ///     will receive the new state of the [`Toggler`] and must produce a
    ///     `Message`.
    pub fn new(is_toggled: bool) -> Self {
        Toggler {
            is_toggled,
            on_toggle: None,
            label: None,
            width: Length::Shrink,
            size: Self::DEFAULT_SIZE,
            text_size: None,
            text_line_height: text::LineHeight::default(),
            text_alignment: text::Alignment::Default,
            text_shaping: text::Shaping::default(),
            text_wrapping: text::Wrapping::default(),
            ellipsis: text::Ellipsis::default(),
            spacing: Self::DEFAULT_SIZE / 2.0,
            font: None,
            class: Theme::default(),
            last_status: None,
            easing: Easing::EaseOutQuint,
            duration: Duration::from_millis(500),
        }
    }

    /// Sets the [`Easing`] of the animation.
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the [`Duration`] of the animation.
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the label of the [`Toggler`].
    pub fn label(mut self, label: impl text::IntoFragment<'a>) -> Self {
        self.label = Some(label.into_fragment());
        self
    }

    /// Sets the message that should be produced when a user toggles
    /// the [`Toggler`].
    ///
    /// If this method is not called, the [`Toggler`] will be disabled.
    pub fn on_toggle(mut self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_toggle = Some(Box::new(on_toggle));
        self
    }

    /// Sets the message that should be produced when a user toggles
    /// the [`Toggler`], if `Some`.
    ///
    /// If `None`, the [`Toggler`] will be disabled.
    pub fn on_toggle_maybe(mut self, on_toggle: Option<impl Fn(bool) -> Message + 'a>) -> Self {
        self.on_toggle = on_toggle.map(|on_toggle| Box::new(on_toggle) as _);
        self
    }

    /// Sets the size of the [`Toggler`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = size.into().0;
        self
    }

    /// Sets the width of the [`Toggler`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the text size o the [`Toggler`].
    pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
        self.text_size = Some(text_size.into());
        self
    }

    /// Sets the text [`text::LineHeight`] of the [`Toggler`].
    pub fn text_line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.text_line_height = line_height.into();
        self
    }

    /// Sets the horizontal alignment of the text of the [`Toggler`]
    pub fn text_alignment(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.text_alignment = alignment.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`Toggler`].
    pub fn text_shaping(mut self, shaping: text::Shaping) -> Self {
        self.text_shaping = shaping;
        self
    }

    /// Sets the [`text::Wrapping`] strategy of the [`Toggler`].
    pub fn text_wrapping(mut self, wrapping: text::Wrapping) -> Self {
        self.text_wrapping = wrapping;
        self
    }

    /// Sets the [`text::Ellipsis`] strategy of the [`Toggler`].
    pub fn text_ellipsis(mut self, ellipsis: text::Ellipsis) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    /// Sets the spacing between the [`Toggler`] and the text.
    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    /// Sets the [`Renderer::Font`] of the text of the [`Toggler`]
    ///
    /// [`Renderer::Font`]: iced::core::text::Renderer
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.font = Some(font.into());
        self
    }

    /// Sets the style of the [`Toggler`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Toggler<'_, Message, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<widget::text::State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new(
            self.is_toggled,
            self.easing,
            self.duration,
        ))
    }

    fn diff(&self, tree: &mut Tree) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        if state.animation.value() != self.is_toggled {
            state.go_mut(self.is_toggled, Instant::now());
        }
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let limits = limits.width(self.width);

        layout::next_to_each_other(
            &limits,
            if self.label.is_some() {
                self.spacing
            } else {
                0.0
            },
            |_| layout::Node::new(Size::new(2.0 * self.size, self.size)),
            |limits| {
                if let Some(label) = self.label.as_deref() {
                    let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

                    widget::text::layout(
                        &mut state.text,
                        renderer,
                        limits,
                        label,
                        widget::text::Format {
                            width: self.width,
                            height: Length::Shrink,
                            line_height: self.text_line_height,
                            size: self.text_size,
                            font: self.font,
                            align_x: self.text_alignment,
                            align_y: alignment::Vertical::Top,
                            shaping: self.text_shaping,
                            wrapping: self.text_wrapping,
                            ellipsis: self.ellipsis,
                        },
                    )
                } else {
                    layout::Node::new(Size::ZERO)
                }
            },
        )
    }

    fn update(
        &mut self,
        tree: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let Some(on_toggle) = &self.on_toggle else {
            return;
        };

        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
            | Event::Touch(touch::Event::FingerPressed { .. }) => {
                let mouse_over = cursor.is_over(layout.bounds());

                if mouse_over {
                    let new = !self.is_toggled;
                    shell.publish(on_toggle(new));
                    state.go_mut(new, Instant::now());
                    shell.capture_event();
                }
            }
            _ => {}
        }

        let current_status = if self.on_toggle.is_none() {
            Status::Disabled {
                is_toggled: self.is_toggled,
            }
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered {
                is_toggled: self.is_toggled,
            }
        } else {
            Status::Active {
                is_toggled: self.is_toggled,
            }
        };

        if let Event::Window(window::Event::RedrawRequested(now)) = event {
            self.last_status = Some(current_status);

            if state.is_animating(*now) {
                state.now = *now;
                shell.request_redraw();
            }
        } else if self
            .last_status
            .is_some_and(|status| status != current_status)
        {
            shell.request_redraw();
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        if cursor.is_over(layout.bounds()) {
            if self.on_toggle.is_some() {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::NotAllowed
            }
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        defaults: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let mut children = layout.children();
        let toggler_layout = children.next().unwrap();
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        let style = theme.style(
            &self.class,
            self.last_status.unwrap_or(Status::Disabled {
                is_toggled: self.is_toggled,
            }),
        );

        if self.label.is_some() {
            let label_layout = children.next().unwrap();

            iced::widget::text::draw(
                renderer,
                defaults,
                label_layout.bounds(),
                state.text.raw(),
                iced::widget::text::Style {
                    color: style.text_color,
                },
                viewport,
            );
        }

        let bounds = toggler_layout.bounds();
        let border_radius = style
            .border_radius
            .unwrap_or_else(|| border::Radius::new(bounds.height / 2.0));

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: Border {
                    radius: border_radius,
                    width: style.background_border_width,
                    color: style.background_border_color,
                },
                ..renderer::Quad::default()
            },
            style.background,
        );

        let padding = (style.padding_ratio * bounds.height).round();
        let position = bounds.width - bounds.height;
        let position = state.interpolate() * position;

        let toggler_foreground_bounds = Rectangle {
            x: bounds.x + position + padding,
            y: bounds.y + padding,
            width: bounds.height - (2.0 * padding),
            height: bounds.height - (2.0 * padding),
        };

        renderer.fill_quad(
            renderer::Quad {
                bounds: toggler_foreground_bounds,
                border: Border {
                    radius: border_radius,
                    width: style.foreground_border_width,
                    color: style.foreground_border_color,
                },
                ..renderer::Quad::default()
            },
            style.foreground,
        );
    }
}

impl<'a, Message, Theme, Renderer> From<Toggler<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: Catalog + 'a,
    Renderer: text::Renderer + 'a,
{
    fn from(
        toggler: Toggler<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(toggler)
    }
}

struct State<Paragraph>
where
    Paragraph: text::Paragraph,
{
    text: widget::text::State<Paragraph>,
    animation: Animation<bool>,
    now: Instant,
}

impl<Paragraph> State<Paragraph>
where
    Paragraph: text::Paragraph,
{
    fn new(is_checked: bool, easing: Easing, duration: Duration) -> Self {
        Self {
            text: widget::text::State::default(),

            animation: Animation::new(is_checked).duration(duration).easing(easing),
            now: Instant::now(),
        }
    }

    fn go_mut(&mut self, new: bool, at: Instant) {
        self.now = at;
        self.animation.go_mut(new, at);
    }

    fn is_animating(&self, at: Instant) -> bool {
        self.animation.is_animating(at)
    }

    fn interpolate(&self) -> f32 {
        self.animation.interpolate(0.0, 1.0, self.now)
    }
}

/// The possible status of a [`Toggler`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Status {
    /// The [`Toggler`] can be interacted with.
    Active {
        /// Indicates whether the [`Toggler`] is toggled.
        is_toggled: bool,
    },
    /// The [`Toggler`] is being hovered.
    Hovered {
        /// Indicates whether the [`Toggler`] is toggled.
        is_toggled: bool,
    },
    /// The [`Toggler`] is disabled.
    Disabled {
        /// Indicates whether the [`Toggler`] is toggled.
        is_toggled: bool,
    },
}

/// The appearance of a toggler.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Style {
    /// The background [`Color`] of the toggler.
    pub background: Background,
    /// The width of the background border of the toggler.
    pub background_border_width: f32,
    /// The [`Color`] of the background border of the toggler.
    pub background_border_color: Color,
    /// The foreground [`Color`] of the toggler.
    pub foreground: Background,
    /// The width of the foreground border of the toggler.
    pub foreground_border_width: f32,
    /// The [`Color`] of the foreground border of the toggler.
    pub foreground_border_color: Color,
    /// The text [`Color`] of the toggler.
    pub text_color: Option<Color>,
    /// The border radius of the toggler.
    ///
    /// If `None`, the toggler will be perfectly round.
    pub border_radius: Option<border::Radius>,
    /// The ratio of separation between the background and the toggle in relative height.
    pub padding_ratio: f32,
}

/// The theme catalog of a [`Toggler`].
pub trait Catalog: Sized {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

/// A styling function for a [`Toggler`].
///
/// This is just a boxed closure: `Fn(&Theme, Status) -> Style`.
pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default style of a [`Toggler`].
pub fn default(theme: &Theme, status: Status) -> Style {
    let palette = theme.extended_palette();

    let background = match status {
        Status::Active { is_toggled } | Status::Hovered { is_toggled } => {
            if is_toggled {
                palette.primary.base.color
            } else {
                palette.background.strong.color
            }
        }
        Status::Disabled { is_toggled } => {
            if is_toggled {
                palette.background.strong.color
            } else {
                palette.background.weak.color
            }
        }
    };

    let foreground = match status {
        Status::Active { is_toggled } => {
            if is_toggled {
                palette.primary.base.text
            } else {
                palette.background.base.color
            }
        }
        Status::Hovered { is_toggled } => {
            if is_toggled {
                Color {
                    a: 0.5,
                    ..palette.primary.base.text
                }
            } else {
                palette.background.weak.color
            }
        }
        Status::Disabled { .. } => palette.background.weakest.color,
    };

    Style {
        background: background.into(),
        foreground: foreground.into(),
        foreground_border_width: 0.0,
        foreground_border_color: Color::TRANSPARENT,
        background_border_width: 0.0,
        background_border_color: Color::TRANSPARENT,
        text_color: None,
        border_radius: None,
        padding_ratio: 0.1,
    }
}
