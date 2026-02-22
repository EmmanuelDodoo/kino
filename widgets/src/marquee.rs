use iced::{
    Color, Element, Event, Length, Pixels, Point, Rectangle, Size,
    advanced::{
        self, Widget, layout,
        text::{self, LineHeight, Shaping, Wrapping},
        widget::{Operation, tree},
    },
    alignment::{Horizontal, Vertical},
    mouse,
    time::{Duration, Instant},
    widget::text::Format,
    window,
};

pub use iced::animation::{Animation, Easing};
pub use iced::widget::text::{Catalog, Style, StyleFn};

pub fn marquee<'a, Theme: Catalog, Renderer: text::Renderer>(
    fragment: impl text::IntoFragment<'a>,
) -> Marquee<'a, Theme, Renderer> {
    Marquee::new(fragment)
}

#[derive(Debug, Default, Clone, Copy, PartialEq)]
/// Determines how the text is scrolled within a [`Marquee`].
pub enum Behavior {
    #[default]
    /// The text bounces back and forth between edges
    Alternate,
    /// The enters from one side
    Slide,
    /// The text continuously enters from one side and exits the other.
    Scroll,
}

/// A single line text widget which scrolls its content when hovered
pub struct Marquee<'a, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fragment: text::Fragment<'a>,
    format: Format<Renderer::Font>,
    duration: f32,
    delay: Duration,
    easing: Easing,
    rtl: bool,
    behavior: Behavior,
    class: Theme::Class<'a>,
}

impl<'a, Theme, Renderer> Marquee<'a, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    /// Create a new fragment of [`Marquee`] with the given contents.
    pub fn new(fragment: impl text::IntoFragment<'a>) -> Self {
        Self {
            fragment: fragment.into_fragment(),
            duration: 3.0,
            delay: Duration::from_millis(1000),
            format: Format::default(),
            rtl: false,
            easing: Easing::EaseInOut,
            class: Theme::default(),
            behavior: Behavior::default(),
        }
    }

    /// Sets the duration for scrolling in seconds
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the delay before scrolling starts
    pub fn delay(mut self, delay: impl Into<Duration>) -> Self {
        self.delay = delay.into();
        self
    }

    /// Sets the easing on the scroll
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Sets the direction of the scroll
    pub fn direction(mut self, right_to_left: bool) -> Self {
        self.rtl = right_to_left;
        self
    }

    /// Sets the [`Behavior`] of the widget
    pub fn behavior(mut self, behavior: Behavior) -> Self {
        self.behavior = behavior;
        self
    }

    /// Sets the size of the [`Text`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.format.size = Some(size.into());
        self
    }

    /// Sets the [`LineHeight`] of the [`Text`].
    pub fn line_height(mut self, line_height: impl Into<LineHeight>) -> Self {
        self.format.line_height = line_height.into();
        self
    }

    /// Sets the [`Font`] of the [`Text`].
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font(mut self, font: impl Into<Renderer::Font>) -> Self {
        self.format.font = Some(font.into());
        self
    }

    /// Sets the [`Font`] of the [`Text`], if `Some`.
    ///
    /// [`Font`]: crate::text::Renderer::Font
    pub fn font_maybe(mut self, font: Option<impl Into<Renderer::Font>>) -> Self {
        self.format.font = font.map(Into::into);
        self
    }

    /// Sets the width of the [`Text`] boundaries.
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.format.width = width.into();
        self
    }

    /// Sets the height of the [`Text`] boundaries.
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.format.height = height.into();
        self
    }

    /// Centers the [`Text`], both horizontally and vertically.
    pub fn center(self) -> Self {
        self.align_x(Horizontal::Center).align_y(Vertical::Center)
    }

    /// Sets the [`alignment::Horizontal`] of the [`Text`].
    pub fn align_x(mut self, alignment: impl Into<text::Alignment>) -> Self {
        self.format.align_x = alignment.into();
        self
    }

    /// Sets the [`alignment::Vertical`] of the [`Text`].
    pub fn align_y(mut self, alignment: impl Into<Vertical>) -> Self {
        self.format.align_y = alignment.into();
        self
    }

    /// Sets the [`Shaping`] strategy of the [`Text`].
    pub fn shaping(mut self, shaping: Shaping) -> Self {
        self.format.shaping = shaping;
        self
    }

    /// Sets the [`Wrapping`] strategy of the [`Text`].
    pub fn wrapping(mut self, wrapping: Wrapping) -> Self {
        self.format.wrapping = wrapping;
        self
    }

    /// Sets the style of the [`Text`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the [`Color`] of the [`Text`].
    pub fn color(self, color: impl Into<Color>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.color_maybe(Some(color))
    }

    /// Sets the [`Color`] of the [`Text`], if `Some`.
    pub fn color_maybe(self, color: Option<impl Into<Color>>) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        let color = color.map(Into::into);

        self.style(move |_theme| Style { color })
    }

    /// Sets the style class of the [`Text`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

