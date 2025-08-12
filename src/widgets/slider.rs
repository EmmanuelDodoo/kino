//! Customised version of the [`iced::widget::Slider`] for video playback.
use iced::{
    Background, Border, Color, Element, Event, Length, Pixels, Point, Rectangle, Size, Theme,
    advanced::{
        self, Clipboard, Shell, image,
        layout::{self, Layout},
        mouse, renderer, text,
        widget::{
            self, Widget,
            tree::{self, Tree},
        },
    },
    alignment, border,
    keyboard::{self, Key, key},
    touch,
    widget::slider::{Catalog, Handle, HandleShape, Rail, Status, Style, StyleFn, default},
    window,
};

use std::{ops::RangeInclusive, time::Duration};

/// A customised version of the [`iced::widget::Slider`] for video playback.
/// This version includes detection for hovering on the rail. The cursor type
/// for interactions is changed to an [`iced::mouse::Interaction::Pointer`].
pub struct VideoSlider<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer>
where
    Theme: Catalog,
    Renderer: image::Renderer + text::Renderer,
{
    range: RangeInclusive<f64>,
    step: f64,
    shift_step: Option<f64>,
    value: f64,
    thumbnails: Vec<Renderer::Handle>,
    thumbnail_font: <Renderer as text::Renderer>::Font,
    duration: Duration,
    default: Option<f64>,
    on_change: Box<dyn Fn(f64) -> Message + 'a>,
    on_release: Option<Message>,
    width: Length,
    height: f32,
    class: Theme::Class<'a>,
    status: Option<Status>,
}

impl<'a, Message, Theme, Renderer> VideoSlider<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: image::Renderer + text::Renderer,
{
    /// The default height of a [`Slider`].
    pub const DEFAULT_HEIGHT: f32 = 16.0;

    /// Creates a new [`Slider`].
    ///
    /// It expects:
    ///   * an inclusive range of possible values
    ///   * the current value of the [`Slider`]
    ///   * a function that will be called when the [`Slider`] is dragged.
    ///     It receives the new value of the [`Slider`] and must produce a
    ///     `Message`.
    pub fn new<F>(
        range: RangeInclusive<f64>,
        value: f64,
        on_change: F,
        thumbnails: Vec<Renderer::Handle>,
        thumbnail_font: <Renderer as text::Renderer>::Font,
        duration: Duration,
    ) -> Self
    where
        F: 'a + Fn(f64) -> Message,
    {
        let value = if value >= *range.start() {
            value
        } else {
            *range.start()
        };

        let value = if value <= *range.end() {
            value
        } else {
            *range.end()
        };

        VideoSlider {
            value,
            default: None,
            range,
            step: 1.0,
            thumbnails,
            duration,
            thumbnail_font,
            shift_step: None,
            on_change: Box::new(on_change),
            on_release: None,
            width: Length::Fill,
            height: Self::DEFAULT_HEIGHT,
            class: Theme::default(),
            status: None,
        }
    }

    /// Sets the optional default value for the [`Slider`].
    ///
    /// If set, the [`Slider`] will reset to this value when ctrl-clicked or command-clicked.
    pub fn default(mut self, default: impl Into<f64>) -> Self {
        self.default = Some(default.into());
        self
    }

    /// Sets the release message of the [`Slider`].
    /// This is called when the mouse is released from the slider.
    ///
    /// Typically, the user's interaction with the slider is finished when this message is produced.
    /// This is useful if you need to spawn a long-running task from the slider's result, where
    /// the default on_change message could create too many events.
    pub fn on_release(mut self, on_release: Message) -> Self {
        self.on_release = Some(on_release);
        self
    }

    /// Sets the width of the [`Slider`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the [`Slider`].
    pub fn height(mut self, height: impl Into<Pixels>) -> Self {
        self.height = height.into().0;
        self
    }

    /// Sets the step size of the [`Slider`].
    pub fn step(mut self, step: impl Into<f64>) -> Self {
        self.step = step.into();
        self
    }

