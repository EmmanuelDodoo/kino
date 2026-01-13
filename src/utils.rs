use glib::object::Cast;
use gstreamer::{
    self as gst,
    prelude::{ElementExt, ElementExtManual, GstBinExt},
};
use iced::animation::{Animation, Easing};
use iced::widget::image;
use std::path::{Path, PathBuf};

use crate::error::{Error, GStreamerError};
use crate::variants;

pub mod config;
pub use config::*;
pub mod playlist;
pub use playlist::*;
pub mod filter;
pub mod icons;
pub mod typo;
pub use filter::Filter;
pub mod sort;
pub use sort::{Sort, SortKind};
pub mod image_ops;
pub mod styles;
pub use image_ops::*;

/// Returns an empty [`iced::Element`].
pub fn empty<'a, Message: 'a>() -> iced::Element<'a, Message> {
    iced::widget::Space::new().width(0).height(0).into()
}

pub fn loading_svg(
    animation: &Animation<bool>,
    now: iced::time::Instant,
) -> iced::widget::Svg<'static> {
    use iced::widget::svg::{Style, Svg};
    use iced::{Radians, Rotation};

    let handle = &*icons::LOADING_SVG_HANDLE;
    let rotation = animation.interpolate(0.0, std::f32::consts::TAU, now);
    let rotation = Rotation::Floating(Radians(rotation));

    Svg::new(handle.clone())
        .width(50)
        .height(50)
        .style(|theme: &iced::Theme, _status| {
            let color = theme.extended_palette().background.base.text;

            Style { color: Some(color) }
        })
        .rotation(rotation)
}

pub fn loading_animation(now: iced::time::Instant) -> Animation<bool> {
    Animation::new(false)
        .easing(Easing::EaseInOut)
        .duration(iced::time::Duration::from_millis(1000))
        .repeat_forever()
        .go(true, now)
}

pub fn save_btn<'a, Message: 'a + Clone>() -> iced::widget::Button<'a, Message> {
    iced::widget::button(typo::medium("Save")).style(styles::button::primary)
}

pub fn cancel_btn<'a, Message: 'a + Clone>() -> iced::widget::Button<'a, Message> {
    iced::widget::button(typo::medium("Cancel")).style(styles::button::secondary)
}

pub fn tooltip<'a, Message: 'a>(
    content: impl Into<iced::Element<'a, Message>>,
    label: impl iced::widget::text::IntoFragment<'a>,
    position: iced::widget::tooltip::Position,
) -> iced::widget::Tooltip<'a, Message> {
    use iced::widget::{container, tooltip};
    use iced::{Shadow, Theme};

    tooltip(
        content,
        container(typo::sized_regular(label, typo::H8))
            .clip(true)
            .max_width(350)
            .max_height(100)
            .padding([3, 6])
            .style(|theme: &Theme| {
                let color = theme.extended_palette().secondary.weak.color;
                let default = styles::container::bw2(theme);
                let border = default.border.rounded(5.0).width(1.0).color(color);
                let shadow = Shadow {
                    color,
                    blur_radius: 8.0,
                    offset: [0.0, 0.0].into(),
                };

                container::Style {
                    border,
                    shadow,
                    ..default
                }
            }),
        position,
    )
    .delay(iced::time::Duration::from_millis(500))
    .gap(2.0)
}

pub fn modal_container<'a, Message: 'a>(
    content: impl Into<iced::Element<'a, Message>>,
) -> iced::widget::Container<'a, Message> {
    use iced::alignment::{Horizontal, Vertical};
    use iced::widget::container;

    container(content)
        .padding([8, 12])
        .style(|theme| {
            let default = styles::container::bw2(theme);
            let border = default.border.rounded(5.0);

            container::Style { border, ..default }
        })
        .clip(true)
        .align_y(Vertical::Center)
        .align_x(Horizontal::Center)
}

pub fn picklist_handle(size: f32) -> iced::widget::pick_list::Handle<iced::Font> {
    use iced::widget::{pick_list, text};

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

    pick_list::Handle::Dynamic {
        closed: down,
        open: up,
    }
}

