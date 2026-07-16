use iced::{
    Background, Border, Color, Element, Event, Length, Padding, Pixels, Rectangle, Size,
    advanced::{
        self, layout, text,
        widget::{Widget, tree},
    },
    alignment::Alignment,
    border, mouse,
};
use std::ops::Deref;

const ELLIPSIS: &str = "…";

fn fit(n: usize, l: usize, r: usize) -> (usize, usize) {
    if l + r <= n {
        return (l, r);
    }

    let half = n / 2;

    if l <= half {
        return (l, n - l);
    }

    if r <= half {
        return (n - r, r);
    }

    let left = (n + 1) / 2;
    let right = n / 2;

    (left, right)

    // Wasn't good enough for n=15,l=12,r=12
    // if l + r <= n {
    //     return (l, r);
    // }
    //
    // if l >= r {
    //     (n.saturating_sub(r), r)
    // } else {
    //     (l, n.saturating_sub(l))
    // }

    // Wasn't good enough for n=11,l=11,r=3
    // let s = l + r;
    // if s <= n {
    //     return (l, r);
    // }
    //
    // let d = s - n;
    //
    // let frac = l as f32 / s as f32;
    //
    // let ld = ((d as f32) * frac) as usize;
    //
    // (l.saturating_sub(ld), r.saturating_sub(d - ld))
}

#[derive(Debug, Clone, Copy, PartialEq)]
/// A page option
pub enum Page {
    /// A numbered page
    Number(usize),
    /// An ellipsis
    Ellipsis {
        /// The immediate value to the left of the ellipsis
        left: usize,
        /// The immediate value to the right of the ellipsis
        right: usize,
    },
}

impl PartialEq<usize> for Page {
    fn eq(&self, other: &usize) -> bool {
        match self {
            Self::Number(x) => x.eq(other),
            Self::Ellipsis { .. } => false,
        }
    }
}

#[derive(Debug, Clone, Copy)]
/// An ellipsis content used by a [`Pagination`].
pub struct Ellipsis<Font> {
    /// The font that will be used to display the `code_point`.
    pub font: Font,
    /// The unicode code point that will be used as the ellipsis.
    pub code_point: char,
    /// The font size of the ellipsis.
    pub size: Option<Pixels>,
    /// The line height of the ellipsis.
    pub line_height: Option<text::LineHeight>,
    /// The shapping stragegy of the ellipsis.
    pub shaping: Option<text::Shaping>,
}

pub fn pagination<'a, Message, Theme: Catalog, Renderer: text::Renderer>(
    start: usize,
    current: usize,
    end: usize,
) -> Pagination<'a, Message, Theme, Renderer> {
    Pagination::new(start, current, end)
}

/// A pagination widget
pub struct Pagination<'a, Message, Theme: Catalog, Renderer: text::Renderer> {
    start: usize,
    current: usize,
    end: usize,
    ellipsis: Option<Ellipsis<Renderer::Font>>,
    width: Length,
    padding: Padding,
    spacing: f32,
    size: Option<Pixels>,
    font: Option<Renderer::Font>,
    on_select: Option<Box<dyn Fn(Page) -> Message + 'a>>,
    class: Theme::Class<'a>,
}

