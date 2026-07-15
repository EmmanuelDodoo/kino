//! A version of [`iced::widget::ComboBox`] customised for font selection
use iced::{
    Element, Event, Length, Padding, Pixels, Rectangle, Renderer, Size, Vector,
    advanced::{
        Shell, Widget,
        layout::{self, Layout},
        mouse, overlay, renderer, text,
        widget::tree,
    },
    font::Family,
    keyboard::{self, key},
    time::Instant,
};

use iced::widget::combo_box::Catalog;
use iced::widget::text_input::{self, TextInput};

use std::cell::RefCell;

pub fn font_selection<'a, Message, Theme>(
    state: &'a State,
    placeholder: &str,
    selection: Option<Family>,
    on_selected: impl Fn(Family) -> Message + 'a,
) -> FontSelection<'a, Message, Theme>
where
    Theme: Catalog,
{
    FontSelection::new(state, placeholder, selection, on_selected)
}

/// A version of [`iced::widget::ComboBox`] customised for font selection
pub struct FontSelection<'a, Message, Theme = iced::Theme>
where
    Theme: Catalog,
{
    state: &'a State,
    text_input: TextInput<'a, TextInputEvent, Theme, Renderer>,
    selection: text_input::Value,
    on_selected: Box<dyn Fn(Family) -> Message + 'a>,
    on_option_hovered: Option<Box<dyn Fn(Family) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    padding: Padding,
    size: Option<f32>,
    shaping: text::Shaping,
    ellipsis: text::Ellipsis,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    menu_height: Length,
}

