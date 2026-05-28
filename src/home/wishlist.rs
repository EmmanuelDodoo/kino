use super::{HomeMessage, PageKind, shared::*};
use crate::utils::typo;
use crate::utils::{Scroll, delete_btn, empty, icons, styles, typo::*};
use devutils::image_ops::sample_complement;
use iced::{
    Color, Element, Length, Padding, Task,
    alignment::{Horizontal, Vertical},
    animation::{Animation, Easing},
    task,
    time::Instant,
    widget::{
        self, button, center_x, column, container, grid, image, operation, row, scrollable, space,
        stack, text,
    },
};
use widgets::modal;

use registry::models::{self, Wish, WishId, WishKind};

#[derive(Debug, Clone)]
pub enum WishlistMessage {
    Scroll(scrollable::Viewport),
    Hover(WishId, bool),
    New,
    Select(WishId),
    Edit(WishId),
    Complete(WishId),
    Delete(WishId, String),
}

#[derive(Debug, Clone)]
pub struct Wishlist {
    scroll: Scroll,
}

impl Wishlist {
    pub fn boot() -> (Self, Task<WishlistMessage>) {
        let (new, id) = Self::new();
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new() -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (Self { scroll }, id)
    }

    pub fn update(&mut self, message: WishlistMessage) -> Option<HomeMessage> {
        match message {
            WishlistMessage::Complete(wish) => Some(HomeMessage::WishCompletion(wish)),
            WishlistMessage::Delete(id, name) => {
                Some(HomeMessage::OpenView(super::ViewMessage::RemoveWish {
                    id,
                    name,
                }))
            }
            WishlistMessage::Hover(id, is_hovered) => {
                let msg = HomeMessage::WishHovered(id, is_hovered);
                Some(msg)
            }
            WishlistMessage::Scroll(viewport) => {
                self.scroll.offset = viewport.absolute_offset();
                None
            }
            WishlistMessage::New => Some(HomeMessage::OpenView(super::ViewMessage::Wish(None))),
            WishlistMessage::Edit(id) => {
                Some(HomeMessage::OpenView(super::ViewMessage::Wish(Some(id))))
            }
            WishlistMessage::Select(id) => {
                Some(HomeMessage::OpenView(super::ViewMessage::WishModal(id)))
            }
        }
    }

    pub fn view<'a>(
        &self,
        wishlist: impl Iterator<Item = &'a WishThumbnail>,
        now: Instant,
    ) -> Element<'a, WishlistMessage> {
        let add = {
            let btn = button(typo::medium("New"))
                .style(styles::button::text_primary)
                .on_press(WishlistMessage::New);

            row!(space::horizontal(), btn)
        };

        let content = wishlist.map(|wish| wish.card(now));

        let content = grid(content)
            .spacing(16)
            .fluid(WishThumbnail::WIDTH)
            .height(grid::aspect_ratio(
                WishThumbnail::WIDTH,
                WishThumbnail::HEIGHT,
            ));

        let content = scrollable(container(content).padding(16))
            .auto_scroll(true)
            .height(Length::Fill)
            .id(self.scroll.id.clone())
            .on_scroll(WishlistMessage::Scroll);

        let content = column!(add, content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding(10);

        content.into()
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }
}

#[derive(Debug, Clone)]
pub struct WishThumbnailTask {
    pub id: WishId,
    pub kind: ThumbnailTaskKind,
}

#[derive(Debug, Clone)]
pub struct WishThumbnail {
    poster: Image,
    sample_text: Option<Color>,
    sample_color: Option<Color>,
    background: Animation<bool>,
    icon: Animation<bool>,
    float: Animation<bool>,
    _tasks: task::Handle,
    hovered: bool,
    pub item: Box<Wish>,
}

impl WishThumbnail {
    pub const WIDTH: f32 = CARD_WIDTH;
    pub const HEIGHT: f32 = CARD_HEIGHT;

    pub fn new(wish: Wish) -> (Self, Task<WishThumbnailTask>) {
        let id = wish.id;

        let (poster, task) = Image::load(wish.poster.as_ref());
        let (task, handle) = task
            .map(move |kind| WishThumbnailTask { id, kind })
            .abortable();
        let handle = handle.abort_on_drop();

        let (sample_color, sample_text) = match wish.poster.as_ref() {
            Some(poster) => (
                poster.get_main().map(to_color),
                poster.get_accent().map(to_color),
            ),
            None => (None, None),
        };

        let new = Self {
            poster,
            sample_color,
            sample_text,
            background: background_animation(),
            icon: icon_animation(),
            float: float_animation(),
            item: Box::new(wish),
            hovered: false,
            _tasks: handle,
        };

        (new, task)
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        let poster = match &self.poster {
            Image::Shown { fade_in, .. } => fade_in.is_animating(now),
            _ => false,
        };

        self.background.is_animating(now)
            || self.icon.is_animating(now)
            || self.float.is_animating(now)
            || poster
    }

