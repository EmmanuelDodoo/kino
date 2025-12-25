use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::models::{
    CollectionId, ItemId, Media, Season, SeasonId, Show, ShowId, SimpleCollection,
};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::tooltip;
use crate::utils::typo::*;
use crate::utils::{Layout, Scroll, Sort, styles};
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
    Add(SeasonId),
    AddSelf,
    Hovered(SeasonId, bool),
    Details(SeasonId),
    Resume,
    Tab(Tab),
    Play(SeasonId),
    Scroll(scrollable::Viewport),
    Rate(Option<f32>),
    Goto(CollectionId),
    Rename(String),
    Synopsis(String),
    Refetch,
    Remove(String),
    TMDB,
}

#[derive(Debug, Clone)]
pub struct ShowPageMessage {
    pub id: ShowId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct ShowPage {
    id: ShowId,
    tab: Tab,
    scroll: Scroll,
}

impl ShowPage {
    pub fn boot(show: ShowId) -> (Self, Task<ShowPageMessage>) {
        let (new, id) = Self::new(show);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::<f32>::default());

        (new, scroll)
    }

    fn new(show: ShowId) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                id: show,
                tab: Tab::Items,
                scroll,
            },
            id,
        )
    }

    pub fn update(&mut self, message: ShowPageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Hovered(id, is_hovered) => {
                let msg = HomeMessage::Hovered(ItemId::Season(id), is_hovered);

                Some(msg)
            }
            Message::AddSelf => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Show(self.id)));

                Some(msg)
            }
            Message::Add(id) => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Season(id)));

                Some(msg)
            }
            Message::Resume => {
                let msg = HomeMessage::Play(ItemId::Show(self.id));

                Some(msg)
            }
            Message::Tab(tab) => {
                self.tab = tab;
                None
            }
            Message::Play(season) => {
                let msg = HomeMessage::Play(ItemId::Season(season));

                Some(msg)
            }
            Message::Details(id) => {
                let msg = HomeMessage::Goto(PageKind::Season(id));

                Some(msg)
            }
            Message::Scroll(view) => {
                self.scroll.offset = view.absolute_offset();
                None
            }
            Message::Rate(rating) => {
                let msg = HomeMessage::OpenView(ViewMessage::Rating(ItemId::Show(self.id), rating));
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
            Message::Synopsis(synopsis) => {
                let msg = HomeMessage::OpenView(ViewMessage::Synopsis {
                    id: self.id.into(),
                    old: synopsis,
                });

                Some(msg)
            }
            Message::Refetch => {
                let msg = HomeMessage::Refetch(self.id.into());
                Some(msg)
            }
            Message::Remove(name) => {
                let msg = HomeMessage::OpenView(ViewMessage::RemoveMedia {
                    id: self.id.into(),
                    name,
                });
                Some(msg)
            }
            Message::TMDB => {
                let msg = HomeMessage::OpenView(ViewMessage::TMDBId(self.id.into()));
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
        thumbnails: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, ShowPageMessage> {
        let show = self.id;

        let content = thumbnails.map(|thumbnail| {
            thumbnail.list(
                now,
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Add(id),
                },
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Details(id),
                },
                move |id, is_hovered| ShowPageMessage {
                    id: show,
                    message: Message::Hovered(id, is_hovered),
                },
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Play(id),
                },
                |season| {
                    let episodes = season.episodes;
                    let episodes = format!(
                        "{} episodes{}",
                        episodes,
                        if episodes > 1 { "s" } else { "" }
                    );
                    h7(episodes).into()
                },
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(move |viewport| ShowPageMessage {
                    id: show,
                    message: Message::Scroll(viewport),
                }),
        );

        content.into()
    }

    fn compact<'a>(
        &self,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, ShowPageMessage> {
        let show = self.id;

        let content = thumbnails.map(|thumbnail| {
            thumbnail.compact(
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Add(id),
                },
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Details(id),
                },
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Play(id),
                },
            )
        });

        let content = column(content).spacing(16);

        let content = container(
            scrollable(content)
                .spacing(20.0)
                .id(self.scroll.id.clone())
                .on_scroll(move |viewport| ShowPageMessage {
                    id: show,
                    message: Message::Scroll(viewport),
                }),
        );

        content.into()
    }

    fn grid<'a>(
        &self,
        now: Instant,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, ShowPageMessage> {
        let show = self.id;

        let content = thumbnails.map(|thumbnail| {
            thumbnail.card(
                now,
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Add(id),
                },
                move |id| ShowPageMessage {
                    id: show,
                    message: Message::Details(id),
                },
                move |id, is_hovered| ShowPageMessage {
                    id: show,
                    message: Message::Hovered(id, is_hovered),
                },
                move |id| ShowPageMessage {
                    id: show,
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
                .on_scroll(move |viewport| ShowPageMessage {
                    id: show,
                    message: Message::Scroll(viewport),
                }),
        )
        .padding(10);

        content.into()
    }

    fn top<'a>(&self, show: &'a Thumbnail<Show>) -> Element<'a, ShowPageMessage> {
        let id = self.id;

        let img_height = CARD_HEIGHT * 0.65;
        let img: Element<'_, ShowPageMessage> = {
            let ratio = 2.0 / 3.0;
            show.poster(img_height * ratio, img_height)
        };

        let header = {
            let separator = || Element::from(text("•").line_height(0.9).size(H4));

            let title = title(show.media.name());

            let duration = duration(&show.media);
            let rating = button(ratings(&show.media, true))
                .on_press(ShowPageMessage {
                    id,
                    message: Message::Rate(show.media.rating()),
                })
                .style(styles::button::text)
                .padding(0);
            let release = sized_medium(show.media.release_year(), H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let tags = {
                let mut tags = vec![];
                let tag_len = show.media.tags.len();

                for (i, tag) in show.media.tags.iter().enumerate() {
                    tags.push(Element::from(h8(tag)));

                    if i < tag_len - 1 {
                        tags.push(separator())
                    }
                }

                row(tags).spacing(6).align_y(Vertical::Center)
            };

            let synopsis = tab_synopsis(show.media.synopsis());

            let actions = row!(
                button(
                    row!(icon(PLAY).size(H5), sized_medium("Play", P))
                        .spacing(10.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(ShowPageMessage {
                    id,
                    message: Message::Resume
                })
                .style(|theme, status| {
                    let default = styles::button::primary(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
                button(
                    row!(
                        icon(ADD_COLLECTION).size(H5),
                        sized_medium("Add to Collection", P)
                    )
                    .spacing(10.0)
                    .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(ShowPageMessage {
                    id,
                    message: Message::AddSelf
                })
                .style(|theme, status| {
                    let default = styles::button::primary(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                })
            )
            .align_y(Vertical::Center)
            .spacing(16.0);

            let details = column!(tags, details, rating).spacing(8.0);

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

        let backdrop: Element<'_, ShowPageMessage> = {
            let height = img_height + 71.0;

            show.backdrop(Length::Fill, height)
        };

        let content = row!(img, header).align_y(Vertical::Center).spacing(36.0);

        let item = "Seasons";
        let tabs = Tab::VARIANTS.into_iter().map(move |tab| {
            let is_selected = self.tab == *tab;
            let text = if is_selected {
                bold(tab.to_str(item))
            } else {
                regular(tab.to_str(item))
            };

            Element::from(
                column!(
                    button(text)
                        .padding([3, 6])
                        .on_press(ShowPageMessage {
                            id,
                            message: Message::Tab(*tab)
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
        show: &'a Thumbnail<Show>,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Season>>,
        memberships: impl Iterator<Item = &'a SimpleCollection>,
    ) -> Element<'a, ShowPageMessage> {
        let content = {
            let width = 750.0;
            let id = self.id;

            match self.tab {
                Tab::Items => match layout {
                    Layout::Grid => self.grid(now, thumbnails),
                    Layout::List => self.list(now, thumbnails),
                    Layout::Compact => self.compact(thumbnails),
                },
                Tab::Data => data_tab(
                    &show.media,
                    width,
                    Message::Rename(show.media.name().to_owned()),
                    Message::Refetch,
                    Message::Remove(show.media.name().to_owned()),
                    Message::Synopsis(show.media.synopsis().to_owned()),
                    Some(Message::TMDB),
                )
                .map(move |message| ShowPageMessage { id, message }),
                // todo
                // Tab::Comments => {
                //     let comments = ["Some comment here: "; 7]
                //         .into_iter()
                //         .enumerate()
                //         .map(|(i, comment)| Element::from(regular(format!("{comment}{i}"))));
                //
                //     let comments =
                //         scrollable(column(comments).spacing(4.0).width(Length::Fill)).spacing(4.0);
                //
                //     column!(comments).spacing(8.0).width(width).into()
                // }
                Tab::Collections => {
                    let collections = memberships.map(|collection| {
                        draw_collection_tab(collection, move |collection| ShowPageMessage {
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

        let content = column!(self.top(show), content).spacing(20.0).padding(10);

        content.into()
    }
}