pub fn trim_path(path: &Path, components: usize) -> String {
    let path = path
        .components()
        .rev()
        .take(components)
        .collect::<Vec<_>>()
        .into_iter()
        .rev()
        .collect::<PathBuf>();
    let path = path.display().to_string();
    path.strip_prefix(r"\\?\").unwrap_or(&path).to_owned()
}

pub fn draw_subtitles<'a, Message: 'a>(
    subtitles: &'a str,
    description: SubtitleDescription,
) -> iced::Element<'a, Message> {
    use iced::alignment::Vertical;
    use iced::widget::container;

    let SubtitleDescription {
        size,
        color,
        font,
        background_color,
    } = description;

    let content = typo::regular(subtitles)
        .size(size.max(5))
        .color(u32_to_rgba(color))
        .font(font)
        .align_y(Vertical::Center);

    let subtitles = container(content)
        .align_y(Vertical::Center)
        .padding([6, 6])
        .style(move |_| {
            let border = iced::border::rounded(5);
            container::Style {
                background: Some(u32_to_rgba(background_color).into()),
                text_color: None,
                border,
                ..Default::default()
            }
        });

    subtitles.into()
}

pub fn convert_color_str(input: &str) -> Option<u32> {
    if input.is_empty() {
        return None;
    }

    let input = input.trim();

    let color = if input.contains(",") {
        let values = input
            .trim_start_matches("rgb(")
            .trim_end_matches(")")
            .split(",")
            .enumerate()
            .filter_map(|(idx, value)| {
                if idx != 3 {
                    value.trim().parse::<u8>().ok()
                } else {
                    match value.trim().parse::<u8>().ok() {
                        Some(alpha) => Some(alpha),
                        None => {
                            let alpha = value.trim().parse::<f32>().ok()?;
                            let alpha = alpha.max(0.0);

                            Some((255.0 * alpha).trunc() as u8)
                        }
                    }
                }
            })
            .collect::<Vec<u8>>();

        if values.len() < 3 || values.len() > 4 {
            return None;
        }

        let r = *values.first()? as u32;
        let g = *values.get(1)? as u32;
        let b = *values.get(2)? as u32;
        let a = values.get(3).copied().unwrap_or(255) as u32;

        (r << 24) | (g << 16) | (b << 8) | a
    } else if input.contains("#") {
        u32::from_str_radix(input.trim_start_matches("#"), 16).ok()?
    } else {
        u32::from_str_radix(input.trim(), 16).ok()?
    };

    Some(color)
}

pub fn u32_to_rgba(color: u32) -> iced::Color {
    let r = (color & 0xff000000) >> 24;
    let g = (color & 0x00ff0000) >> 16;
    let b = (color & 0x0000ff00) >> 8;
    let a = color & 0xff;

    let a = (a as f32) / 255.0;

    iced::color!(r as u8, g as u8, b as u8, a)
}

/// Far faster at generating multiple thumbnails than
/// [`iced_video_player::Video::thumbnails`].
///
pub struct ThumbnailGenerator {
    pipeline: gst::Pipeline,
    bus: gst::Bus,
    width: i32,
    height: i32,
    downscale: u32,
    /// The duration of the video playback
    pub duration: gst::ClockTime,
    sink: gstreamer_app::AppSink,
}

impl Drop for ThumbnailGenerator {
    fn drop(&mut self) {
        if let Err(err) = self.pipeline.set_state(gst::State::Null) {
            tracing::error!("Error droping ThumbnailGenerator: \n{err}");
        }
    }
}