impl<'a, Message, Theme> FontSelection<'a, Message, Theme>
where
    Theme: Catalog,
{
    /// Creates a new [`FontSelection`] with the given list of options, a placeholder,
    /// the current selected value, and the message to produce when an option is
    /// selected.
    pub fn new(
        state: &'a State,
        placeholder: &str,
        selection: Option<Family>,
        on_selected: impl Fn(Family) -> Message + 'a,
    ) -> Self {
        let text_input = TextInput::new(placeholder, &state.value())
            .on_input(TextInputEvent::TextChanged)
            .class(Theme::default_input())
            .font(selection.unwrap_or_default().into());

        let selection = selection
            .map(|family| family.to_string())
            .unwrap_or_default();

        Self {
            state,
            text_input,
            selection: text_input::Value::new(&selection),
            on_selected: Box::new(on_selected),
            on_option_hovered: None,
            on_input: None,
            on_open: None,
            on_close: None,
            padding: text_input::DEFAULT_PADDING,
            size: None,
            shaping: text::Shaping::default(),
            ellipsis: text::Ellipsis::End,
            menu_class: <Theme as Catalog>::default_menu(),
            menu_height: Length::Shrink,
        }
    }

    /// Sets the message that should be produced when some text is typed into
    /// the [`TextInput`] of the [`FontSelection`].
    pub fn on_input(mut self, on_input: impl Fn(String) -> Message + 'a) -> Self {
        self.on_input = Some(Box::new(on_input));
        self
    }

    /// Sets the message that will be produced when an option of the
    /// [`FontSelection`] is hovered using the arrow keys.
    pub fn on_option_hovered(mut self, on_option_hovered: impl Fn(Family) -> Message + 'a) -> Self {
        self.on_option_hovered = Some(Box::new(on_option_hovered));
        self
    }

    /// Sets the message that will be produced when the  [`FontSelection`] is
    /// opened.
    pub fn on_open(mut self, message: Message) -> Self {
        self.on_open = Some(message);
        self
    }

    /// Sets the message that will be produced when the outside area
    /// of the [`FontSelection`] is pressed.
    pub fn on_close(mut self, message: Message) -> Self {
        self.on_close = Some(message);
        self
    }

    /// Sets the [`Padding`] of the [`FontSelection`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self.text_input = self.text_input.padding(self.padding);
        self
    }

    /// Sets the [`text_input::Icon`] of the [`FontSelection`].
    pub fn icon(mut self, icon: text_input::Icon<iced::Font>) -> Self {
        self.text_input = self.text_input.icon(icon);
        self
    }

    /// Sets the text sixe of the [`FontSelection`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        let size = size.into();

        self.text_input = self.text_input.size(size);
        self.size = Some(size.0);

        self
    }

    /// Sets the [`LineHeight`] of the [`FontSelection`].
    pub fn line_height(self, line_height: impl Into<text::LineHeight>) -> Self {
        Self {
            text_input: self.text_input.line_height(line_height),
            ..self
        }
    }

    /// Sets the width of the [`FontSelection`].
    pub fn width(self, width: impl Into<Length>) -> Self {
        Self {
            text_input: self.text_input.width(width),
            ..self
        }
    }

    /// Sets the height of the menu of the [`FontSelection`].
    pub fn menu_height(mut self, menu_height: impl Into<Length>) -> Self {
        self.menu_height = menu_height.into();
        self
    }

    /// Sets the [`text::Shaping`] strategy of the [`FontSelection`].
    pub fn shaping(mut self, shaping: text::Shaping) -> Self {
        self.shaping = shaping;
        self
    }

    /// Sets the [`text::Ellipsis`] strategy of the [`FontSelection`].
    pub fn ellipsis(mut self, ellipsis: text::Ellipsis) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    /// Sets the style of the input of the [`FontSelection`].
    #[must_use]
    pub fn input_style(
        mut self,
        style: impl Fn(&Theme, text_input::Status) -> text_input::Style + 'a,
    ) -> Self
    where
        <Theme as text_input::Catalog>::Class<'a>: From<text_input::StyleFn<'a, Theme>>,
    {
        self.text_input = self.text_input.style(style);
        self
    }

    /// Sets the style of the menu of the [`FontSelection`].
    #[must_use]
    pub fn menu_style(mut self, style: impl Fn(&Theme) -> menu::Style + 'a) -> Self
    where
        <Theme as menu::Catalog>::Class<'a>: From<menu::StyleFn<'a, Theme>>,
    {
        self.menu_class = (Box::new(style) as menu::StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the input of the [`FontSelection`].
    #[must_use]
    pub fn input_class(
        mut self,
        class: impl Into<<Theme as text_input::Catalog>::Class<'a>>,
    ) -> Self {
        self.text_input = self.text_input.class(class);
        self
    }

    /// Sets the style class of the menu of the [`FontSelection`].
    #[must_use]
    pub fn menu_class(mut self, class: impl Into<<Theme as menu::Catalog>::Class<'a>>) -> Self {
        self.menu_class = class.into();
        self
    }
}

/// The local state of a [`FontSelection`].
#[derive(Debug, Clone)]
pub struct State {
    options: Vec<Family>,
    inner: RefCell<Inner>,
}

#[derive(Debug, Clone)]
struct Inner {
    value: String,
    option_matchers: Vec<String>,
    filtered_options: Filtered,
}

#[derive(Debug, Clone)]
struct Filtered {
    options: Vec<Family>,
    updated: Instant,
}

impl State {
    /// Creates a new [`State`] for a [`FontSelection`] with the given list of options.
    pub fn new(options: Vec<Family>) -> Self {
        Self::with_selection(options, None)
    }

    /// Creates a new [`State`] for a [`FontSelection`] with the given list of options
    /// and selected value.
    pub fn with_selection(options: Vec<Family>, selection: Option<Family>) -> Self {
        let value = selection
            .map(|family| family.to_string())
            .unwrap_or_default();

        // Pre-build "matcher" strings ahead of time so that search is fast
        let option_matchers = build_matchers(&options);

        let filtered_options = Filtered::new(
            search(&options, &option_matchers, &value)
                .cloned()
                .collect(),
        );

        Self {
            options,
            inner: RefCell::new(Inner {
                value,
                option_matchers,
                filtered_options,
            }),
        }
    }

    /// Returns the options of the [`State`].
    ///
    /// These are the options provided when the [`State`]
    /// was constructed with [`State::new`].
    pub fn options(&self) -> &[Family] {
        &self.options
    }

    /// Pushes a new option to the [`State`].
    pub fn push(&mut self, new_option: Family) {
        let mut inner = self.inner.borrow_mut();

        inner.option_matchers.push(build_matcher(&new_option));
        self.options.push(new_option);

        inner.filtered_options = Filtered::new(
            search(&self.options, &inner.option_matchers, &inner.value)
                .cloned()
                .collect(),
        );
    }

    /// Returns ownership of the options of the [`State`].
    pub fn into_options(self) -> Vec<Family> {
        self.options
    }

    fn value(&self) -> String {
        let inner = self.inner.borrow();

        inner.value.clone()
    }

    fn with_inner<O>(&self, f: impl FnOnce(&Inner) -> O) -> O {
        let inner = self.inner.borrow();

        f(&inner)
    }

    fn with_inner_mut(&self, f: impl FnOnce(&mut Inner)) {
        let mut inner = self.inner.borrow_mut();

        f(&mut inner);
    }

    fn sync_filtered_options(&self, options: &mut Filtered) {
        let inner = self.inner.borrow();

        inner.filtered_options.sync(options);
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl Filtered {
    fn new(options: Vec<Family>) -> Self {
        Self {
            options,
            updated: Instant::now(),
        }
    }

    fn empty() -> Self {
        Self {
            options: vec![],
            updated: Instant::now(),
        }
    }

    fn update(&mut self, options: Vec<Family>) {
        self.options = options;
        self.updated = Instant::now();
    }

    fn sync(&self, other: &mut Filtered) {
        if other.updated != self.updated {
            *other = self.clone();
        }
    }
}

struct Menu {
    menu: menu::State,
    hovered_option: Option<usize>,
    new_selection: Option<Family>,
    filtered_options: Filtered,
}

#[derive(Debug, Clone)]
enum TextInputEvent {
    TextChanged(String),
}

impl<Message, Theme> Widget<Message, Theme, Renderer> for FontSelection<'_, Message, Theme>
where
    Message: Clone,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Widget::<TextInputEvent, Theme, Renderer>::size(&self.text_input)
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<<Renderer as text::Renderer>::Paragraph>>();

            text_input_state.is_focused()
        };

        self.text_input.layout(
            &mut tree.children[0],
            renderer,
            limits,
            (!is_focused).then_some(&self.selection),
        )
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Menu>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Menu {
            menu: menu::State::new(),
            filtered_options: Filtered::empty(),
            hovered_option: Some(0),
            new_selection: None,
        })
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        tree.diff_children(&mut [&mut self.text_input as &mut dyn Widget<_, _, _>]);
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        viewport: &Rectangle,
    ) {
        let menu = tree.state.downcast_mut::<Menu>();

        let started_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<<Renderer as text::Renderer>::Paragraph>>();

            text_input_state.is_focused()
        };
        // This is intended to check whether or not the message buffer was empty,
        // since `Shell` does not expose such functionality.
        let mut published_message_to_shell = false;

        // Create a new list of local messages
        let mut local_messages = Vec::new();
        let mut local_shell = shell.local(&mut local_messages);

        // Provide it to the widget
        self.text_input.update(
            &mut tree.children[0],
            event,
            layout,
            cursor,
            renderer,
            &mut local_shell,
            viewport,
        );

        if local_shell.is_event_captured() {
            shell.capture_event();
        }

        shell.request_redraw_at(local_shell.redraw_request());
        shell.request_input_method(local_shell.input_method());
        shell.clipboard_mut().merge(local_shell.clipboard_mut());

        // Then finally react to them here
        for message in local_messages {
            let TextInputEvent::TextChanged(new_value) = message;

            if let Some(on_input) = &self.on_input {
                shell.publish((on_input)(new_value.clone()));
            }

            // Couple the filtered options with the `FontSelection`
            // value and only recompute them when the value changes,
            // instead of doing it in every `view` call
            self.state.with_inner_mut(|state| {
                menu.hovered_option = Some(0);
                state.value = new_value;

                state.filtered_options.update(
                    search(&self.state.options, &state.option_matchers, &state.value)
                        .cloned()
                        .collect(),
                );
            });
            shell.invalidate_layout();
            shell.request_redraw();
        }

        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<<Renderer as text::Renderer>::Paragraph>>();

            text_input_state.is_focused()
        };

        if is_focused {
            self.state.with_inner(|state| {
                if !started_focused && let Some(on_option_hovered) = &mut self.on_option_hovered {
                    let hovered_option = menu.hovered_option.unwrap_or(0);

                    if let Some(option) = state.filtered_options.options.get(hovered_option) {
                        shell.publish(on_option_hovered(option.clone()));
                        published_message_to_shell = true;
                    }
                }

                if let Event::Keyboard(keyboard::Event::KeyPressed {
                    key: keyboard::Key::Named(named_key),
                    modifiers,
                    ..
                }) = event
                {
                    let shift_modifier = modifiers.shift();
                    match (named_key, shift_modifier) {
                        (key::Named::Enter, _) => {
                            if let Some(index) = &menu.hovered_option
                                && let Some(option) = state.filtered_options.options.get(*index)
                            {
                                menu.new_selection = Some(option.clone());
                            }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        (key::Named::ArrowUp, _) | (key::Named::Tab, true) => {
                            if let Some(index) = &mut menu.hovered_option {
                                if *index == 0 {
                                    *index = state.filtered_options.options.len().saturating_sub(1);
                                } else {
                                    *index = index.saturating_sub(1);
                                }
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_option_hovered) = &mut self.on_option_hovered
                                && let Some(option) = menu
                                    .hovered_option
                                    .and_then(|index| state.filtered_options.options.get(index))
                            {
                                // Notify the selection
                                shell.publish((on_option_hovered)(option.clone()));
                                published_message_to_shell = true;
                            }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        (key::Named::ArrowDown, _) | (key::Named::Tab, false)
                            if !modifiers.shift() =>
                        {
                            if let Some(index) = &mut menu.hovered_option {
                                if *index >= state.filtered_options.options.len().saturating_sub(1)
                                {
                                    *index = 0;
                                } else {
                                    *index = index.saturating_add(1).min(
                                        state.filtered_options.options.len().saturating_sub(1),
                                    );
                                }
                            } else {
                                menu.hovered_option = Some(0);
                            }

                            if let Some(on_option_hovered) = &mut self.on_option_hovered
                                && let Some(option) = menu
                                    .hovered_option
                                    .and_then(|index| state.filtered_options.options.get(index))
                            {
                                // Notify the selection
                                shell.publish((on_option_hovered)(option.clone()));
                                published_message_to_shell = true;
                            }

                            shell.capture_event();
                            shell.request_redraw();
                        }
                        _ => {}
                    }
                }
            });
        }

        // If the overlay menu has selected something
        self.state.with_inner_mut(|state| {
            if let Some(selection) = menu.new_selection.take() {
                // Clear the value and reset the options and menu
                state.value = String::new();
                state.filtered_options.update(self.state.options.clone());
                menu.menu = menu::State::default();

                // Notify the selection
                shell.publish((self.on_selected)(selection));
                published_message_to_shell = true;

                // Unfocus the input
                let mut local_messages = Vec::new();
                let mut local_shell = shell.local(&mut local_messages);
                self.text_input.update(
                    &mut tree.children[0],
                    &Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)),
                    layout,
                    mouse::Cursor::Unavailable,
                    renderer,
                    &mut local_shell,
                    viewport,
                );
                shell.request_input_method(local_shell.input_method());
            }
        });

        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<<Renderer as text::Renderer>::Paragraph>>();

            text_input_state.is_focused()
        };

        if started_focused != is_focused {
            // Focus changed, invalidate widget tree to force a fresh `view`
            shell.invalidate_widgets();

            if !published_message_to_shell {
                if is_focused {
                    if let Some(on_open) = self.on_open.take() {
                        shell.publish(on_open);
                    }
                } else if let Some(on_close) = self.on_close.take() {
                    shell.publish(on_close);
                }
            }
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
        self.text_input
            .mouse_interaction(&tree.children[0], layout, cursor, viewport, renderer)
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<<Renderer as text::Renderer>::Paragraph>>();

            text_input_state.is_focused()
        };

        let selection = if is_focused || self.selection.is_empty() {
            None
        } else {
            Some(&self.selection)
        };

        self.text_input.draw(
            &tree.children[0],
            renderer,
            theme,
            layout,
            cursor,
            selection,
            viewport,
        );
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let is_focused = {
            let text_input_state = tree.children[0]
                .state
                .downcast_ref::<text_input::State<<Renderer as text::Renderer>::Paragraph>>();

            text_input_state.is_focused()
        };

        if is_focused {
            let Menu {
                menu,
                filtered_options,
                hovered_option,
                ..
            } = tree.state.downcast_mut::<Menu>();

            self.state.sync_filtered_options(filtered_options);

            if filtered_options.options.is_empty() {
                None
            } else {
                let bounds = layout.bounds();

                let mut menu = menu::Menu::new(
                    menu,
                    &filtered_options.options,
                    hovered_option,
                    |selection| {
                        self.state.with_inner_mut(|state| {
                            state.value = String::new();
                            state.filtered_options.update(self.state.options.clone());
                        });

                        tree.children[0]
                            .state
                            .downcast_mut::<text_input::State<<Renderer as text::Renderer>::Paragraph>>()
                            .unfocus();

                        (self.on_selected)(selection)
                    },
                    self.on_option_hovered.as_deref(),
                    &self.menu_class,
                )
                .width(bounds.width)
                .padding(self.padding)
                .shaping(self.shaping)
                .ellipsis(self.ellipsis);

                if let Some(size) = self.size {
                    menu = menu.text_size(size);
                }

                Some(menu.overlay(
                    layout.position() + translation,
                    *viewport,
                    bounds.height,
                    self.menu_height,
                ))
            }
        } else {
            None
        }
    }
}

