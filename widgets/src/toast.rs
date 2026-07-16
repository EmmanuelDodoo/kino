// Code derived from Iced Github repo
//
use std::fmt::Display;

use iced::{
    Element, Length, Point, Rectangle, Renderer, Size, Vector,
    advanced::{
        Layout, Shell, layout, overlay, renderer,
        widget::{self, Operation, Tree},
    },
    alignment::{Alignment, Vertical},
    event::Event,
    mouse,
    time::{self, Duration, Instant},
    widget::{button, container, row, space, text},
    window,
};

use iced::advanced::Widget;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
/// The status for a [`Toast`].
pub enum Status {
    #[default]
    Info,
    Warn,
    Success,
    Error,
}

impl Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Info => "Info",
            Status::Error => "Error",
            Status::Success => "Success",
            Status::Warn => "Warning",
        }
        .fmt(f)
    }
}

#[derive(Debug, Clone, Default)]
/// A message toast
pub struct Toast {
    pub message: String,
    pub status: Status,
}

impl Toast {
    pub fn new(message: impl Into<String>, status: Status) -> Self {
        Self {
            message: message.into(),
            status,
        }
    }

    pub fn success(message: impl Into<String>) -> Self {
        Self::new(message, Status::Success)
    }

    pub fn error(message: impl Into<String>) -> Self {
        Self::new(message, Status::Error)
    }

    pub fn warn(message: impl Into<String>) -> Self {
        Self::new(message, Status::Warn)
    }

    pub fn info(message: impl Into<String>) -> Self {
        Self::new(message, Status::Info)
    }
}

#[derive(Debug, Clone, Copy)]
/// The settings for a [`Manager`]
pub struct Settings {
    pub text_size: f32,
    pub close_icon: char,
    pub close_size: f32,
    pub close_font: iced::Font,
}

pub fn manager<'a, Message, Theme>(
    content: impl Into<Element<'a, Message, Theme>>,
    toasts: &'a [Toast],
    on_close: impl Fn(usize) -> Message + 'a,
    settings: Settings,
) -> Manager<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a + Catalog,
{
    Manager::new(content, toasts, on_close, settings)
}

/// A widget for displaying toasts on top of content
pub struct Manager<'a, Message, Theme = iced::Theme>
where
    Message: 'a + Clone,
    Theme: 'a + Catalog,
{
    content: Element<'a, Message, Theme>,
    toasts: Vec<Element<'a, Message, Theme>>,
    timeout: u64,
    on_close: Box<dyn Fn(usize) -> Message + 'a>,
}

impl<'a, Message, Theme> Manager<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a + Catalog,
{
    /// Creates a new [`Manager`].
    pub fn new(
        content: impl Into<Element<'a, Message, Theme>>,
        toasts: &'a [Toast],
        on_close: impl Fn(usize) -> Message + 'a,
        settings: Settings,
    ) -> Self {
        let toasts = toasts
            .iter()
            .enumerate()
            .map(|(index, toast)| {
                let class = Theme::toast_status(toast.status);
                let side = container(space())
                    .class(class)
                    .width(5.0)
                    .height(Length::Fill);

                let content = text(toast.message.as_str()).size(settings.text_size);

                let class = Theme::button_text();
                let close = button(
                    text(settings.close_icon)
                        .font(settings.close_font)
                        .size(settings.close_size),
                )
                .on_press((on_close)(index))
                .class(class);

                container(
                    row!(side, content, space::horizontal(), close)
                        .width(Length::Shrink)
                        .align_y(Vertical::Center)
                        .spacing(5),
                )
                .class(Theme::container_rounded())
                .clip(true)
                .width(Length::Fit.max(500))
                .height(Length::Shrink)
                .padding([5.0, 5.0])
                .into()
            })
            .collect();

        Self {
            content: content.into(),
            toasts,
            timeout: 3,
            on_close: Box::new(on_close),
        }
    }

    /// Sets the timeout for toasts
    pub fn timeout(mut self, seconds: u64) -> Self {
        self.timeout = seconds;
        self
    }
}

impl<'a, Message, Theme> Widget<Message, Theme, Renderer> for Manager<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a + Catalog,
{
    fn size(&self) -> Size<Length> {
        self.content.as_widget().size()
    }

    fn layout(
        &mut self,
        tree: &mut Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.content
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn tag(&self) -> widget::tree::Tag {
        struct Marker;
        widget::tree::Tag::of::<Marker>()
    }

    fn state(&self) -> widget::tree::State {
        widget::tree::State::new(Vec::<Option<Instant>>::new())
    }

    fn diff(&mut self, tree: &mut Tree) {
        let instants = tree.state.downcast_mut::<Vec<Option<Instant>>>();

        // Invalidating removed instants to None allows us to remove
        // them here so that diffing for removed / new toast instants
        // is accurate
        instants.retain(Option::is_some);

        match (instants.len(), self.toasts.len()) {
            (old, new) if old > new => {
                instants.truncate(new);
            }
            (old, new) if old < new => {
                instants.extend(std::iter::repeat_n(
                    Some(Instant::now()),
                    new.saturating_sub(old),
                ));
            }
            _ => {}
        }

        tree.diff_children(
            &mut std::iter::once(&mut self.content)
                .chain(&mut self.toasts)
                .collect::<Vec<_>>(),
        );
    }

    fn operate(
        &mut self,
        state: &mut Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.content.as_widget_mut().operate(
                &mut state.children[0],
                layout,
                renderer,
                operation,
            );
        });
    }

    fn update(
        &mut self,
        state: &mut Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        self.content.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            shell,
            viewport,
        )
    }

    fn draw(
        &self,
        state: &Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        self.content.as_widget().draw(
            &state.children[0],
            renderer,
            theme,
            style,
            layout,
            cursor,
            viewport,
        );
    }

    fn mouse_interaction(
        &self,
        state: &Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        )
    }

    fn overlay<'b>(
        &'b mut self,
        state: &'b mut Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let instants = state.state.downcast_mut::<Vec<Option<Instant>>>();

        let (content_state, toasts_state) = state.children.split_at_mut(1);

        let content = self.content.as_widget_mut().overlay(
            &mut content_state[0],
            layout,
            renderer,
            viewport,
            translation,
        );

        let toasts = (!self.toasts.is_empty()).then(|| {
            overlay::Element::new(Box::new(Overlay {
                position: layout.bounds().position() + translation,
                viewport: *viewport,
                toasts: &mut self.toasts,
                state: toasts_state,
                instants,
                on_close: &self.on_close,
                timeout_secs: self.timeout,
            }))
        });
        let overlays = content.into_iter().chain(toasts).collect::<Vec<_>>();

        (!overlays.is_empty()).then(|| overlay::Group::with_children(overlays).overlay())
    }
}

