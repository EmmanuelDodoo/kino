use super::*;
use crate::utils::{
    cancel_btn, empty, icons, icons::*, modal_container, picklist_handle, save_btn, styles,
    toggler, tooltip, trim_path, typo::*,
};
use registry::models::collection::{
    CollectionView, ItemId,
    triggers::{self, Comparison, DeleteTrigger, InsertTrigger},
};

use registry::models::WishKind;

use devutils::source::SourceSet;
use iced::{
    Color, Element, Length, Padding, Theme,
    alignment::{Horizontal, Vertical},
    color, mouse,
    time::Instant,
    widget::{
        self, bottom_right, button, center, checkbox, column, container, grid, mouse_area,
        pick_list, radio, row, rule, scrollable, space, table, text, text_editor, text_input,
        tooltip as tp,
    },
};
use widgets::expandable;

const PADDING_5: Padding = Padding::new(5.0);

pub fn draw_config(config: &CollectionConfig) -> Element<'_, HomeMessage> {
    let width = 550;
    let height = 550;
    let radius = 5.0;
    let padding = Padding::from([6, 6]);

    let icon_height = 40.0;
    let icon_width = 40.0;

    fn icon_btn<'a>(
        content: impl Into<Element<'a, HomeMessage>>,
        selected: bool,
        message: ConfigMessage,
        label: &'a str,
    ) -> Element<'a, HomeMessage> {
        let radius = 5.0;
        tooltip(
            button(content)
                .padding([0, 0])
                .on_press(HomeMessage::CollectionConfig(message))
                .style(move |theme, status| {
                    let default = if selected {
                        styles::button::weak_primary(theme, status)
                    } else {
                        styles::button::subtle(theme, status)
                    };
                    let border = default.border.rounded(radius);

                    button::Style { border, ..default }
                }),
            label,
            tp::Position::Top,
        )
        .into()
    }

    let name = {
        let label = bold("Name");

        let value = config.name.as_str();
        let input_style = styles::text_input::required(config.empty_name);

        let input = text_input("", value)
            .id(config.name_input.clone())
            .on_input(|input| HomeMessage::CollectionConfig(ConfigMessage::Name(input)))
            .on_submit(HomeMessage::CollectionConfig(ConfigMessage::Save))
            .font(regular_font())
            .padding(padding)
            .style(input_style)
            .width(Length::Fill);

        column!(label, input).spacing(2)
    };

    let description = {
        let label = bold("Description");

        let content = &config.description;
        let editor = text_editor(content)
            .on_action(move |action| {
                HomeMessage::CollectionConfig(ConfigMessage::Description(action))
            })
            .font(regular_font())
            .padding(padding)
            .style(move |theme, status| {
                let default = text_editor::default(theme, status);
                let border = default.border.rounded(radius);

                text_editor::Style { border, ..default }
            })
            .height(height as f32 * 0.2);

        column!(label, editor).spacing(2)
    };

    let view = {
        let selected = config.view;

        let label = bold("Visibility");

        let views = [
            CollectionView::Pinned,
            CollectionView::Shown,
            CollectionView::Hidden,
        ]
        .into_iter()
        .map(|view| {
            let unicode = view_unicode(view);

            let content = center(icon(unicode).size(P));
            let label = match view {
                CollectionView::Shown => "Shown",
                CollectionView::Pinned => "Pinned",
                CollectionView::Hidden => "Hidden",
            };

            icon_btn(content, view == selected, ConfigMessage::View(view), label)
        });

        let views = grid(views)
            .spacing(16)
            .fluid(icon_width)
            .height(grid::aspect_ratio(icon_width, icon_height));

        let views = container(views)
            .padding(padding)
            .style(move |theme: &Theme| {
                let color = theme.palette().secondary.weak.color;
                let default = styles::container::transparent(theme);
                let border = default.border.rounded(radius).color(color).width(1.5);

                container::Style { border, ..default }
            });

        column!(label, views).spacing(2)
    };

    let icons = {
        let selected = config.icon;

        let label = bold("Icon");

        let icons = Icon::all().into_iter().map(|value| {
            let content = center(icon(value.unicode()).size(P));

            icon_btn(
                content,
                value == selected,
                ConfigMessage::Icon(value),
                value.label(),
            )
        });

        let icons = grid(icons)
            .spacing(16)
            .fluid(icon_width)
            .height(grid::aspect_ratio(icon_width, icon_height));

        let icons = container(icons)
            .padding(padding)
            .style(move |theme: &Theme| {
                let color = theme.palette().secondary.weak.color;
                let default = styles::container::transparent(theme);
                let border = default.border.rounded(radius).color(color).width(1.5);

                container::Style { border, ..default }
            });

        column!(label, icons).spacing(2)
    };

    let actions = {
        let save = save_btn().on_press(HomeMessage::CollectionConfig(ConfigMessage::Save));

        let cancel = cancel_btn().on_press(HomeMessage::CloseView);

        column!(row!(save, cancel).spacing(80))
            .align_x(Horizontal::Center)
            .width(Length::Fill)
    };

    let content = column!(name, description, view, icons, space::vertical(), actions).spacing(16);

    modal_container(content).width(width).height(height).into()
}

pub fn draw_search<'a, F: Fn(ItemId) -> HomeMessage + Clone>(
    state: &'a SearchState,
    primary: F,
    theme: &Theme,
    set_play: bool,
) -> Element<'a, HomeMessage> {
    let items = state.items.iter().map(|item| {
        item.view(
            theme,
            HomeMessage::Play,
            primary.clone(),
            |_| HomeMessage::None,
            set_play,
        )
    });

    let input = {
        let filter: Element<'_, HomeMessage> = match state.filter {
            Some(filter) => {
                let content = row!(medium(filter.to_str()), icon(CANCEL))
                    .align_y(Vertical::Center)
                    .spacing(4.0);

                button(content)
                    .on_press(HomeMessage::SearchMessage(SearchMessage::ClearFilter))
                    .style(|theme, status| {
                        let default = styles::button::text_primary(theme, status);
                        let border = default.border.rounded(5);

                        button::Style { border, ..default }
                    })
                    .into()
            }
            None => empty(),
        };

        let size = H6;
        let icon = text_input::Icon {
            font: icons::FONT,
            code_point: icons::SEARCH,
            side: text_input::Side::Right,
            size: Some(size.into()),
            spacing: 5.0,
        };
        let input = text_input("Search Media", &state.search)
            .id(state.text_input.clone())
            .size(size)
            .icon(icon)
            .font(regular_font())
            .on_input(|search| HomeMessage::SearchMessage(SearchMessage::Search(search)))
            .on_submit(HomeMessage::SearchMessage(SearchMessage::Load));

        row!(filter, input).spacing(10.0).align_y(Vertical::Center)
    };

    let content = column!(input).extend(items).spacing(16.0);

    modal_container(content)
        .padding([8, 8])
        .max_width(550)
        .height(Length::Shrink)
        .into()
}

