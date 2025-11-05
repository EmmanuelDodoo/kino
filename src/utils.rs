use glib::object::{Cast, ObjectExt};
use gstreamer::{
    self as gst,
    prelude::{ElementExt, ElementExtManual, GstBinExt, GstBinExtManual, PadExt},
};
use iced::widget::image;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::error::*;
use crate::models::ItemId;
use crate::models::{EpisodeId, MovieId};

pub mod icons;
pub use icons::*;
pub mod typo;
pub use typo::*;
pub mod filter;
pub use filter::{Comments, Comp, Duration, Filter, FilterMode, Progress, ProgressKind, Release};
pub mod sort;
pub use sort::{Sort, SortKind};

/// Returns an empty [`iced::Element`].
pub fn empty<'a, Message: 'a>() -> iced::Element<'a, Message> {
    iced::widget::Space::new().width(0).height(0).into()
}

pub fn loading_svg() -> iced::widget::Svg<'static> {
    use iced::widget::svg::{Style, Svg};

    let handle = &*LOADING_SVG_HANDLE;

    Svg::new(handle.clone())
        .width(50)
        .height(50)
        .style(|theme: &iced::Theme, _| {
            let color = theme.extended_palette().background.base.text;

            Style { color: Some(color) }
        })
}

/// Returns a single thumbnail frame handle in rgba format.
pub fn _generate_thumbnail(
    path: impl AsRef<Path>,
    width: u32,
    height: u32,
    downscale: u32,
    save: bool,
) -> Result<image::Handle> {
    gst::init().map_err(GStreamerError::Glib)?;

    let src = gst::ElementFactory::make("filesrc")
        .name("file-source")
        .build()
        .map_err(GStreamerError::BoolError)?;
    src.set_property("location", path.as_ref());
    let decodebin = gst::ElementFactory::make("decodebin")
        .name("decoder")
        .build()
        .map_err(GStreamerError::BoolError)?;
    let sink = gst::ElementFactory::make("appsink")
        .build()
        .map_err(GStreamerError::BoolError)?;
    let sink_ref = sink.clone();

    let pipeline = gst::Pipeline::with_name("Thumbnail");
    pipeline
        .add_many([&src, &decodebin])
        .map_err(GStreamerError::BoolError)?;

    src.link(&decodebin).map_err(GStreamerError::BoolError)?;

    let pipeline_weak = pipeline.downgrade();
    decodebin.connect_pad_added(move |_dbin, src_pad| {
        if src_pad.query_caps(None).to_string().contains("video/") {
            let pipeline = match pipeline_weak.upgrade() {
                Some(p) => p,
                None => return,
            };
            let convert = gst::ElementFactory::make("videoconvert").build().unwrap();
            let scale = gst::ElementFactory::make("videoscale").build().unwrap();
            let jpegenc = gst::ElementFactory::make("jpegenc").build().unwrap();
            jpegenc.set_property("quality", 50);

            pipeline
                .add_many([&convert, &scale, &jpegenc, &sink])
                .unwrap();
            gst::Element::link_many([&convert, &scale, &jpegenc, &sink]).unwrap();

            for e in [&convert, &scale, &jpegenc, &sink] {
                e.sync_state_with_parent().unwrap();
            }

            let sink_pad = convert.static_pad("sink").unwrap();
            src_pad.link(&sink_pad).unwrap();
        }
    });

    // Start pipeline in paused state (so we can seek without playing)
    pipeline
        .set_state(gst::State::Paused)
        .map_err(GStreamerError::StateChangeError)?;

    // Wait until preroll (pipeline ready to process)
    let (res, _, _) = pipeline.state(gst::ClockTime::NONE);
    if let Err(err) = res {
        eprintln!("{err:?}");
    }

    let duration = pipeline
        .query_duration::<gst::ClockTime>()
        .ok_or(Error::ThumbnailEmptyVideo)?;
    let position = (duration * 10) / 100;

    // Seek to 10% into the video
    pipeline
        .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, position)
        .map_err(GStreamerError::BoolError)?;

    // Pull one frame from appsink
    let sink = sink_ref.dynamic_cast::<gstreamer_app::AppSink>().unwrap();
    let sample = sink
        .try_pull_preroll(gst::ClockTime::from_mseconds(250))
        .expect("Couldn't pull sample");
    let buffer = sample.buffer().expect("Could get sample buffer");
    let frame = buffer.map_readable().map_err(GStreamerError::BoolError)?;

    if save {
        let mut thumbnail_path = PathBuf::from("assets").join(".thumbnails");
        let thumbnail_stem = path
            .as_ref()
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("untitled");

        thumbnail_path.push(thumbnail_stem);
        thumbnail_path.set_extension("jpeg");

        let mut file = File::create(thumbnail_path)?;
        file.write_all(&frame)?;
    }

    pipeline
        .set_state(gst::State::Null)
        .map_err(GStreamerError::StateChangeError)?;

    Ok(image::Handle::from_rgba(
        width / downscale,
        height / downscale,
        yuv_to_rgba(frame.as_slice(), width as _, height as _, downscale),
    ))
}