impl ThumbnailGenerator {
    pub fn new(path: url::Url, width: i32, height: i32, downscale: u32) -> Self {
        gst::init().map_err(GStreamerError::Glib).unwrap();

        let template = format!(
            "urisourcebin uri=\"{}\" ! decodebin ! videoconvert ! videoscale ! appsink name=sink drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1",
            path.as_str()
        );
        let pipeline = gst::parse::launch(template.as_ref())
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();

        let sink = pipeline.by_name("sink").expect("Missing appsink");
        let sink = sink.downcast::<gstreamer_app::AppSink>().unwrap();

        pipeline
            .set_state(gst::State::Paused)
            .map_err(GStreamerError::StateChangeError)
            .unwrap();

        // Wait until preroll (pipeline ready to process)
        let (res, _, _) = pipeline.state(gst::ClockTime::NONE);
        if let Err(err) = res {
            tracing::error!("{err:?}");
        }

        let duration = pipeline
            .query_duration::<gst::ClockTime>()
            .ok_or(Error::ThumbnailEmptyVideo)
            .unwrap();

        Self {
            bus: pipeline.bus().unwrap(),
            pipeline,
            sink,
            width,
            height,
            downscale,
            duration,
        }
    }

    fn sample(&self, position: gst::ClockTime) -> gstreamer::Sample {
        self.pipeline
            .set_state(gst::State::Paused)
            .map_err(GStreamerError::StateChangeError)
            .unwrap();

        // Wait until preroll (pipeline ready to process)
        let (res, _, _) = self.pipeline.state(gst::ClockTime::NONE);
        if let Err(err) = res {
            tracing::error!("{err:?}");
        }

        self.pipeline
            .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, position)
            .map_err(GStreamerError::BoolError)
            .unwrap();

        

        self.sink.pull_preroll().unwrap()
    }

    fn frame<'a>(
        &self,
        sample: &'a gstreamer::Sample,
    ) -> (
        gstreamer::BufferMap<'a, gstreamer::buffer::Readable>,
        Option<u32>,
    ) {
        let stride = sample.buffer().and_then(|buffer| {
            buffer
                .meta::<gstreamer_video::VideoMeta>()
                .map(|meta| meta.stride()[0] as u32)
        });

        let buffer = sample.buffer().expect("Could get sample buffer");
        let frame = buffer
            .map_readable()
            .map_err(GStreamerError::BoolError)
            .unwrap();

        while let Some(msg) = self.bus.pop() {
            if let gst::MessageView::Error(error) = msg.view() {
                tracing::error!("{error:?}")
            }
        }

        (frame, stride)
    }

    pub fn generate(&self, position: gst::ClockTime) -> image::Handle {
        let width = self.width;
        let height = self.height;
        let downscale = self.downscale;

        let sample = self.sample(position);

        let (frame, stride) = self.frame(&sample);

        image::Handle::from_rgba(
            width as u32 / downscale,
            height as u32 / downscale,
            yuv_to_rgba(frame.as_slice(), width as _, height as _, downscale, stride),
        )
    }

    pub fn generate_with_poster(&self, position: gst::ClockTime) -> (image::Handle, image::Handle) {
        let width = self.width;
        let height = self.height;
        let downscale = self.downscale;

        let sample = self.sample(position);

        let (frame, stride) = self.frame(&sample);

        (
            image::Handle::from_rgba(
                width as u32 / downscale,
                height as u32 / downscale,
                yuv_to_rgba(frame.as_slice(), width as _, height as _, downscale, stride),
            ),
            image::Handle::from_rgba(
                width as u32,
                height as u32,
                yuv_to_rgba(frame.as_slice(), width as _, height as _, 1, stride),
            ),
        )
    }
}

/// Credit to iced_video_player
fn yuv_to_rgba(
    yuv: &[u8],
    width: u32,
    height: u32,
    downscale: u32,
    stride: Option<u32>,
) -> Vec<u8> {
    let stride = stride.unwrap_or(width);

    let uv_start = stride * height;
    let mut rgba = vec![];

    for y in 0..height / downscale {
        for x in 0..width / downscale {
            let x_src = x * downscale;
            let y_src = y * downscale;

            // NV12 memory layout with stride:
            // Y plane: stride bytes per row, starting at offset 0
            // UV plane: stride bytes per row (same stride), starting at offset stride * height
            // Each pixel is 1 byte Y, and every 2x2 block shares 2 bytes (U, V)
            let y_offset = (y_src * stride + x_src) as usize;
            let uv_offset = (uv_start + (y_src / 2) * stride + (x_src / 2) * 2) as usize;

            let y = yuv[y_offset] as f32;
            let u = yuv[uv_offset] as f32;
            let v = yuv[uv_offset + 1] as f32;

            let r = 1.164 * (y - 16.0) + 1.596 * (v - 128.0);
            let g = 1.164 * (y - 16.0) - 0.813 * (v - 128.0) - 0.391 * (u - 128.0);
            let b = 1.164 * (y - 16.0) + 2.018 * (u - 128.0);

            rgba.push(r as u8);
            rgba.push(g as u8);
            rgba.push(b as u8);
            rgba.push(0xFF);
        }
    }

    rgba
}