pub fn draw_collection_add<'a>(
    state: &'a CollectionAddState,
    collections: impl Iterator<Item = &'a SimpleCollection>,
    is_empty: bool,
) -> Element<'a, HomeMessage> {
    let title = h6("Collections");

    fn btn(collection: &SimpleCollection, selected: bool) -> Element<'_, HomeMessage> {
        let size = P;
        let unicode = Icon::new(collection.icon).unicode();
        let icon = icons::icon(unicode).size(size);
        let text = container(regular(&collection.name))
            .max_height(48.0)
            .max_width(275);
        let check = checkbox(selected).on_toggle(|value| {
            HomeMessage::CollectionAdd(CollectionAddMessage::Toggle(!value, collection.id))
        });

        button(
            row!(icon, text, space::horizontal(), check)
                .align_y(Vertical::Center)
                .width(Length::Fill)
                .spacing(8.0),
        )
        .padding([8, 12])
        .on_press(HomeMessage::CollectionAdd(CollectionAddMessage::Toggle(
            selected,
            collection.id,
        )))
        .style(move |theme, status| {
            let default = if selected {
                styles::button::subtle(theme, status)
            } else {
                styles::button::subtlest(theme, status)
            };

            let border = default.border.rounded(5.0);

            button::Style { border, ..default }
        })
        .into()
    }

    let collections = column(
        collections.map(|collection| btn(collection, state.selected.contains(&collection.id))),
    )
    .spacing(8.0);

    let collections = scrollable(collections).spacing(16.0);

    let collections = container(collections)
        .padding(if is_empty { [0, 0] } else { [6, 8] })
        .style(|theme: &Theme| {
            let color = theme.palette().secondary.strong.color;
            let default = styles::container::transparent(theme);
            let border = default.border.rounded(5).color(color).width(1.5);

            container::Style { border, ..default }
        });

    let new = button(
        row!(icons::icon(icons::ADD).size(H7), sized_bold("New", H7))
            .align_y(Vertical::Center)
            .spacing(8),
    )
    .padding([2, 4])
    .on_press(HomeMessage::NewCollection)
    .style(styles::button::text_primary);

    let collections = column!(new, collections)
        .spacing(5.0)
        .align_x(Horizontal::Right);

    let actions = {
        let save = save_btn().on_press(HomeMessage::CollectionAdd(CollectionAddMessage::Save));

        let cancel = cancel_btn().on_press(HomeMessage::CloseView);

        row!(save, cancel).spacing(100)
    };

    let content = column!(title, collections, actions)
        .spacing(24)
        .align_x(Horizontal::Center);

    modal_container(content).max_width(400).into()
}

pub fn draw_rating<'a>(state: &Rating) -> Element<'a, HomeMessage> {
    let title = h4("Rating");

    let size = H6;

    let value: Element<'_, HomeMessage> = {
        let size = H6;
        let extra = sized_regular("/5", H7);

        let value: Element<'_, HomeMessage> = match state {
            Rating::Value(value) => {
                let rating = (value * 100.0).round() / 100.0;
                mouse_area(h6(format!("{rating:.2}")))
                    .interaction(mouse::Interaction::Text)
                    .on_press(HomeMessage::Rating(RatingMessage::Type))
                    .into()
            }
            Rating::Input { id, input } => text_input("", input)
                .id(id.clone())
                .size(size)
                .font(regular_font())
                .width(48.0)
                .on_submit(HomeMessage::Rating(RatingMessage::Submit))
                .on_input(|input| HomeMessage::Rating(RatingMessage::Input(input)))
                .into(),
        };

        row!(value, extra)
            .spacing(2.0)
            .align_y(Vertical::Center)
            .into()
    };

    let ratings = {
        let rating = match state {
            Rating::Value(value) => *value,
            Rating::Input { input, .. } => input.parse::<f32>().unwrap_or_default().clamp(0.0, 5.0),
        };

        let stars = (rating.trunc() as u8).clamp(0, 5);
        let rem = 5 - stars;
        let frac = rating.fract() >= 0.5;
        let unstars = if frac { rem.saturating_sub(1) } else { rem };
        let frac = rem - unstars;

        let color = |theme: &Theme| -> text::Style {
            let color = theme.palette().primary.strong.color;
            text::Style { color: Some(color) }
        };

        let stars = (0..stars).map(|_| Element::from(icon(STAR).size(size).style(color)));
        let frac = (0..frac).map(|_| Element::from(icon(HALF_STAR).size(size).style(color)));
        let unstars = (0..unstars).map(|_| Element::from(icon(UNSTAR).size(size).style(color)));

        let stars = stars
            .chain(frac)
            .chain(unstars)
            .enumerate()
            .map(|(idx, elem)| {
                Element::from(
                    button(elem)
                        .on_press(HomeMessage::Rating(RatingMessage::Star((idx + 1) as u8)))
                        .padding(0)
                        .style(styles::button::text),
                )
            });

        row(stars).spacing(6.0).align_y(Vertical::Center)
    };

    let content = column!(title, value, ratings)
        .spacing(16.0)
        .align_x(Horizontal::Center);

    modal_container(content).max_width(400).into()
}

pub fn draw_rename<'a>(
    input: &widget::Id,
    placeholder: &str,
    value: &str,
    is_empty: bool,
) -> Element<'a, HomeMessage> {
    let input = text_input(placeholder, value)
        .on_input(|new| HomeMessage::Rename(RenameMessage::Input(new)))
        .on_submit(HomeMessage::Rename(RenameMessage::Submit))
        .font(regular_font())
        .id(input.clone())
        .size(H7)
        .width(250)
        .style(move |theme: &Theme, status| {
            let error = theme.palette().danger.strong.color;
            let default = text_input::default(theme, status);
            let border = default.border.rounded(5);
            let border = if is_empty && matches!(status, text_input::Status::Focused { .. }) {
                border.color(error)
            } else {
                border
            };

            text_input::Style { border, ..default }
        });

    modal_container(input).padding([6, 8]).into()
}

pub fn draw_synopsis<'a>(
    editor: &widget::Id,
    content: &'a text_editor::Content,
) -> Element<'a, HomeMessage> {
    let content = text_editor(content)
        .id(editor.clone())
        .font(regular_font())
        .width(575)
        .on_action(|action| HomeMessage::Synopsis(SynopsisMessage::Action(action)))
        .key_binding(|press| {
            use iced::keyboard::{Key, key::Named};
            use text_editor::{Binding, Status};

            let is_focused = matches!(press.status, Status::Focused { .. });

            match press.key {
                Key::Named(Named::Enter) if press.modifiers.command() && is_focused => Some(
                    Binding::Custom(HomeMessage::Synopsis(SynopsisMessage::Submit)),
                ),
                _ => Binding::from_key_press(press),
            }
        })
        .size(P);

    modal_container(content).padding([6, 6]).into()
}

pub fn draw_tmdb<'a>(input: &widget::Id, value: &str, top_level: bool) -> Element<'a, HomeMessage> {
    let input = text_input(
        if top_level {
            "TMDB ID"
        } else {
            "Season/Episode number"
        },
        value,
    )
    .on_input(|new| HomeMessage::TMDBId(TMDBMessage::Input(new)))
    .on_submit(HomeMessage::TMDBId(TMDBMessage::Submit))
    .font(regular_font())
    .id(input.clone())
    .size(H7)
    .width(250);

    modal_container(input).padding([6, 8]).into()
}

