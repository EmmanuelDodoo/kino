use super::{HomeMessage, ViewMessage, shared::*};
use crate::models::{ItemId, Media, Movie, MovieId};
use crate::utils::icons::*;
use crate::utils::typo::*;
use iced::widget::Space;
use iced::{
    Color, Element, Length, Shadow,
    alignment::{Horizontal, Vertical},
    widget::{bottom_center, button, center_x, column, container, row, scrollable, stack, text},
};

#[derive(Debug, Clone, Copy)]
pub enum Message {
    Tab(Tab),
    Play,
    AddCollection,
}

#[derive(Debug, Clone, Copy)]
pub struct MoviePageMessage {
    pub id: MovieId,
    pub message: Message,
}

#[derive(Debug, Clone)]
pub struct MoviePage {
    pub id: MovieId,
    pub tab: Tab,
}

impl MoviePage {
    pub fn new(id: MovieId) -> Self {
        Self {
            id,
            tab: Tab::Items,
        }
    }

    pub fn update(&mut self, message: MoviePageMessage) -> Option<HomeMessage> {
        if message.id != self.id {
            return None;
        }

        match message.message {
            Message::Tab(tab) => {
                self.tab = tab;
                None
            }
            Message::Play => {
                let msg = HomeMessage::Play(ItemId::Movie(self.id));
                Some(msg)
            }
            Message::AddCollection => {
                let msg = HomeMessage::OpenView(ViewMessage::Add(ItemId::Movie(self.id)));
                Some(msg)
            }
        }
    }

    pub fn overlay<'a>(&self, movie: &'a Thumbnail<Movie>) -> Element<'a, MoviePageMessage> {
        let id = self.id;

        let img: Element<'_, MoviePageMessage> = {
            let img_height = 300.0;
            let ratio = 2.0 / 3.0;
            movie.poster(img_height * ratio, img_height)
        };

        let header = {
            let separator = || Element::from(text("•").size(H3));

            let title = text(movie.media.name()).size(H4);
            let duration = duration(&movie.media);
            let rating = ratings(&movie.media);
            let release = text(movie.media.release_year()).size(H7);

            let details = row!(release, separator(), duration)
                .spacing(6)
                .align_y(Vertical::Center);

            let mut tags = vec![];
            let tag_len = movie.media.tags.len();

            for (i, tag) in movie.media.tags.iter().enumerate() {
                tags.push(Element::from(text(tag).size(H7)));

                if i < tag_len - 1 {
                    tags.push(separator())
                }
            }

            let tags = row(tags).spacing(6).align_y(Vertical::Center);
            column!(title, tags, details, rating)
        };

        let item = "Overview";
        let tabs = Tab::ALL.into_iter().map(|tab| {
            let is_selected = self.tab == tab;

            Element::from(
                column!(
                    button(text(tab.to_str(item)).size(H7))
                        .on_press(MoviePageMessage {
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

        let view: Element<'_, MoviePageMessage> = {
            let width = 750.0;

            match self.tab {
                Tab::Items => {
                    let synapsis = text(movie.media.synapsis());

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
                Tab::Data => data_tab(&movie.media, width),
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
                .on_press(MoviePageMessage {
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
                .on_press(MoviePageMessage {
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

    pub fn view<'a>(&self, movie: &'a Thumbnail<Movie>) -> Element<'a, MoviePageMessage> {
        let overlay = bottom_center(self.overlay(movie));

        let img = movie.backdrop(Length::Fill, Length::FillPortion(3));

        let content = container(column!(img,)).style(container::dark);

        let content = stack![content, overlay];

        content.into()
    }

    pub fn show_tools(&self) -> bool {
        false
    }
}