    pub fn go_mut(&mut self, new_state: bool, at: Instant) {
        self.hovered = new_state;
        self.background.go_mut(new_state, at);
        self.icon.go_mut(new_state, at);
        self.float.go_mut(new_state, at);
    }

    pub fn task(&mut self, task: ThumbnailTaskKind, now: Instant) {
        match task {
            ThumbnailTaskKind::Samples { main, accent } => {
                self.sample_color = Some(main);
                self.sample_text = Some(accent);
            }
            ThumbnailTaskKind::Image(Ok(allocation)) => {
                self.poster = Image::Shown {
                    allocation,
                    fade_in: fade_in(now),
                };
            }
            ThumbnailTaskKind::Image(Err(error)) => {
                tracing::error!("Wish Thumbnail poster allocation error: \n{error}");
            }
        }
    }

    fn card(&self, now: Instant) -> Element<'_, WishlistMessage> {
        let background_inter = if self.item.completed {
            1.0
        } else {
            self.background.interpolate(0.0, 1.0, now)
        };

        let card = Card {
            sample_color: self.sample_color,
            background_inter,
            selected: false,
            item: self.item.id,
            poster: &self.poster,
            title: card_title(&self.item.name, self.hovered),
            details: Some(self.card_details()),
            overlay: Some(self.card_overlay()),
            float_anim: (!self.item.completed).then_some(&self.float),
        };