pub fn draw_delete_confirm<'a>(name: &'a str, message: HomeMessage) -> Element<'a, HomeMessage> {
    let title = h6("Confirm Deletion");

    let body = sized_medium(format!("Are you sure you want to delete \"{name}\""), P);

    let delete = button(medium("Delete"))
        .on_press(message)
        .style(styles::button::danger);

    let cancel = button(medium("Cancel"))
        .on_press(HomeMessage::CloseView)
        .style(styles::button::primary);

    let actions = row!(cancel, delete).spacing(80.0).align_y(Vertical::Center);

    let content = column!(title, body, actions)
        .spacing(40)
        .align_x(Horizontal::Center);

    modal_container(content).max_width(500).into()
}

pub fn draw_collection_triggers<'a>(
    view_inserts: bool,
    itriggers: &'a [(bool, InsertTrigger, bool, String, String)],
    dtriggers: &'a [(bool, DeleteTrigger, bool, String, String)],
) -> Element<'a, HomeMessage> {
    let title = h6("Collection Rules");
    let text_size = P;
    let input_padding = [3.5, 5.0];

    fn label_maker<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
        sized_medium(label, P)
    }

    let content: Element<'_, TriggerMessage> = {
        let tabs = {
            let pop = {
                let text = "Auto-Populate";
                let text = if view_inserts {
                    bold(text)
                } else {
                    regular(text)
                };

                column!(
                    button(text)
                        .on_press(TriggerMessage::Tab)
                        .style(styles::button::text),
                    container(iced::widget::Space::new().width(88).height(2)).style(
                        if view_inserts {
                            styles::container::pb
                        } else {
                            styles::container::transparent
                        }
                    ),
                )
                .align_x(Horizontal::Center)
                .padding([3, 6])
                .spacing(0.0)
            };

            let remove = {
                let text = "Auto-Remove";
                let text = if !view_inserts {
                    bold(text)
                } else {
                    regular(text)
                };

                column!(
                    button(text)
                        .on_press(TriggerMessage::Tab)
                        .style(styles::button::text),
                    container(iced::widget::Space::new().width(88).height(2)).style(
                        if !view_inserts {
                            styles::container::pb
                        } else {
                            styles::container::transparent
                        }
                    ),
                )
                .align_x(Horizontal::Center)
                .padding([3, 6])
                .spacing(0.0)
            };

            row!(pop, remove).spacing(200)
        };

        let new = {
            let new = {
                let icon = icons::icon(icons::ADD).size(text_size * RATIO);
                let label = label_maker("New");

                row!(icon, label).spacing(8.0).align_y(Vertical::Center)
            };

            row!(
                space::horizontal(),
                button(new)
                    .on_press(if view_inserts {
                        TriggerMessage::AddInsert
                    } else {
                        TriggerMessage::AddDelete
                    })
                    .style(styles::button::text_primary)
            )
        };

        let triggers = if view_inserts {
            let triggers = itriggers.iter().map(|(open, trigger, roe, last, release)| {
                draw_insert_trigger(*open, trigger, roe, last, release, text_size, input_padding)
            });

            scrollable(column(triggers).spacing(12).padding(6))
        } else {
            let triggers = dtriggers.iter().map(|(open, trigger, roe, last, release)| {
                draw_delete_trigger(*open, trigger, roe, last, release, text_size, input_padding)
            });

            scrollable(column(triggers).spacing(12).padding(6))
        };

        column!(tabs, new, triggers)
            .height(Length::Fill)
            .spacing(12)
            .align_x(Horizontal::Center)
            .into()
    };

    let actions = {
        let save = save_btn().on_press(HomeMessage::Trigger(TriggerMessage::Save));

        let cancel = cancel_btn().on_press(HomeMessage::CloseView);

        row!(save, cancel).spacing(100)
    };

    let content = column!(title, content.map(HomeMessage::Trigger), actions)
        .spacing(24)
        .align_x(Horizontal::Center);

    modal_container(content)
        .padding([16, 16])
        .width(700)
        .height(700)
        .into()
}