pub fn rand_u32() -> u32 {
    use std::time::{SystemTime, UNIX_EPOCH};

    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .subsec_millis()
}

#[derive(Debug, Clone)]
pub struct Scroll {
    pub id: iced::widget::Id,
    pub offset: iced::widget::operation::AbsoluteOffset,
}

impl Scroll {
    pub fn new() -> Self {
        Self {
            id: iced::widget::Id::unique(),
            offset: iced::widget::operation::AbsoluteOffset::default(),
        }
    }
}

impl Default for Scroll {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Clone, Debug, Copy)]
pub enum Screen {
    Home,
    Player,
    Settings,
    // Log,
}

variants! {
#[derive(Debug, Clone, Copy, Default, PartialEq, serde::Serialize, serde::Deserialize)]
    pub enum Layout {
        #[default]
        Grid,
        List,
        Compact,
    }

}

impl Layout {
    pub fn icon(&self) -> char {
        match self {
            Self::Grid => icons::GRID,
            Self::List => icons::LIST,
            Self::Compact => icons::COMPACT_LIST,
        }
    }
}

impl std::fmt::Display for Layout {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Grid => "Grid",
                Self::List => "List",
                Self::Compact => "Compact list",
            }
        )
    }
}

variants! {
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
    pub enum HomeAction {
        SettingsOpen,
        CloseModal,
        LayoutToggle,
        RefreshContent,
        /// Refreshes both content and the side menu
        #[serde(rename = "RefreshAll")]
        Refresh,
        SearchToggle,
        Back,
        Forward,
        SelectionStart,
    }
}

impl HomeAction {
    pub fn descr(&self) -> &str {
        match self {
            Self::SettingsOpen => "Opens the settings screen",
            Self::CloseModal => "Closes the current modal",
            Self::LayoutToggle => "Toggles the content layout",
            Self::RefreshContent => "Refreshes only the current content",
            Self::Refresh => "Refreshes both the current content and collections",
            Self::SearchToggle => "Opens the search dialog",
            Self::Back => "Navigates back to the previous page",
            Self::Forward => "Navigates forward to the next page",
            Self::SelectionStart => "Enters selection mode, adding clicked media to a new playlist",
        }
    }
}

impl std::fmt::Display for HomeAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Back => "Back",
                Self::Forward => "Forward",
                Self::SearchToggle => "Search Toggle",
                Self::Refresh => "Refresh",
                Self::RefreshContent => "Refresh Content",
                Self::LayoutToggle => "Layout Toggle",
                Self::CloseModal => "Close Modal",
                Self::SettingsOpen => "Settings Open",
                Self::SelectionStart => "Selection Start",
            }
        )
    }
}

variants! {
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
    pub enum PlayerAction {
        PlayToggle,
        PlayNext,
        PlayPrevious,
        FullscreenToggle,
        /// Used for either exiting fullscreen or closing modal windows
        Exit,
        SeekBack,
        SeekBackShift,
        SeekFront,
        SeekFrontShift,
        VolumeIncrease,
        VolumeDecrease,
        MuteToggle,
        SpeedIncrease,
        SpeedDecrease,
        SpeedReset,
        VideoConfig,
        VideoComment,
        SubtitlesToggle,
        #[serde(rename = "CollectionAdd")]
        Add,
        CloseView,
        Back,
        PlaylistToggle,
    }

}

