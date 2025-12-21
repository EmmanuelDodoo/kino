use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::models::{
    CollectionId, Episode, EpisodeId, ItemId, Media, Season, SeasonId, SimpleCollection,
};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::tooltip;
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll, Sort, empty, styles};
use iced::widget::Space;
use iced::{
    Element, Length, Task,
    alignment::{Horizontal, Vertical},
    time::Instant,
    widget::{
        self, button, column, container, grid, operation, row, rule, scrollable, space, stack,
        text, tooltip as tp,
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    AddSelf,
    Add(EpisodeId),
    Hovered(EpisodeId, bool),
    Details(EpisodeId),
    Resume,
    Tab(Tab),
    Play(EpisodeId),
    Scroll(scrollable::Viewport),
    Rate(Option<f32>),
    Goto(CollectionId),
    Rename(String),
    Refetch,
    Remove,
}

#[derive(Debug, Clone)]
pub struct SeasonPageMessage {
    pub id: SeasonId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct SeasonPage {
    id: SeasonId,
    tab: Tab,
    scroll: Scroll,
}

impl SeasonPage {
    pub fn boot(season: SeasonId) -> (Self, Task<SeasonPageMessage>) {
        let (new, id) = Self::new(season);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new(season: SeasonId) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                id: season,
                tab: Tab::Items,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: SeasonPageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Hovered(id, is_hovered) => {
                let msg = HomeMessage::Hovered(ItemId::Episode(id), is_hovered);

                Some(msg)
            }
            Message::AddSelf => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Season(self.id)));

                Some(msg)
            }
            Message::Add(id) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Episode(id)));

                Some(msg)
            }
            Message::Details(id) => {
                let msg = HomeMessage::Goto(PageKind::Episode(id));

                Some(msg)
            }
            Message::Tab(tab) => {
                self.tab = tab;
                None
            }
            Message::Resume => {
                let msg = HomeMessage::Play(ItemId::Season(self.id));

                Some(msg)
            }
            Message::Play(id) => {
                let msg = HomeMessage::Play(ItemId::Episode(id));

                Some(msg)
            }
            Message::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                None
            }
            Message::Rate(rating) => {
                let msg =
                    HomeMessage::OpenView(ViewMessage::Rating(ItemId::Season(self.id), rating));
                Some(msg)
            }
            Message::Goto(id) => {
                let msg = HomeMessage::Goto(PageKind::Collection(id));
                Some(msg)
            }
            Message::Rename(name) => {
                let msg = HomeMessage::OpenView(ViewMessage::Rename {
                    id: self.id.into(),
                    old: name,
                });

                Some(msg)
            }
            Message::Refetch => {
                let msg = HomeMessage::Refetch(self.id.into());
                Some(msg)
            }
            Message::Remove => {
                let msg = HomeMessage::Remove(self.id.into());
                Some(msg)
            }
        }
    }

    pub fn show_tools(&self) -> bool {
        true
    }

    pub fn update_scroll(&mut self) -> Task<()> {
        operation::scroll_to(self.scroll.id.clone(), self.scroll.offset)
    }

    fn list<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Episode>>,
    ) -> Element<'a, SeasonPageMessage> {
        let season = self.id;

        let content = thumbnails.map(|thumbnail| {
            thumbnail.list(
                now,
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Add(id),
                },
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Details(id),
                },
                move |id, hovered| SeasonPageMessage {
                    id: season,
                    message: Message::Hovered(id, hovered),
                },
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Play(id),
                },
                |_| empty(),
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(move |viewpport| SeasonPageMessage {
                    id: season,

                    message: Message::Scroll(viewpport),
                }),
        );

        content.into()
    }

    fn compact<'a>(
        &self,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Episode>>,
    ) -> Element<'a, SeasonPageMessage> {
        let season = self.id;

        let content = thumbnails.map(|thumbnail| {
            thumbnail.compact(
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Add(id),
                },
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Details(id),
                },
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Play(id),
                },
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(move |viewpport| SeasonPageMessage {
                    id: season,

                    message: Message::Scroll(viewpport),
                }),
        );

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Episode>>,
    ) -> Element<'a, SeasonPageMessage> {
        let season = self.id;

        let content = thumbnails.map(|thumbnail| {
            thumbnail.card(
                now,
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Add(id),
                },
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Details(id),
                },
                move |id, hovered| SeasonPageMessage {
                    id: season,
                    message: Message::Hovered(id, hovered),
                },
                move |id| SeasonPageMessage {
                    id: season,
                    message: Message::Play(id),
                },
            )
        });

        let content = grid(content)
            .spacing(12)
            .fluid(CARD_WIDTH)
            .height(grid::aspect_ratio(CARD_WIDTH, CARD_HEIGHT));

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(move |viewpport| SeasonPageMessage {
                    id: season,

                    message: Message::Scroll(viewpport),
                }),
        )
        .padding(10);

        content.into()
    }

    fn top<'a>(&self, season: &'a Thumbnail<Season>) -> Element<'a, SeasonPageMessage> {
        let id = self.id;

        let img_height = CARD_HEIGHT * 0.65;
        let img: Element<'_, SeasonPageMessage> = {
            let ratio = 2.0 / 3.0;
            season.poster(img_height * ratio, img_height)
        };

        let header = {
            let separator = || Element::from(text("•").line_height(0.9).size(H4));

            let title = title(
                season.media.name(),
                SeasonPageMessage {
                    id,
                    message: Message::Rename(season.media.name().to_owned()),
                },
                SeasonPageMessage {
                    id,
                    message: Message::Refetch,
                },
                SeasonPageMessage {
                    id,
                    message: Message::Remove,
                },
            );
            let duration = duration(&season.media);
            let rating = button(ratings(&season.media, true))
                .on_press(SeasonPageMessage {
                    id,
                    message: Message::Rate(season.media.rating()),
                })
                .style(styles::button::text)
                .padding(0);
            let release = text(season.media.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let synopsis = container(text(season.media.synopsis()))
                .max_width(750)
                .height(Length::Fill);

            let actions = row!(
                button(
                    row!(icon(PLAY).size(P), text("Resume").size(H7))
                        .spacing(10.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(SeasonPageMessage {
                    id,
                    message: Message::Resume
                })
                .style(|theme, status| {
                    let default = styles::button::subtlest(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
                button(
                    row!(
                        icon(ADD_COLLECTION).size(P),
                        text("Add to Collection").size(H7)
                    )
                    .spacing(10.0)
                    .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(SeasonPageMessage {
                    id,
                    message: Message::AddSelf
                })
                .style(|theme, status| {
                    let default = styles::button::subtlest(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
            )
            .align_y(Vertical::Center)
            .spacing(16.0);

            let details = column!(details, rating).spacing(8.0);

            column!(
                title,
                details,
                synopsis,
                space::vertical().height(3),
                actions
            )
            .height(img_height)
            .spacing(10.0)
        };

        let backdrop: Element<'_, SeasonPageMessage> = {
            let height = img_height + 68.5;

            season.backdrop(Length::Fill, height)
        };

        let content = row!(img, header).align_y(Vertical::Center).spacing(36.0);

        let item = "Episodes";
        let tabs = Tab::ALL.into_iter().map(move |tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .padding([3, 6])
                        .on_press(SeasonPageMessage {
                            id,
                            message: Message::Tab(tab)
                        })
                        .style(|theme, status| {
                            let default = styles::button::text_white(theme, status);

                            button::Style {
                                border: iced::Border::default(),
                                ..default
                            }
                        }),
                    container(Space::new().width(68).height(2)).style(if is_selected {
                        styles::container::pb
                    } else {
                        styles::container::transparent
                    }),
                )
                .align_x(Horizontal::Center)
                .padding([3, 6])
                .spacing(0.0),
            )
        });

        let tabs = row(tabs).spacing(40.0).align_y(Vertical::Center);
        let tabs = column!(tabs, rule::horizontal(2.0)).spacing(4.0);

        let content = container(column!(content, tabs).spacing(24))
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([4, 6])
            .style(|theme| {
                let default = styles::container::dark(theme);

                container::Style {
                    background: default
                        .background
                        .map(|background| background.scale_alpha(0.85)),
                    ..default
                }
            });

        let content = stack![backdrop, content];

        content.into()
    }

    pub fn view<'a>(
        &self,
        now: Instant,
        layout: Layout,
        season: &'a Thumbnail<Season>,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Episode>>,
        memberships: impl Iterator<Item = &'a SimpleCollection>,
    ) -> Element<'a, SeasonPageMessage> {
        let content = {
            let width = 750.0;
            let id = self.id;

            match self.tab {
                Tab::Items => match layout {
                    Layout::Grid => self.grid(now, thumbnails),
                    Layout::List => self.list(now, thumbnails),
                    Layout::Compact => self.compact(thumbnails),
                },
                Tab::Data => data_tab(&season.media, width),
                Tab::Comments => {
                    // todo
                    let comments = ["Some comment here: "; 7]
                        .into_iter()
                        .enumerate()
                        .map(|(i, comment)| Element::from(text(format!("{comment}{i}"))));

                    let comments =
                        scrollable(column(comments).spacing(4.0).width(Length::Fill)).spacing(4.0);

                    column!(comments).spacing(8.0).width(width).into()
                }
                Tab::Collections => {
                    let collections = memberships.map(|collection| {
                        draw_collection_tab(collection, move |collection| SeasonPageMessage {
                            id,
                            message: Message::Goto(collection),
                        })
                    });

                    scrollable(column(collections).spacing(4.0).width(Length::Fill))
                        .spacing(4.0)
                        .into()
                }
            }
        };

        let content = column!(self.top(season), content).spacing(20.0).padding(10);

        content.into()
    }
}