impl<'a, Message, Theme> From<FontSelection<'a, Message, Theme>> for Element<'a, Message, Theme>
where
    Message: Clone + 'a,
    Theme: Catalog + 'a,
{
    fn from(font_selection: FontSelection<'a, Message, Theme>) -> Self {
        Self::new(font_selection)
    }
}

fn search<'a, T, A>(
    options: impl IntoIterator<Item = T> + 'a,
    option_matchers: impl IntoIterator<Item = &'a A> + 'a,
    query: &'a str,
) -> impl Iterator<Item = T> + 'a
where
    A: AsRef<str> + 'a,
{
    let query: Vec<String> = query
        .to_lowercase()
        .split(|c: char| !c.is_ascii_alphanumeric())
        .map(String::from)
        .collect();

    options
        .into_iter()
        .zip(option_matchers)
        // Make sure each part of the query is found in the option
        .filter_map(move |(option, matcher)| {
            if query.iter().all(|part| matcher.as_ref().contains(part)) {
                Some(option)
            } else {
                None
            }
        })
}

fn build_matchers<'a>(options: impl IntoIterator<Item = &'a Family>) -> Vec<String> {
    options.into_iter().map(build_matcher).collect()
}

fn build_matcher(option: &Family) -> String {
    let mut matcher = option.to_string();
    matcher.retain(|c| c.is_ascii_alphanumeric());
    matcher.to_lowercase()
}