/// Far faster at generating multiple thumbnails than
/// [`iced_video_player::Video::thumbnails`].
///
pub struct ThumbnailGenerator {
    pipeline: gst::Pipeline,
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
            eprintln!("Error droping ThumbnailGenerator: \n{err}");
        }
    }
}

impl ThumbnailGenerator {
    pub fn new(path: impl AsRef<Path>, width: i32, height: i32, downscale: u32) -> Self {
        gst::init().map_err(GStreamerError::Glib).unwrap();

        let template = format!(
            "filesrc location=\"{}\" ! decodebin ! videoconvert ! videoscale ! appsink name=sink drop=true caps=video/x-raw,format=NV12,pixel-aspect-ratio=1/1",
            path.as_ref().to_str().unwrap_or_default()
        );
        let pipeline = gst::parse::launch(template.as_ref())
            .unwrap()
            .downcast::<gst::Pipeline>()
            .unwrap();
        // .map_err(|_| Error::Cast)?;
        let sink = pipeline.by_name("sink").expect("Missing appsink");
        let sink = sink.downcast::<gstreamer_app::AppSink>().unwrap();

        // let src = gst::ElementFactory::make("filesrc")
        //     .name("file-source")
        //     .build()
        //     .map_err(GStreamerError::BoolError)
        //     .unwrap();
        // src.set_property("location", path.as_ref());
        // let decodebin = gst::ElementFactory::make("decodebin")
        //     .name("decoder")
        //     .build()
        //     .map_err(GStreamerError::BoolError)
        //     .unwrap();
        // let sink = gst::ElementFactory::make("appsink")
        //     .build()
        //     .map_err(GStreamerError::BoolError)
        //     .unwrap();
        // let caps = gst::Caps::builder("video/x-raw")
        //     .field("format", &"NV12")
        //     .field("pixel-aspect-ratio", &gst::Fraction::new(1, 1))
        //     .build();
        // sink.set_property("caps", &caps);
        // let sink_ref = sink.clone();
        //
        // let pipeline = gst::Pipeline::with_name("Thumbnail");
        // pipeline
        //     .add_many(&[&src, &decodebin])
        //     .map_err(GStreamerError::BoolError)
        //     .unwrap();
        //
        // src.link(&decodebin)
        //     .map_err(GStreamerError::BoolError)
        //     .unwrap();
        //
        // let pipeline_weak = pipeline.downgrade();
        // decodebin.connect_pad_added(move |_dbin, src_pad| {
        //     if src_pad.query_caps(None).to_string().contains("video/") {
        //         let pipeline = match pipeline_weak.upgrade() {
        //             Some(p) => p,
        //             None => return,
        //         };
        //         let convert = gst::ElementFactory::make("videoconvert").build().unwrap();
        //         let scale = gst::ElementFactory::make("videoscale").build().unwrap();
        //         // let jpegenc = gst::ElementFactory::make("jpegenc").build().unwrap();
        //         // jpegenc.set_property("quality", 50);
        //
        //         pipeline.add_many(&[&convert, &scale, &sink]).unwrap();
        //         gst::Element::link_many(&[&convert, &scale, &sink]).unwrap();
        //
        //         for e in [&convert, &scale, &sink] {
        //             e.sync_state_with_parent().unwrap();
        //         }
        //
        //         let sink_pad = convert.static_pad("sink").unwrap();
        //         src_pad.link(&sink_pad).unwrap();
        //     }
        // });

        pipeline
            .set_state(gst::State::Paused)
            .map_err(GStreamerError::StateChangeError)
            .unwrap();

        // Wait until preroll (pipeline ready to process)
        let (res, _, _) = pipeline.state(gst::ClockTime::NONE);
        if let Err(err) = res {
            eprintln!("{err:?}");
        }

        // let sink = sink_ref.dynamic_cast::<gstreamer_app::AppSink>().unwrap();
        let duration = pipeline
            .query_duration::<gst::ClockTime>()
            .ok_or(Error::ThumbnailEmptyVideo)
            .unwrap();

        Self {
            pipeline,
            sink,
            width,
            height,
            downscale,
            duration,
        }
    }

    pub fn generate(&self, position: gst::ClockTime) -> image::Handle {
        let width = self.width;
        let height = self.height;
        let downscale = self.downscale;

        self.pipeline
            .set_state(gst::State::Paused)
            .map_err(GStreamerError::StateChangeError)
            .unwrap();

        // Wait until preroll (pipeline ready to process)
        let (res, _, _) = self.pipeline.state(gst::ClockTime::NONE);
        if let Err(err) = res {
            eprintln!("{err:?}");
        }

        self.pipeline
            .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, position)
            .map_err(GStreamerError::BoolError)
            .unwrap();