struct State<P>
where
    P: text::Paragraph,
{
    paragraph: text::paragraph::Plain<P>,
    diff: f32,
    duration: Duration,
    animation: Option<Animation<f32>>,
}

impl<P> State<P>
where
    P: text::Paragraph,
{
    fn new() -> Self {
        Self {
            paragraph: text::paragraph::Plain::default(),
            diff: 0.0,
            duration: Duration::ZERO,
            animation: None,
        }
    }

    fn reset(&mut self) {
        self.animation.take();
    }

    fn start(&self, rtl: bool) -> f32 {
        if rtl { self.diff } else { 0.0 }
    }

    fn end(&self, rtl: bool) -> f32 {
        if rtl { 0.0 } else { self.diff }
    }

    fn anchor(&self, bounds: Rectangle, rtl: bool) -> Point {
        let x = match &self.animation {
            Some(animation) => animation.interpolate_with(std::convert::identity, Instant::now()),
            None => self.start(rtl),
        };

        let anchor = bounds.anchor(
            self.paragraph.min_bounds(),
            self.paragraph.align_x(),
            self.paragraph.align_y(),
        );

        Point::new(anchor.x + x, anchor.y)
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer> for Marquee<'_, Theme, Renderer>
where
    Theme: Catalog,
    Renderer: text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.format.width,
            height: self.format.height,
        }
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        let limits = limits.width(self.format.width).height(self.format.height);
        let max_width = limits.max().width;

        let bounds = Size::new(f32::INFINITY, limits.max().height);

        let size = self.format.size.unwrap_or_else(|| renderer.default_size());
        let font = self.format.font.unwrap_or_else(|| renderer.default_font());

        let _ = state.paragraph.update(text::Text {
            content: &self.fragment,
            bounds,
            size,
            line_height: self.format.line_height,
            font,
            align_x: self.format.align_x,
            align_y: self.format.align_y,
            shaping: self.format.shaping,
            wrapping: self.format.wrapping,
            ellipsis: text::Ellipsis::None,
            hint_factor: renderer.scale_factor(),
        });

        let paragraph_size = state.paragraph.min_bounds();

        let duration = self.duration * paragraph_size.width / max_width;
        state.diff = (max_width - paragraph_size.width).min(0.0);
        state.duration = Duration::from_secs_f32(duration);

        layout::Node::new(limits.resolve(self.format.width, self.format.height, paragraph_size))
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: advanced::mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();
        let color = theme.style(&self.class).color.unwrap_or(style.text_color);

        let bounds = layout.bounds();

        let anchor = state.anchor(bounds, self.rtl);

        if let Some(clipped) = bounds.intersection(viewport) {
            renderer.fill_paragraph(state.paragraph.raw(), anchor, color, clipped);
        }
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &iced::Event,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        if state.diff == 0.0 {
            return;
        }

        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                if !cursor.is_over(layout.bounds()) {
                    if state.animation.is_some() {
                        state.reset();
                        shell.request_redraw();
                    }

                    return;
                }

                if state.animation.is_some() {
                    return;
                }

                let animation = match self.behavior {
                    Behavior::Alternate => Animation::new(state.start(self.rtl))
                        .auto_reverse()
                        .repeat_forever()
                        .go(state.end(self.rtl), Instant::now()),
                    Behavior::Slide => Animation::new(state.start(self.rtl))
                        .go(state.end(self.rtl), Instant::now()),
                    Behavior::Scroll => {
                        let width = state.paragraph.min_width();

                        let (start, end) = if self.rtl {
                            (-width, width + state.diff)
                        } else {
                            (width + state.diff, -width)
                        };

                        Animation::new(start)
                            .repeat_forever()
                            .go(end, Instant::now())
                    }
                }
                .easing(self.easing)
                .duration(state.duration)
                .delay(self.delay);

                state.animation = Some(animation);
                shell.request_redraw();
            }
            Event::Window(window::Event::RedrawRequested(at)) => {
                let Some(animation) = state.animation.as_mut() else {
                    return;
                };

                if animation.is_animating(*at) {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn operate(
        &mut self,
        _tree: &mut tree::Tree,
        layout: layout::Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.text(None, layout.bounds(), &self.fragment);
    }
}

impl<'a, Message, Theme, Renderer> From<Marquee<'a, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Theme: 'a + Catalog,
    Renderer: 'a + text::Renderer,
{
    fn from(value: Marquee<'a, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