impl<'a, Message, Theme: Catalog, Renderer: text::Renderer>
    Pagination<'a, Message, Theme, Renderer>
{
    pub const DEFAULT_PADDING: Padding = Padding {
        top: 4.0,
        right: 8.0,
        bottom: 4.0,
        left: 8.0,
    };

    /// Creates a new [`Pagination`] with the given `start`, `current` and
    /// `end` pages.
    pub fn new(start: usize, current: usize, end: usize) -> Self {
        Self {
            start,
            current,
            end,
            ellipsis: None,
            width: Length::Fit,
            padding: Self::DEFAULT_PADDING,
            spacing: 2.0,
            size: None,
            font: None,
            on_select: None,
            class: Theme::default(),
        }
    }

    /// Sets the function that will be called when a [`Page`] is selected.
    pub fn on_select(mut self, on_select: impl Fn(Page) -> Message + 'a) -> Self {
        self.on_select = Some(Box::new(on_select));
        self
    }

    /// Sets the [`Ellipsis`] used by the the [`Pagination`].
    pub fn ellipsis(self, ellipsis: Ellipsis<Renderer::Font>) -> Self {
        self.ellipsis_maybe(Some(ellipsis))
    }

    /// Sets the [`Ellipsis`] used by the the [`Pagination`].
    fn ellipsis_maybe(mut self, ellipsis: Option<Ellipsis<Renderer::Font>>) -> Self {
        self.ellipsis = ellipsis;
        self
    }

    /// Sets the width of the [`Pagination`].
    pub fn width(mut self, width: impl Into<Length>) -> Self {
        self.width = width.into();
        self
    }

    /// Sets the padding for each [`Page`] in the [`Pagination`].
    pub fn padding(mut self, padding: impl Into<Padding>) -> Self {
        self.padding = padding.into();
        self
    }

    /// Sets the spacing between [`Page`]s in the [`Pagination`].
    pub fn spacing(mut self, spacing: f32) -> Self {
        self.spacing = spacing;
        self
    }

    /// Sets the text size of the [`Pagination`].
    pub fn size(self, size: impl Into<Pixels>) -> Self {
        self.size_maybe(Some(size.into()))
    }

    /// Sets the text size of the [`Pagination`].
    fn size_maybe(mut self, size: Option<Pixels>) -> Self {
        self.size = size;
        self
    }

    /// Sets the font of the [`Pagination`].
    pub fn font(self, font: Renderer::Font) -> Self {
        self.font_maybe(Some(font))
    }

    /// Sets the font of the [`Pagination`].
    fn font_maybe(mut self, font: Option<Renderer::Font>) -> Self {
        self.font = font;
        self
    }

    /// Sets the style of the [`Pagination`].
    pub fn style(mut self, style: impl Fn(&Theme, Status) -> Style + 'a) -> Self
    where
        Theme::Class<'a>: From<StyleFn<'a, Theme>>,
    {
        self.class = (Box::new(style) as StyleFn<'a, Theme>).into();
        self
    }

    /// Sets the style class of the [`Pagination`].
    pub fn class(mut self, class: impl Into<Theme::Class<'a>>) -> Self {
        self.class = class.into();
        self
    }

    /// Converts the [`Pagination`] to a [`buttoned::Buttoned`].
    pub fn buttoned(self) -> buttoned::Buttoned<'a, Message, Theme, Renderer>
    where
        Theme: 'a + buttoned::Catalog,
        Renderer: 'a,
    {
        buttoned::Buttoned::new(self)
    }
}

impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
    for Pagination<'a, Message, Theme, Renderer>
where
    Renderer: advanced::Renderer + text::Renderer,
    Theme: Catalog,
{
    fn size(&self) -> Size<Length> {
        Size {
            width: self.width,
            height: Length::Fit,
        }
    }

    fn tag(&self) -> tree::Tag {
        tree::Tag::of::<State<Renderer::Paragraph>>()
    }

    fn state(&self) -> tree::State {
        tree::State::new(State::<Renderer::Paragraph>::new())
    }

    fn layout(
        &mut self,
        tree: &mut tree::Tree,
        renderer: &Renderer,
        limits: &layout::Limits,
    ) -> layout::Node {
        let none = layout::Node::with_children(Size::ZERO, vec![]);

        if self.start >= self.end {
            return none;
        }

        let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();

        let max = state.max_size(
            renderer,
            self.end,
            self.font,
            self.size,
            self.padding,
            self.ellipsis.as_ref(),
        );
        let width = limits.max().width;

        // available space - space for current
        let free_width = width - (max.width + self.spacing);

        let max_no = (free_width / (max.width + self.spacing).max(1.0)) as usize;

        let l = self.current.saturating_sub(self.start);
        let r = self.end.saturating_sub(self.current);

        let (left, right) = fit(max_no, l, r);

        let left_ellipsis = (self.start + left) < self.current;
        let right_ellipsis = (self.current + right) < self.end;

        let mut width = 0.0;
        let height = max.height;
        let mut pages = Vec::new();
        let mut nodes = Vec::new();

        let mut proc = |page: Page| {
            let page = PageState::new(renderer, page, self.font, self.size, self.ellipsis.as_ref());

            pages.push(page);

            let node = layout::Node::new(max).translate([width, 0.0]);
            nodes.push(node);

            width += max.width + self.spacing;
        };

        if left_ellipsis {
            let start = self.current.saturating_sub(left) + 2;
            // start
            if left > 0 {
                let page = Page::Number(self.start);

                proc(page);
            }

            // ellipsis
            if left > 1 {
                let page = Page::Ellipsis {
                    left: self.start,
                    right: start,
                };

                proc(page);
            }

            // the rest
            if left > 2 {
                for n in start..self.current {
                    let page = Page::Number(n);

                    proc(page)
                }
            }
        } else {
            for n in self.start..(self.start + l) {
                let page = Page::Number(n);

                proc(page);
            }
        }

        let page = Page::Number(self.current);
        proc(page);

        if right_ellipsis {
            let end = (self.current + right).saturating_sub(2);

            if right > 2 {
                for n in (self.current..=end).skip(1) {
                    let page = Page::Number(n);
                    proc(page)
                }
            }

            if right > 1 {
                let page = Page::Ellipsis {
                    left: end,
                    right: self.end,
                };
                proc(page)
            }

            if right > 0 {
                let page = Page::Number(self.end);
                proc(page);
            }
        } else {
            for n in (self.current..=self.end).skip(1) {
                let page = Page::Number(n);

                proc(page);
            }
        }

        state.pages = pages;

        width = (width - self.spacing).max(0.0);
        let intrinsic = Size::new(width, height);

        let size = limits.resolve(self.width, Length::Fit, intrinsic);

        layout::Node::with_children(size, nodes)
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
        let Some(viewport) = layout.bounds().intersection(viewport) else {
            return;
        };

        let state = tree.state.downcast_ref::<State<Renderer::Paragraph>>();

        let text_color = style.text_color;

        let active = theme.style(&self.class, Status::Active);
        let hovered = theme.style(&self.class, Status::Hovered);
        let idle = theme.style(&self.class, Status::Idle);

        for (page, layout) in state.pages.iter().zip(layout.children()) {
            let bounds = layout.bounds();
            let style = if page.page == self.current {
                &active
            } else if let Some(hovered_page) = state.hovered
                && hovered_page == page.page
            {
                &hovered
            } else {
                &idle
            };

            renderer.fill_quad(
                advanced::renderer::Quad {
                    bounds,
                    border: style.border,
                    ..Default::default()
                },
                style
                    .background
                    .unwrap_or(Background::Color(Color::TRANSPARENT)),
            );

            let position =
                bounds.anchor(page.text.min_bounds(), Alignment::Center, Alignment::Center);

            renderer.fill_paragraph(
                page.text.raw(),
                position,
                style.text.unwrap_or(text_color),
                viewport,
            );
        }
    }

    fn update(
        &mut self,
        tree: &mut tree::Tree,
        event: &Event,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _renderer: &Renderer,
        shell: &mut advanced::Shell<'_, Message>,
        _viewport: &Rectangle,
    ) {
        match event {
            Event::Mouse(mouse::Event::CursorMoved { .. }) => {
                let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
                let prev = state.hovered.take();

                for (page, layout) in state.pages.iter().zip(layout.children()) {
                    if cursor.is_over(layout.bounds()) {
                        state.hovered = Some(page.page);
                        break;
                    }
                }

                if prev != state.hovered {
                    shell.request_redraw();
                }
            }
            Event::Mouse(mouse::Event::ButtonPressed(mouse::Button::Left))
                if cursor.is_over(layout.bounds()) =>
            {
                let state = tree.state.downcast_mut::<State<Renderer::Paragraph>>();
                for (page, layout) in state.pages.iter().zip(layout.children()) {
                    if cursor.is_over(layout.bounds()) {
                        state.hovered = Some(page.page);

                        if let Some(on_select) = self.on_select.as_ref() {
                            shell.publish((on_select)(page.page));
                            shell.capture_event();
                            shell.invalidate_layout();
                        }
                    }
                }
            }
            _ => {}
        }
    }

    fn mouse_interaction(
        &self,
        _tree: &tree::Tree,
        layout: layout::Layout<'_>,
        cursor: advanced::mouse::Cursor,
        _viewport: &Rectangle,
        _renderer: &Renderer,
    ) -> advanced::mouse::Interaction {
        for layout in layout.children() {
            if cursor.is_over(layout.bounds()) {
                return mouse::Interaction::Pointer;
            }
        }

        mouse::Interaction::None
    }
}

impl<'a, Message, Theme, Renderer> From<Pagination<'a, Message, Theme, Renderer>>
    for Element<'a, Message, Theme, Renderer>
where
    Message: 'a,
    Theme: 'a + Catalog,
    Renderer: 'a + advanced::Renderer + text::Renderer,
{
    fn from(value: Pagination<'a, Message, Theme, Renderer>) -> Self {
        Element::new(value)
    }
}

pub struct Style {
    pub background: Option<Background>,
    pub text: Option<Color>,
    pub border: Border,
}

#[derive(Debug, Clone, Copy)]
/// The possible status of a [`Page`].
pub enum Status {
    /// The [`Page`] is currently selected.
    Active,
    /// The [`Page`] is being hovered.
    Hovered,
    /// The [`Page`] can be pressed.
    Idle,
}

/// The theme catalog of a [`Page`].
pub trait Catalog {
    /// The item class of the [`Catalog`].
    type Class<'a>;

    /// The default class produced by the [`Catalog`].
    fn default<'a>() -> Self::Class<'a>;

    /// The [`Style`] of a class with the given status.
    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style;
}

pub type StyleFn<'a, Theme> = Box<dyn Fn(&Theme, Status) -> Style + 'a>;

impl Catalog for iced::Theme {
    type Class<'a> = StyleFn<'a, Self>;

    fn default<'a>() -> Self::Class<'a> {
        Box::new(default)
    }

    fn style(&self, class: &Self::Class<'_>, status: Status) -> Style {
        class(self, status)
    }
}

/// The default styling of a [`Pagination`].
pub fn default(theme: &iced::Theme, status: Status) -> Style {
    let palette = theme.palette();

    let pair = match status {
        Status::Active => palette.primary.weak,
        Status::Idle => palette.background.base,
        Status::Hovered => palette.background.weaker,
    };

    Style {
        background: Some(Background::Color(pair.color)),
        text: Some(pair.text),
        border: border::rounded(2),
    }
}

struct PageState<P: text::Paragraph> {
    text: text::paragraph::Plain<P>,
    page: Page,
}

impl<P: text::Paragraph> PageState<P> {
    fn new<Renderer>(
        renderer: &Renderer,
        page: Page,
        font: Option<Renderer::Font>,
        size: Option<Pixels>,
        ellipsis: Option<&Ellipsis<Renderer::Font>>,
    ) -> Self
    where
        Renderer: text::Renderer<Font = P::Font>,
    {
        fn text<'a, Renderer: text::Renderer>(
            renderer: &Renderer,
            value: String,
            font: Option<Renderer::Font>,
            size: Option<Pixels>,
        ) -> text::Text<String, Renderer::Font> {
            text::Text {
                content: value,
                size: size.unwrap_or(renderer.default_size()),
                font: font.unwrap_or(renderer.default_font()),
                line_height: text::LineHeight::default(),
                bounds: Size::INFINITE,
                align_x: text::Alignment::Default,
                align_y: iced::alignment::Vertical::Top,
                shaping: text::Shaping::default(),
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: renderer.scale_factor(),
            }
        }
        let text = match page {
            Page::Number(n) => {
                let value = n.to_string();
                text(renderer, value, font, size)
            }
            Page::Ellipsis { .. } => match ellipsis {
                Some(ellipsis) => {
                    let mut content = [0; 4];
                    let value = ellipsis.code_point.encode_utf8(&mut content);
                    let value = value.to_owned();

                    text::Text {
                        content: value,
                        size: ellipsis.size.unwrap_or(renderer.default_size()),
                        font: ellipsis.font,
                        bounds: Size::INFINITE,
                        line_height: ellipsis.line_height.unwrap_or_default(),
                        align_x: text::Alignment::Default,
                        align_y: iced::alignment::Vertical::Top,
                        shaping: ellipsis.shaping.unwrap_or(text::Shaping::Advanced),
                        wrapping: text::Wrapping::None,
                        ellipsis: text::Ellipsis::None,
                        hint_factor: renderer.scale_factor(),
                    }
                }
                None => {
                    let value = ELLIPSIS.to_owned();
                    text(renderer, value, font, size)
                }
            },
        };

        let text = text::paragraph::Plain::new(text);

        Self { text, page }
    }
}

struct State<P: text::Paragraph> {
    reference: text::paragraph::Plain<P>,
    pages: Vec<PageState<P>>,
    hovered: Option<Page>,
}

impl<P: text::Paragraph> State<P> {
    fn new() -> Self {
        Self {
            reference: text::paragraph::Plain::default(),
            pages: Vec::default(),
            hovered: None,
        }
    }

    fn max_size<Renderer>(
        &mut self,
        renderer: &Renderer,
        end: usize,
        font: Option<Renderer::Font>,
        size: Option<Pixels>,
        padding: Padding,
        ellipsis: Option<&Ellipsis<Renderer::Font>>,
    ) -> Size
    where
        Renderer: text::Renderer<Font = P::Font>,
    {
        fn text<'a, Renderer: text::Renderer>(
            renderer: &Renderer,
            value: &'a str,
            font: Option<Renderer::Font>,
            size: Option<Pixels>,
        ) -> text::Text<&'a str, Renderer::Font> {
            text::Text {
                content: value,
                size: size.unwrap_or(renderer.default_size()),
                font: font.unwrap_or(renderer.default_font()),
                line_height: text::LineHeight::default(),
                bounds: Size::INFINITE,
                align_x: text::Alignment::Default,
                align_y: iced::alignment::Vertical::Top,
                shaping: text::Shaping::default(),
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: renderer.scale_factor(),
            }
        }

        let mut ellipsis_content = [0; 4];
        let ellipsis = match ellipsis {
            Some(ellipsis) => text::Text {
                content: ellipsis.code_point.encode_utf8(&mut ellipsis_content) as &_,
                size: ellipsis.size.unwrap_or(renderer.default_size()),
                font: ellipsis.font,
                bounds: Size::INFINITE,
                line_height: ellipsis.line_height.unwrap_or_default(),
                align_x: text::Alignment::Default,
                align_y: iced::alignment::Vertical::Top,
                shaping: ellipsis.shaping.unwrap_or(text::Shaping::Advanced),
                wrapping: text::Wrapping::None,
                ellipsis: text::Ellipsis::None,
                hint_factor: renderer.scale_factor(),
            },
            None => text(renderer, ELLIPSIS, font, size),
        };

        self.reference.update(ellipsis);

        let ellipsis = self.reference.min_bounds();

        let max = end.to_string();
        let max = text(renderer, max.deref(), font, size);

        self.reference.update(max);

        let end = self.reference.min_bounds();

        if end.width >= ellipsis.width {
            end
        } else {
            ellipsis
        }
        .expand(padding)
    }
}

