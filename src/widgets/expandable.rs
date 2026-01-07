use iced::{
    Element, Length, Pixels, Point, Rectangle, Size,
    advanced::{
        self, Widget,
        layout::{self, Node},
        overlay,
        widget::tree,
    },
    animation::{Animation, Easing},
    mouse,
    time::{Duration, Instant},
    window,
};

pub fn expandable<'a, Message>(
    root: impl Into<Element<'a, Message>>,
    content: impl Into<Element<'a, Message>>,
) -> Expandable<'a, Message> {
    Expandable::new(root, content)
}

pub struct Expandable<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    root: Element<'a, Message, Theme, Renderer>,
    content: Element<'a, Message, Theme, Renderer>,
    expanded: bool,
    spacing: f32,
    duration: Duration,
    easing: Easing,
    width: Length,
    height: Length,
    on_expand: Option<Box<dyn Fn(bool) -> Message + 'a>>,
}

impl<'a, Message, Theme, Renderer> Expandable<'a, Message, Theme, Renderer> {
    pub fn new(
        root: impl Into<Element<'a, Message, Theme, Renderer>>,
        content: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            root: root.into(),
            content: content.into(),
            expanded: false,
            spacing: 0.0,
            duration: Duration::from_millis(200),
            easing: Easing::EaseInOut,
            width: Length::Shrink,
            height: Length::Shrink,
            on_expand: None,
        }
    }

    pub fn expanded(mut self, expanded: bool) -> Self {
        self.expanded = expanded;
        self
    }

    pub fn spacing(mut self, spacing: impl Into<Pixels>) -> Self {
        self.spacing = spacing.into().0;
        self
    }

    pub fn duration(mut self, duration: Duration) -> Self {
        self.duration = duration;
        self
    }

    pub fn easing(mut self, easing: Easing) -> Self {
        self.easing = easing;
        self
    }

    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    pub fn height(mut self, height: impl Into<Length>) -> Self {
        self.height = height.into();
        self
    }

    pub fn on_expand(mut self, on_expand: impl Fn(bool) -> Message + 'a) -> Self {
        self.on_expand = Some(Box::new(on_expand));
        self
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Expandable<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn size(&self) -> iced::Size<Length> {
        Size::new(self.width, self.height)
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::new(self.expanded, self.duration, self.easing))
    }

    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.root), tree::Tree::new(&self.content)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        let state = tree.state.downcast_mut::<State>();
        state.expanded = self.expanded;
        state.go_mut(Instant::now());

        tree.diff_children(&[&self.root, &self.content])
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<State>();
        let factor = state
            .animation
            .interpolate_with(std::convert::identity, Instant::now());

        state.factor = factor;

        let spacing = self.spacing * factor;

        let root = self
            .root
            .as_widget_mut()
            .layout(&mut tree.children[0], renderer, limits);

        let root_size = root.size();

        let content = {
            let full = self
                .content
                .as_widget_mut()
                .layout(&mut tree.children[1], renderer, &limits)
                .size();

            let max_height = full.height * factor;

            self.content
                .as_widget_mut()
                .layout(
                    &mut tree.children[1],
                    renderer,
                    &limits.max_height(max_height),
                )
                .move_to(Point::new(0.0, root_size.height + spacing))
        };

        let content_size = content.size();

        let intrinsic = {
            let width = root_size.width.max(content_size.width);
            let height = root_size.height + spacing + content_size.height;

            Size::new(width, height)
        };

        let size = limits.resolve(self.width, self.height, intrinsic);

        Node::with_children(size, vec![root, content])
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &iced::advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: iced::advanced::mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };

        let state = tree.state.downcast_ref::<State>();

        let mut children = layout.children();

        let root_layout = children.next().expect("Missing root layout");

        if let Some(viewport) = root_layout.bounds().intersection(&viewport) {
            self.root.as_widget().draw(
                &tree.children[0],
                renderer,
                theme,
                style,
                root_layout,
                cursor,
                &viewport,
            );
        }

        if state.factor < 0.5 {
            return;
        }

        let content_layout = children.next().expect("Missing innter content layout");

        if let Some(viewport) = content_layout.bounds().intersection(&viewport) {
            self.content.as_widget().draw(
                &tree.children[1],
                renderer,
                theme,
                style,
                content_layout,
                cursor,
                &viewport,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &iced::Event,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let state = tree.state.downcast_mut::<State>();
        let mut children = layout.children();

        let root = children.next().expect("Missing root layout");

        self.root.as_widget_mut().update(
            &mut tree.children[0],
            event,
            root,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        if self.expanded {
            let content = children.next().expect("Missing content layout");

            self.content.as_widget_mut().update(
                &mut tree.children[1],
                event,
                content,
                cursor,
                renderer,
                clipboard,
                shell,
                viewport,
            );
        }

        if shell.is_event_captured() {
            return;
        }

        match event {
            iced::Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(root.bounds()) =>
            {
                state.expanded = !state.expanded;

                if let Some(on_expand) = self.on_expand.as_ref() {
                    shell.publish((on_expand)(state.expanded));
                }

                shell.capture_event();
                shell.request_redraw();
            }
            iced::Event::Window(window::Event::RedrawRequested(at)) => {
                state.go_mut(*at);
                shell.invalidate_layout();

                if state.animation.is_animating(*at) {
                    shell.request_redraw();
                }
            }
            _ => {}
        }
    }

    fn operate(
        &mut self,
        tree: &mut tree::Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::widget::Operation,
    ) {
        let mut children = layout.children();
        let root = children.next().expect("Missing root layout");

        self.root
            .as_widget_mut()
            .operate(&mut tree.children[0], root, renderer, operation);

        if self.expanded {
            let content = children.next().expect("Missing content layout");

            self.content.as_widget_mut().operate(
                &mut tree.children[1],
                content,
                renderer,
                operation,
            );
        }
    }

    fn mouse_interaction(
        &self,
        tree: &tree::Tree,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> advanced::mouse::Interaction {
        let mut children = layout.children();
        let root = children.next().expect("Missing root layout");

        if cursor.is_over(root.bounds()) {
            let interaction = self.root.as_widget().mouse_interaction(
                &tree.children[0],
                root,
                cursor,
                viewport,
                renderer,
            );

            if !matches!(interaction, mouse::Interaction::None) || self.on_expand.is_none() {
                interaction
            } else {
                mouse::Interaction::Pointer
            }
        } else {
            let content = children.next().expect("Missing content layout");
            self.content.as_widget().mouse_interaction(
                &tree.children[1],
                content,
                cursor,
                viewport,
                renderer,
            )
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: layout::Layout<'b>,
        renderer: &Renderer,
        viewport: &Rectangle,
        translation: iced::Vector,
    ) -> Option<advanced::overlay::Element<'b, Message, Theme, Renderer>> {
        let mut children = layout.children();

        let root = children.next().expect("Missing root layout");

        let (root_tree, rest) = tree.children.split_at_mut(1);

        let root = self.root.as_widget_mut().overlay(
            &mut root_tree[0],
            root,
            renderer,
            viewport,
            translation,
        );

        let content = if self.expanded {
            let content = children.next().expect("Missing content layout");

            self.content.as_widget_mut().overlay(
                &mut rest[0],
                content,
                renderer,
                viewport,
                translation,
            )
        } else {
            None
        };

        let overlays = [root, content].into_iter().flatten();

        Some(overlay::Group::with_children(overlays.collect()).overlay())
    }
}

impl<'a, Message, Theme, Renderer> From<Expandable<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a,
    Renderer: advanced::Renderer + 'a,
{
    fn from(value: Expandable<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

struct State {
    expanded: bool,
    animation: Animation<f32>,
    factor: f32,
}

impl State {
    fn new(expanded: bool, duration: Duration, easing: Easing) -> Self {
        State {
            expanded,
            animation: Animation::new(f32::from(expanded))
                .duration(duration)
                .easing(easing),
            factor: 0.0,
        }
    }

    fn go_mut(&mut self, now: Instant) {
        self.animation.go_mut(f32::from(self.expanded), now);
    }
}