impl<'a, Message, Theme> From<Manager<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: 'a + Clone,
    Theme: 'a + Catalog,
{
    fn from(value: Manager<'a, Message, Theme>) -> Self {
        Element::new(value)
    }
}

struct Overlay<'a, 'b, Message, Theme> {
    position: Point,
    viewport: Rectangle,
    toasts: &'b mut [Element<'a, Message, Theme>],
    state: &'b mut [Tree],
    instants: &'b mut [Option<Instant>],
    on_close: &'b dyn Fn(usize) -> Message,
    timeout_secs: u64,
}

impl<Message, Theme> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'_, '_, Message, Theme>
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);

        layout::flex::resolve(
            layout::flex::Axis::Vertical,
            renderer,
            &limits,
            Length::Fill,
            Length::Fill,
            10.into(),
            10.0,
            Alignment::End,
            self.toasts,
            self.state,
        )
        .translate(Vector::new(self.position.x, self.position.y))
    }

    fn update(
        &mut self,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
    ) {
        if let Event::Window(window::Event::RedrawRequested(now)) = &event {
            // self.instants.iter_mut().zip(self.state.iter_mut()).enumerate().for_each(f);
            self.instants
                .iter_mut()
                .zip(layout.children())
                .enumerate()
                .for_each(|(index, (maybe_instant, layout))| {
                    if let Some(instant) = maybe_instant.as_mut() {
                        let remaining =
                            time::seconds(self.timeout_secs).saturating_sub(instant.elapsed());

                        if remaining == Duration::ZERO && !cursor.is_over(layout.bounds()) {
                            maybe_instant.take();
                            shell.publish((self.on_close)(index));
                        } else {
                            shell.request_redraw_at(*now + remaining);
                        }
                    }
                });
        }

        let viewport = layout.bounds();

        for (((child, state), layout), instant) in self
            .toasts
            .iter_mut()
            .zip(self.state.iter_mut())
            .zip(layout.children())
            .zip(self.instants.iter_mut())
        {
            let mut local_messages = vec![];
            let mut local_shell = shell.local(&mut local_messages);

            child.as_widget_mut().update(
                state,
                event,
                layout,
                cursor,
                renderer,
                &mut local_shell,
                &viewport,
            );

            if !local_shell.is_empty() {
                instant.take();
            }

            shell.merge(local_shell, std::convert::identity);
        }
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        let viewport = layout.bounds();

        for ((child, state), layout) in self
            .toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
        {
            child
                .as_widget()
                .draw(state, renderer, theme, style, layout, cursor, &viewport);
        }
    }

    fn operate(
        &mut self,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        operation.container(None, layout.bounds());
        operation.traverse(&mut |operation| {
            self.toasts
                .iter_mut()
                .zip(self.state.iter_mut())
                .zip(layout.children())
                .for_each(|((child, state), layout)| {
                    child
                        .as_widget_mut()
                        .operate(state, layout, renderer, operation);
                });
        });
    }

    fn mouse_interaction(
        &self,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.toasts
            .iter()
            .zip(self.state.iter())
            .zip(layout.children())
            .map(|((child, state), layout)| {
                child
                    .as_widget()
                    .mouse_interaction(state, layout, cursor, &self.viewport, renderer)
                    .max(if cursor.is_over(layout.bounds()) {
                        mouse::Interaction::Idle
                    } else {
                        Default::default()
                    })
            })
            .max()
            .unwrap_or_default()
    }
}

pub trait Catalog: container::Catalog + button::Catalog + text::Catalog {
    fn toast_status<'a>(status: Status) -> <Self as container::Catalog>::Class<'a>;

    fn button_text<'a>() -> <Self as button::Catalog>::Class<'a>;

    fn container_rounded<'a>() -> <Self as container::Catalog>::Class<'a>;
}

impl Catalog for iced::Theme {
    fn toast_status<'a>(status: Status) -> <Self as container::Catalog>::Class<'a> {
        match status {
            Status::Info => Box::new(container::primary),
            Status::Success => Box::new(container::success),
            Status::Warn => Box::new(container::warning),
            Status::Error => Box::new(container::danger),
        }
    }

    fn container_rounded<'a>() -> <Self as container::Catalog>::Class<'a> {
        Box::new(|theme: &iced::Theme| {
            let default = container::rounded_box(theme);
            let border = default
                .border
                .rounded(5)
                .width(0.5)
                .color(default.text_color.unwrap_or_default());

            container::Style { border, ..default }
        })
    }

    fn button_text<'a>() -> <Self as button::Catalog>::Class<'a> {
        Box::new(button::text)
    }
}
