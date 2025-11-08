use super::{HomeMessage, PageKind, PageUpdate, ViewMessage, shared::*};
use crate::models::{ItemId, Media, shows::*};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort};
use iced::widget::Space;
use iced::{
    Element, Length, Task,
    alignment::{Horizontal, Vertical},
    time::Instant,
    widget::{
        self, button, column, container, grid, operation, row, rule, scrollable, space, stack, text,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Add(SeasonId),
    AddSelf,
    Hovered(SeasonId, bool),
    Details(SeasonId),
    Resume,
    Tab(Tab),
    Play(SeasonId),
    Scroll(scrollable::Viewport),
}

#[derive(Debug, Clone, Copy)]
pub struct ShowPageMessage {
    pub id: ShowId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct ShowPage {
    id: ShowId,
    layout: Layout,
    sort: Sort,
    filters: Filter,
    tab: Tab,
    scroll: Scroll,
}

impl ShowPage {
    pub fn boot(
        show: ShowId,
        sort: Sort,
        filters: Filter,
        layout: Layout,
    ) -> (Self, Task<ShowPageMessage>) {
        let (new, id) = Self::new(show, sort, filters, layout);
        let scroll = operation::scroll_to(id, scrollable::AbsoluteOffset::default());

        (new, scroll)
    }

    fn new(show: ShowId, sort: Sort, filters: Filter, layout: Layout) -> (Self, widget::Id) {
        let scroll = Scroll::new();
        let id = scroll.id.clone();

        (
            Self {
                id: show,
                layout,
                sort,
                filters,
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
        }
    }

    pub fn page_update(&mut self, update: PageUpdate) {
        let PageUpdate {
            layout,
            sort,
            filters,
        } = update.clone();

        self.sort = sort;
        self.layout = layout;
        self.filters = filters;
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

        let content = filter_sort(thumbnails, &self.filters, &self.sort).map(|thumbnail| {
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
                    text(episodes).size(H7).into()
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

        let content = filter_sort(thumbnails, &self.filters, &self.sort).map(|thumbnail| {
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
            .spacing(16)
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

        let img_height = CARD_HEIGHT * 0.85;
        let img: Element<'_, ShowPageMessage> = {
            let ratio = 2.0 / 3.0;
            show.poster(img_height * ratio, img_height)
        };

        let header = {
            let separator = || Element::from(text("•").line_height(0.9).size(H4));

            let title = text(show.media.name()).size(H2);
            let duration = duration(&show.media);
            let rating = ratings(&show.media);
            let release = text(show.media.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let tags = {
                let mut tags = vec![];
                let tag_len = show.media.tags.len();

                for (i, tag) in show.media.tags.iter().enumerate() {
                    tags.push(Element::from(text(tag).size(H7)));

                    if i < tag_len - 1 {
                        tags.push(separator())
                    }
                }

                row(tags).spacing(6).align_y(Vertical::Center)
            };

            let synapsis = container(text(show.media.synapsis()))
                .max_width(750)
                .height(Length::Fill);

            let actions = row!(
                button(
                    row!(icon(PLAY).size(P), text("Resume").size(H7))
                        .spacing(10.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(ShowPageMessage {
                    id,
                    message: Message::Resume
                })
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
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
                .on_press(ShowPageMessage {
                    id,
                    message: Message::AddSelf
                })
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
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
                space::vertical().height(3),
                synapsis,
                actions
            )
            .height(img_height)
            .spacing(10.0)
        };

        let backdrop: Element<'_, ShowPageMessage> = {
            let height = img_height + 68.5;

            show.backdrop(Length::Fill, height)
        };

        let content = row!(img, header).align_y(Vertical::Center).spacing(36.0);

        let item = "Seasons";
        let tabs = Tab::ALL.into_iter().map(move |tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .padding([3, 6])
                        .on_press(ShowPageMessage {
                            id,
                            message: Message::Tab(tab)
                        })
                        .style(|theme, status| {
                            let default = button::text(theme, status);

                            button::Style {
                                border: iced::Border::default(),
                                ..default
                            }
                        }),
                    container(Space::new().width(68).height(4)).style(if is_selected {
                        container::primary
                    } else {
                        container::transparent
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
                let default = container::dark(theme);

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
        show: &'a Thumbnail<Show>,
        thumbnails: impl Iterator<Item = &'a Thumbnail<Season>>,
    ) -> Element<'a, ShowPageMessage> {
        let content = {
            let width = 750.0;

            match self.tab {
                Tab::Items => match self.layout {
                    Layout::Grid => self.grid(now, thumbnails),
                    Layout::List => self.list(now, thumbnails),
                },
                Tab::Data => data_tab(&show.media, width),
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
                    // todo
                    let collections = ["Some Collection here: "; 7]
                        .into_iter()
                        .enumerate()
                        .map(|(i, collection)| Element::from(text(format!("{collection}{i}"))));

                    let collections =
                        scrollable(column(collections).spacing(4.0).width(Length::Fill))
                            .spacing(4.0);

                    column!(collections).spacing(8.0).width(width).into()
                }
            }
        };

        let content = column!(self.top(show), content).spacing(20.0).padding(10);

        content.into()
    }
}
