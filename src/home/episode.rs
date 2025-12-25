use super::{HomeMessage, PageKind, ViewMessage, shared::*};
use crate::models::{CollectionId, Episode, EpisodeId, ItemId, Media, SimpleCollection};
use crate::utils::icons::*;
use crate::utils::styles;
use crate::utils::tooltip;
use crate::utils::typo::*;
use iced::widget::Space;
use iced::{
    Color, Element, Length, Shadow,
    alignment::{Horizontal, Vertical},
    widget::{
        bottom_center, button, center_x, column, container, row, scrollable, stack, text,
        tooltip as tp,
    },
};

#[derive(Debug, Clone)]
pub enum Message {
    Tab(Tab),
    AddCollection,
    Rate(Option<f32>),
    Play,
    Goto(CollectionId),
    Rename(String),
    Synopsis(String),
    Refetch,
    Remove(String),
}

#[derive(Debug, Clone)]
pub struct EpisodePageMessage {
    pub id: EpisodeId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct EpisodePage {
    pub tab: Tab,
    pub id: EpisodeId,
}

impl EpisodePage {
    pub fn new(id: EpisodeId) -> Self {
        Self {
            id,
            tab: Tab::Items,
        }
    }

    pub fn update(&mut self, message: EpisodePageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Tab(tab) => {
                self.tab = tab;
                None
            }
            Message::Play => {
                let msg = HomeMessage::Play(ItemId::Episode(self.id));
                Some(msg)
            }
            Message::AddCollection => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Episode(self.id)));
                Some(msg)
            }
            Message::Rate(rating) => {
                let msg =
                    HomeMessage::OpenView(ViewMessage::Rating(ItemId::Episode(self.id), rating));
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
        }
    }

    fn overlay<'a>(
        &self,
        episode: &'a Thumbnail<Episode>,
        memberships: impl Iterator<Item = &'a SimpleCollection>,
    ) -> Element<'a, EpisodePageMessage> {
        let id = self.id;

        let img: Element<'_, EpisodePageMessage> = {
            let img_height = 300.0;
            let ratio = 2.0 / 3.0;
            episode.poster(img_height * ratio, img_height)
        };

        let header = {
            let separator = || Element::from(text("•").size(H3));

            let title = title(episode.media.name());

            let duration = duration(&episode.media);
            let rating = button(ratings(&episode.media, true))
                .on_press(EpisodePageMessage {
                    id,
                    message: Message::Rate(episode.media.rating()),
                })
                .style(styles::button::text)
                .padding(0);
            let release = sized_medium(episode.media.release_year(), H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            column!(title, details, rating)
        };

        let item = "Overview";
        let tabs = Tab::VARIANTS.into_iter().map(|tab| {
            let is_selected = self.tab == *tab;
            let text = if is_selected {
                bold(tab.to_str(item))
            } else {
                regular(tab.to_str(item))
            };

            Element::from(
                column!(
                    button(text)
                        .on_press(EpisodePageMessage {
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

        let tabs = row(tabs).spacing(8.0);

        let view: Element<'_, EpisodePageMessage> = {
            let width = 750.0;

            match self.tab {
                Tab::Items => tab_synopsis(episode.media.synopsis()),
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
                Tab::Data => data_tab(
                    &episode.media,
                    width,
                    Message::Rename(episode.media.name().to_owned()),
                    Message::Refetch,
                    Message::Remove(episode.media.name().to_owned()),
                    Message::Synopsis(episode.media.synopsis().to_owned()),
                    None,
                )
                .map(move |message| EpisodePageMessage { id, message }),
                Tab::Collections => {
                    let collections = memberships.map(|collection| {
                        draw_collection_tab(collection, move |collection| EpisodePageMessage {
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

        let actions = center_x(
            row!(
                button(
                    row!(icon(PLAY).size(H5), sized_medium("Play", P))
                        .spacing(16.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(EpisodePageMessage {
                    id,
                    message: Message::Play
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
                    .spacing(16.0)
                    .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(EpisodePageMessage {
                    id,
                    message: Message::AddCollection
                })
                .style(|theme, status| {
                    let default = styles::button::primary(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
            )
            .align_y(Vertical::Center)
            .spacing(16.0),
        );

        let tabs = column!(tabs, view).height(Length::Fill).spacing(16.0);

        let content = column!(header, tabs).spacing(24.0).width(675.0);

        let content = center_x(row!(img, content).spacing(20.0));

        container(column!(content, actions))
            .padding([20, 28])
            .max_height(500.0)
            .align_x(Horizontal::Center)
            .width(Length::Fill)
            .style(|theme| {
                let default = styles::container::dark(theme);
                let background = default
                    .background
                    .map(|background| background.scale_alpha(0.75));

                let shadow = default.shadow;
                let shadow = Shadow {
                    color: Color::BLACK.scale_alpha(0.85),
                    blur_radius: 10.0,
                    ..shadow
                };

                container::Style {
                    background,
                    shadow,
                    ..default
                }
            })
            .into()
    }

    pub fn view<'a>(
        &self,
        episode: &'a Thumbnail<Episode>,
        memberships: impl Iterator<Item = &'a SimpleCollection>,
    ) -> Element<'a, EpisodePageMessage> {
        let overlay = bottom_center(self.overlay(episode, memberships));

        let img = episode.backdrop(Length::Fill, Length::FillPortion(3));

        let content = container(column!(img,)).style(styles::container::dark);

        let content = stack![content, overlay];

        content.into()
    }

    pub fn show_tools(&self) -> bool {
        false
    }
}