        card.view(
            now,
            Self::WIDTH,
            Self::HEIGHT,
            |id| Some(WishlistMessage::Select(id)),
            WishlistMessage::Hover,
            WishlistMessage::Hover,
        )
    }

    fn card_overlay(&self) -> Element<'_, WishlistMessage> {
        let padding = [3, 6];
        let sample = self.sample_text;

        let color = move |theme: &iced::Theme| {
            if sample.is_some() {
                text::Style { color: sample }
            } else {
                text::Style {
                    color: Some(theme.palette().primary.strong.text),
                }
            }
        };

        let (tag, bottom) = match &self.item.kind {
            WishKind::Movie { duration, .. } => {
                let duration = if *duration > 0 {
                    models::duration_full(*duration)
                } else {
                    String::default()
                };
                ("movie", duration)
            }
            WishKind::Show { seasons, .. } => ("show", format!("{seasons} seasons")),
            WishKind::Season { episodes, .. } => ("season", format!("{episodes} episodes")),
            WishKind::Episode { duration, .. } => {
                let duration = if *duration > 0 {
                    models::duration_full(*duration)
                } else {
                    String::default()
                };
                ("episode", duration)
            }
        };

        let tag = container(sized_bold(tag, H8))
            .padding([2, 4])
            .style(move |theme| {
                let color = sample.unwrap_or_else(|| theme.palette().primary.strong.color);
                let default = container::rounded_box(theme);
                let border = default.border.color(color).rounded(3.0).width(1.50);

                container::Style {
                    text_color: Some(color),
                    border,
                    background: None,
                    ..Default::default()
                }
            });

        let bottom = sized_medium(bottom, H8).style(color);

        let top = row!(tag, space::horizontal())
            .align_y(Vertical::Center)
            .padding(padding);

        let bottom = row!(space::horizontal(), bottom)
            .align_y(Vertical::Center)
            .padding(padding);

        let content = column!(top, space::vertical(), bottom);

        if self.item.completed {
            use iced::color;
            let curtain = container("")
                .width(Length::Fill)
                .height(Length::Fill)
                .style(|theme: &iced::Theme| {
                    let color = theme.palette().primary.weak.color.scale_alpha(0.5);

                    container::background(color)
                });

            stack![content, curtain].into()
        } else {
            content.into()
        }
    }

    fn card_details(&self) -> Element<'_, WishlistMessage> {
        let ratings = ratings(self.item.rating, true);

        let right = match &self.item.kind {
            WishKind::Movie { .. } | WishKind::Show { .. } => {
                let release = self.item.release_year();
                let icon = icons::icon(icons::CALENDAR).size(H7);

                row!(icon, sized_medium(release, H8))
                    .align_y(Vertical::Center)
                    .spacing(3)
            }

            WishKind::Season { number, .. } => {
                let season = format!("Season {number}");
                let icon = icons::icon(icons::NUMBER).size(H7);

                row!(icon, sized_medium(season, H8)).align_y(Vertical::Center)
            }
            WishKind::Episode { season, number, .. } => {
                let episode = format!("S{season:02}E{number:02}");
                let icon = icons::icon(icons::NUMBER).size(H7);

                row!(icon, sized_medium(episode, H8)).align_y(Vertical::Center)
            }
        };

        row!(ratings, space::horizontal(), right)
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .into()
    }

    pub fn modal(&self, now: Instant) -> Element<'_, WishlistMessage> {
        use iced::ContentFit;

        let primary_text = |theme: &iced::Theme| text::Style {
            color: Some(theme.palette().primary.base.color),
        };

        let poster = {
            let sample = self.sample_color.unwrap_or(Color::BLACK);
            container(image_poster(
                &self.poster,
                Length::Fill,
                Length::Shrink,
                ContentFit::Contain,
                now,
            ))
            .max_height(250)
            .style(move |_| {
                let mut style = container::background(sample);
                style.border = style.border.rounded(IMAGE_RADIUS);

                style
            })
        };

        let name = {
            let tag = match &self.item.kind {
                WishKind::Movie { .. } => "movie",
                WishKind::Show { .. } => "show",
                WishKind::Season { .. } => "season",
                WishKind::Episode { .. } => "episode",
            };

            let color = self.sample_color;
            let tag = sized_bold(tag, H8).color_maybe(color);

            let name = sized_medium(&self.item.name, H6);

            column!(tag, name).spacing(4)
        };

        let details = {
            let details = match &self.item.kind {
                WishKind::Movie { tags, .. } => tags_row(tags),
                WishKind::Show { tags, .. } => tags_row(tags),
                WishKind::Season { number, .. } => {
                    h8(format!("Season {number}")).style(primary_text).into()
                }

                WishKind::Episode { season, number, .. } => {
                    h8(format!("Season {season} Episode {number}"))
                        .style(primary_text)
                        .into()
                }
            };

            let rating = ratings(self.item.rating, true);

            column!(details, rating).spacing(4)
        };

        let title = column!(name, details).spacing(8);

        let overview = {
            let overview = container(regular(&self.item.synopsis)).max_height(300);

            scrollable(overview)
        };

        let actions = {
            let style = |theme: &iced::Theme, status: button::Status| {
                let default = styles::button::subtle(theme, status);

                let border = default.border.rounded(5);

                button::Style { border, ..default }
            };

            let edit = icon_button(icons::RENAME, "Edit", None)
                .style(style)
                .on_press(WishlistMessage::Edit(self.item.id));

            let complete = {
                let label = if self.item.completed {
                    "Done"
                } else {
                    "Pending"
                };

                icon_button(icons::CHECK, label, Some(self.item.completed))
                    .style(style)
                    .on_press(WishlistMessage::Complete(self.item.id))
            };

            let delete_name = match &self.item.kind {
                WishKind::Movie { .. } | WishKind::Show { .. } => self.item.name.clone(),
                WishKind::Season { number, .. } => format!("{} Season {number}", self.item.name),
                WishKind::Episode { season, number, .. } => {
                    format!("{} S{season:02}{number:02}", self.item.name)
                }
            };

            let delete = delete_btn("Delete")
                .style(style)
                .on_press(WishlistMessage::Delete(self.item.id, delete_name));

            row!(complete, edit, delete)
                .spacing(20)
                .align_y(Vertical::Center)
        };

        let content = column!(poster, title, overview, actions).spacing(36);

        content.into()
    }
}

fn tags_row<'a, Message: 'a>(tags: &'a [String]) -> Element<'a, Message> {
    let separator = || Element::from(text("•").line_height(1.0).size(H6));
    let len = 3;

    let mut res = vec![];
    let tag_len = tags.len().min(len);

    for (i, tag) in tags.iter().enumerate().take(len) {
        let tag = h8(tag).style(|theme| text::Style {
            color: Some(theme.palette().primary.base.color),
        });

        res.push(Element::from(tag));

        if i < tag_len - 1 {
            res.push(separator())
        }
    }

    row(res).align_y(Vertical::Center).spacing(3).into()
}

fn icon_button<'a, Message: 'a + Clone>(
    icon: char,
    label: &'a str,
    primary: Option<bool>,
) -> button::Button<'a, Message> {
    let icon = icons::icon(icon).size(P).style(move |theme| {
        let color = match primary {
            Some(true) => Some(theme.palette().primary.base.color),
            Some(false) => Some(theme.palette().success.base.color),
            None => None,
        };

        text::Style { color }
    });

    button(
        row!(icon, typo::sized_medium(label, typo::H7))
            .spacing(10.0)
            .align_y(iced::alignment::Vertical::Center),
    )
    .padding([6, 12])
    .style(|theme, status| {
        let default = styles::button::subtlest(theme, status);
        let border = default.border.rounded(5);

        button::Style { border, ..default }
    })
}
