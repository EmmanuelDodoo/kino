use crate::utils::{self, load_fonts};
use iced::{
    Element, Length, Padding, Subscription, Task, Theme,
    alignment::Vertical,
    animation::{Animation, Easing},
    border::{Border, Radius},
    font, keyboard,
    time::Instant,
    widget::{
        button, center, column, container, horizontal_rule, horizontal_space, pick_list, row,
        scrollable, text, text_input, vertical_rule, vertical_space,
    },
    window,
};

mod movies;
mod pages;

use movies::{Movies, MoviesMessage};
use pages::{Page, PageKind, PageUpdate};
use utils::empty;
use utils::filter::*;
use utils::icons;
use utils::typo;
use utils::typo::*;
use utils::{Sort, SortKind, ViewType};

#[derive(Debug, Clone)]
pub enum FilterMessage {
    Mode,
    Clear,
    ProgressKind(ProgressKind),
    ProgressComp,
    RatingKind(RatingKind),
    RatingComp,
    CommentsNum(String),
    CommentsComp,
    ReleaseYear(String),
    ReleaseComp,
    DurationHours(String),
    DurationMinutes(String),
    DurationComp,
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    FontLoad(Result<(), font::Error>),
    Search(String),
    AddSort(SortKind),
    RemoveSort(SortKind),
    ToggleSort,
    ToggleFilter,
    Filter(FilterMessage),
    Movies(MoviesMessage),
    Settings,
    Randomize,
    Back,
    Forward,
    ToggleView,
    Home,
    Goto(PageKind),
    NewCollection,
    Animate,
    None,
}

pub struct Home {
    forward: Vec<Page>,
    backward: Vec<Page>,
    search: String,
    view: ViewType,
    sort: Sort,
    now: Instant,
    show_sorts: bool,
    show_filters: bool,
    filters: Filter,
}

impl Home {
    pub fn boot() -> (Self, Task<HomeMessage>) {
        let load_font = load_fonts().map(HomeMessage::FontLoad);

        (
            Self::new(ViewType::default(), FilterMode::default()),
            load_font,
        )
    }

    fn new(view: ViewType, filter_mode: FilterMode) -> Self {
        Self {
            forward: vec![],
            backward: vec![],
            search: String::default(),
            view,
            sort: Sort::default(),
            show_sorts: false,
            show_filters: false,
            now: Instant::now(),
            filters: Filter::new(filter_mode),
        }
    }

