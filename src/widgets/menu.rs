use iced::{
    Element, Event, Length, Point, Rectangle, Size, Vector,
    advanced::{
        self, Widget, layout, mouse, overlay,
        widget::{operation, tree},
    },
};

pub fn menu<'a, Message>(
    base: impl Into<Element<'a, Message>>,
    overlay: impl Into<Element<'a, Message>>,
) -> Menu<'a, Message> {
    Menu::new(base, overlay)
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Position {
    Top,
    Right,
    Bottom,
    Left,
}

pub struct Menu<'a, Message, Theme = iced::Theme, Renderer = iced::Renderer> {
    base: Element<'a, Message, Theme, Renderer>,
    overlay: Element<'a, Message, Theme, Renderer>,
    on_toggle: Option<Box<dyn Fn(bool) -> Message + 'a>>,
    position: Position,
    auto_close: bool,
}

impl<'a, Message, Theme, Renderer> Menu<'a, Message, Theme, Renderer> {
    pub fn new(
        base: impl Into<Element<'a, Message, Theme, Renderer>>,
        overlay: impl Into<Element<'a, Message, Theme, Renderer>>,
    ) -> Self {
        Self {
            base: base.into(),
            overlay: overlay.into(),
            on_toggle: None,
            position: Position::Top,
            auto_close: true,
        }
    }

    pub fn position(self, position: Position) -> Self {
        Self { position, ..self }
    }

    pub fn on_toggle(self, on_toggle: impl Fn(bool) -> Message + 'a) -> Self {
        Self {
            on_toggle: Some(Box::new(on_toggle)),
            ..self
        }
    }

    pub fn auto_close(self, auto_close: bool) -> Self {
        Self { auto_close, ..self }
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Menu<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn size(&self) -> Size<Length> {
        self.base.as_widget().size()
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State {
            is_open: false,
            overlay: false,
        })
    }

    fn children(&self) -> Vec<tree::Tree> {
        vec![tree::Tree::new(&self.base), tree::Tree::new(&self.overlay)]
    }

    fn diff(&self, tree: &mut tree::Tree) {
        tree.diff_children(&[&self.base, &self.overlay]);
    }

    fn layout(
        &self,
        tree: &mut advanced::widget::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        self.base
            .as_widget()
            .layout(&mut tree.children[0], renderer, limits)
    }

    fn draw(
        &self,
        tree: &advanced::widget::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &iced::Rectangle,
    ) {
        self.base.as_widget().draw(
            &tree.children[0],
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
        state: &tree::Tree,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        let interaction = self.base.as_widget().mouse_interaction(
            &state.children[0],
            layout,
            cursor,
            viewport,
            renderer,
        );

        if matches!(interaction, mouse::Interaction::None) && cursor.is_over(layout.bounds()) {
            mouse::Interaction::Pointer
        } else {
            interaction
        }
    }

    fn operate(
        &self,
        state: &mut tree::Tree,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn operation::Operation,
    ) {
        operation.container(None, layout.bounds(), &mut |operation| {
            self.base
                .as_widget()
                .operate(&mut state.children[0], layout, renderer, operation);
        });
    }

    fn update(
        &mut self,
        state: &mut advanced::widget::Tree,
        event: &iced::Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
        viewport: &iced::Rectangle,
    ) {
        self.base.as_widget_mut().update(
            &mut state.children[0],
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            viewport,
        );

        if shell.is_event_captured() {
            return;
        }

        let state = state.state.downcast_mut::<State>();

        if let Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) = event {
            if state.is_open && !state.overlay {
                state.is_open = false;
                if let Some(on_toggle) = &self.on_toggle {
                    shell.publish((on_toggle)(false));
                }
            } else if cursor.is_over(layout.bounds()) {
                state.is_open = true;
                if let Some(on_toggle) = &self.on_toggle {
                    shell.publish((on_toggle)(true));
                }
                shell.capture_event();
            }
        }
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: layout::Layout<'b>,
        renderer: &Renderer,
        viewport: &iced::Rectangle,
        translation: iced::Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let base_size = layout.bounds().size();
        let state = tree.state.downcast_mut::<State>();

        if state.is_open {
            Some(overlay::Element::new(Box::new(Overlay {
                tree: &mut tree.children[1],
                content: &mut self.overlay,
                state,
                point: layout.position() + translation,
                base_size,
                viewport: *viewport,
                position: self.position,
                auto_close: self.auto_close,
            })))
        } else {
            self.base.as_widget_mut().overlay(
                &mut tree.children[0],
                layout,
                renderer,
                viewport,
                translation,
            )
        }
    }
}