impl PlayerAction {
    pub fn descr(&self) -> &str {
        match self {
            Self::PlayToggle => "Toggles Play/Pause",
            Self::PlayNext => "Plays the next video",
            Self::PlayPrevious => "Plays the previous video",
            Self::FullscreenToggle => "Toggles full screen mode",
            Self::Exit => "Exits either full screen mode or closes the current modal",
            Self::SeekBack => "Rewind video",
            Self::SeekBackShift => "Rewind video",
            Self::SeekFront => "Fast forward video",
            Self::SeekFrontShift => "Fast forward video",
            Self::VolumeIncrease => "Increases the volume",
            Self::VolumeDecrease => "Decreases the volume",
            Self::MuteToggle => "Toggles mute",
            Self::SpeedIncrease => "Increase playback rate",
            Self::SpeedDecrease => "Decrease playback rate",
            Self::SpeedReset => "Reset playback rate",
            Self::VideoConfig => "Opens the video configuration menu",
            Self::VideoComment => "Opens the video comment dialog",
            Self::SubtitlesToggle => "Toggles video subtitles",
            Self::Add => "Opens the add to collection dialog",
            Self::CloseView => "Closes the current modal",
            Self::Back => "Exits the video player",
            Self::PlaylistToggle => "Toggles the playlist view",
        }
    }
}

impl std::fmt::Display for PlayerAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::PlayToggle => "Play Toggle",
                Self::PlayNext => "Play Next",
                Self::PlayPrevious => "Play Previous",
                Self::FullscreenToggle => "Fullscreen Toggle",
                Self::Exit => "Exit",
                Self::SeekBack => "Seek Back",
                Self::SeekBackShift => "Seek Back Shift",
                Self::SeekFront => "Seek Front",
                Self::SeekFrontShift => "Seek Front Shift",
                Self::VolumeIncrease => "Volume Increase",
                Self::VolumeDecrease => "Volume Decrease",
                Self::MuteToggle => "Mute Toggle",
                Self::SpeedIncrease => "Speed Increase",
                Self::SpeedDecrease => "Speed Decrease",
                Self::SpeedReset => "Speed Reset",
                Self::VideoConfig => "Video Config",
                Self::VideoComment => "Video Comment",
                Self::SubtitlesToggle => "Subtitles Toggle",
                Self::Add => "Add",
                Self::CloseView => "Close View",
                Self::Back => "Back",
                Self::PlaylistToggle => "Playlist Toggle",
            }
        )
    }
}

variants! {

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, serde::Deserialize, serde::Serialize)]
    pub enum SettingsAction {
        Up,
        Down,
        Cancel,
    }
}

impl SettingsAction {
    pub fn descr(&self) -> &str {
        match self {
            Self::Cancel => "Discards changes and exits the settings screen",
            Self::Up => "Navigates to the next sub-menu",
            Self::Down => "Navigates to the previous sub-menu",
        }
    }
}

impl std::fmt::Display for SettingsAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}",
            match self {
                Self::Cancel => "Cancel",
                Self::Down => "Down",
                Self::Up => "Up",
            }
        )
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Home(HomeAction),
    Player(PlayerAction),
    Settings(SettingsAction),
}

impl Action {
    pub fn descr(&self) -> &str {
        match self {
            Self::Home(action) => action.descr(),
            Self::Player(action) => action.descr(),
            Self::Settings(action) => action.descr(),
        }
    }
}

impl From<PlayerAction> for Action {
    fn from(value: PlayerAction) -> Self {
        Self::Player(value)
    }
}

impl From<HomeAction> for Action {
    fn from(value: HomeAction) -> Self {
        Self::Home(value)
    }
}

impl From<SettingsAction> for Action {
    fn from(value: SettingsAction) -> Self {
        Self::Settings(value)
    }
}

#[macro_export]
macro_rules! variants {
	(
		$(#[$meta:meta])*
		$vis:vis enum $name:ident {
			$(
				$(#[$field_meta:meta])*
				$variant:ident,
			)+
		}
	) => {
		$(#[$meta])*
		$vis enum $name {
			$(
				$(#[$field_meta])*
				$variant,
			)+
		}

		impl $name {
			pub const VARIANTS: &[Self] = &[$(Self::$variant,)+];
                        pub const NAMES: &[&str] = &[$(stringify!($variant)),+];
		}
	};
}
