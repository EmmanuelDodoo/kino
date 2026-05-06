//! A hybrid modal and drawer widget with animated translations

use iced::{
    Element, Event, Length, Rectangle, Size, Transformation, Vector,
    advanced::{
        self, Shell, Widget,
        layout::{Layout, Node},
        mouse,
        renderer::Quad,
        widget::{self, tree},
    },
    animation::{Animation, Easing},
    color,
    time::{Duration, Instant},
    window,
};

pub fn modal<'a, Message, Theme, Renderer>(
    base: impl Into<Element<'a, Message, Theme, Renderer>>,
    content: impl Into<Element<'a, Message, Theme, Renderer>>,
) -> Modal<'a, Message, Theme, Renderer> {
    Modal::new(base, content)
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
/// The final position of a [`Modal`].
pub enum Position {
    #[default]
    /// Modal transitions from the left
    Left,
    /// Modal transitions from the right
    Right,
    /// Modal transitions from the top
    Top,
    /// Modal transitions from the bottom
    Bottom,
    /// Modal scales up from the center
    Center,
}

impl Position {
    pub const ALL: &[Self] = &[
        Self::Top,
        Self::Right,
        Self::Bottom,
        Self::Left,
        Self::Center,
    ];

    fn node_translation(&self, parent: Size, content: Size) -> Vector {
        let diff = (parent - content).max(Size::ZERO);
        let x_diff = diff.width;
        let y_diff = diff.height;

        match self {
            Self::Left => {
                let y = 0.5 * y_diff;
                Vector::new(0.0, y)
            }
            Self::Right => {
                let y = 0.5 * y_diff;

                Vector::new(x_diff, y)
            }
            Self::Top => {
                let x = x_diff * 0.5;
                Vector::new(x, 0.0)
            }
            Self::Bottom => {
                let x = x_diff * 0.5;
                Vector::new(x, y_diff)
            }
            Self::Center => {
                let x = x_diff * 0.5;
                let y = y_diff * 0.5;

                Vector::new(x, y)
            }
        }
    }
}

pub struct Modal<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    content: Element<'a, Message, Theme, Renderer>,
    base: Element<'a, Message, Theme, Renderer>,
    width: Length,
    height: Length,
    position: Position,
    duration: Duration,
    easing: Easing,
    toggle: bool,
    alpha: f32,
    on_blur: Option<Message>,
    on_toggle_complete: Option<Message>,
    passthrough: bool,
}

impl<'a, Message, Theme, Renderer> Modal<'a, Message, Theme, Renderer> {
    /// Creates a new modal with the given `base` element and modal content
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            content: content.into(),
            base: base.into(),
            width: Length::Fill,
            height: Length::Fill,
            position: Position::default(),
            duration: Duration::from_millis(150),
            easing: Easing::EaseInOut,
            toggle: true,
            alpha: 0.0,
            on_blur: None,
            on_toggle_complete: None,
            passthrough: false,
        }
    }

    /// Sets the width of the modal
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the height of the modal
    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    /// Sets the [`Position`] of the modal
    pub fn position(mut self, position: Position) -> Self {
        self.position = position;
        self
    }

    /// Sets the duration of the modal animation
    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    /// Sets the [`Easing`] of the modal animation
    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    /// Toggles the modal
    pub fn toggle(mut self, toggle: bool) -> Self {
        self.toggle = toggle;
        self
    }

    /// Sets the background blur when the modal is open
    pub fn blur_alpha(mut self, alpha: f32) -> Self {
        self.alpha = alpha;
        self
    }

    /// If true, events are passed through the modal to the base element
    /// when the modal is open
    pub fn passthrough(mut self, passthrough: bool) -> Self {
        self.passthrough = passthrough;
        self
    }

    /// Sets the postion to [`Position::Left`]
    pub fn left(self) -> Self {
        self.position(Position::Left)
    }

    /// Sets the postion to [`Position::Right`]
    pub fn right(self) -> Self {
        self.position(Position::Right)
    }

    /// Sets the postion to [`Position::Top`]
    pub fn top(self) -> Self {
        self.position(Position::Top)
    }

    /// Sets the postion to [`Position::Bottom`]
    pub fn bottom(self) -> Self {
        self.position(Position::Bottom)
    }

    /// Sets the postion to [`Position::Center`]
    pub fn center(self) -> Self {
        self.position(Position::Center)
    }

    /// Sets the message produced when the blur is clicked while the modal is
    /// open
    pub fn on_blur(mut self, message: Message) -> Self {
        self.on_blur = Some(message);
        self
    }

    pub fn on_toggle_complete(mut self, message: Message) -> Self {
        self.on_toggle_complete = Some(message);
        self
    }
}

