use crate::theme::{self, Theme};
use crate::utils::{cancel_btn, icons, save_btn, typo};
use fancy_regex::{Captures, Regex};
use iced::{
    Length, Task,
    alignment::{Horizontal, Vertical},
    animation::Animation,
    time::{Instant, milliseconds},
    widget::{
        self, button, center_x, column, container, hover, markdown, operation, rich_text, row,
        scrollable, sensor, text, text_editor,
    },
};
use registry::models::{self, CommentId, VideoId};
use std::collections::HashMap;
use std::sync::LazyLock;

use crate::Element;

static REGEX: LazyLock<Option<Regex>> = LazyLock::new(|| {
    Regex::new(r"@((?<hrs>\d+):)?((?<mins>[0-9]|[0-5][0-9]):)((?<secs>[0-5][0-9]))").ok()
});

static NOW: LazyLock<Option<Regex>> = LazyLock::new(|| Regex::new(r"@now").ok());

pub trait CommentMessage {
    fn link(url: String) -> Self;

    fn image_shown(id: CommentId, timestamp: Option<u64>, url: String) -> Self;

    fn edit_action(id: CommentId, timestamp: Option<u64>, action: text_editor::Action) -> Self;

    fn save(id: CommentId, timestamp: Option<u64>) -> Self;

    fn cancel(id: CommentId, timestamp: Option<u64>) -> Self;

    fn delete(id: CommentId, timestamp: Option<u64>) -> Self;

    fn edit(id: CommentId, timestamp: Option<u64>) -> Self;
}

#[derive(Debug)]
enum Mode {
    View(Box<markdown::Content>),
    Edit(text_editor::Content),
}

pub struct Comment {
    pub inner: models::Comment,

    text_editor: widget::Id,
    mode: Mode,
    pub images: HashMap<String, Image>,
}

impl std::fmt::Debug for Comment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Comment")
            .field("inner", &self.inner)
            .finish()
    }
}

impl Comment {
    fn prep(
        raw: String,
        video: &str,
        now: Option<u64>,
        timestamp: &mut Option<u64>,
    ) -> (String, markdown::Content) {
        let raw = match now {
            Some(now) => {
                let now = {
                    let hours = now / 3600;

                    let mins = (now % 3600) / 60;

                    let secs = (now % 3600) % 60;

                    if hours > 0 {
                        format!("@{hours:02}:{mins:02}:{secs:02}")
                    } else {
                        format!("@{mins:02}:{secs:02}")
                    }
                };

                match NOW.as_ref() {
                    Some(regex) => regex.replace_all(&raw, now).to_string(),
                    None => raw,
                }
            }
            None => raw,
        };

        let replacer = Replacer {
            video,
            first: timestamp,
        };

        let replaced = REGEX
            .as_ref()
            .map(|regex| regex.replace_all(&raw, replacer))
            .unwrap_or_default();
        let markdown = markdown::Content::parse(&replaced);

        (raw, markdown)
    }

    pub fn new(raw: String, kind: VideoId, now: u64, editor: widget::Id) -> Self {
        let mut timestamp = None;
        let video = kind.to_string();

        let (raw, markdown) = Self::prep(raw, &video, Some(now), &mut timestamp);

        let (inner, _) = models::Comment::new(raw, timestamp, kind);

        Self {
            inner,
            text_editor: editor,
            mode: Mode::View(Box::new(markdown)),
            images: HashMap::default(),
        }
    }

    pub fn load(mut inner: models::Comment, editor: Option<widget::Id>) -> Self {
        let video = inner.kind.to_string();

        let replacer = Replacer {
            video: &video,
            first: &mut inner.timestamp,
        };

        let replaced = REGEX
            .as_ref()
            .map(|regex| regex.replace_all(&inner.content, replacer))
            .unwrap_or_default();
        let markdown = markdown::Content::parse(&replaced);
        let mode = Mode::View(Box::new(markdown));
        let text_editor = editor.unwrap_or_else(widget::Id::unique);

        Self {
            inner,
            text_editor,
            mode,
            images: HashMap::default(),
        }
    }

    pub fn perform_action(&mut self, action: text_editor::Action) {
        if let Mode::Edit(editor) = &mut self.mode {
            editor.perform(action);
        }
    }

    pub fn save(&mut self, now: u64) -> Option<u64> {
        let Mode::Edit(editor) = &mut self.mode else {
            return self.inner.timestamp;
        };

        let text = editor.text();
        self.inner.timestamp.take();

        let video = self.inner.kind.to_string();
        let (raw, markdown) = Self::prep(text, &video, Some(now), &mut self.inner.timestamp);

        self.inner.content = raw;
        self.mode = Mode::View(Box::new(markdown));

        self.inner.timestamp
    }