pub mod buttoned {
    use super::*;
    use iced::widget::button;

    /// A pagination widget with buttons
    pub struct Buttoned<'a, Message, Theme: Catalog, Renderer: text::Renderer> {
        pages: Pagination<'a, ButtonedEvent, Theme, Renderer>,
        next: button::Button<'a, ButtonedEvent, Theme, Renderer>,
        prev: button::Button<'a, ButtonedEvent, Theme, Renderer>,
        on_select: Option<Box<dyn Fn(Page) -> Message + 'a>>,
        spacing: f32,
        width: Length,
    }

    impl<'a, Message, Theme: 'a + Catalog, Renderer: 'a + text::Renderer>
        Buttoned<'a, Message, Theme, Renderer>
    {
        pub(super) fn new(pages: Pagination<'a, Message, Theme, Renderer>) -> Self {
            let Pagination {
                start,
                current,
                end,
                ellipsis,
                width,
                padding,
                spacing,
                size,
                font,
                on_select,
                class,
            } = pages;

            let pages = Pagination::new(start, current, end)
                .ellipsis_maybe(ellipsis)
                .width(width)
                .padding(padding)
                .spacing(spacing)
                .size_maybe(size)
                .font_maybe(font)
                .on_select(buttoned::ButtonedEvent::Page)
                .class(class);

            let mut prev = iced::widget::text("❮ Back");
            let mut next = iced::widget::text("Next ❯");

            if let Some(size) = pages.size {
                prev = prev.size(size);
                next = next.size(size);
            }

            if let Some(font) = pages.font {
                prev = prev.font(font);
                next = next.font(font);
            }

            let prev = button(prev)
                .on_press_maybe((pages.start != pages.current).then_some(ButtonedEvent::Previous))
                .clip(true);

            let next = button(next)
                .on_press_maybe((pages.current != pages.end).then_some(ButtonedEvent::Next))
                .clip(true);

            Self {
                pages,
                prev,
                next,
                on_select,
                spacing: 16.0,
                width: Length::Fit,
            }
        }

        /// Sets the function that will be called when a [`Page`] is selected.
        pub fn on_select(mut self, on_select: impl Fn(Page) -> Message + 'a) -> Self {
            self.on_select = Some(Box::new(on_select));
            self
        }

        /// Sets the width of the [`Pagination`].
        pub fn width(mut self, width: impl Into<Length>) -> Self {
            self.width = width.into();
            self
        }

        /// Sets the padding for each button in the [`Buttoned`].
        pub fn button_padding(mut self, padding: impl Into<Padding>) -> Self {
            let padding = padding.into();
            self.prev = self.prev.padding(padding);
            self.next = self.next.padding(padding);
            self
        }

        /// Sets the spacing between components in the [`Buttoned`].
        pub fn spacing(mut self, spacing: f32) -> Self {
            self.spacing = spacing;
            self
        }

        /// Sets the style of the back button in the [`Buttoned`].
        pub fn back_style(
            mut self,
            style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
        ) -> Self
        where
            <Theme as button::Catalog>::Class<'a>: From<button::StyleFn<'a, Theme>>,
        {
            self.prev = self.prev.style(style);
            self
        }

        /// Sets the style of the next button in the [`Buttoned`].
        pub fn next_style(
            mut self,
            style: impl Fn(&Theme, button::Status) -> button::Style + 'a,
        ) -> Self
        where
            <Theme as button::Catalog>::Class<'a>: From<button::StyleFn<'a, Theme>>,
        {
            self.next = self.next.style(style);
            self
        }

        /// Sets the style class of the back button in the [`Buttoned`].
        pub fn back_class(
            mut self,
            class: impl Into<<Theme as button::Catalog>::Class<'a>>,
        ) -> Self {
            self.prev = self.prev.class(class);
            self
        }

        /// Sets the style class of the back button in the [`Buttoned`].
        pub fn next_class(
            mut self,
            class: impl Into<<Theme as button::Catalog>::Class<'a>>,
        ) -> Self {
            self.next = self.next.class(class);
            self
        }
    }

    impl<'a, Message, Theme, Renderer> Widget<Message, Theme, Renderer>
        for Buttoned<'a, Message, Theme, Renderer>
    where
        Theme: 'a + Catalog,
        Renderer: 'a + text::Renderer,
    {
        fn size(&self) -> Size<Length> {
            Size {
                width: self.width,
                height: Length::Fit,
            }
        }

        fn diff(&mut self, tree: &mut tree::Tree) {
            tree.diff_children(&mut [
                &mut self.pages as &mut dyn Widget<_, _, _>,
                &mut self.prev as &mut dyn Widget<_, _, _>,
                &mut self.next as &mut dyn Widget<_, _, _>,
            ]);
        }

        fn layout(
            &mut self,
            tree: &mut tree::Tree,
            renderer: &Renderer,
            limits: &layout::Limits,
        ) -> layout::Node {
            let new_limits = &limits.width(self.width);

            let prev = self
                .prev
                .layout(&mut tree.children[1], renderer, new_limits);
            let prev_size = prev.size();
            let shrink = Size::new(prev_size.width + self.spacing, 0.0);
            let new_limits = &new_limits.shrink(shrink);

            let next = self
                .next
                .layout(&mut tree.children[2], renderer, new_limits);
            let next_size = next.size();
            let shrink = Size::new(next_size.width + self.spacing, 0.0);
            let new_limits = &new_limits.shrink(shrink);

            let pages = self
                .pages
                .layout(&mut tree.children[0], renderer, new_limits);
            let pages_size = pages.size();

            let height = prev_size
                .height
                .max(pages_size.height)
                .max(next_size.height);

            let pages = {
                let y = (height - pages_size.height) / 2.0;
                pages.translate([prev_size.width + self.spacing, y])
            };
            let next = next.translate([
                prev_size.width + pages_size.width + (2.0 * self.spacing),
                0.0,
            ]);

            let intrinsic = Size::new(
                prev_size.width + pages_size.width + next_size.width + (2.0 * self.spacing),
                height,
            );

            let size = limits.resolve(self.width, Length::Fit, intrinsic);

            layout::Node::with_children(size, vec![pages, prev, next])
        }

        fn draw(
            &self,
            tree: &tree::Tree,
            renderer: &mut Renderer,
            theme: &Theme,
            style: &advanced::renderer::Style,
            layout: layout::Layout<'_>,
            cursor: advanced::mouse::Cursor,
            viewport: &Rectangle,
        ) {
            let Some(viewport) = layout.bounds().intersection(viewport) else {
                return;
            };

            let mut children = layout.children();

            {
                let pages_layout = children
                    .next()
                    .expect("Buttoned Pagination draw missing pages layout");
                self.pages.draw(
                    &tree.children[0],
                    renderer,
                    theme,
                    style,
                    pages_layout,
                    cursor,
                    &viewport,
                );
            }

            {
                let prev_layout = children
                    .next()
                    .expect("Buttoned Pagination draw missing prev button layout");
                self.prev.draw(
                    &tree.children[1],
                    renderer,
                    theme,
                    style,
                    prev_layout,
                    cursor,
                    &viewport,
                );
            }

            {
                let next_layout = children
                    .next()
                    .expect("Buttoned Pagination draw missing next button layout");
                self.next.draw(
                    &tree.children[2],
                    renderer,
                    theme,
                    style,
                    next_layout,
                    cursor,
                    &viewport,
                );
            }
        }

        fn update(
            &mut self,
            tree: &mut tree::Tree,
            event: &Event,
            layout: layout::Layout<'_>,
            cursor: advanced::mouse::Cursor,
            renderer: &Renderer,
            shell: &mut advanced::Shell<'_, Message>,
            viewport: &Rectangle,
        ) {
            let mut children = layout.children();
            let mut local_messages = Vec::new();
            let mut local_shell = shell.local(&mut local_messages);

            let pages_layout = children
                .next()
                .expect("Buttoned Pagination update missing pages layout");
            self.pages.update(
                &mut tree.children[0],
                event,
                pages_layout,
                cursor,
                renderer,
                &mut local_shell,
                viewport,
            );

            let prev_layout = children
                .next()
                .expect("Buttoned Pagination update missing prev layout");

            self.prev.update(
                &mut tree.children[1],
                event,
                prev_layout,
                cursor,
                renderer,
                &mut local_shell,
                viewport,
            );

            let next_layout = children
                .next()
                .expect("Buttoned Pagination update missing next layout");
            self.next.update(
                &mut tree.children[2],
                event,
                next_layout,
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

            for message in local_messages {
                let page = match message {
                    ButtonedEvent::Page(page) => page,
                    ButtonedEvent::Next => {
                        let page = (self.pages.current + 1).min(self.pages.end);
                        Page::Number(page)
                    }
                    ButtonedEvent::Previous => {
                        let page = self.pages.current.saturating_sub(1).max(self.pages.start);
                        Page::Number(page)
                    }
                };

                if let Some(on_select) = self.on_select.as_ref() {
                    shell.publish((on_select)(page));
                    shell.invalidate_layout();
                }
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

            {
                let pages_layout = children
                    .next()
                    .expect("Buttoned Pagination interaction missing pages layout");

                let interaction = self.pages.mouse_interaction(
                    &tree.children[0],
                    pages_layout,
                    cursor,
                    viewport,
                    renderer,
                );

                if !matches!(interaction, mouse::Interaction::None) {
                    return interaction;
                }
            }

            {
                let prev_layout = children
                    .next()
                    .expect("Buttoned Pagination interaction missing prev layout");

                let interaction = self.prev.mouse_interaction(
                    &tree.children[1],
                    prev_layout,
                    cursor,
                    viewport,
                    renderer,
                );

                if !matches!(interaction, mouse::Interaction::None) {
                    return interaction;
                }
            }

            {
                let next_layout = children
                    .next()
                    .expect("Buttoned Pagination interaction missing next layout");

                let interaction = self.next.mouse_interaction(
                    &tree.children[2],
                    next_layout,
                    cursor,
                    viewport,
                    renderer,
                );

                if !matches!(interaction, mouse::Interaction::None) {
                    return interaction;
                }
            }

            mouse::Interaction::None
        }
    }

    impl<'a, Message, Theme, Renderer> From<Buttoned<'a, Message, Theme, Renderer>>
        for Element<'a, Message, Theme, Renderer>
    where
        Message: 'a,
        Theme: 'a + Catalog,
        Renderer: 'a + text::Renderer,
    {
        fn from(value: Buttoned<'a, Message, Theme, Renderer>) -> Self {
            Element::new(value)
        }
    }

    #[derive(Clone)]
    pub(super) enum ButtonedEvent {
        Page(Page),
        Next,
        Previous,
    }

    pub trait Catalog: super::Catalog + button::Catalog + iced::widget::text::Catalog {
        fn default_pagination<'a>() -> <Self as super::Catalog>::Class<'a> {
            <Self as super::Catalog>::default()
        }

        fn default_button<'a>() -> <Self as button::Catalog>::Class<'a> {
            <Self as button::Catalog>::default()
        }
    }

    impl<T> Catalog for T where T: super::Catalog + button::Catalog + iced::widget::text::Catalog {}
}
