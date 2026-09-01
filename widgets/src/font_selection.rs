//! A version of [`iced::widget::ComboBox`] customised for font selection
use iced::{
    Element, Event, Length, Padding, Pixels, Rectangle, Renderer, Size, Vector,
    advanced::{
        Shell, Widget,
        layout::{self, Layout},
        mouse, overlay, renderer, text,
        widget::{self, tree},
    },
    font::{self, Family},
    keyboard::{self, key},
    window,
};

use iced::advanced::Renderer as _;
use widget::operation::Focusable;

use iced::widget::combo_box::Catalog;
use iced::widget::text_input;
use std::sync::atomic::{self, AtomicU64};

pub fn font_selection<'a, Message, Theme>(
    state: &'a State,
    placeholder: impl text::IntoFragment<'a>,
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
    id: Option<widget::Id>,
    placeholder: text::Fragment<'a>,
    width: Length,
    line_height: text::LineHeight,
    selection: String,
    on_selected: Box<dyn Fn(Family) -> Message + 'a>,
    on_option_hovered: Option<Box<dyn Fn(Family) -> Message + 'a>>,
    on_open: Option<Message>,
    on_close: Option<Message>,
    on_input: Option<Box<dyn Fn(String) -> Message + 'a>>,
    padding: Padding,
    size: Option<Pixels>,
    shaping: text::Shaping,
    ellipsis: text::Ellipsis,
    input_class: <Theme as text_input::Catalog>::Class<'a>,
    menu_class: <Theme as menu::Catalog>::Class<'a>,
    menu_height: Length,
    last_status: Option<text_input::Status>,
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
        placeholder: impl text::IntoFragment<'a>,
        selection: Option<Family>,
        on_selected: impl Fn(Family) -> Message + 'a,
    ) -> Self {
        Self {
            state,
            id: None,
            placeholder: placeholder.into_fragment(),
            selection: selection
                .map(|selection| selection.to_string())
                .unwrap_or_default(),
            width: Length::Fill,
            line_height: text::LineHeight::default(),
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
            input_class: <Theme as Catalog>::default_input(),
            menu_height: Length::Shrink,
            last_status: None,
        }
    }

    /// Sets the [`widget::Id`] of the [`ComboBox`].
    pub fn id(mut self, id: impl Into<widget::Id>) -> Self {
        self.id = Some(id.into());
        self
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
        self
    }

    /// Sets the text sixe of the [`FontSelection`].
    pub fn size(mut self, size: impl Into<Pixels>) -> Self {
        self.size = Some(size.into());

        self
    }

    /// Sets the [`LineHeight`] of the [`FontSelection`].
    pub fn line_height(mut self, line_height: impl Into<text::LineHeight>) -> Self {
        self.line_height = line_height.into();

        self
    }

    /// Sets the width of the [`FontSelection`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
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
        self.input_class = (Box::new(style) as text_input::StyleFn<'a, Theme>).into();
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
        self.input_class = class.into();
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
    version: u64,
}

static VERSION: AtomicU64 = AtomicU64::new(0);

impl State {
    /// Creates a new [`State`] for a [`FontSelection`] with the given list of options.
    pub fn new(options: Vec<Family>) -> Self {
        Self {
            options,
            version: VERSION.fetch_add(1, atomic::Ordering::Relaxed),
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
        self.options.push(new_option);
        self.version = VERSION.fetch_add(1, atomic::Ordering::Relaxed)
    }

    /// Returns ownership of the options of the [`State`].
    pub fn into_options(self) -> Vec<Family> {
        self.options
    }
}

impl Default for State {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl<Message, Theme> Widget<Message, Theme, Renderer> for FontSelection<'_, Message, Theme>
where
    Message: Clone,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fit,
        }
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let state = tree.state.downcast_mut::<Internal<Renderer>>();

        let font = (!self.selection.is_empty())
            .then_some(font::Font::with_family(Family::name(&self.selection)));