    pub fn cancel(&mut self) {
        if matches!(self.mode, Mode::View(_)) {
            return;
        }

        let video = self.inner.kind.to_string();

        let replacer = Replacer {
            video: &video,
            first: &mut self.inner.timestamp,
        };

        let replaced = REGEX
            .as_ref()
            .map(|regex| regex.replace_all(&self.inner.content, replacer))
            .unwrap_or_default();
        let markdown = markdown::Content::parse(&replaced);

        self.mode = Mode::View(Box::new(markdown))
    }

    pub fn edit<Message>(&mut self) -> Task<Message> {
        if !matches!(&self.mode, Mode::View(_)) {
            return Task::none();
        }
        let mut editor = text_editor::Content::with_text(&self.inner.content);

        editor.perform(text_editor::Action::Move(text_editor::Motion::DocumentEnd));

        self.mode = Mode::Edit(editor);

        operation::focus(self.text_editor.clone())
    }

    pub fn view<'a, Message: 'a + CommentMessage + Clone>(
        &'a self,
        now: Instant,
        theme: &Theme,
    ) -> Element<'a, Message> {
        let id = self.inner.id;
        let timestamp = self.inner.timestamp;
        let padding = 6;
        let border_width = 1.5;
        let size = typo::H7;

        match &self.mode {
            Mode::Edit(editor) => {
                let editor = text_editor(editor)
                    .id(self.text_editor.clone())
                    .font(typo::regular_font())
                    .on_action(move |action| Message::edit_action(id, timestamp, action))
                    .wrapping(text::Wrapping::WordOrGlyph)
                    .key_binding(move |press| {
                        use iced::keyboard::{Key, key::Named};
                        use text_editor::Binding;

                        match press.key {
                            Key::Named(Named::Enter)
                                if press.modifiers.command() && press.is_focused =>
                            {
                                Some(Binding::Custom(Message::save(id, timestamp)))
                            }
                            _ => Binding::from_key_press(press),
                        }
                    })
                    .size(size)
                    .height(Length::Fit.max(250.0))
                    .padding(6)
                    .highlight("markdown", iced::highlighter::Theme::Base16Ocean);
                let cancel = cancel_btn().on_press(Message::cancel(id, timestamp));
                let save = save_btn().on_press(Message::save(id, timestamp));

                let btns = row!(save, cancel).spacing(40).align_y(Vertical::Center);

                let content = column!(editor, btns)
                    .align_x(Horizontal::Center)
                    .spacing(10);

                let content = container(content).padding(padding).style(move |theme| {
                    let default = theme::container::bw(theme);

                    container::Style { ..default }
                });

                content.into()
            }
            Mode::View(markdown) => {
                let settings = {
                    let base = markdown::Settings::with_text_size(size, theme);
                    markdown::Settings {
                        style: markdown::Style {
                            font: typo::regular_font(),
                            ..base.style
                        },
                        ..base
                    }
                };

                let content = markdown::view_with(
                    markdown.items(),
                    settings,
                    &CommentViewer {
                        id,
                        timestamp,
                        images: &self.images,
                        now,
                    },
                );

                let content = container(content)
                    .width(Length::Fill)
                    .height(Length::Fill)
                    .padding(padding);

                let content = container(content);

                let content = scrollable(content);

                let edit = button(icons::icon(icons::DELETE).size(typo::P))
                    .style(theme::button::text_danger)
                    .on_press(Message::delete(id, timestamp))
                    .padding(0);

                let edit = container(edit)
                    .width(Length::Fill)
                    .padding(iced::Padding::new(2.0).right(6))
                    .align_x(Horizontal::Right);

                let content = hover(content, edit);

                let content = button(content)
                    .padding(iced::Padding::ZERO.vertical(6).right(4))
                    .on_press(Message::edit(id, timestamp))
                    .style(move |theme: &Theme, status| {
                        let base = theme::button::subtle_inv(theme, status);
                        let border = base.border.width(border_width);
                        let border = match status {
                            button::Status::Hovered => {
                                border.color(theme.schema().neutral.weak.color)
                            }
                            _ => border,
                        };

                        button::Style {
                            border,
                            background: None,
                            ..base
                        }
                    });

                content.into()
            }
        }
    }

    pub fn is_animating(&self, now: Instant) -> bool {
        self.images.values().any(|image| match image {
            Image::Ready { fade_in, .. } => fade_in.is_animating(now),
            _ => false,
        })
    }
}