struct State {
    animation: Option<Animation<bool>>,
    now: Instant,
    done: bool,
}

impl State {
    fn new() -> Self {
        let now = Instant::now();
        Self {
            animation: None,
            now: now,
            done: true,
        }
    }

    fn go_mut(&mut self, new_state: bool, duration: Duration, easing: Easing, now: Instant) {
        self.now = now;
        self.done = self.animation_value() != new_state;
        match self.animation.as_mut() {
            Some(animation) => animation.go_mut(new_state, self.now),
            None => {
                self.animation = Some(
                    Animation::new(!new_state)
                        .duration(duration)
                        .easing(easing)
                        .go(new_state, self.now),
                );
            }
        }
    }

    fn is_animating(&self, now: Instant) -> bool {
        self.animation
            .as_ref()
            .map(|animation| animation.is_animating(now))
            .unwrap_or_default()
    }

    fn translation(&self, position: Position, content: Rectangle) -> Vector {
        let Some(animation) = &self.animation else {
            return Vector::ZERO;
        };

        match position {
            Position::Left => {
                let interpolation = animation.interpolate(-1.1, 0.1, self.now);
                let interpolation = interpolation.clamp(-1.0, 0.0);
                let diff = content.width * interpolation;

                Vector::new(diff, 0.0)
            }
            Position::Right => {
                let interpolation = animation.interpolate(1.1, -0.1, self.now);
                let interpolation = interpolation.clamp(0.0, 1.0);
                let diff = content.width * interpolation;

                Vector::new(diff, 0.0)
            }
            Position::Top => {
                let interpolation = animation.interpolate(-1.1, 0.1, self.now);
                let interpolation = interpolation.clamp(-1.0, 0.0);
                let diff = content.height * interpolation;

                Vector::new(0.0, diff)
            }
            Position::Bottom => {
                let interpolation = animation.interpolate(1.1, -0.1, self.now);
                let interpolation = interpolation.clamp(0.0, 1.0);
                let diff = content.height * interpolation;

                Vector::new(0.0, diff)
            }
            Position::Center => {
                let interpolation = animation.interpolate(1.1, -0.1, self.now);
                let interpolation = interpolation.clamp(0.0, 1.0);
                let position = content.position();

                let x_diff = position.x + (content.width * 0.5);
                let y_diff = position.y + (content.height * 0.5);

                let x = x_diff * interpolation;
                let y = y_diff * interpolation;

                Vector::new(x, y)
            }
        }
    }

    fn scale(&self, position: Position) -> f32 {
        let Some(animation) = &self.animation else {
            return 1.0;
        };

        match position {
            Position::Center => {
                let interpolation = animation.interpolate(-0.1, 1.1, self.now);

                interpolation.clamp(0.0, 1.0)
            }
            _ => 1.0,
        }
    }

    fn alpha(&self, alpha: f32) -> f32 {
        self.animation
            .as_ref()
            .map(|animation| animation.interpolate(0.0, alpha, self.now))
            .unwrap_or(alpha)
    }