        state.editor.input.layout(
            renderer,
            limits,
            text::input::Layout {
                width: self.width,
                height: Length::Fit,
                padding: self.padding,
                placeholder: &self.placeholder,
                font,
                size: self.size,
                line_height: self.line_height,
                alignment: text::Alignment::Default,
                multiline: None,
                is_secure: false,
            },
        )
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<Internal<Renderer>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(Internal::<Renderer> {
            editor: Editor {
                input: text::Input::new(),
                selection: None,
            },
            menu: menu::State::new(),
            filtered_options: Vec::new(),
            option_matchers: Vec::new(),
            hovered_option: Some(0),
            version: 0,
        })
    }

    fn diff(&mut self, tree: &mut tree::Tree) {
        let state = tree.state.downcast_mut::<Internal<Renderer>>();

        if state.version != self.state.version
            || state.editor.selection.as_deref() != Some(&self.selection)
        {
            state.editor.input.overwrite(&self.selection);
            state.editor.selection = Some(self.selection.clone());
            state.filter(&self.state.options, &self.selection);

            state.version = self.state.version;
        }
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: Layout<'_>,
        cursor: mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        let internal = tree.state.downcast_mut::<Internal<Renderer>>();

        let was_focused = internal.editor.input.is_focused();

        let edit = internal.editor.input.update::<Message>(
            event,
            layout.bounds(),
            cursor,
            shell,
            text::editor::Binding::from_key_press,
        );

        if edit.is_some() {
            let value = internal.editor.input.value();

            if let Some(on_input) = &self.on_input {
                shell.publish(on_input(value.clone()));
            }

            internal.filter(&self.state.options, &value);
        }

        let is_focused = internal.editor.input.is_focused();

        if is_focused {
            if !was_focused {
                internal.editor.input.overwrite("");
                internal.filtered_options = self.state.options.clone();

                if let Some(on_option_hovered) = &mut self.on_option_hovered {
                    let hovered_option = internal.hovered_option.unwrap_or(0);

                    if let Some(option) = internal.filtered_options.get(hovered_option) {
                        shell.publish(on_option_hovered(option.clone()));
                    }
                }
            }

            if let Event::Keyboard(keyboard::Event::KeyPressed {
                key: keyboard::Key::Named(named_key),
                modifiers,
                ..
            }) = event
            {
                match (named_key, modifiers.shift()) {
                    (key::Named::Enter, _) => {
                        if let Some(option) = internal
                            .filtered_options
                            .get(internal.hovered_option())
                            .cloned()
                        {
                            internal.menu = menu::State::default();
                            internal.editor.selection = None;
                            internal.editor.input.overwrite("");
                            internal.editor.input.unfocus();

                            shell.publish((self.on_selected)(option));
                        }

                        shell.capture_event();
                        shell.request_redraw();
                    }
                    (key::Named::ArrowUp, _) | (key::Named::Tab, true) => {
                        let index = internal.hovered_option();

                        internal.hovered_option = Some(if index == 0 {
                            internal.filtered_options.len().saturating_sub(1)
                        } else {
                            index.saturating_sub(1)
                        });

                        if let Some(on_option_hovered) = &mut self.on_option_hovered
                            && let Some(option) = internal
                                .hovered_option
                                .and_then(|index| internal.filtered_options.get(index))
                        {
                            shell.publish((on_option_hovered)(option.clone()));
                        }

                        shell.capture_event();
                        shell.request_redraw();
                    }
                    (key::Named::ArrowDown, _) | (key::Named::Tab, false) => {
                        let index = internal.hovered_option();

                        internal.hovered_option = Some(
                            if index >= internal.filtered_options.len().saturating_sub(1) {
                                0
                            } else {
                                index
                                    .saturating_add(1)
                                    .min(internal.filtered_options.len().saturating_sub(1))
                            },
                        );

                        if let Some(on_option_hovered) = &mut self.on_option_hovered
                            && let Some(option) = internal
                                .hovered_option
                                .and_then(|index| internal.filtered_options.get(index))
                        {
                            shell.publish((on_option_hovered)(option.clone()));
                        }

                        shell.capture_event();
                        shell.request_redraw();
                    }
                    _ => {}
                }
            }
        }

        if was_focused != is_focused {
            if is_focused {
                if let Some(on_open) = self.on_open.take() {
                    shell.publish(on_open);
                }
            } else if let Some(on_close) = self.on_close.take() {
                internal.editor.input.overwrite(&self.selection);
                shell.publish(on_close);
            }
        }

        let status = if internal.editor.input.is_focused() {
            text_input::Status::Focused {
                is_hovered: cursor.is_over(layout.bounds()),
            }
        } else if cursor.is_over(layout.bounds()) {
            text_input::Status::Hovered
        } else {
            text_input::Status::Active
        };

        if let Event::Window(window::Event::RedrawRequested(_now)) = event {
            self.last_status = Some(status);

            shell.request_input_method(
                &internal
                    .editor
                    .input
                    .input_method(layout.bounds().shrink(self.padding).position()),
            );
        } else if self
            .last_status
            .is_some_and(|last_status| status != last_status)
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
        if cursor.is_over(layout.bounds()) {
            mouse::Interaction::Text
        } else {
            mouse::Interaction::default()
        }
    }

    fn draw(
        &self,
        tree: &tree::Tree,
        renderer: &mut Renderer,
        theme: &Theme,
        _style: &renderer::Style,
        layout: Layout<'_>,
        _cursor: mouse::Cursor,
        viewport: &Rectangle,
    ) {
        let internal = tree.state.downcast_ref::<Internal<Renderer>>();

        let bounds = layout.bounds();
        let style = text_input::Catalog::style(
            theme,
            &self.input_class,
            self.last_status.unwrap_or(text_input::Status::Disabled),
        );

        renderer.fill_quad(
            renderer::Quad {
                bounds,
                border: style.border,
                ..renderer::Quad::default()
            },
            style.background,
        );

        internal.editor.input.draw(
            renderer,
            bounds,
            *viewport,
            text::input::Style {
                value: style.value,
                selection: style.selection,
                placeholder: style.placeholder,
            },
        );
    }

    fn operate(
        &mut self,
        tree: &mut widget::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        operation: &mut dyn widget::Operation,
    ) {
        let state = tree.state.downcast_mut::<Internal<Renderer>>();
        let bounds = layout.bounds();

        operation.focusable(self.id.as_ref(), bounds, &mut state.editor.input);
        operation.text_input(self.id.as_ref(), bounds, &mut state.editor.input);
    }

    fn overlay<'b>(
        &'b mut self,
        tree: &'b mut tree::Tree,
        layout: Layout<'_>,
        _renderer: &Renderer,
        viewport: &Rectangle,
        translation: Vector,
    ) -> Option<overlay::Element<'b, Message, Theme, Renderer>> {
        let internal = tree.state.downcast_mut::<Internal<Renderer>>();
        let is_focused = internal.editor.input.is_focused();

        if is_focused {
            let Internal {
                menu,
                filtered_options,
                hovered_option,
                editor,
                ..
            } = internal;

            if filtered_options.is_empty() {
                None
            } else {
                let bounds = layout.bounds();

                let mut menu = menu::Menu::new(
                    menu,
                    filtered_options,
                    hovered_option,
                    |selection| {
                        editor.selection = None;
                        editor.input.overwrite("");
                        editor.input.unfocus();

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

struct Internal<R: text::Renderer> {
    editor: Editor<R>,
    menu: menu::State,
    hovered_option: Option<usize>,
    option_matchers: Vec<String>,
    filtered_options: Vec<Family>,
    version: u64,
}

impl<R: text::Renderer> Internal<R> {
    fn hovered_option(&self) -> usize {
        let index = self.hovered_option.unwrap_or_default();

        index.min(self.filtered_options.len().saturating_sub(1))
    }

    fn filter(&mut self, options: &[Family], value: &str) {
        self.option_matchers = build_matchers(options);
        self.filtered_options = search(options, &self.option_matchers, value)
            .cloned()
            .collect();
    }
}

struct Editor<R: text::Renderer> {
    input: text::Input<R>,
    selection: Option<String>,
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
                        shell.publish((self.on_selected)(*option));
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
                                shell.publish(on_option_hovered(*option));
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
                            shell.publish((self.on_selected)(*option));
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
                        hint_factor: <Renderer as advanced::Renderer>::hint_factor(renderer),
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