pub struct Replacer<'r> {
    pub video: &'r str,
    pub first: &'r mut Option<u64>,
}

impl<'r> Replacer<'r> {
    pub fn replacer<'cap>(&mut self, captures: &Captures<'cap>) -> String {
        let hrs = captures.get(1);

        let mins = captures.get(3);
        let secs = captures.get(5);

        let original = format!(
            "{}{}{}",
            hrs.map(|mat| mat.as_str()).unwrap_or_default(),
            mins.map(|mat| mat.as_str()).unwrap_or_default(),
            secs.map(|mat| mat.as_str()).unwrap_or_default()
        );

        let hrs = captures
            .name("hrs")
            .and_then(|mat| mat.as_str().parse::<u64>().ok())
            .unwrap_or_default();

        let mins = captures
            .name("mins")
            .and_then(|mat| mat.as_str().parse::<u64>().ok())
            .unwrap_or_default();

        let secs = captures
            .name("secs")
            .and_then(|mat| mat.as_str().parse::<u64>().ok())
            .unwrap_or_default();

        let time = (hrs * 3600) + (mins * 60) + secs;

        if self.first.is_none() {
            *self.first = Some(time)
        }

        format!("[{original}](video://{}/{time})", self.video)
    }
}

impl<'r> fancy_regex::Replacer for Replacer<'r> {
    fn replace_append(&mut self, caps: &Captures<'_>, dst: &mut String) {
        dst.push_str(self.replacer(caps).as_ref());
    }
}

#[derive(Debug)]
pub enum Image {
    Loading,
    Ready {
        handle: widget::image::Handle,
        fade_in: Animation<bool>,
    },
    Errored(String),
}

pub struct CommentViewer<'a> {
    pub id: CommentId,
    pub timestamp: Option<u64>,
    pub images: &'a HashMap<String, Image>,
    pub now: Instant,
}

impl<'a, Message: 'a + CommentMessage> markdown::Viewer<'a, Message, Theme> for CommentViewer<'a> {
    fn on_link_click(url: String) -> Message {
        Message::link(url)
    }

    fn image(
        &self,
        settings: markdown::Settings,
        url: &'a markdown::Uri,
        title: &'a str,
        alt: &markdown::Text,
    ) -> Element<'a, Message> {
        use iced::widget::markdown::Catalog;
        let id = self.id;
        let timestamp = self.timestamp;

        match self.images.get(url) {
            Some(Image::Ready { handle, fade_in }) => center_x(
                widget::image(handle)
                    .opacity(fade_in.interpolate(0.0, 1.0, self.now))
                    .scale(fade_in.interpolate(1.2, 1.0, self.now)),
            )
            .height(Length::Fit.max(200))
            .into(),
            Some(Image::Errored(error)) => {
                let msg = format!("{title} image download errored. {error}");

                typo::sized_regular(msg, typo::H7)
                    .style(theme::text::danger)
                    .into()
            }
            None | Some(Image::Loading) => sensor(
                container(rich_text(alt.spans(settings.style)).on_link_click(Self::on_link_click))
                    .padding(settings.spacing.0)
                    .class(Theme::code_block()),
            )
            .key_ref(url.as_str())
            .delay(milliseconds(500))
            .on_show(move |_size| Message::image_shown(id, timestamp, url.clone()))
            .into(),
        }
    }
}

pub async fn download_image(uri: markdown::Uri) -> Result<widget::image::Handle, String> {
    use std::io;
    use tokio::task;
    use url::Url;

    let bytes = match Url::parse(&uri) {
        Ok(url) if url.scheme() == "http" || url.scheme() == "https" => {
            tracing::debug!("Trying to download image: {url}");

            let client = reqwest::Client::new();

            client
                .get(url)
                .send()
                .await
                .map_err(|error| error.to_string())?
                .error_for_status()
                .map_err(|error| error.to_string())?
                .bytes()
                .await
                .map_err(|error| error.to_string())?
        }
        _ => return Err("unsupported uri: {uri}".to_owned()),
    };

    let image = task::spawn_blocking(move || {
        image::ImageReader::new(io::Cursor::new(bytes))
            .with_guessed_format()
            .map_err(|error| error.to_string())
            .and_then(|img| img.decode().map_err(|error| error.to_string()))
            .map(|img| img.to_rgba8())
    })
    .await
    .map_err(|error| error.to_string())?
    .map_err(|error| error.to_string())?;

    Ok(widget::image::Handle::from_rgba(
        image.width(),
        image.height(),
        image.into_raw(),
    ))
}