    pub fn update(&mut self, message: HomeMessage, now: Instant) -> Task<HomeMessage> {
        self.now = now;
        match message {
            HomeMessage::None => Task::none(),
            HomeMessage::Animate => Task::none(),
            HomeMessage::FontLoad(Err(error)) => {
                eprintln!("Font load error: \n{error:?}");
                Task::none()
            }
            HomeMessage::FontLoad(Ok(_)) => Task::none(),
            HomeMessage::Search(input) => {
                self.search = input;
                Task::none()
            }
            HomeMessage::Settings => Task::none(),
            HomeMessage::Home => {
                self.forward.clear();
                std::mem::swap(&mut self.forward, &mut self.backward);
                Task::none()
            }
            HomeMessage::Goto(kind) => {
                match kind {
                    PageKind::Movies => {
                        let (movies, task) = Movies::boot(
                            self.sort.clone(),
                            self.filters,
                            matches!(self.view, ViewType::Grid),
                        );
                        self.forward.clear();
                        self.backward.push(Page::Movies(movies));

                        task.map(HomeMessage::Movies)
                    }
                    _ => {
                        todo!()

                        // self.forward.clear();
                        // self.backward.push(kind);
                        // Task::none()
                    }
                }
            }
            HomeMessage::Movies(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.movies_update(message, now).map(HomeMessage::Movies)
            }
            HomeMessage::Back => {
                if self
                    .current_page_mut()
                    .map(|collection| collection.back())
                    .unwrap_or_default()
                {
                    return Task::none();
                }

                let Some(current) = self.backward.pop() else {
                    return Task::none();
                };

                self.forward.push(current);

                Task::none()
            }
            HomeMessage::Forward => {
                if self
                    .current_page_mut()
                    .map(|collection| collection.forward())
                    .unwrap_or_default()
                {
                    return Task::none();
                }

                let Some(forward) = self.forward.pop() else {
                    return Task::none();
                };

                self.backward.push(forward);
                Task::none()
            }
            HomeMessage::ToggleView => {
                if self.view == ViewType::Grid {
                    self.view = ViewType::List
                } else {
                    self.view = ViewType::Grid
                }

                let view = self.view;

                if let Some(page) = self.current_page_mut() {
                    page.page_update(PageUpdate::Layout(view), now);
                };

                Task::none()
            }
            HomeMessage::AddSort(sort) => {
                self.sort.kinds.push(sort);
                let sort = self.sort.clone();

                if let Some(page) = self.current_page_mut() {
                    page.page_update(PageUpdate::Sort(sort), now);
                };
                Task::none()
            }
            HomeMessage::RemoveSort(remove) => {
                self.sort.kinds.retain(|sort| *sort != remove);
                let sort = self.sort.clone();

                if let Some(page) = self.current_page_mut() {
                    page.page_update(PageUpdate::Sort(sort), now);
                };

                Task::none()
            }
            HomeMessage::ToggleSort => {
                self.show_sorts = !self.show_sorts;
                Task::none()
            }
            HomeMessage::ToggleFilter => {
                self.show_filters = !self.show_filters;
                Task::none()
            }
            HomeMessage::Filter(fsg) => {
                match fsg {
                    FilterMessage::Mode => self.filters.mode.toggle(),
                    FilterMessage::ProgressKind(kind) => {
                        self.filters.progress.kind = kind;
                    }
                    FilterMessage::ProgressComp => {
                        self.filters.progress.comp.toggle();
                    }
                    FilterMessage::RatingKind(kind) => {
                        self.filters.rating.kind = kind;
                    }
                    FilterMessage::RatingComp => {
                        self.filters.rating.comp.toggle();
                    }
                    FilterMessage::CommentsNum(number) => {
                        let number = number.trim();
                        if number.is_empty() {
                            self.filters.comments = None;
                            return Task::none();
                        }

                        let Ok(number) = number.parse::<u32>() else {
                            todo!("Error handling for Home");
                        };

                        match self.filters.comments.as_mut() {
                            Some(comments) => {
                                comments.number = number;
                            }
                            None => {
                                self.filters.comments = Some(Comments {
                                    number,
                                    comp: Comp::default(),
                                })
                            }
                        }
                    }
                    FilterMessage::CommentsComp => {
                        if let Some(comments) = self.filters.comments.as_mut() {
                            comments.comp.toggle();
                        }
                    }
                    FilterMessage::DurationMinutes(minutes) => {
                        let minutes = minutes.trim();

                        if minutes.is_empty() {
                            if let Some(duration) = self.filters.duration.as_mut() {
                                duration.secs = (duration.secs / 3600) * 3600;
                                if duration.secs == 0 {
                                    self.filters.duration = None;
                                }
                            }

                            return Task::none();
                        }

                        let Ok(minutes) = minutes.parse::<u64>() else {
                            todo!("Error handling for Home");
                        };

                        let secs = minutes * 60;

                        match self.filters.duration.as_mut() {
                            Some(duration) => {
                                let hours = (duration.secs / 3600) * 3600;

                                duration.secs = hours + secs;
                            }
                            None => {
                                self.filters.duration = Some(utils::Duration {
                                    secs,
                                    comp: Comp::default(),
                                });
                            }
                        }
                    }
                    FilterMessage::DurationHours(hours) => {
                        let hours = hours.trim();

                        if hours.is_empty() {
                            if let Some(duration) = self.filters.duration.as_mut() {
                                duration.secs %= 3600;
                                if duration.secs == 0 {
                                    self.filters.duration = None;
                                }
                            }

                            return Task::none();
                        }

                        let Ok(hours) = hours.parse::<u64>() else {
                            todo!("Error handling for Home")
                        };

                        let secs = hours * 3600;

                        match self.filters.duration.as_mut() {
                            Some(duration) => {
                                let minutes = duration.secs % 3600;
                                duration.secs = secs + minutes;
                            }
                            None => {
                                self.filters.duration = Some(utils::Duration {
                                    secs,
                                    comp: Comp::default(),
                                });
                            }
                        }
                    }
                    FilterMessage::DurationComp => {
                        if let Some(duration) = self.filters.duration.as_mut() {
                            duration.comp.toggle();
                        }
                    }
                    FilterMessage::ReleaseYear(year) => {
                        let year = year.trim();

                        if year.is_empty() {
                            self.filters.release = None;
                            return Task::none();
                        }

                        let Ok(year) = year.parse::<u16>() else {
                            todo!("Error handling for Home")
                        };

                        match self.filters.release.as_mut() {
                            Some(release) => release.year = year,
                            None => {
                                self.filters.release = Some(Release {
                                    year,
                                    comp: Comp::default(),
                                })
                            }
                        }
                    }
                    FilterMessage::ReleaseComp => {
                        if let Some(release) = self.filters.release.as_mut() {
                            release.comp.toggle();
                        }
                    }
                    FilterMessage::Clear => {
                        self.filters.clear();
                    }
                }
                let filters = self.filters;

                if let Some(page) = self.current_page_mut() {
                    page.page_update(PageUpdate::Filters(filters), now);
                };
                Task::none()
            }
            HomeMessage::NewCollection => Task::none(),
            HomeMessage::Randomize => Task::none(),
        }
    }