    /// Sets the optional "shift" step for the [`Slider`].
    ///
    /// If set, this value is used as the step while the shift key is pressed.
    pub fn shift_step(mut self, shift_step: impl Into<f64>) -> Self {
        self.shift_step = Some(shift_step.into());
        self
    }

    /// Sets the style of the [`Slider`].
    #[must_use]
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Slider`].
    #[must_use]
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }
}

impl<Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for VideoSlider<'_, Message, Theme, Renderer>
where
    Message: Clone,
    Theme: Catalog,
    Renderer: advanced::Renderer + image::Renderer + text::Renderer,
{
    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::default())
    }

    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Shrink,
        }
    }

    fn layout(
        &self,
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
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        _clipboard: &mut dyn Clipboard,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();

        let mut update = || {
            let current_value = self.value;

            let locate = |cursor_position: iced::Point| -> f64 {
                let bounds = layout.bounds();
                let new_value = if cursor_position.x <= bounds.x {
                    *self.range.start()
                } else if cursor_position.x >= bounds.x + bounds.width {
                    *self.range.end()
                } else {
                    let start = *self.range.start();
                    let end = *self.range.end();

                    let percent = f64::from(cursor_position.x - bounds.x) / f64::from(bounds.width);

                    let steps = (percent * (end - start) / self.step).round();
                    let value = steps * self.step + start;

                    value.min(end)
                };

                new_value
            };

            let increment = |value: f64| -> f64 {
                let steps = (value / self.step).round();
                let new_value = self.step * (steps + 1.0);

                if new_value > (*self.range.end()).into() {
                    return *self.range.end();
                }

                new_value
            };

            let decrement = |value: f64| -> f64 {
                let steps = (value / self.step).round();
                let new_value = self.step * (steps - 1.0);

                if new_value < (*self.range.start()).into() {
                    return *self.range.start();
                }

                new_value
            };

            let mut change = |new_value: f64| {
                if (self.value - new_value).abs() > f64::EPSILON {
                    shell.publish((self.on_change)(new_value));
                    self.value = new_value;
                }
            };

            match &event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(cursor_position) = cursor.position_over(layout.bounds()) {
                        if state.keyboard_modifiers.command() {
                            let _ = self.default.map(change);
                            state.is_dragging = false;
                        } else {
                            let _ = change(locate(cursor_position));
                            state.is_dragging = true;
                        }

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::ButtonReleased(mouse::Button::Left))
                | Event::Touch(touch::Event::FingerLifted { .. })
                | Event::Touch(touch::Event::FingerLost { .. }) => {
                    if state.is_dragging {
                        if let Some(on_release) = self.on_release.clone() {
                            shell.publish(on_release);
                        }
                        state.is_dragging = false;

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. })
                | Event::Touch(touch::Event::FingerMoved { .. }) => {
                    state.is_hovering = cursor.is_over(layout.bounds());
                    state.cursor_position = cursor.position().unwrap_or_default();
                    state.cursor_location = cursor.position().map(locate).unwrap_or_default();

                    if state.is_dragging {
                        let _ = cursor.position().map(locate).map(change);

                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::WheelScrolled { delta })
                    if state.keyboard_modifiers.control() =>
                {
                    if cursor.is_over(layout.bounds()) {
                        let delta = match delta {
                            mouse::ScrollDelta::Lines { x: _, y } => y,
                            mouse::ScrollDelta::Pixels { x: _, y } => y,
                        };

                        if *delta < 0.0 {
                            let _ = change(decrement(current_value));
                        } else {
                            let _ = change(increment(current_value));
                        }

                        shell.capture_event();
                    }
                }
                Event::Keyboard(keyboard::Event::KeyPressed { key, .. }) => {
                    if cursor.is_over(layout.bounds()) {
                        match key {
                            Key::Named(key::Named::ArrowUp) => {
                                let _ = change(increment(current_value));
                            }
                            Key::Named(key::Named::ArrowDown) => {
                                let _ = change(decrement(current_value));
                            }
                            _ => (),
                        }

                        shell.capture_event();
                    }
                }
                Event::Keyboard(keyboard::Event::ModifiersChanged(modifiers)) => {
                    state.keyboard_modifiers = *modifiers;
                }
                _ => {}
            }
        };

        update();

        let current_status = if state.is_dragging {
            Status::Dragged
        } else if cursor.is_over(layout.bounds()) {
            Status::Hovered
        } else {
            Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.status = Some(current_status);
        } else if self.status.is_some_and(|status| status != current_status) {
            shell.request_redraw();
        }
    }

    fn draw(
        &self,
        _tree: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        _viewport: &Rectangle,
    ) {
        let bounds = layout.bounds();

        let style = theme.style(&self.class, self.status.unwrap_or(Status::Active));

        let (handle_width, handle_height, handle_border_radius) = match style.handle.shape {
            HandleShape::Circle { radius } => (radius * 2.0, radius * 2.0, radius.into()),
            HandleShape::Rectangle {
                width,
                border_radius,
            } => (f32::from(width), bounds.height, border_radius),
        };

        let value = self.value as f32;
        let (range_start, range_end) = {
            let (start, end) = self.range.clone().into_inner();

            (start as f32, end as f32)
        };

        let offset = if range_start >= range_end {
            0.0
        } else {
            (bounds.width - handle_width) * (value - range_start) / (range_end - range_start)
        };

        let rail_y = bounds.y + bounds.height / 2.0;

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x,
                    y: rail_y - style.rail.width / 2.0,
                    width: offset + handle_width / 2.0,
                    height: style.rail.width,
                },
                border: style.rail.border,
                ..renderer::Quad::default()
            },
            style.rail.backgrounds.0,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + offset + handle_width / 2.0,
                    y: rail_y - style.rail.width / 2.0,
                    width: bounds.width - offset - handle_width / 2.0,
                    height: style.rail.width,
                },
                border: style.rail.border,
                ..renderer::Quad::default()
            },
            style.rail.backgrounds.1,
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds: Rectangle {
                    x: bounds.x + offset,
                    y: rail_y - handle_height / 2.0,
                    width: handle_width,
                    height: handle_height,
                },
                border: Border {
                    radius: handle_border_radius,
                    width: style.handle.border_width,
                    color: style.handle.border_color,
                },
                ..renderer::Quad::default()
            },
            style.handle.background,
        );
    }

    fn mouse_interaction(
        &self,
        tree: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();
        let is_mouse_over = cursor.is_over(bounds);

        if state.is_dragging || is_mouse_over {
            mouse::Interaction::Pointer
        } else {
            mouse::Interaction::default()
        }
    }

    fn overlay<'a>(
        &'a mut self,
        state: &'a mut Tree,
        layout: Layout<'a>,
        renderer: &Renderer,
        _viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<advanced::overlay::Element<'a, Message, Theme, Renderer>> {
        let state = state.state.downcast_ref::<State>();

        if !state.is_hovering && !state.is_dragging {
            return None;
        }

        let cursor_percent =
            (state.cursor_location - *self.range.start()) / (self.range.end() - self.range.start());
        let image_size = self
            .thumbnails
            .first()
            .map(|img| renderer.measure_image(img))
            .unwrap_or_default();
        let image_index = (cursor_percent * self.thumbnails.len() as f64) as usize;
        let image_index = image_index.min(self.thumbnails.len().max(1) - 1);

        let position = (cursor_percent * self.duration.as_secs_f64()) as u64;
        let timestamp = format!(
            "{:02}:{:02}:{:02}",
            position as u64 / 3600,
            position as u64 % 3600 / 60,
            position as u64 % 60
        );

        let mut overlay = vec![];

        if let Some(image) = self.thumbnails.get(image_index).cloned() {
            overlay.push(advanced::overlay::Element::new(Box::new(
                ThumbnailOverlay {
                    position: layout.position() + translation,
                    content_bounds: layout.bounds(),
                    image,
                    image_size: iced::Size::new(
                        image_size.width as f32 / image_size.height as f32 * 100.0,
                        100.0,
                    ),
                    cursor_position: state.cursor_position,
                },
            )));
        }

        overlay.push(advanced::overlay::Element::new(Box::new(
            TimestampOverlay {
                position: layout.position() + translation,
                content_bounds: layout.bounds(),
                timestamp,
                size: iced::Size::new(80.0, 20.0),
                cursor_position: state.cursor_position,
                font: self.thumbnail_font,
            },
        )));

        Some(advanced::overlay::Group::with_children(overlay).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<VideoSlider<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
    Renderer: advanced::Renderer + image::Renderer + text::Renderer + 'a,
{
    fn from(
        slider: VideoSlider<'a, Message, Theme, Renderer>,
    ) -> Element<'a, Message, Theme, Renderer> {
        Element::new(slider)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
struct State {
    is_hovering: bool,
    cursor_position: Point,
    cursor_location: f64,
    is_dragging: bool,
    keyboard_modifiers: keyboard::Modifiers,
}

struct ThumbnailOverlay<Renderer: image::Renderer> {
    position: iced::Point,
    content_bounds: iced::Rectangle,
    image: Renderer::Handle,
    image_size: iced::Size,
    cursor_position: iced::Point,
}

impl<Message, Theme, Renderer> advanced::Overlay<Message, Theme, Renderer>
    for ThumbnailOverlay<Renderer>
where
    Renderer: image::Renderer + advanced::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: iced::Size) -> layout::Node {
        let translation = self.position - self.content_bounds.position();
        let position = iced::Vector::new(
            self.cursor_position.x - self.image_size.width / 2.0,
            self.position.y - self.image_size.height - 30.0,
        ) + translation;

        layout::Node::new(self.image_size).translate(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: advanced::mouse::Cursor,
    ) {
        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds: layout.bounds().expand(2.0),
                border: iced::Border::default().rounded(3.0),
                shadow: iced::Shadow {
                    color: iced::Color::BLACK.scale_alpha(1.2),
                    offset: iced::Vector::new(0.0, 1.0),
                    blur_radius: 20.0,
                },
                snap: false,
            },
            iced::Background::Color(iced::Color::WHITE.scale_alpha(0.5)),
        );

        let image = image::Image {
            handle: self.image.clone(),
            filter_method: image::FilterMethod::default(),
            rotation: iced::Radians(0.0),
            opacity: 1.0,
            snap: false,
        };

        renderer.draw_image(image, layout.bounds());
    }
}

struct TimestampOverlay<Renderer>
where
    Renderer: text::Renderer,
{
    position: iced::Point,
    content_bounds: iced::Rectangle,
    size: iced::Size,
    timestamp: String,
    cursor_position: iced::Point,
    font: Renderer::Font,
}

impl<Message, Theme, Renderer> advanced::Overlay<Message, Theme, Renderer>
    for TimestampOverlay<Renderer>
where
    Renderer: text::Renderer,
{
    fn layout(&mut self, _renderer: &Renderer, _bounds: iced::Size) -> layout::Node {
        let translation = self.position - self.content_bounds.position();
        let position = iced::Vector::new(
            self.cursor_position.x - self.size.width / 2.0,
            self.position.y - self.size.height - 5.0,
        ) + translation;

        layout::Node::new(self.size).translate(position)
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        _theme: &Theme,
        _style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        _cursor: advanced::mouse::Cursor,
    ) {
        renderer.fill_quad(
            advanced::renderer::Quad {
                bounds: layout.bounds(),
                border: iced::Border::default().rounded(3.0),
                shadow: Default::default(),
                snap: false,
            },
            iced::Background::Color(iced::Color::BLACK.scale_alpha(0.9)),
        );

        renderer.fill_text(
            text::Text {
                content: self.timestamp.clone(),
                bounds: self.size,
                size: 13.into(),
                line_height: Default::default(),
                font: self.font,
                align_x: iced::Alignment::Center.into(),
                align_y: iced::Alignment::Center.into(),
                shaping: text::Shaping::Basic,
                wrapping: text::Wrapping::None,
            },
            layout.bounds().center() + iced::Vector::new(0.0, 1.5),
            iced::Color::from_rgb8(210, 210, 210),
            layout.bounds(),
        );
    }
}