mod menu {
    use iced::{
        Element, Event, Length, Padding, Pixels, Point, Rectangle, Renderer, Size, Vector,
        advanced::{
            self, Shell,
            layout::{self, Layout},
            overlay, renderer,
            text::{self, Text},
            widget::{Widget, tree},
        },
        alignment,
        border::{self},
        font::{Family, Font},
        mouse, touch,
        widget::scrollable::{self, Scrollable},
        window,
    };

    pub use iced::overlay::menu::{Catalog, Style, StyleFn};

    /// A list of selectable options.
    pub struct Menu<'a, 'b, Message, Theme = iced::Theme>
    where
        Theme: Catalog,
        'b: 'a,
    {
        state: &'a mut State,
        options: &'a [Family],
        hovered_option: &'a mut Option<usize>,
        on_selected: Box<dyn FnMut(Family) -> Message + 'a>,
        on_option_hovered: Option<&'a dyn Fn(Family) -> Message>,
        width: f32,
        padding: Padding,
        text_size: Option<Pixels>,
        line_height: text::LineHeight,
        shaping: text::Shaping,
        ellipsis: text::Ellipsis,
        class: &'a <Theme as Catalog>::Class<'b>,
    }

    impl<'a, 'b, Message, Theme> Menu<'a, 'b, Message, Theme>
    where
        Message: 'a,
        Theme: Catalog + 'a,
        'b: 'a,
    {
        /// Creates a new [`Menu`] with the given [`State`], a list of options,
        /// the message to produced when an option is selected, and its [`Style`].
        pub fn new(
            state: &'a mut State,
            options: &'a [Family],
            hovered_option: &'a mut Option<usize>,
            on_selected: impl FnMut(Family) -> Message + 'a,
            on_option_hovered: Option<&'a dyn Fn(Family) -> Message>,
            class: &'a <Theme as Catalog>::Class<'b>,
        ) -> Self {
            Menu {
                state,
                options,
                hovered_option,
                on_selected: Box::new(on_selected),
                on_option_hovered,
                width: 0.0,
                padding: Padding::ZERO,
                text_size: None,
                line_height: text::LineHeight::default(),
                shaping: text::Shaping::default(),
                ellipsis: text::Ellipsis::default(),
                class,
            }
        }

        /// Sets the width of the [`Menu`].
        pub fn width(mut self, width: f32) -> Self {
            self.width = width;
            self
        }

        /// Sets the [`Padding`] of the [`Menu`].
        pub fn padding<P: Into<Padding>>(mut self, padding: P) -> Self {
            self.padding = padding.into();
            self
        }

        /// Sets the text size of the [`Menu`].
        pub fn text_size(mut self, text_size: impl Into<Pixels>) -> Self {
            self.text_size = Some(text_size.into());
            self
        }

        #[allow(dead_code)]
        /// Sets the text [`text::LineHeight`] of the [`Menu`].
        pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
            self.line_height = line_height.into();
            self
        }