pub fn draw_insert_trigger<'a>(
    open: bool,
    trigger: &'a InsertTrigger,
    roe: &bool,
    last: &'a str,
    release: &'a str,
    text_size: f32,
    input_padding: [f32; 2],
) -> Element<'a, TriggerMessage> {
    let id = trigger.id;
    let width = 200;
    let padding = [10, 12];

    let title = {
        let size = text_size;

        let kind = regular(trigger.media.to_string()).size(size / RATIO);

        let kind = container(kind).padding([1, 5]).style(|theme| {
            let default = styles::container::text_ps(theme);
            let border = default
                .border
                .rounded(3.0)
                .color(default.text_color.unwrap_or_default())
                .width(0.75);

            container::Style { border, ..default }
        });

        let name = sized_medium(&trigger.name, size);

        let expand = if open { CHEV_DOWN } else { CHEV_LEFT };
        let expand = icons::icon(expand).size(size);

        let name = row!(expand, kind, name)
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .spacing(6);

        let duplicate = button(icons::icon(COPY).size(size))
            .padding(0)
            .style(styles::button::text_primary)
            .on_press(TriggerMessage::DuplicateInsert(id));

        let remove = button(icons::icon(DELETE).size(size))
            .padding(0)
            .style(styles::button::text_danger)
            .on_press(TriggerMessage::RemoveInsert(id));

        row!(name, duplicate, remove,)
            .spacing(8)
            .align_y(Vertical::Center)
    };

    let name = {
        let name = sized_medium("Rule name", text_size);
        let input = text_input("", &trigger.name)
            .size(text_size)
            .on_input(move |name| TriggerMessage::NameInsert(id, name))
            .font(regular_font())
            .padding(input_padding)
            .width(Length::FillPortion(2));

        row!(name, space::horizontal().width(Length::Fill), input).align_y(Vertical::Center)
    };

    let media = {
        let label = sized_medium("Target Media", text_size);

        let options = if trigger.logic.tags.is_none() && trigger.logic.dir.is_none() {
            triggers::Media::VARIANTS
        } else {
            triggers::Media::ROOTS
        };

        let pick = pick_list(Some(trigger.media), options, ToString::to_string)
            .width(88.0)
            .on_select(move |media| TriggerMessage::MediaInsert(id, media))
            .padding([2, 5])
            .style(styles::pick_list::default)
            .handle(picklist_handle(text_size))
            .padding([5, 10])
            .font(regular_font())
            .text_size(text_size);

        row!(label, space::horizontal(), pick).align_y(Vertical::Center)
    };

    let roe = {
        let label = sized_medium("Run on existing media", text_size);
        let label = button(label)
            .padding(0)
            .style(styles::button::text)
            .on_press(TriggerMessage::ToggleROEInsert(id));
        let toggle = toggler(*roe).on_toggle(move |checked| TriggerMessage::ROEInsert(id, checked));

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let generate = {
        let label = sized_medium("Generate Delete Rule", text_size / RATIO);

        button(label)
            .style(styles::button::subtlest)
            .on_press(TriggerMessage::GenerateDelete(id))
    };

    let cond: Element<'_, LogicMessage> = {
        let title = sized_medium("Conditions", text_size);
        let size = text_size / RATIO;
        let pick_padding = [2, 5];

        let input = |value: &str| -> text_input::TextInput<'_, LogicMessage> {
            text_input("", value)
                .align_x(Horizontal::Right)
                .size(size)
                .font(regular_font())
                .padding(input_padding)
                .width(160)
        };

        let name = {
            let label = sized_medium("Name contains", size).width(width);

            let (not, name) = trigger
                .logic
                .name
                .as_ref()
                .map(|(not, name)| (*not, name.as_str()))
                .unwrap_or((false, ""));
            let input = input(name).on_input(LogicMessage::Name);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::NameComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let synopsis = {
            let label = sized_medium("Overview contains", size).width(width);

            let (not, synopsis) = trigger
                .logic
                .synopsis
                .as_ref()
                .map(|(not, synopsis)| (*not, synopsis.as_str()))
                .unwrap_or((false, ""));
            let input = input(synopsis).on_input(LogicMessage::Synopsis);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::SynopsisComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let tags = {
            let label = sized_medium("Tags contain", size).width(width);

            let (not, tags) = trigger
                .logic
                .tags
                .as_ref()
                .map(|(not, tags)| (*not, tags.as_str()))
                .unwrap_or((false, ""));
            let input = input(tags).on_input(LogicMessage::Tags);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::TagsComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let dir = {
            let label = sized_medium("Directory contains", size).width(width);

            let (not, dir) = trigger
                .logic
                .dir
                .as_ref()
                .map(|(not, dir)| (*not, dir.as_str()))
                .unwrap_or((false, ""));
            let input = input(dir).on_input(LogicMessage::Dir);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::DirComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let last = {
            let label = sized_medium("Last watched", size).width(width);

            let comp = trigger
                .logic
                .last_watched
                .as_ref()
                .map(|(comp, _)| *comp)
                .unwrap_or_default();

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::LastComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let input = input(last).on_input(LogicMessage::Last);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let duration = {
            let label = sized_medium("Duration", size).width(width);

            let (comp, duration) = trigger
                .logic
                .duration
                .as_ref()
                .map(|(comp, duration)| (*comp, *duration))
                .unwrap_or((Comparison::default(), 0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::DurationComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let duration = duration.to_string();
            let input = input(&duration).on_input(LogicMessage::Duration);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let progress = {
            let label = sized_medium("Progress", size).width(width);

            let (comp, progress) = trigger
                .logic
                .progress
                .as_ref()
                .map(|(comp, progress)| (*comp, *progress))
                .unwrap_or((Comparison::default(), 0.0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::ProgressComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let progress = format!("{progress:.2}");
            let input = input(&progress).on_input(LogicMessage::Progress);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let watch = {
            let label = sized_medium("Watch count", size).width(width);

            let (comp, count) = trigger
                .logic
                .watch_count
                .as_ref()
                .map(|(comp, count)| (*comp, *count))
                .unwrap_or((Comparison::default(), 0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::WatchComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let count = count.to_string();
            let input = input(&count).on_input(LogicMessage::Watch);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let release = {
            let label = sized_medium("Release date", size).width(width);

            let comp = trigger
                .logic
                .release
                .as_ref()
                .map(|(comp, _)| *comp)
                .unwrap_or_default();

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::ReleaseComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let input = input(release).on_input(LogicMessage::Release);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let rating = {
            let label = sized_medium("Rating", size).width(width);

            let (comp, rating) = trigger
                .logic
                .rating
                .as_ref()
                .map(|(comp, rating)| (*comp, *rating))
                .unwrap_or((Comparison::default(), 0.0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::RatingComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let rating = format!("{rating:.2}");
            let input = input(&rating).on_input(LogicMessage::Rating);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let comment = {
            let label = sized_medium("Comments", size).width(width);

            let (comp, count) = trigger
                .logic
                .comment
                .as_ref()
                .map(|(comp, count)| (*comp, *count))
                .unwrap_or((Comparison::default(), 0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::CommentComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let count = count.to_string();
            let input = input(&count).on_input(LogicMessage::Comment);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        column!(
            title, name, synopsis, tags, dir, last, duration, progress, watch, release, rating,
            comment
        )
        .spacing(4.0)
        .align_x(Horizontal::Center)
        .into()
    };

    let content = column!(
        rule::horizontal(1.0),
        name,
        media,
        roe,
        generate,
        cond.map(move |lsg| TriggerMessage::LogicInsert(id, lsg))
    )
    .spacing(8);

    let content = expandable(title, content)
        .width(Length::Fill)
        .expanded(open)
        .spacing(8)
        .on_expand(move |expand| TriggerMessage::ToggleExpandInsert(id, expand));

    let content = container(content)
        .style(styles::container::bw)
        .padding(padding);
    content.into()
}

pub fn draw_delete_trigger<'a>(
    open: bool,
    trigger: &'a DeleteTrigger,
    roe: &bool,
    last: &'a str,
    release: &'a str,
    text_size: f32,
    input_padding: [f32; 2],
) -> Element<'a, TriggerMessage> {
    let id = trigger.id;
    let width = 200;
    let padding = [10, 12];

    let title = {
        let size = text_size;

        let kind = regular(trigger.media.to_string()).size(size / RATIO);
        let kind = container(kind).padding([1, 5]).style(|theme| {
            let default = styles::container::text_ps(theme);
            let border = default
                .border
                .rounded(3.0)
                .color(default.text_color.unwrap_or_default())
                .width(0.75);

            container::Style { border, ..default }
        });

        let name = sized_medium(&trigger.name, size);

        let expand = if open { CHEV_DOWN } else { CHEV_LEFT };
        let expand = icons::icon(expand).size(size);

        let name = row!(expand, kind, name)
            .width(Length::Fill)
            .align_y(Vertical::Center)
            .spacing(6);

        let duplicate = button(icons::icon(COPY).size(size))
            .padding(0)
            .style(styles::button::text_primary)
            .on_press(TriggerMessage::DuplicateDelete(id));

        let remove = button(icons::icon(DELETE).size(size))
            .padding(0)
            .style(styles::button::text_danger)
            .on_press(TriggerMessage::RemoveDelete(id));

        row!(name, duplicate, remove,)
            .spacing(8)
            .align_y(Vertical::Center)
    };

    let name = {
        let name = sized_medium("Rule name", text_size);
        let input = text_input("", &trigger.name)
            .size(text_size)
            .on_input(move |name| TriggerMessage::NameDelete(id, name))
            .font(regular_font())
            .padding(input_padding)
            .width(Length::FillPortion(2));

        row!(name, space::horizontal().width(Length::Fill), input).align_y(Vertical::Center)
    };

    let media = {
        let label = sized_medium("Target Media", text_size);

        let options = if trigger.logic.tags.is_none() && trigger.logic.dir.is_none() {
            triggers::Media::VARIANTS
        } else {
            triggers::Media::ROOTS
        };

        let pick = pick_list(Some(trigger.media), options, ToString::to_string)
            .width(88.0)
            .style(styles::pick_list::default)
            .on_select(move |media| TriggerMessage::MediaDelete(id, media))
            .padding([2, 5])
            .handle(picklist_handle(text_size))
            .font(regular_font())
            .padding([5, 10])
            .text_size(text_size);

        row!(label, space::horizontal(), pick).align_y(Vertical::Center)
    };

    let roe = {
        let label = sized_medium("Run on existing media", text_size);
        let label = button(label)
            .padding(0)
            .style(styles::button::text)
            .on_press(TriggerMessage::ToggleROEDelete(id));
        let toggle = toggler(*roe).on_toggle(move |checked| TriggerMessage::ROEDelete(id, checked));

        row!(label, space::horizontal(), toggle).align_y(Vertical::Center)
    };

    let cond: Element<'_, LogicMessage> = {
        let title = sized_medium("Conditions", text_size);
        let size = text_size / RATIO;
        let pick_padding = [2, 5];

        let input = |value: &str| -> text_input::TextInput<'_, LogicMessage> {
            text_input("", value)
                .size(size)
                .font(regular_font())
                .padding(input_padding)
                .align_x(Horizontal::Right)
                .width(160)
        };

        let name = {
            let label = sized_medium("Name contains", size).width(width);

            let (not, name) = trigger
                .logic
                .name
                .as_ref()
                .map(|(not, name)| (*not, name.as_str()))
                .unwrap_or((false, ""));
            let input = input(name).on_input(LogicMessage::Name);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::NameComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let synopsis = {
            let label = sized_medium("Overview contains", size).width(width);

            let (not, synopsis) = trigger
                .logic
                .synopsis
                .as_ref()
                .map(|(not, synopsis)| (*not, synopsis.as_str()))
                .unwrap_or((false, ""));
            let input = input(synopsis).on_input(LogicMessage::Synopsis);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::SynopsisComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let tags = {
            let label = sized_medium("Tags contain", size).width(width);

            let (not, tags) = trigger
                .logic
                .tags
                .as_ref()
                .map(|(not, tags)| (*not, tags.as_str()))
                .unwrap_or((false, ""));
            let input = input(tags).on_input(LogicMessage::Tags);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::TagsComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let dir = {
            let label = sized_medium("Directory contains", size).width(width);

            let (not, dir) = trigger
                .logic
                .dir
                .as_ref()
                .map(|(not, dir)| (*not, dir.as_str()))
                .unwrap_or((false, ""));
            let input = input(dir).on_input(LogicMessage::Dir);

            let not = checkbox(not)
                .label("NOT")
                .text_size(size)
                .on_toggle(LogicMessage::DirComp)
                .size(size);

            row!(label, space::horizontal(), not, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let last = {
            let label = sized_medium("Last watched", size).width(width);

            let comp = trigger
                .logic
                .last_watched
                .as_ref()
                .map(|(comp, _)| *comp)
                .unwrap_or_default();

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::LastComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let input = input(last).on_input(LogicMessage::Last);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let duration = {
            let label = sized_medium("Duration", size).width(width);

            let (comp, duration) = trigger
                .logic
                .duration
                .as_ref()
                .map(|(comp, duration)| (*comp, *duration))
                .unwrap_or((Comparison::default(), 0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .on_select(LogicMessage::DurationComp)
                .style(styles::pick_list::default)
                .padding(pick_padding)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let duration = duration.to_string();
            let input = input(&duration).on_input(LogicMessage::Duration);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let progress = {
            let label = sized_medium("Progress", size).width(width);

            let (comp, progress) = trigger
                .logic
                .progress
                .as_ref()
                .map(|(comp, progress)| (*comp, *progress))
                .unwrap_or((Comparison::default(), 0.0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::ProgressComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let progress = format!("{progress:.2}");
            let input = input(&progress).on_input(LogicMessage::Progress);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let watch = {
            let label = sized_medium("Watch count", size).width(width);

            let (comp, count) = trigger
                .logic
                .watch_count
                .as_ref()
                .map(|(comp, count)| (*comp, *count))
                .unwrap_or((Comparison::default(), 0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::WatchComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let count = count.to_string();
            let input = input(&count).on_input(LogicMessage::Watch);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let release = {
            let label = sized_medium("Release date", size).width(width);

            let comp = trigger
                .logic
                .release
                .as_ref()
                .map(|(comp, _)| *comp)
                .unwrap_or_default();

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::ReleaseComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let input = input(release).on_input(LogicMessage::Release);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let rating = {
            let label = sized_medium("Rating", size).width(width);

            let (comp, rating) = trigger
                .logic
                .rating
                .as_ref()
                .map(|(comp, rating)| (*comp, *rating))
                .unwrap_or((Comparison::default(), 0.0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::RatingComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let rating = format!("{rating:.2}");
            let input = input(&rating).on_input(LogicMessage::Rating);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        let comment = {
            let label = sized_medium("Comments", size).width(width);

            let (comp, count) = trigger
                .logic
                .comment
                .as_ref()
                .map(|(comp, count)| (*comp, *count))
                .unwrap_or((Comparison::default(), 0));

            let comp = pick_list(Some(comp), Comparison::VARIANTS, ToString::to_string)
                .padding(pick_padding)
                .style(styles::pick_list::default)
                .on_select(LogicMessage::CommentComp)
                .handle(picklist_handle(text_size))
                .font(regular_font())
                .text_size(text_size);

            let count = count.to_string();
            let input = input(&count).on_input(LogicMessage::Comment);

            row!(label, space::horizontal(), comp, input)
                .spacing(6.0)
                .align_y(Vertical::Center)
        };

        column!(
            title, name, synopsis, tags, dir, last, duration, progress, watch, release, rating,
            comment
        )
        .spacing(4.0)
        .align_x(Horizontal::Center)
        .into()
    };

    let content = column!(
        rule::horizontal(1.0),
        name,
        media,
        roe,
        cond.map(move |lsg| TriggerMessage::LogicDelete(id, lsg))
    )
    .spacing(8);

    let content = expandable(title, content)
        .width(Length::Fill)
        .expanded(open)
        .spacing(8)
        .on_expand(move |expand| TriggerMessage::ToggleExpandDelete(id, expand));

    let content = container(content)
        .style(styles::container::bw)
        .padding(padding);
    content.into()
}

pub fn draw_selection<'a>(items: usize) -> Element<'a, HomeMessage> {
    let dimensions = 76;

    let play = button(icons::icon(PLAY).size(40).center())
        .on_press(SelectionMessage::Play)
        .width(56)
        .height(56)
        .style(|theme, status| {
            let default = styles::button::weak_primary(theme, status);
            let text = styles::button::text_primary(theme, status);
            let border = default.border.rounded(100.0);

            button::Style {
                border,
                text_color: text.text_color,
                ..default
            }
        });
    let count = sized_medium(items.to_string(), P);

    let cancel = button(icons::icon(CANCEL).size(H6))
        .padding(0)
        .style(styles::button::text_danger)
        .on_press(SelectionMessage::Cancel);

    let extra = column!(cancel, space::vertical(), count);

    let content: Element<'_, SelectionMessage> = container(
        row!(play, extra)
            .width(dimensions)
            .height(dimensions)
            .spacing(4.0)
            .align_y(Vertical::Center),
    )
    .align_y(Vertical::Center)
    .align_x(Horizontal::Center)
    .padding([40, 20])
    .style(|theme| {
        let default = styles::container::transparent(theme);
        let border = default.border.rounded(5.0);
        let background = default
            .background
            .map(|background| background.scale_alpha(0.25));

        container::Style {
            border,
            background,
            ..default
        }
    })
    .into();

    content.map(HomeMessage::Selection)
}

pub fn draw_wishlist<'a>(state: &'a WishNewState) -> Element<'a, HomeMessage> {
    let width = 550;
    let height = 325;
    let size = P;
    let padding = Padding::from([6, 6]);

    let name = {
        let name = match state.kind {
            WishKindSelection::Movie => "Movie name",
            _ => "Show name",
        };
        let is_root = matches!(
            state.kind,
            WishKindSelection::Show | WishKindSelection::Movie
        );

        let label = medium(name);

        let input_style = styles::text_input::required(state.invalid_name());

        let input = text_input("", state.name())
            .id(state.name_input.clone())
            .on_input(|input| HomeMessage::WishViewMessage(WishViewMessage::Name(input)))
            .on_submit_maybe(is_root.then_some(HomeMessage::WishViewMessage(WishViewMessage::Save)))
            .font(regular_font())
            .padding(padding)
            .size(P)
            .style(input_style)
            .width(Length::Fill);

        column!(label, input).spacing(3)
    };

    let extra = match state.kind {
        WishKindSelection::Show | WishKindSelection::Movie => empty(),
        WishKindSelection::Season => {
            let label = medium("Season number:");
            let input_style = styles::text_input::required(state.invalid_season());

            let input = text_input("", state.season())
                .id(state.season_input.clone())
                .on_input(|input| HomeMessage::WishViewMessage(WishViewMessage::Season(input)))
                .align_x(Horizontal::Right)
                .font(regular_font())
                .size(P)
                .padding([4, 6])
                .style(input_style)
                .width(40.0);

            row!(label, input)
                .spacing(12.0)
                .align_y(Vertical::Center)
                .into()
        }
        WishKindSelection::Episode => {
            let padding = [4, 6];

            let season = {
                let label = medium("Season number:");
                let input_style = styles::text_input::required(state.invalid_season());

                let input = text_input("", state.season())
                    .id(state.season_input.clone())
                    .on_input(|input| HomeMessage::WishViewMessage(WishViewMessage::Season(input)))
                    .font(regular_font())
                    .align_x(Horizontal::Right)
                    .size(P)
                    .padding(padding)
                    .style(input_style)
                    .width(40.0);

                row!(label, input).spacing(12.0).align_y(Vertical::Center)
            };

            let episode = {
                let label = medium("Episode number:");
                let input_style = styles::text_input::required(state.invalid_episode());

                let input = text_input("", state.episode())
                    .id(state.episode_input.clone())
                    .on_input(|input| HomeMessage::WishViewMessage(WishViewMessage::Episode(input)))
                    .size(P)
                    .align_x(Horizontal::Right)
                    .font(regular_font())
                    .padding(padding)
                    .style(input_style)
                    .width(40.0);

                row!(label, input).spacing(12.0).align_y(Vertical::Center)
            };

            row!(season, episode)
                .spacing(40)
                .align_y(Vertical::Center)
                .into()
        }
    };

    let source = {
        let label = medium("Source:");
        let handle = picklist_handle(size);

        let source = pick_list(Some(state.source), SourceSet::VARIANTS, |source| {
            source.to_str().to_owned()
        })
        .style(styles::pick_list::default)
        .font(regular_font())
        .on_select(|kind| HomeMessage::WishViewMessage(WishViewMessage::Source(kind)))
        .handle(handle)
        .padding([5, 10])
        .text_size(size);

        row!(label, space::horizontal(), source)
            .spacing(40.0)
            .align_y(Vertical::Center)
    };

    let kind = {
        let label = medium("Media Type:");
        let handle = picklist_handle(size);

        let kind = pick_list(Some(state.kind), WishKindSelection::VARIANTS, |kind| {
            match kind {
                WishKindSelection::Movie => "Movie",
                WishKindSelection::Show => "Show",
                WishKindSelection::Season => "Season",
                WishKindSelection::Episode => "Episode",
            }
            .to_owned()
        })
        .style(styles::pick_list::default)
        .font(regular_font())
        .on_select(|kind| HomeMessage::WishViewMessage(WishViewMessage::Kind(kind)))
        .handle(handle)
        .padding([5, 10])
        .text_size(size);

        row!(label, space::horizontal(), kind)
            .spacing(40.0)
            .align_y(Vertical::Center)
    };

    let id = {
        let label = medium("Source Id (optional):");

        let input = text_input("", &state.source_id)
            .on_input(|input| HomeMessage::WishViewMessage(WishViewMessage::SourceId(input)))
            .align_x(Horizontal::Right)
            .style(styles::text_input::default)
            .font(regular_font())
            .size(P)
            .padding([4, 6])
            .width(80.0);

        row!(label, space::horizontal(), input).align_y(Vertical::Center)
    };

    let actions = {
        let save = save_btn().on_press(HomeMessage::WishViewMessage(WishViewMessage::Save));

        let cancel = cancel_btn().on_press(HomeMessage::CloseView);

        column!(row!(save, cancel).spacing(80))
            .align_x(Horizontal::Center)
            .width(Length::Fill)
    };

    let content = column!(name, extra, kind, source, id).spacing(16.0);

    let content = column!(content, actions).spacing(20.0);

    modal_container(content)
        .width(width)
        .max_height(height)
        .into()
}

pub fn draw_wish<'a>(wish: &'a WishThumbnail, now: Instant) -> Element<'a, HomeMessage> {
    modal_container(wish.modal(now).map(HomeMessage::Wishlist))
        .width(450)
        .padding([8, 12])
        .align_y(Vertical::Top)
        .into()
}

pub fn draw_movie_edit<'a>(
    state: &'a MovieEditState,
    videos: &'a [VideoInfo],
    audio: &'a [Audio],
    subs: &'a [Subtitle],
) -> Element<'a, HomeMessage> {
    let name = media_name(&state.placeholder, state.name(), MovieEditMessage::Name);

    let overview = media_overview(&state.overview, MovieEditMessage::Overview);

    let ratings = media_rating(&state.ratings, MovieEditMessage::Rating);

    let watched = media_mark(state.watched, MovieEditMessage::MarkWatched);

    let videos = draw_videos(videos, state.selected_video, MovieEditMessage::Video);

    let audio = draw_audio(audio, state.selected_audio, MovieEditMessage::Audio);

    let subtitles = draw_subs(
        subs,
        state.selected_sub,
        MovieEditMessage::Subtitle,
        MovieEditMessage::SubDelete,
    );

    let source = media_source(
        &state.source,
        Some(|source| MovieEditMessage::Source(source)),
    );

    let source_id = media_source_id(state.source, &state.source_id, MovieEditMessage::SourceId);

    let refetch = media_icon_btn(REFRESH, "Refetch").on_press(MovieEditMessage::Refetch);

    let remove = media_icon_btn(DELETE, "Delete").on_press(MovieEditMessage::Remove);

    let poster = media_image(
        "Poster: ",
        state.poster.as_deref(),
        MovieEditMessage::PickPoster,
    );
    let backdrop = media_image(
        "Backdrop: ",
        state.backdrop.as_deref(),
        MovieEditMessage::PickPoster,
    );

    let content: Element<'_, MovieEditMessage> = column!(
        name, overview, ratings, watched, videos, audio, subtitles, source, source_id, poster,
        backdrop, refetch, remove,
    )
    .spacing(24)
    .into();

    media_layout(
        content.map(HomeMessage::MovieEdit),
        HomeMessage::MovieEdit(MovieEditMessage::Save),
    )
}

pub fn draw_episode_edit<'a>(
    state: &'a EpisodeEditState,
    videos: &'a [VideoInfo],
    audio: &'a [Audio],
    subs: &'a [Subtitle],
) -> Element<'a, HomeMessage> {
    let name = media_name(&state.placeholder, state.name(), EpisodeEditMessage::Name);

    let overview = media_overview(&state.overview, EpisodeEditMessage::Overview);

    let ratings = media_rating(&state.ratings, EpisodeEditMessage::Rating);

    let watched = media_mark(state.watched, EpisodeEditMessage::MarkWatched);

    let videos = draw_videos(videos, state.selected_video, EpisodeEditMessage::Video);

    let audio = draw_audio(audio, state.selected_audio, EpisodeEditMessage::Audio);

    let subtitles = draw_subs(
        subs,
        state.selected_sub,
        EpisodeEditMessage::Subtitle,
        EpisodeEditMessage::SubDelete,
    );

    let source = media_source(&state.source, None::<fn(_) -> EpisodeEditMessage>);

    let source_id = media_source_id(state.source, &state.source_id, EpisodeEditMessage::SourceId);

    let refetch = media_icon_btn(REFRESH, "Refetch").on_press(EpisodeEditMessage::Refetch);

    let remove = media_icon_btn(DELETE, "Delete").on_press(EpisodeEditMessage::Remove);

    let poster = media_image(
        "Poster: ",
        state.poster.as_deref(),
        EpisodeEditMessage::PickPoster,
    );

    let content: Element<'_, EpisodeEditMessage> = column!(
        name, overview, ratings, watched, videos, audio, subtitles, source, source_id, refetch,
        remove, poster
    )
    .spacing(24)
    .into();

    media_layout(
        content.map(HomeMessage::EpisodeEdit),
        HomeMessage::EpisodeEdit(EpisodeEditMessage::Save),
    )
}

fn draw_videos<'a, Message: 'a + Clone, F>(
    videos: &'a [VideoInfo],
    selected: Option<VideoInfoId>,
    on_select: F,
) -> Element<'a, Message>
where
    F: Fn(VideoInfoId) -> Message + 'a + Clone,
{
    if videos.is_empty() {
        return empty();
    }

    let label = mouse_area(
        row!(media_label("Videos"))
            .width(Length::Fill)
            .padding([2, 2]),
    )
    .interaction(mouse::Interaction::Pointer);
    let size = H7;

    let rad = table::column(empty(), |video: &VideoInfo| {
        radio("", video.id, selected, on_select.clone())
            .size(12)
            .spacing(0)
    })
    .align_y(Vertical::Center);

    let codec = table::column(empty(), |video: &VideoInfo| {
        let codec = video.codec.as_deref().unwrap_or("unknown codec");
        button(sized_regular(codec, size))
            .on_press(on_select(video.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let resolution = table::column(empty(), |video: &VideoInfo| {
        button(sized_regular(video.resolution(), size))
            .on_press(on_select(video.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let framerate = table::column(empty(), |video: &VideoInfo| {
        if video.framerate > 0.0 {
            let framerate = format!("{:.2} fps", video.framerate);

            Some(
                button(sized_regular(framerate, size))
                    .on_press(on_select(video.id))
                    .padding(0)
                    .style(styles::button::text),
            )
        } else {
            None
        }
    })
    .align_y(Vertical::Center);

    let bitrate = table::column(empty(), |video: &VideoInfo| {
        if video.bitrate > 0 {
            let bitrate = format!("{:.2} Mbps", video.bitrate as f32 / 1000_000.0);

            Some(
                button(sized_regular(bitrate, size))
                    .on_press(on_select(video.id))
                    .padding(0)
                    .style(styles::button::text),
            )
        } else {
            None
        }
    })
    .align_y(Vertical::Center);

    let content = table([rad, codec, resolution, bitrate, framerate], videos).separator(0);

    expandable(label, content)
        .width(Length::Fill)
        .expanded(true)
        .spacing(0)
        .into()
}

fn draw_audio<'a, Message: 'a + Clone, F>(
    audio: &'a [Audio],
    selected: Option<AudioId>,
    on_select: F,
) -> Element<'a, Message>
where
    F: Fn(AudioId) -> Message + 'a + Clone,
{
    if audio.is_empty() {
        return empty();
    }

    let size = H7;
    let label = mouse_area(
        row!(media_label("Audios"))
            .width(Length::Fill)
            .padding([2, 2]),
    )
    .interaction(mouse::Interaction::Pointer);

    let radio = table::column(empty(), |audio: &Audio| {
        radio("", audio.id, selected, on_select.clone())
            .size(12)
            .spacing(0)
    })
    .align_y(Vertical::Center);

    let lang = table::column(empty(), |audio: &Audio| {
        let lang = audio.lang.as_deref().unwrap_or("unk. lang");
        button(sized_regular(lang, size))
            .on_press(on_select(audio.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let codec = table::column(empty(), |audio: &Audio| {
        let codec = audio.codec.as_deref().unwrap_or("unk. codec");
        button(sized_regular(codec, size))
            .on_press(on_select(audio.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let bitrate = table::column(empty(), |audio: &Audio| {
        if audio.bitrate > 0 {
            let bitrate = format!("{:.2} kbps", audio.bitrate as f32 / 1000.0);

            Some(
                button(sized_regular(bitrate, size))
                    .on_press(on_select(audio.id))
                    .padding(0)
                    .style(styles::button::text),
            )
        } else {
            None
        }
    })
    .align_y(Vertical::Center);

    let sample = table::column(empty(), |audio: &Audio| {
        if audio.sample_rate > 0 {
            let sample = format!("{} Hz", audio.sample_rate);

            Some(
                button(sized_regular(sample, size))
                    .on_press(on_select(audio.id))
                    .padding(0)
                    .style(styles::button::text),
            )
        } else {
            None
        }
    })
    .align_y(Vertical::Center);

    let channels = table::column(empty(), |audio: &Audio| {
        if audio.channels > 0 {
            let channels = format!("{} chan.", audio.channels);

            Some(
                button(sized_regular(channels, size))
                    .on_press(on_select(audio.id))
                    .padding(0)
                    .style(styles::button::text),
            )
        } else {
            None
        }
    })
    .align_y(Vertical::Center);

    let content = table([radio, lang, codec, bitrate, sample, channels], audio).separator(0);

    expandable(label, content)
        .width(Length::Fill)
        .expanded(true)
        .spacing(0)
        .into()
}

fn draw_subs<'a, Message: 'a + Clone, F, D>(
    subs: &'a [Subtitle],
    selected: Option<SubtitleId>,
    on_select: F,
    on_delete: D,
) -> Element<'a, Message>
where
    F: Fn(SubtitleId) -> Message + 'a + Clone,
    D: Fn(SubtitleId) -> Message + 'a,
{
    if subs.is_empty() {
        return empty();
    }

    let label = mouse_area(
        row!(media_label("Subtitles"))
            .width(Length::Fill)
            .padding([2, 2]),
    )
    .interaction(mouse::Interaction::Pointer);
    let size = H7;

    let radio = table::column(empty(), |sub: &Subtitle| {
        radio("", sub.id, selected, on_select.clone())
            .size(12)
            .spacing(0)
    })
    .align_y(Vertical::Center);

    let title = table::column(empty(), |sub: &Subtitle| {
        button(sized_regular(&sub.title, size))
            .on_press(on_select(sub.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let lang = table::column(empty(), |sub: &Subtitle| {
        button(sized_regular(&sub.lang, size))
            .on_press(on_select(sub.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let kind = table::column(empty(), |sub: &Subtitle| {
        let kind = match &sub.kind {
            registry::models::SubtitleKind::Embedded => "embedded".to_owned(),
            registry::models::SubtitleKind::Loaded { path, .. } => trim_path(path, 3),
        };

        button(marquee(kind).size(size).width(225))
            .on_press(on_select(sub.id))
            .padding(0)
            .style(styles::button::text)
    })
    .align_y(Vertical::Center);

    let delete = table::column(empty(), |sub: &Subtitle| match &sub.kind {
        registry::models::SubtitleKind::Embedded => empty(),
        registry::models::SubtitleKind::Loaded { .. } => icons::text_button(icons::CANCEL)
            .padding(0)
            .on_press(on_delete(sub.id))
            .into(),
    })
    .align_y(Vertical::Center);

    let content = table([radio, title, lang, kind, delete], subs).separator(0);

    expandable(label, content)
        .width(Length::Fill)
        .expanded(true)
        .spacing(0)
        .into()
}

fn media_label<'a>(label: impl text::IntoFragment<'a>) -> text::Text<'a> {
    sized_medium(label, P)
}

fn media_name<'a, Message: 'a + Clone>(
    placeholder: &str,
    value: &str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let label = media_label("Name");

    let input = text_input(placeholder, value)
        .on_input(on_input)
        .font(regular_font())
        .style(styles::text_input::default)
        .padding(PADDING_5)
        .style(styles::text_input::default)
        .width(Length::Fill);

    column!(label, input).spacing(3).into()
}

fn media_overview<'a, Message: 'a + Clone>(
    content: &'a text_editor::Content,
    on_edit: impl Fn(text_editor::Action) -> Message + 'a,
) -> Element<'a, Message> {
    let label = media_label("Overview");

    let content = text_editor(content)
        .padding(PADDING_5)
        .height(175)
        .font(regular_font())
        .on_action(on_edit);

    column!(label, content).spacing(3).into()
}

fn media_rating<'a, Message: 'a + Clone>(
    rating: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let label = media_label("Rating: ");

    let content = {
        let extra = sized_regular("/5", H7);

        let value = text_input("", rating)
            .font(regular_font())
            .padding(PADDING_5)
            .align_x(Horizontal::Right)
            .width(80.0)
            .style(styles::text_input::default)
            .on_input(on_input);

        row!(value, extra).spacing(2.0).align_y(Vertical::Center)
    };

    row!(label, space::horizontal(), content)
        .align_y(Vertical::Center)
        .into()
}

fn media_mark<'a, Message: 'a + Clone>(
    watched: bool,
    on_toggle: impl Fn(bool) -> Message + 'a,
) -> Element<'a, Message> {
    let label = media_label("Mark as Watched");

    let content = checkbox(watched).size(P).on_toggle(on_toggle);

    row!(label, space::horizontal(), content)
        .align_y(Vertical::Center)
        .into()
}

fn media_source<'a, Message: 'a + Clone>(
    source: &'a SourceSet,
    on_select: Option<impl Fn(SourceSet) -> Message + 'a>,
) -> Element<'a, Message> {
    let size = H7;
    let label = media_label("Source:");
    let handle = picklist_handle(size);

    let source: Element<'_, Message> = match on_select {
        Some(on_select) => pick_list(Some(source), SourceSet::VARIANTS, |source| {
            source.to_str().to_owned()
        })
        .style(styles::pick_list::default)
        .font(regular_font())
        .on_select(on_select)
        .handle(handle)
        .padding([5, 10])
        .text_size(size)
        .into(),
        None => sized_medium(source.to_str(), H7).into(),
    };

    row!(label, space::horizontal(), source)
        .spacing(40.0)
        .align_y(Vertical::Center)
        .into()
}

fn media_source_id<'a, Message: 'a + Clone>(
    source: SourceSet,
    id: &'a str,
    on_input: impl Fn(String) -> Message + 'a,
) -> Element<'a, Message> {
    let source_id = match source {
        SourceSet::Tmdb => {
            let label = media_label("Source Id (optional):");

            let input = text_input("", id)
                .on_input(on_input)
                .align_x(Horizontal::Right)
                .style(styles::text_input::default)
                .font(regular_font())
                .size(P)
                .padding(PADDING_5)
                .width(80.0);

            Some(row!(label, space::horizontal(), input).align_y(Vertical::Center))
        }
        SourceSet::None => None,
    };

    source_id.into()
}

fn media_icon_btn<'a, Message: 'a + Clone>(
    codepoint: char,
    label: &'a str,
) -> button::Button<'a, Message> {
    let size = H7;

    let refetch = row!(icon(codepoint).size(size), sized_medium(label, size))
        .spacing(4.0)
        .align_y(Vertical::Center);

    button(refetch).style(styles::button::subtle)
}

fn media_image<'a, Message: 'a + Clone>(
    label: &'a str,
    path: Option<&'a std::path::Path>,
    on_press: Message,
) -> Element<'a, Message> {
    let size = H7;
    let label = media_label(label);

    let action: Element<'_, Message> = match path {
        Some(path) => {
            let path = trim_path(path, 3);
            let path = marquee(path)
                .size(size)
                .width(250)
                .font(mono_font())
                .direction(true);

            let redo = text_button(REFRESH).on_press(on_press);

            row!(path, redo)
                .spacing(3.0)
                .align_y(Vertical::Center)
                .into()
        }
        None => {
            let upload = row!(icon(UPLOAD).size(size), sized_medium("Upload", size))
                .spacing(4.0)
                .align_y(Vertical::Center);

            button(upload)
                .style(styles::button::subtle)
                .on_press(on_press)
                .into()
        }
    };

    row!(label, space::horizontal(), action)
        .align_y(Vertical::Center)
        .into()
}

fn media_layout<'a>(
    content: impl Into<Element<'a, HomeMessage>>,
    on_save: HomeMessage,
) -> Element<'a, HomeMessage> {
    let actions = {
        let save = save_btn().on_press(on_save);

        let cancel = cancel_btn().on_press(HomeMessage::CloseView);

        column!(row!(save, cancel).spacing(80))
            .align_x(Horizontal::Center)
            .width(Length::Fill)
    };

    let content = scrollable(content).spacing(4).height(Length::Fill);

    let content = column!(content, actions).spacing(28);

    modal_container(content)
        .width(500)
        .padding([8, 12])
        .align_y(Vertical::Top)
        .into()
}