    fn animation_value(&self) -> bool {
        self.animation
            .as_ref()
            .map(|anim| anim.value())
            .unwrap_or_default()
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Modal<'a, Message, Theme, Renderer>
where
    Message: Clone,
    Renderer: advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new())
    }

    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.base), tree::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        let state = tree.state.downcast_mut::<State>();

        if self.toggle != state.animation_value() {
            state.go_mut(self.toggle, self.duration, self.easing, Instant::now());
        }

        tree.diff_children(&[&self.base, &self.content]);
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &advanced::layout::Limits,
    ) -> advanced::layout::Node {
        let limits = limits.width(self.width).height(self.height);

        let base_layout =
            self.base
                .as_widget_mut()
                .layout(&mut tree.children[0], renderer, &limits);

        let content_layout =
            self.content
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &limits);

        let content_size = content_layout.size();

        let size = limits.resolve(self.width, self.height, Size::INFINITE);

        let position =
            content_layout.bounds().position() + self.position.node_translation(size, content_size);

        let content_layout = content_layout.move_to(position);

        let node = Node::with_children(size, vec![base_layout, content_layout]);

        node
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: advanced::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_ref::<State>();
        let bounds = layout.bounds();

        let Some(viewport) = bounds.intersection(viewport) else {
            return;
        };

        self.base.as_widget().draw(
            &tree.children[0],
            renderer,
            theme,
            style,
            layout.child(0),
            cursor,
            &viewport,
        );

        if !state.done || state.animation_value() {
            let alpha = state.alpha(self.alpha);

            renderer.fill_quad(
                Quad {
                    bounds,
                    ..Default::default()
                },
                color!(0, 0, 0, alpha),
            );

            let content_layout = layout.child(1);

            let translation = state.translation(self.position, content_layout.bounds());

            let translation = Transformation::translate(translation.x, translation.y);
            let scale = Transformation::scale(state.scale(self.position));

            let transformation = translation * scale;

            renderer.with_layer(viewport, |renderer| {
                renderer.with_transformation(transformation, |renderer| {
                    self.content.as_widget().draw(
                        &tree.children[1],
                        renderer,
                        theme,
                        style,
                        content_layout,
                        cursor,
                        &viewport,
                    );
                });
            });
        }
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &iced::Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let content_layout = layout.child(1);

        if state.animation_value() {
            self.content.as_widget_mut().update(
                &mut tree.children[1],
                event,
                content_layout,
                cursor,
                renderer,
                shell,
                viewport,
            );

            let interactive_event = !matches!(event, iced::Event::Window(_));
            let passthrough = !(interactive_event && cursor.is_over(content_layout.bounds()));

            if self.passthrough && !shell.is_event_captured() && passthrough {
                self.base.as_widget_mut().update(
                    &mut tree.children[0],
                    event,
                    layout.child(0),
                    cursor,
                    renderer,
                    shell,
                    viewport,
                );
            }
        } else {
            self.base.as_widget_mut().update(
                &mut tree.children[0],
                event,
                layout.child(0),
                cursor,
                renderer,
                shell,
                viewport,
            );
        }

        match event {
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if state.animation_value() && !shell.is_event_captured() =>
            {
                let Some(message) = self.on_blur.clone() else {
                    return;
                };

                if cursor.is_over(layout.bounds()) && !cursor.is_over(content_layout.bounds()) {
                    shell.publish(message);
                    shell.capture_event();
                }
            }
            Event::Window(window::Event::RedrawRequested(now)) => {
                if state.is_animating(*now) {
                    state.done = false;
                    state.now = *now;
                    shell.request_redraw();
                } else if !state.done {
                    state.done = true;
                    if let Some(message) = self.on_toggle_complete.clone() {
                        shell.publish(message);
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let state = tree.state.downcast_ref::<State>();
        let content_layout = layout.child(1);

        if cursor.is_over(content_layout.bounds()) && state.animation_value() {
            self.content.as_widget().mouse_interaction(
                &tree.children[1],
                content_layout,
                cursor,
                viewport,
                renderer,
            )
        } else if !state.animation_value() || self.passthrough {
            self.base.as_widget().mouse_interaction(
                &tree.children[0],
                layout.child(0),
                cursor,
                viewport,
                renderer,
            )
        } else {
            mouse::Interaction::Idle
        }
    }

    fn operate(
        &mut self,
        tree: &mut tree::Tree,
        layout: Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_ref::<State>();

        if state.animation_value() {
            self.content.as_widget_mut().operate(
                &mut tree.children[1],
                layout.child(1),
                renderer,
                operation,
            );
        } else {
            self.base.as_widget_mut().operate(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                operation,
            );
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let state = tree.state.downcast_ref::<State>();

        if state.animation_value() {
            if self.passthrough || !state.done {
                let overlays = [&mut self.base, &mut self.content]
                    .into_iter()
                    .zip(tree.children.iter_mut().zip(layout.children()))
                    .flat_map(|(element, (tree, layout))| {
                        element.as_widget_mut().overlay(
                            tree,
                            layout,
                            renderer,
                            viewport,
                            translation,
                        )
                    })
                    .collect();

                let group = advanced::overlay::Group::with_children(overlays);

                Some(group.overlay())
            } else {
                self.content.as_widget_mut().overlay(
                    &mut tree.children[1],
                    layout.child(1),
                    renderer,
                    viewport,
                    translation,
                )
            }
        } else if self.passthrough {
            self.base.as_widget_mut().overlay(
                &mut tree.children[0],
                layout.child(0),
                renderer,
                viewport,
                translation,
            )
        } else {
            None
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Modal<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + 'a,
    Message: Clone + 'a,
    Theme: 'a,
{
    fn from(value: Modal<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}