    fn current_page(&self) -> Option<&Page> {
        self.backward.last()
    }

    fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.backward.last_mut()
    }

    fn side(&self) -> Element<'_, HomeMessage> {
        let header = {
            let icon = icons::icon(icons::LOGO).size(H2);
            let text = text("Kino").size(H2);

            row!(icon, text)
                .padding([5, 10])
                .align_y(Vertical::Center)
                .spacing(12.0)
        };

        let collections = column!(
            icon_button(
                icons::HOME,
                "Home",
                HomeMessage::Home,
                self.current_page().is_none()
            ),
            icon_button(
                icons::SHOW,
                "Shows",
                HomeMessage::Goto(Page::goto_shows()),
                self.current_page().map(Page::is_shows).unwrap_or_default()
            ),
            icon_button(
                icons::MOVIE,
                "Movies",
                HomeMessage::Goto(Page::goto_movies()),
                self.current_page().map(Page::is_movies).unwrap_or_default(),
            ),
            icon_button(
                icons::NEW_COLLECTION,
                "New collection",
                HomeMessage::NewCollection,
                false
            ),
        )
        .spacing(16.0)
        .width(Length::Fill);
        let collections = scrollable(collections)
            .width(Length::Fill)
            .height(Length::Fill);

        let bottom = column!(
            icon_button(
                icons::COMMENT,
                "Comments",
                HomeMessage::Goto(Page::goto_comments()),
                self.current_page()
                    .map(Page::is_comments)
                    .unwrap_or_default()
            ),
            icon_button(icons::SETTINGS, "Settings", HomeMessage::Settings, false)
        )
        .spacing(16.0);

        let content = column!(collections, vertical_space(), bottom,)
            .padding([0, 5])
            .height(Length::Fill);

        let content = column!(header, vertical_space().height(24.0), content,)
            .width(240.0)
            .height(Length::Fill);

        content.into()
    }

    fn inner(&self) -> Element<'_, HomeMessage> {
        match self.current_page() {
            None => center(text("Home Page"))
                .width(Length::Fill)
                .height(Length::Fill)
                .into(),
            Some(collection) => collection.view(),
        }
    }

    fn navigation(&self) -> Element<'_, HomeMessage> {
        let current = self.current_page();

        let can_back = current
            .map(|collection| collection.can_back())
            .unwrap_or_default()
            || !self.backward.is_empty();

        let can_forward = current
            .map(|collection| collection.can_forward())
            .unwrap_or_default()
            || !self.forward.is_empty();

        let navigation = row!(
            icons::text_button(icons::BACK).on_press_maybe(can_back.then_some(HomeMessage::Back)),
            icons::text_button(icons::FORWARD)
                .on_press_maybe(can_forward.then_some(HomeMessage::Forward))
        )
        .spacing(5.0);

        navigation.into()
    }

    fn filters_view(&self) -> Element<'_, HomeMessage> {
        let size = typo::H7;
        let padding = Padding::new(2.0).left(5.0).right(5.0);

        let vertical_rule = || container(vertical_rule(2.0)).height(20.0);
        let comp = |icon: char, msg: FilterMessage| {
            icons::sized_button(icon, size)
                .padding([5, 5])
                .style(button::background)
                .on_press(HomeMessage::Filter(msg))
        };

        let up = pick_list::Icon {
            font: icons::FONT,
            code_point: icons::CHEV_UP,
            size: Some(size.into()),
            line_height: text::LineHeight::Relative(1.0),
            shaping: text::Shaping::Basic,
        };

        let down = pick_list::Icon {
            font: icons::FONT,
            code_point: icons::CHEV_DOWN,
            size: Some(size.into()),
            line_height: text::LineHeight::Relative(1.0),
            shaping: text::Shaping::Basic,
        };

        let handle = pick_list::Handle::Dynamic {
            closed: down,
            open: up,
        };

        let progress = {
            let text = text("Progress:").size(size);
            let progress = pick_list(
                ProgressKind::ALL,
                Some(self.filters.progress.kind),
                |selected| HomeMessage::Filter(FilterMessage::ProgressKind(selected)),
            )
            .padding(padding)
            .width(60.0)
            .handle(handle.clone())
            .text_size(size);

            let comp = comp(
                self.filters.progress.comp.icon(),
                FilterMessage::ProgressComp,
            );

            row!(text, comp, progress)
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let rating = {
            let text = text("Rating:").size(size);
            let rating = pick_list(
                RatingKind::ALL,
                Some(self.filters.rating.kind),
                |selected| HomeMessage::Filter(FilterMessage::RatingKind(selected)),
            )
            .padding(padding)
            .width(52.0)
            .handle(handle)
            .text_size(size);

            let comp = comp(self.filters.rating.comp.icon(), FilterMessage::RatingComp);

            row!(text, comp, rating)
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let comments = {
            let text = text("Comments:").size(size);
            let icon = self
                .filters
                .comments
                .map(|comments| comments.comp.icon())
                .unwrap_or(Comp::default().icon());
            let comp = comp(icon, FilterMessage::CommentsComp);

            let content = self
                .filters
                .comments
                .map(|comments| comments.number.to_string())
                .unwrap_or_default();
            let input = text_input("", &content)
                .width(32.0)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::CommentsNum(input)));

            row!(text, comp, input)
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let release = {
            let text = text("Release:").size(size);
            let icon = self
                .filters
                .release
                .map(|release| release.comp.icon())
                .unwrap_or(Comp::default().icon());
            let comp = comp(icon, FilterMessage::ReleaseComp);

            let content = self
                .filters
                .release
                .map(|release| release.year.to_string())
                .unwrap_or_default();
            let input = text_input("", &content)
                .width(48.0)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::ReleaseYear(input)));

            row!(text, comp, input)
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let duration = {
            let hr = text("hrs").size(size);
            let min = text("mins").size(size);
            let text = text("Duration:").size(size);
            let icon = self
                .filters
                .duration
                .map(|duration| duration.comp.icon())
                .unwrap_or(Comp::default().icon());
            let comp = comp(icon, FilterMessage::DurationComp);

            let hours = self
                .filters
                .duration
                .map(|duration| format!("{}", duration.secs / 3600))
                .unwrap_or_default();
            let hours = text_input("", &hours)
                .width(28.0)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::DurationHours(input)));

            let minutes = self
                .filters
                .duration
                .map(|duration| format!("{}", (duration.secs % 3600) / 60))
                .unwrap_or_default();
            let minutes = text_input("", &minutes)
                .width(28.0)
                .size(size)
                .padding(padding)
                .on_input(|input| HomeMessage::Filter(FilterMessage::DurationMinutes(input)));

            let duration = row!(hours, hr, minutes, min)
                .spacing(4.0)
                .align_y(Vertical::Center);

            row!(text, comp, duration)
                .spacing(5.0)
                .align_y(Vertical::Center)
        };

        let mode = {
            let mode = text(self.filters.mode.to_string()).size(size);
            let text = text("Combination mode:").size(size);

            let button = button(mode)
                .style(button::background)
                .padding(padding)
                .on_press(HomeMessage::Filter(FilterMessage::Mode));

            row!(text, button).spacing(5.0).align_y(Vertical::Center)
        };

        let clear = button(text("Clear filters").size(size))
            .padding(padding)
            .style(button::text)
            .on_press(HomeMessage::Filter(FilterMessage::Clear));

        let content = row!(
            progress,
            vertical_rule(),
            rating,
            vertical_rule(),
            comments,
            vertical_rule(),
            release,
            vertical_rule(),
            duration,
            vertical_rule(),
            mode,
            vertical_rule(),
            clear,
        )
        .spacing(10.0)
        .align_y(Vertical::Center)
        .wrap();

        let content = column!(text("Filters").size(size), content).spacing(5.0);

        content.into()
    }

    fn toolbar(&self) -> Element<'_, HomeMessage> {
        let size = typo::P;

        let filter = {
            let icon = if self.show_filters {
                icons::CHEV_UP
            } else {
                icons::CHEV_DOWN
            };
            let icon = icons::icon(icon).size(size).line_height(0.5);
            let text = icons::icon(icons::FILTER).size(size);

            let content = row!(text, icon).spacing(2.0).align_y(Vertical::Center);

            button(content)
                .style(if self.filters.is_any() {
                    button::subtle
                } else {
                    button::background
                })
                .on_press(HomeMessage::ToggleFilter)
                .padding([5, 5])
        };

        let sort = {
            let icon = if self.show_sorts {
                icons::CHEV_UP
            } else {
                icons::CHEV_DOWN
            };
            let icon = icons::icon(icon).size(size).line_height(0.5);
            let text = icons::icon(icons::SORT).size(size);

            let content = row!(text, icon).spacing(2.0).align_y(Vertical::Center);

            button(content)
                .style(if self.sort.kinds.is_empty() {
                    button::subtle
                } else {
                    button::background
                })
                .on_press(HomeMessage::ToggleSort)
                .padding([5, 5])
        };

        let curr_filters: Element<'_, HomeMessage> = if !self.show_filters {
            empty()
        } else {
            self.filters_view()
        };

        let curr_sorts: Element<'_, HomeMessage> = if !self.show_sorts {
            empty()
        } else {
            row!(
                text("Sort by: ").size(H7),
                row(SortKind::ALL.iter().map(|sort| {
                    let order = self.sort.kinds.iter().position(|selected| sort == selected);
                    sort.view(order)
                }))
                .spacing(5.0)
                .width(Length::Fill)
            )
            .align_y(Vertical::Center)
            .spacing(10.0)
            .into()
        };

        let left = row!(filter, sort).align_y(Vertical::Center).spacing(10.0);

        let right = row!(
            icons::sized_button(icons::RAND, size).on_press(HomeMessage::Randomize),
            icons::sized_button(self.view.icon(), size).on_press(HomeMessage::ToggleView),
        )
        .align_y(Vertical::Center)
        .spacing(5.0);

        let tools = row!(left, horizontal_space(), right).width(Length::Fill);

        let sorts_rule = if self.show_sorts {
            horizontal_rule(2.0).into()
        } else {
            empty()
        };
        let filters_rule = if self.show_filters {
            horizontal_rule(2.0).into()
        } else {
            empty()
        };

        let tools = column!(tools, sorts_rule, curr_sorts, filters_rule, curr_filters)
            .width(Length::Fill)
            .spacing(5.0)
            .padding(Padding::default().top(5.0).right(5.0).bottom(8.0).left(5.0));

        let content = container(tools).width(Length::Fill).style(container_style);

        content.into()
    }

    fn content_area(&self) -> Element<'_, HomeMessage> {
        let title = self.current_page().map(Page::name).unwrap_or("Home");
        let title = text(title).size(H5);

        let search = {
            let size = H7;
            let icon = text_input::Icon {
                font: icons::FONT,
                code_point: icons::SEARCH,
                side: text_input::Side::Right,
                size: Some(size.into()),
                spacing: 5.0,
            };

            text_input("Search", &self.search)
                .icon(icon)
                .size(size)
                .on_input(HomeMessage::Search)
        };

        let top = container(
            row!(
                self.navigation(),
                horizontal_space(),
                title,
                horizontal_space(),
                search,
            )
            .padding(Padding::ZERO.right(5))
            .align_y(Vertical::Center)
            .height(H2 * 1.50)
            .width(Length::Fill),
        )
        .style(container_style);

        let content_area = container(self.inner()).style(container_style);

        let content = column!(top, self.toolbar(), content_area)
            .height(Length::Fill)
            .width(Length::Fill);

        content.into()
    }

    pub fn view(&self) -> Element<'_, HomeMessage> {
        let content = row!(self.side(), self.content_area())
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([6, 5]);

        content.into()
    }

    pub fn subscription(&self) -> Subscription<HomeMessage> {
        let page = self
            .current_page()
            .map(|page| page.subscription())
            .unwrap_or(Subscription::none());

        let keys = keyboard::on_key_press(|key, modifiers| match key {
            keyboard::Key::Named(keyboard::key::Named::ArrowLeft) if modifiers.alt() => {
                Some(HomeMessage::Back)
            }
            keyboard::Key::Named(keyboard::key::Named::ArrowRight) if modifiers.alt() => {
                Some(HomeMessage::Forward)
            }

            _ => None,
        });

        Subscription::batch([page, keys])
    }
}

fn icon_button<'a>(
    unicode: char,
    value: &'a str,
    message: HomeMessage,
    current: bool,
) -> Element<'a, HomeMessage> {
    let size = H6;
    let icon = icons::icon(unicode).size(size);
    let text = text(value).size(size);

    button(
        row!(icon, text)
            .align_y(Vertical::Center)
            .width(Length::Fill)
            .spacing(16.0),
    )
    .style(move |theme, status| {
        use button::{Status, Style, background};
        let default = background(theme, status);

        match status {
            Status::Active if current => {
                let background = theme.extended_palette().background.weakest;
                Style {
                    background: Some(background.color.into()),
                    ..default
                }
            }
            _ => default,
        }
    })
    .on_press(message)
    .into()
}

fn container_style(theme: &Theme) -> container::Style {
    let style = container::bordered_box(theme);
    let border = Border {
        radius: Radius::default(),
        ..style.border
    };

    container::Style { border, ..style }
}
