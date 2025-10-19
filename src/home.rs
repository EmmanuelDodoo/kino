use crate::utils::{self, icons::*, load_fonts};
use crate::widgets::menu::{Position, menu};
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
use rand::seq::SliceRandom;
use std::collections::HashMap;

mod movies;
mod pages;
mod shared;
mod shows;

use crate::models::{Movie, MovieId, Show, ShowId};
use movies::{Movies, MoviesMessage};
use pages::{Page, PageKind, PageUpdate};
use shared::{Scroll, Thumbnail, filter_sort};
use shows::{TvShows, TvShowsMessage};
use utils::empty;
use utils::filter::*;
use utils::icons;
use utils::typo;
use utils::typo::*;
use utils::{Layout, Sort, SortKind};

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

#[derive(Debug, Clone, Copy)]
pub enum SortMessage {
    AddSort(SortKind),
    RemoveSort(SortKind),
    Clear,
    ToggleReverse,
}

#[derive(Debug, Clone, Copy)]
enum Focused {
    Movie(MovieId),
    Show(ShowId),
}

#[derive(Debug, Clone)]
struct HomeScroll {
    home: Scroll,
    movies: Scroll,
    shows: Scroll,
}

impl HomeScroll {
    fn new() -> Self {
        Self {
            home: Scroll::new(),
            movies: Scroll::new(),
            shows: Scroll::new(),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HomeScrollMessage {
    Home(scrollable::Viewport),
    Movies(scrollable::Viewport),
    Shows(scrollable::Viewport),
}

#[derive(Debug, Clone)]
pub enum RecentMessage {
    Movies(Vec<Thumbnail<Movie>>),
    Shows(Vec<Thumbnail<Show>>),
    AddCollectionMovie(MovieId),
    AddCollectionShow(ShowId),
    HoveredMovie(MovieId, bool),
    HoveredShow(ShowId, bool),
    PlayMovie(MovieId),
    PlayShow(ShowId),
    DetailsMovie(MovieId),
    DetailsShow(ShowId),
}

#[derive(Debug, Clone)]
pub enum HomeMessage {
    FontLoad(Result<(), font::Error>),
    Search(String),
    ToggleSort,
    ToggleFilter,
    Filter(FilterMessage),
    Sort(SortMessage),
    Movies(MoviesMessage),
    Shows(TvShowsMessage),
    Settings,
    Random,
    Back,
    Forward,
    ToggleLayout,
    Home,
    Goto(PageKind),
    NewCollection,
    Animate,
    None,
    Recent(RecentMessage),
    HomeScroll(HomeScrollMessage),
    Refresh,
}

pub struct Home {
    forward: Vec<PageKind>,
    backward: Vec<PageKind>,
    search: String,
    layout: Layout,
    sort: Sort,
    now: Instant,
    show_sorts: bool,
    show_filters: bool,
    filters: Filter,
    recent_movies: HashMap<MovieId, Thumbnail<Movie>>,
    recent_shows: HashMap<ShowId, Thumbnail<Show>>,
    focused: Option<Focused>,
    home_scroll: HomeScroll,
    pages: HashMap<PageKind, Page>,
    current_page: Option<PageKind>,
}

impl Home {
    pub fn boot() -> (Self, Task<HomeMessage>) {
        let load_font = load_fonts().map(HomeMessage::FontLoad);

        let recent_movies = Task::perform(
            async { (0..6).map(|_| Movie::testing()).collect::<Vec<_>>() },
            |videos| {
                HomeMessage::Recent(RecentMessage::Movies(
                    videos.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        );

        let recent_shows = Task::perform(
            async { (0..6).map(|_| Show::testing()).collect::<Vec<_>>() },
            |shows| {
                HomeMessage::Recent(RecentMessage::Shows(
                    shows.into_iter().map(Thumbnail::new).collect(),
                ))
            },
        );

        let tasks = Task::batch([load_font, recent_movies, recent_shows]);

        (Self::new(Layout::default(), FilterMode::default()), tasks)
    }

    fn new(view: Layout, filter_mode: FilterMode) -> Self {
        Self {
            forward: vec![],
            backward: vec![],
            search: String::default(),
            layout: view,
            sort: Sort::default(),
            show_sorts: false,
            show_filters: false,
            now: Instant::now(),
            filters: Filter::new(filter_mode),
            recent_shows: HashMap::default(),
            recent_movies: HashMap::default(),
            focused: None,
            home_scroll: HomeScroll::new(),
            pages: HashMap::default(),
            current_page: None,
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
                if let Some(old) = self.current_page.take() {
                    self.backward.push(old);
                };
                self.forward.clear();
                self.focused = None;
                self.update_scroll()
            }
            HomeMessage::Goto(kind) => {
                if let Some(old) = self.current_page.replace(kind) {
                    self.backward.push(old)
                };
                self.forward.clear();
                self.focused = None;

                if let Some(page) = self.pages.get_mut(&kind) {
                    let update = PageUpdate {
                        filters: self.filters,
                        sort: self.sort,
                        layout: self.layout,
                    };
                    page.page_update(update, now);
                    return Task::none();
                }

                match kind {
                    PageKind::Movies => {
                        let (movies, id, task) = Movies::boot(
                            self.sort,
                            self.filters,
                            matches!(self.layout, Layout::Grid),
                        );

                        self.pages.insert(kind, Page::Movies(Box::new(movies)));

                        let scroll =
                            scrollable::scroll_to(id, scrollable::AbsoluteOffset::default());

                        Task::batch([task.map(HomeMessage::Movies), scroll])
                    }
                    PageKind::Shows => {
                        let (shows, id, tasks) = TvShows::boot(
                            self.sort,
                            self.filters,
                            matches!(self.layout, Layout::Grid),
                        );

                        self.pages.insert(kind, Page::Shows(Box::new(shows)));

                        let scroll =
                            scrollable::scroll_to(id, scrollable::AbsoluteOffset::default());

                        Task::batch([tasks.map(HomeMessage::Shows), scroll])
                    }
                    _ => {
                        todo!()
                    }
                }
            }
            HomeMessage::Movies(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.movies_update(message, now).map(HomeMessage::Movies)
            }
            HomeMessage::Shows(message) => {
                let Some(page) = self.current_page_mut() else {
                    return Task::none();
                };

                page.shows_update(message, now).map(HomeMessage::Shows)
            }
            HomeMessage::Back => {
                self.focused = None;
                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                match self.current_page.take() {
                    Some(current) => {
                        if let Some(task) = self
                            .pages
                            .get_mut(&current)
                            .and_then(|page| page.back(update.clone(), now))
                        {
                            self.current_page = Some(current);
                            return task.map(|_| HomeMessage::None);
                        }

                        self.forward.push(current);

                        match self.backward.pop() {
                            Some(new) => {
                                let page = self
                                    .pages
                                    .get_mut(&new)
                                    .expect("Page cannot be in back without being recorded first");
                                self.current_page = Some(new);
                                page.page_update(update, now);
                                page.update_scroll().map(|_| HomeMessage::None)
                            }
                            None => self.update_scroll(),
                        }
                    }
                    None => {
                        let Some(new) = self.backward.pop() else {
                            return Task::none();
                        };
                        let page = self
                            .pages
                            .get_mut(&new)
                            .expect("Page cannot be in back without being recorded first");
                        self.current_page = Some(new);
                        page.page_update(update, now);
                        page.update_scroll().map(|_| HomeMessage::None)
                    }
                }
            }
            HomeMessage::Forward => {
                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                match self.current_page.take() {
                    Some(current) => {
                        if let Some(task) = self
                            .pages
                            .get_mut(&current)
                            .and_then(|page| page.forward(update.clone(), now))
                            .map(|task| task.map(|_| HomeMessage::None))
                        {
                            self.current_page = Some(current);
                            return task;
                        }

                        self.backward.push(current);
                        let Some(new) = self.forward.pop() else {
                            return Task::none();
                        };

                        let page = self
                            .pages
                            .get_mut(&new)
                            .expect("Page cannot be in forward without being recorded");

                        self.current_page = Some(new);
                        page.page_update(update, now);
                        page.update_scroll().map(|_| HomeMessage::None)
                    }
                    None => {
                        let Some(new) = self.forward.pop() else {
                            return Task::none();
                        };

                        let page = self
                            .pages
                            .get_mut(&new)
                            .expect("Page cannot be in forward without being recorded");
                        self.current_page = Some(new);
                        page.page_update(update, now);
                        page.update_scroll().map(|_| HomeMessage::None)
                    }
                }
            }
            HomeMessage::ToggleLayout => {
                if self.layout == Layout::Grid {
                    self.layout = Layout::List
                } else {
                    self.layout = Layout::Grid
                }

                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                if let Some(page) = self.current_page_mut() {
                    page.page_update(update, now);
                };

                Task::none()
            }
            HomeMessage::Sort(ssg) => {
                match ssg {
                    SortMessage::AddSort(kind) => self.sort.push(kind),
                    SortMessage::RemoveSort(kind) => self.sort.remove(kind),
                    SortMessage::Clear => self.sort.clear(),
                    SortMessage::ToggleReverse => self.sort.reverse(),
                }

                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                if let Some(page) = self.current_page_mut() {
                    page.page_update(update, now);
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

                        let Ok(year) = year.parse::<i32>() else {
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

                let update = PageUpdate {
                    layout: self.layout,
                    sort: self.sort,
                    filters: self.filters,
                };

                if let Some(page) = self.current_page_mut() {
                    page.page_update(update, now);
                };
                Task::none()
            }
            HomeMessage::NewCollection => Task::none(),
            HomeMessage::Random => match self.current_page_mut().map(|page| page.rand()) {
                Some(task) => task,
                None => {
                    let choices = [0, 1];
                    let mut rng = rand::thread_rng();
                    let choice = choices
                        .choose(&mut rng)
                        .expect("choices as defined above is not empty");

                    let msg = if *choice == 0 {
                        let recents = self.recent_movies.keys().collect::<Vec<_>>();
                        let Some(choice) = recents.choose(&mut rng).copied() else {
                            return Task::none();
                        };
                        HomeMessage::Recent(RecentMessage::DetailsMovie(*choice))
                    } else {
                        let recents = self.recent_shows.keys().collect::<Vec<_>>();
                        let Some(choice) = recents.choose(&mut rng).copied() else {
                            return Task::none();
                        };
                        HomeMessage::Recent(RecentMessage::DetailsShow(*choice))
                    };

                    Task::done(msg)
                }
            },
            HomeMessage::Refresh => match self.current_page_mut() {
                Some(page) => page.refresh(),
                None => todo!("Refresh recents"),
            },
            HomeMessage::Recent(rsg) => match rsg {
                RecentMessage::Shows(shows) => {
                    for show in shows {
                        self.recent_shows.insert(show.id(), show);
                    }
                    Task::none()
                }
                RecentMessage::Movies(movies) => {
                    for movie in movies {
                        self.recent_movies.insert(movie.id(), movie);
                    }
                    Task::none()
                }
                RecentMessage::PlayShow(id) => {
                    println!("Play show {id:?}");
                    Task::none()
                }
                RecentMessage::PlayMovie(id) => {
                    println!("Play movie {id:?}");
                    Task::none()
                }
                RecentMessage::HoveredShow(id, is_hovered) => {
                    let Some(media) = self.recent_shows.get_mut(&id) else {
                        return Task::none();
                    };

                    media.zoom.go_mut(is_hovered, now);
                    self.focused = Some(Focused::Show(id));
                    Task::none()
                }
                RecentMessage::HoveredMovie(id, is_hovered) => {
                    let Some(media) = self.recent_movies.get_mut(&id) else {
                        return Task::none();
                    };

                    media.zoom.go_mut(is_hovered, now);
                    self.focused = Some(Focused::Movie(id));
                    Task::none()
                }
                RecentMessage::DetailsMovie(id) => {
                    let Some(movie) = self.pages.get_mut(&PageKind::Movies) else {
                        return Task::done(HomeMessage::Goto(PageKind::Movies)).chain(Task::done(
                            HomeMessage::Recent(RecentMessage::DetailsMovie(id)),
                        ));
                    };

                    let Page::Movies(movie) = movie else {
                        return Task::none();
                    };

                    if !movie.contains(&id) {
                        return Task::perform(async {}, move |_| {
                            HomeMessage::Recent(RecentMessage::DetailsMovie(id))
                        });
                    }

                    movie.preview(id);

                    match self.current_page.take() {
                        Some(old) => {
                            if !matches!(old, PageKind::Movies) {
                                self.backward.push(old);
                                self.current_page = Some(PageKind::Movies);
                            } else {
                                self.current_page = Some(old);
                            }
                        }
                        None => self.current_page = Some(PageKind::Movies),
                    }

                    self.forward.clear();
                    self.focused = None;
                    Task::none()
                }
                RecentMessage::DetailsShow(id) => {
                    let Some(show) = self.pages.get_mut(&PageKind::Shows) else {
                        return Task::done(HomeMessage::Goto(PageKind::Shows)).chain(Task::done(
                            HomeMessage::Recent(RecentMessage::DetailsShow(id)),
                        ));
                    };

                    let Page::Shows(show) = show else {
                        return Task::none();
                    };

                    if !show.contains(&id) {
                        return Task::perform(async {}, move |_| {
                            HomeMessage::Recent(RecentMessage::DetailsShow(id))
                        });
                    }

                    match self.current_page.take() {
                        Some(old) => {
                            if !matches!(old, PageKind::Shows) {
                                self.backward.push(old);
                                self.current_page = Some(PageKind::Shows);
                            } else {
                                self.current_page = Some(old);
                            }
                        }
                        None => self.current_page = Some(PageKind::Shows),
                    }

                    self.forward.clear();
                    self.focused = None;
                    show.preview(id).map(HomeMessage::Shows)
                }
                RecentMessage::AddCollectionShow(id) => {
                    println!("Add {id:?} to collection pressed");
                    Task::none()
                }
                RecentMessage::AddCollectionMovie(id) => {
                    println!("Add {id:?} to collection pressed");
                    Task::none()
                }
            },
            HomeMessage::HomeScroll(hsg) => match hsg {
                HomeScrollMessage::Home(viewport) => {
                    self.home_scroll.home.offset = viewport.absolute_offset();
                    Task::none()
                }
                HomeScrollMessage::Movies(viewport) => {
                    self.home_scroll.movies.offset = viewport.absolute_offset();
                    Task::none()
                }
                HomeScrollMessage::Shows(viewport) => {
                    self.home_scroll.shows.offset = viewport.absolute_offset();
                    Task::none()
                }
            },
        }
    }

    fn update_scroll(&mut self) -> Task<HomeMessage> {
        use scrollable::scroll_to;
        let HomeScroll {
            home,
            shows,
            movies,
        } = self.home_scroll.clone();

        let home: Task<()> = scroll_to(home.id, home.offset);
        let movies = scroll_to(movies.id, movies.offset);
        let shows = scroll_to(shows.id, shows.offset);

        Task::batch([home, movies, shows]).map(|_| HomeMessage::None)
    }

    fn current_page(&self) -> Option<&Page> {
        self.current_page
            .as_ref()
            .and_then(|kind| self.pages.get(kind))
    }

    fn current_page_mut(&mut self) -> Option<&mut Page> {
        self.current_page
            .as_ref()
            .and_then(|kind| self.pages.get_mut(kind))
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

    fn recents(&self) -> Element<'_, HomeMessage> {
        let movies = {
            let label = text("Recent Movies").size(H4);
            let label = column!(label, horizontal_rule(2.0)).spacing(4.0);
            let movies = filter_sort(self.recent_movies.values(), &self.filters, &self.sort);

            let movies: Element<'_, HomeMessage> = match self.layout {
                Layout::Grid => {
                    let content = movies.map(|thumbnail| {
                        thumbnail.card(
                            self.now,
                            |id| HomeMessage::Recent(RecentMessage::AddCollectionMovie(id)),
                            |id| HomeMessage::Recent(RecentMessage::DetailsMovie(id)),
                            |id, hovered| {
                                HomeMessage::Recent(RecentMessage::HoveredMovie(id, hovered))
                            },
                            |id| HomeMessage::Recent(RecentMessage::PlayMovie(id)),
                        )
                    });

                    scrollable(row(content).spacing(16.0).align_y(Vertical::Center))
                        .id(self.home_scroll.movies.id.clone())
                        .on_scroll(|view| HomeMessage::HomeScroll(HomeScrollMessage::Movies(view)))
                        .direction(scrollable::Direction::Horizontal(
                            scrollable::Scrollbar::default().spacing(16.0),
                        ))
                        .into()
                }
                Layout::List => {
                    let content = movies.map(|thumbnail| {
                        thumbnail.list(
                            self.now,
                            |id| HomeMessage::Recent(RecentMessage::AddCollectionMovie(id)),
                            |id| HomeMessage::Recent(RecentMessage::DetailsMovie(id)),
                            |id, hovered| {
                                HomeMessage::Recent(RecentMessage::HoveredMovie(id, hovered))
                            },
                            |id| HomeMessage::Recent(RecentMessage::PlayMovie(id)),
                            movies::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                }
            };

            column!(label, movies).spacing(10.0)
        };

        let shows = {
            let label = text("Recent Shows").size(H4);
            let label = column!(label, horizontal_rule(2.0)).spacing(4.0);
            let shows = filter_sort(self.recent_shows.values(), &self.filters, &self.sort);

            let shows: Element<'_, HomeMessage> = match self.layout {
                Layout::Grid => {
                    let shows = shows.map(|show| {
                        show.card(
                            self.now,
                            |id| HomeMessage::Recent(RecentMessage::AddCollectionShow(id)),
                            |id| HomeMessage::Recent(RecentMessage::DetailsShow(id)),
                            |id, hovered| {
                                HomeMessage::Recent(RecentMessage::HoveredShow(id, hovered))
                            },
                            |id| HomeMessage::Recent(RecentMessage::PlayShow(id)),
                        )
                    });

                    scrollable(row(shows).spacing(16.0).align_y(Vertical::Center))
                        .id(self.home_scroll.shows.id.clone())
                        .on_scroll(|view| HomeMessage::HomeScroll(HomeScrollMessage::Shows(view)))
                        .direction(scrollable::Direction::Horizontal(
                            scrollable::Scrollbar::default().spacing(16.0),
                        ))
                        .into()
                }
                Layout::List => {
                    let content = shows.map(|thumbnail| {
                        thumbnail.list(
                            self.now,
                            |id| HomeMessage::Recent(RecentMessage::AddCollectionShow(id)),
                            |id| HomeMessage::Recent(RecentMessage::DetailsShow(id)),
                            |id, hovered| {
                                HomeMessage::Recent(RecentMessage::HoveredShow(id, hovered))
                            },
                            |id| HomeMessage::Recent(RecentMessage::PlayShow(id)),
                            shows::unique,
                        )
                    });

                    column(content).spacing(16.0).into()
                }
            };

            column!(label, shows).spacing(10.0)
        };

        let content = scrollable(column!(movies, shows).spacing(40.0).padding(10))
            .spacing(16.0)
            .id(self.home_scroll.home.id.clone())
            .on_scroll(|view| HomeMessage::HomeScroll(HomeScrollMessage::Home(view)));

        content.into()
    }

    fn inner(&self) -> Element<'_, HomeMessage> {
        match self.current_page() {
            None => self.recents(),
            Some(collection) => collection.view(),
        }
    }

    fn navigation(&self) -> Element<'_, HomeMessage> {
        let current = self.current_page();

        let can_back = current
            .map(|collection| collection.can_back())
            .unwrap_or_default()
            || !self.backward.is_empty()
            || (self.backward.is_empty() && self.current_page.is_some());

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

        let clear = button(text("Clear").size(size))
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

    fn sort_view(&self) -> Element<'_, HomeMessage> {
        let size = H7;
        let vertical_rule = || container(vertical_rule(2.0)).height(20.0);

        let clear = button(text("Clear").size(size))
            .padding([2, 5])
            .style(button::text)
            .on_press(HomeMessage::Sort(SortMessage::Clear));

        let reverse = button(text("Reverse").size(size))
            .padding([2, 5])
            .style(button::text)
            .on_press(HomeMessage::Sort(SortMessage::ToggleReverse));

        let base = row!(text("More").size(size), icon(ELLIPSIS_VER).size(size))
            .spacing(2.0)
            .align_y(Vertical::Center);

        let hidden = {
            container(
                column(SortKind::HIDDEN.iter().map(|kind| {
                    let order = self.sort.position(*kind);

                    kind.view(
                        |kind| HomeMessage::Sort(SortMessage::AddSort(kind)),
                        |kind| HomeMessage::Sort(SortMessage::RemoveSort(kind)),
                        order,
                    )
                }))
                .spacing(8),
            )
            .style(container::bordered_box)
            .padding([3, 6])
        };

        let more = menu(base, hidden)
            .auto_close(false)
            .position(Position::Right)
            .on_toggle(|_| HomeMessage::None);

        row!(
            text("Sort by: ").size(size),
            row(SortKind::VISIBLE.iter().map(|sort| {
                let order = self.sort.position(*sort);

                sort.view(
                    |kind| HomeMessage::Sort(SortMessage::AddSort(kind)),
                    |kind| HomeMessage::Sort(SortMessage::RemoveSort(kind)),
                    order,
                )
            }))
            .spacing(5.0),
            vertical_rule(),
            more,
            vertical_rule(),
            reverse,
            clear,
        )
        .align_y(Vertical::Center)
        .spacing(10.0)
        .into()
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
                .style(if self.sort.is_empty() {
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
            self.sort_view()
        };

        let left = row!(filter, sort).align_y(Vertical::Center).spacing(10.0);

        let right = row!(
            icons::sized_button(icons::REFRESH, size).on_press(HomeMessage::Refresh),
            icons::sized_button(icons::RAND, size).on_press(HomeMessage::Random),
            icons::sized_button(self.layout.icon(), size).on_press(HomeMessage::ToggleLayout),
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
        let title = self
            .current_page()
            .map(Page::name)
            .unwrap_or("Home".to_owned());
        let title = container(text(title).size(H6)).max_width(400.0);

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
                .width(175.0)
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

        let content_area = container(self.inner())
            .style(container_style)
            .height(Length::Fill)
            .width(Length::Fill);

        let show_tools = self
            .current_page()
            .map(|page| page.show_tools())
            .unwrap_or(true);

        let content = column!(
            top,
            if show_tools { self.toolbar() } else { empty() },
            content_area
        )
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

    pub fn is_animating(&self) -> bool {
        match &self.focused {
            Some(Focused::Show(id)) => self
                .recent_shows
                .get(id)
                .map(|media| media.is_animating(self.now))
                .unwrap_or_default(),
            Some(Focused::Movie(id)) => self
                .recent_movies
                .get(id)
                .map(|media| media.is_animating(self.now))
                .unwrap_or_default(),
            None => false,
        }
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

        let animating = if self.is_animating() {
            window::frames().map(|_| HomeMessage::Animate)
        } else {
            Subscription::none()
        };

        Subscription::batch([page, keys, animating])
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
                    text_color: background.text,
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
