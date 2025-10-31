use super::{HomeMessage, PageUpdate, ViewMessage, shared::*};
use crate::models::{ItemId, Media, shows::*};
use crate::utils::filter::*;
use crate::utils::icons::*;
use crate::utils::typo::*;
use crate::utils::{Layout, Sort, empty};
use iced::widget::Space;
use iced::{
    Color, ContentFit, Element, Length, Shadow, Subscription, Task,
    alignment::{Horizontal, Vertical},
    time::Instant,
    widget::{
        self, bottom_center, button, center_x, column, container, grid, image, operation, row,
        rule, scrollable, space, stack, text,
    },
    window,
};
use std::collections::HashMap;

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Tab(Tab),
    AddCollection,
    Play,
}

#[derive(Debug, Clone, Copy)]
pub struct EpisodePageMessage {
    pub id: EpisodeId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct EpisodePage {
    pub tab: Tab,
    pub id: EpisodeId,
    pub name: String,
}

impl EpisodePage {
    pub fn new(episode: &Episode) -> Self {
        Self {
            id: episode.id,
            tab: Tab::Items,
            name: episode.name().to_owned(),
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
        }
    }

    fn overlay<'a>(&self, episode: &'a Thumbnail<Episode>) -> Element<'a, EpisodePageMessage> {
        let id = self.id;

        let img: Element<'_, EpisodePageMessage> = {
            let img_height = 300.0;
            let ratio = 2.0 / 3.0;
            episode.poster(img_height * ratio, img_height)
        };

        let header = {
            let separator = || Element::from(text("•").size(H3));

            let title = text(episode.media.name()).size(H4);
            let duration = duration(&episode.media);
            let rating = ratings(&episode.media);
            let release = text(episode.media.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            column!(title, details, rating)
        };

        let item = "Overview";
        let tabs = Tab::ALL.into_iter().map(|tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .on_press(EpisodePageMessage {
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

        let tabs = row(tabs).spacing(8.0);

        let view: Element<'_, EpisodePageMessage> = {
            let width = 750.0;

            match self.tab {
                Tab::Items => {
                    let synapsis = text(episode.media.synapsis());

                    scrollable(column!(synapsis).spacing(4.0).width(width))
                        .spacing(4.0)
                        .into()
                }
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
                Tab::Data => data_tab(&episode.media, width),
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

        let actions = center_x(
            row!(
                button(
                    row!(icon(PLAY).size(H5), text("Play").size(H5))
                        .spacing(16.0)
                        .align_y(Vertical::Center),
                )
                .padding([6, 12])
                .on_press(EpisodePageMessage {
                    id,
                    message: Message::Play
                })
                .style(|theme, status| {
                    let default = button::subtle(theme, status);
                    let border = default.border.rounded(5);

                    button::Style { border, ..default }
                }),
                button(
                    row!(
                        icon(ADD_COLLECTION).size(H5),
                        text("Add to Collection").size(H5)
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
                    let default = button::subtle(theme, status);
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
            .max_height(465.0)
            .align_x(Horizontal::Center)
            .width(Length::Fill)
            .style(|theme| {
                let default = container::dark(theme);
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

    pub fn view<'a>(&self, episode: &'a Thumbnail<Episode>) -> Element<'a, EpisodePageMessage> {
        let overlay = bottom_center(self.overlay(episode));

        let img = episode.backdrop(Length::Fill, Length::FillPortion(3));

        let content = container(column!(img,)).style(container::dark);

        let content = stack![content, overlay];

        content.into()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn show_tools(&self) -> bool {
        false
    }
}