        let sample = self
            .sink
            .pull_preroll()
            // .try_pull_preroll(gst::ClockTime::from_mseconds(250))
            // .expect("Couldn't pull sample");
            .unwrap();
        let buffer = sample.buffer().expect("Could get sample buffer");
        let frame = buffer
            .map_readable()
            .map_err(GStreamerError::BoolError)
            .unwrap();

        image::Handle::from_rgba(
            width as u32 / downscale,
            height as u32 / downscale,
            yuv_to_rgba(frame.as_slice(), width as _, height as _, downscale),
        )
    }
}

/// Credit to iced_video_player
fn yuv_to_rgba(yuv: &[u8], width: u32, height: u32, downscale: u32) -> Vec<u8> {
    let uv_start = width * height;
    let mut rgba = vec![];

    for y in 0..height / downscale {
        for x in 0..width / downscale {
            let x_src = x * downscale;
            let y_src = y * downscale;

            let uv_i = uv_start + width * (y_src / 2) + x_src / 2 * 2;

            let y = yuv[(y_src * width + x_src) as usize] as f32;
            let u = yuv[uv_i as usize] as f32;
            let v = yuv[(uv_i + 1) as usize] as f32;

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum PlayId {
    Movie(MovieId),
    Episode(EpisodeId),
}

impl From<PlayId> for ItemId {
    fn from(value: PlayId) -> Self {
        match value {
            PlayId::Movie(id) => ItemId::Movie(id),
            PlayId::Episode(id) => ItemId::Episode(id),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlayItem {
    pub id: PlayId,
    pub name: String,
    pub path: PathBuf,
}

#[derive(Debug, Clone)]
pub struct Playlist {
    current: usize,
    items: Vec<PlayItem>,
}

impl Playlist {
    pub fn new(items: impl Iterator<Item = PlayItem>) -> Self {
        Self {
            current: 0,
            items: items.collect(),
        }
    }

    #[allow(clippy::should_implement_trait)]
    pub fn next(&mut self) -> Option<&PlayItem> {
        self.current = (self.current + 1).min(self.items.len());

        self.current()
    }

    pub fn next_peek(&self) -> Option<&PlayItem> {
        self.items.get(self.current + 1)
    }

    pub fn current(&self) -> Option<&PlayItem> {
        self.items.get(self.current)
    }

    pub fn previous(&mut self) -> Option<&PlayItem> {
        self.current = self.current.saturating_sub(1);

        self.current()
    }

    pub fn restart(&mut self) {
        self.current = 0;
    }

    pub fn len(&self) -> usize {
        self.items.len()
    }

    pub fn is_empty(&self) -> bool {
        self.items.is_empty()
    }

    pub fn has_next(&self) -> bool {
        self.current < self.items.len() - 1
    }

    pub fn has_previous(&self) -> bool {
        self.current != 0
    }

    pub fn is_done(&self) -> bool {
        self.current >= self.items.len()
    }
}

#[derive(Debug, Clone)]
pub struct VideoSettings {
    pub thumbnail_interval: u32,
    pub volume: f64,
    pub speed: f64,
    pub gamma: f64,
    pub seek_mult: f64,
    pub seek_shift_mult: f64,
    pub volume_change_amt: f64,
    pub seek_change_amt: f64,
    pub speed_change_amt: f64,
    pub show_subtitles: bool,
    pub muted: bool,
    /// Whether a loaded video automatically starts playing
    pub autoplay: bool,
}

impl Default for VideoSettings {
    fn default() -> Self {
        Self {
            thumbnail_interval: 10,
            volume: 1.0,
            speed: 1.0,
            gamma: 1.5,
            seek_mult: 1.0,
            seek_shift_mult: 2.0,
            volume_change_amt: 0.05,
            seek_change_amt: 10.0,
            speed_change_amt: 0.1,
            show_subtitles: false,
            muted: false,
            autoplay: true,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub enum Layout {
    #[default]
    Grid,
    List,
}

impl Layout {
    pub fn icon(&self) -> char {
        match self {
            Self::Grid => icons::LIST,
            Self::List => icons::GRID,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum HomeAction {
    // todo
    // SettingsOpen,
    LayoutToggle,
    // Refresh,
    // SearchToggle,
}

#[derive(Debug, Clone, Copy)]
pub enum PlayerAction {
    PlayToggle,
    PlayNext,
    PlayPrevious,
    FullscreenToggle,
    FullscreenExit,
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
    Add,
    Favorite,
}

#[derive(Debug, Clone, Copy)]
pub enum Action {
    Back,
    Forward,
    Home(HomeAction),
    Player(PlayerAction),
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