        /// Sets the [`text::Shaping`] strategy of the [`Menu`].
        pub fn shaping(mut self, shaping: text::Shaping) -> Self {
            self.shaping = shaping;
            self
        }

        /// Sets the [`text::Ellipsis`] strategy of the [`Menu`].
        pub fn ellipsis(mut self, ellipsis: text::Ellipsis) -> Self {
            self.ellipsis = ellipsis;
            self
        }

        /// Turns the [`Menu`] into an overlay [`Element`] at the given target
        /// position.
        ///
        /// The `target_height` will be used to display the menu either on top
        /// of the target or under it, depending on the screen position and the
        /// dimensions of the [`Menu`].
        pub fn overlay(
            self,
            position: Point,
            viewport: Rectangle,
            target_height: f32,
            menu_height: Length,
        ) -> overlay::Element<'a, Message, Theme, Renderer> {
            overlay::Element::new(Box::new(Overlay::new(
                position,
                viewport,
                self,
                target_height,
                menu_height,
            )))
        }
    }

    /// The local state of a [`Menu`].
    #[derive(Debug)]
    pub struct State {
        tree: tree::Tree,
    }

    impl State {
        /// Creates a new [`State`] for a [`Menu`].
        pub fn new() -> Self {
            Self {
                tree: tree::Tree::empty(),
            }
        }
    }

    impl Default for State {
        fn default() -> Self {
            Self::new()
        }
    }

    struct Overlay<'a, 'b, Message, Theme>
    where
        Theme: Catalog,
    {
        position: Point,
        viewport: Rectangle,
        tree: &'a mut tree::Tree,
        list: Scrollable<'a, Message, Theme, Renderer>,
        width: f32,
        target_height: f32,
        class: &'a <Theme as Catalog>::Class<'b>,
    }

    impl<'a, 'b, Message, Theme> Overlay<'a, 'b, Message, Theme>
    where
        Message: 'a,
        Theme: Catalog + scrollable::Catalog + 'a,
        'b: 'a,
    {
        pub fn new(
            position: Point,
            viewport: Rectangle,
            menu: Menu<'a, 'b, Message, Theme>,
            target_height: f32,
            menu_height: Length,
        ) -> Self {
            let Menu {
                state,
                options,
                hovered_option,
                on_selected,
                on_option_hovered,
                width,
                padding,
                text_size,
                line_height,
                shaping,
                ellipsis,
                class,
            } = menu;

            let mut list = Scrollable::new(List {
                options,
                hovered_option,
                on_selected,
                on_option_hovered,
                text_size,
                line_height,
                shaping,
                ellipsis,
                padding,
                class,
            })
            .height(menu_height);

            state.tree.diff(&mut list as &mut dyn Widget<_, _, _>);

            Self {
                position,
                viewport,
                tree: &mut state.tree,
                list,
                width,
                target_height,
                class,
            }
        }
    }

    impl<Message, Theme> overlay::Overlay<Message, Theme, iced::Renderer>
        for Overlay<'_, '_, Message, Theme>
    where
        Theme: Catalog,
    {
        fn layout(&mut self, renderer: &Renderer, bounds: Size) -> layout::Node {
            let space_below = bounds.height - (self.position.y + self.target_height);
            let space_above = self.position.y;

            let limits = layout::Limits::new(
                Size::ZERO,
                Size::new(
                    bounds.width - self.position.x,
                    if space_below > space_above {
                        space_below
                    } else {
                        space_above
                    },
                ),
            )
            .width(self.width);

            let node = self.list.layout(self.tree, renderer, &limits);
            let size = node.size();

            node.move_to(if space_below > space_above {
                self.position + Vector::new(0.0, self.target_height)
            } else {
                self.position - Vector::new(0.0, size.height)
            })
        }

        fn update(
            &mut self,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            shell: &mut Shell<'_, Message>,
        ) {
            let bounds = layout.bounds();

            self.list
                .update(self.tree, event, layout, cursor, renderer, shell, &bounds);
        }

        fn mouse_interaction(
            &self,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
        ) -> mouse::Interaction {
            self.list
                .mouse_interaction(self.tree, layout, cursor, &self.viewport, renderer)
        }

        fn draw(
            &self,
            renderer: &mut Renderer,
            theme: &Theme,
            defaults: &renderer::Style,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
        ) {
            use advanced::Renderer;
            let bounds = layout.bounds();

            let style = Catalog::style(theme, self.class);

            renderer.fill_quad(
                renderer::Quad {
                    bounds,
                    border: style.border,
                    shadow: style.shadow,
                    ..renderer::Quad::default()
                },
                style.background,
            );

            self.list.draw(
                self.tree, renderer, theme, defaults, layout, cursor, &bounds,
            );
        }
    }

    struct List<'a, 'b, Message, Theme>
    where
        Theme: Catalog,
    {
        options: &'a [Family],
        hovered_option: &'a mut Option<usize>,
        on_selected: Box<dyn FnMut(Family) -> Message + 'a>,
        on_option_hovered: Option<&'a dyn Fn(Family) -> Message>,
        padding: Padding,
        text_size: Option<Pixels>,
        line_height: text::LineHeight,
        shaping: text::Shaping,
        ellipsis: text::Ellipsis,
        class: &'a <Theme as Catalog>::Class<'b>,
    }

    struct ListState {
        is_hovered: Option<bool>,
    }

    impl<Message, Theme> Widget<Message, Theme, Renderer> for List<'_, '_, Message, Theme>
    where
        Theme: Catalog,
    {
        fn tag(&self) -> tree::Tag {
            tree::Tag::of::<Option<bool>>()
        }

        fn state(&self) -> tree::State {
            tree::State::new(ListState { is_hovered: None })
        }

        fn size(&self) -> Size<Length> {
            Size {
                width: Length::Fill,
                height: Length::Shrink,
            }
        }

        fn layout(
            &mut self,
            _tree: &mut tree::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            use std::f32;
            use text::Renderer;

            let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());

            let text_line_height = self.line_height.to_absolute(text_size);

            let size = {
                let intrinsic = Size::new(
                    0.0,
                    (f32::from(text_line_height) + self.padding.y()) * self.options.len() as f32,
                );

                limits.resolve(Length::Fill, Length::Shrink, intrinsic)
            };

            layout::Node::new(size)
        }

        fn update(
            &mut self,
            tree: &mut tree::Tree,
            event: &Event,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            renderer: &Renderer,
            shell: &mut Shell<'_, Message>,
            _viewport: &Rectangle,
        ) {
            match event {
                Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left)) => {
                    if cursor.is_over(layout.bounds())
                        && let Some(index) = *self.hovered_option
                        && let Some(option) = self.options.get(index)
                    {
                        shell.publish((self.on_selected)(option.clone()));
                        shell.capture_event();
                    }
                }
                Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                    if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                        use text::Renderer;
                        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());

                        let option_height =
                            f32::from(self.line_height.to_absolute(text_size)) + self.padding.y();

                        let new_hovered_option = (cursor_position.y / option_height) as usize;

                        if *self.hovered_option != Some(new_hovered_option)
                            && let Some(option) = self.options.get(new_hovered_option)
                        {
                            if let Some(on_option_hovered) = self.on_option_hovered {
                                shell.publish(on_option_hovered(option.clone()));
                            }

                            shell.request_redraw();
                        }

                        *self.hovered_option = Some(new_hovered_option);
                    }
                }
                Event::Touch(touch::Event::FingerPressed { .. }) => {
                    if let Some(cursor_position) = cursor.position_in(layout.bounds()) {
                        use text::Renderer;
                        let text_size = self.text_size.unwrap_or_else(|| renderer.default_size());

                        let option_height =
                            f32::from(self.line_height.to_absolute(text_size)) + self.padding.y();

                        *self.hovered_option = Some((cursor_position.y / option_height) as usize);

                        if let Some(index) = *self.hovered_option
                            && let Some(option) = self.options.get(index)
                        {
                            shell.publish((self.on_selected)(option.clone()));
                            shell.capture_event();
                        }
                    }
                }
                _ => {}
            }

            let state = tree.state.downcast_mut::<ListState>();

            if let Event::Window(window::Event::RedrawRequested(_now)) = event {
                state.is_hovered = Some(cursor.is_over(layout.bounds()));
            } else if state
                .is_hovered
                .is_some_and(|is_hovered| is_hovered != cursor.is_over(layout.bounds()))
            {
                shell.request_redraw();
            }
        }

        fn mouse_interaction(
            &self,
            _tree: &tree::Tree,
            layout: Layout<'_>,
            cursor: mouse::Cursor,
            _viewport: &Rectangle,
            _renderer: &Renderer,
        ) -> mouse::Interaction {
            let is_mouse_over = cursor.is_over(layout.bounds());

            if is_mouse_over {
                mouse::Interaction::Pointer
            } else {
                mouse::Interaction::default()
            }
        }

        fn draw(
            &self,
            _tree: &tree::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            _style: &renderer::Style,
            layout: Layout<'_>,
            _cursor: mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let style = Catalog::style(theme, self.class);
            let bounds = layout.bounds();

            let text_size = self.text_size.unwrap_or_else(|| {
                use text::Renderer;
                renderer.default_size()
            });
            let option_height =
                f32::from(self.line_height.to_absolute(text_size)) + self.padding.y();

            let offset = viewport.y - bounds.y;
            let start = (offset / option_height) as usize;
            let end = ((offset + viewport.height) / option_height).ceil() as usize;

            let visible_options = &self.options[start..end.min(self.options.len())];

            for (i, option) in visible_options.iter().enumerate() {
                let i = start + i;
                let is_selected = *self.hovered_option == Some(i);

                let bounds = Rectangle {
                    x: bounds.x,
                    y: bounds.y + (option_height * i as f32),
                    width: bounds.width,
                    height: option_height,
                };

                if is_selected {
                    use advanced::Renderer;
                    renderer.fill_quad(
                        renderer::Quad {
                            bounds: Rectangle {
                                x: bounds.x + style.border.width,
                                width: bounds.width - style.border.width * 2.0,
                                ..bounds
                            },
                            border: border::rounded(style.border.radius),
                            ..renderer::Quad::default()
                        },
                        style.selected_background,
                    );
                }

                <Renderer as text::Renderer>::fill_text(
                    renderer,
                    Text {
                        content: option.to_string(),
                        bounds: Size::new(bounds.width - self.padding.x(), bounds.height),
                        size: text_size,
                        line_height: self.line_height,
                        font: Font::from(*option),
                        align_x: text::Alignment::Default,
                        align_y: alignment::Vertical::Center,
                        shaping: self.shaping,
                        wrapping: text::Wrapping::None,
                        ellipsis: self.ellipsis,
                        hint_factor: <Renderer as advanced::Renderer>::scale_factor(renderer),
                    },
                    Point::new(bounds.x + self.padding.left, bounds.center_y()),
                    if is_selected {
                        style.selected_text_color
                    } else {
                        style.text_color
                    },
                    *viewport,
                );
            }
        }
    }

    impl<'a, 'b, Message, Theme> From<List<'a, 'b, Message, Theme>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a + Catalog,
        'b: 'a,
    {
        fn from(list: List<'a, 'b, Message, Theme>) -> Self {
            Element::new(list)
        }
    }
}