impl<'a, Message, Theme, Renderer> From<Menu<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Renderer: advanced::Renderer + 'a,
    Theme: 'a,
{
    fn from(value: Menu<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

struct State {
    is_open: bool,
    overlay: bool,
}

struct Overlay<'a, 'b, Message, Theme, Renderer> {
    tree: &'a mut tree::Tree,
    content: &'a mut Element<'b, Message, Theme, Renderer>,
    state: &'a mut State,
    point: Point,
    base_size: Size,
    viewport: Rectangle,
    position: Position,
    auto_close: bool,
}

impl<'a, 'b, Message, Theme, Renderer> overlay::Overlay<Message, Theme, Renderer>
    for Overlay<'a, 'b, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer,
{
    fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
        let limits = layout::Limits::new(Size::ZERO, bounds);
        let layout = self
            .content
            .as_widget_mut()
            .layout(self.tree, renderer, &limits)
            .move_to(self.point);

        let overlay_bounds = layout.bounds();

        let offset_x = match self.position {
            Position::Left => -overlay_bounds.width - 5.0,
            Position::Right => self.base_size.width + 5.0,
            _ => 0.0,
        };

        let offset_y = match self.position {
            Position::Top => -5.0 - overlay_bounds.height,
            Position::Bottom => self.base_size.height + 5.0,
            _ => 0.0,
        };

        layout.translate([offset_x, offset_y])
    }

    fn draw(
        &self,
        renderer: &mut Renderer,
        theme: &Theme,
        style: &advanced::renderer::Style,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
    ) {
        self.content.as_widget().draw(
            self.tree,
            renderer,
            theme,
            style,
            layout,
            cursor,
            &self.viewport,
        )
    }

    fn update(
        &mut self,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        clipboard: &mut dyn advanced::Clipboard,
        shell: &mut advanced::Shell<'_, Message>,
    ) {
        self.content.as_widget_mut().update(
            self.tree,
            event,
            layout,
            cursor,
            renderer,
            clipboard,
            shell,
            &self.viewport,
        );

        self.state.overlay = cursor.is_over(layout.bounds())
            && matches!(event, Event::Mouse(mouse::Event::ButtonPressed(_)));

        if !shell.is_event_captured()
            && cursor.is_over(layout.bounds())
            && matches!(event, Event::Mouse(mouse::Event::ButtonReleased(_)))
        {
            self.state.is_open = true;
        } else if shell.is_event_captured() && self.auto_close && cursor.is_over(layout.bounds()) {
            self.state.is_open = false
        }
    }

    fn operate(
        &mut self,
        layout: layout::Layout<'_>,
        renderer: &Renderer,
        operation: &mut dyn advanced::graphics::core::widget::Operation,
    ) {
        self.content
            .as_widget_mut()
            .operate(self.tree, layout, renderer, operation);
    }

    fn mouse_interaction(
        &self,
        layout: layout::Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
    ) -> mouse::Interaction {
        self.content.as_widget().mouse_interaction(
            self.tree,
            layout,
            cursor,
            &self.viewport,
            renderer,
        )
    }

    fn overlay<'c>(
        &'c mut self,
        layout: layout::Layout<'c>,
        renderer: &Renderer,
    ) -> Option<overlay::Element<'c, Message, Theme, Renderer>> {
        self.content.as_widget_mut().overlay(
            self.tree,
            layout,
            renderer,
            &self.viewport,
            Vector::ZERO,
        )
    }
}
