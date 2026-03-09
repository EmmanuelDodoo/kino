use core::error::{Error, GStreamerError};

use glib::object::Cast;
use gstreamer::{
    self as gst,
    prelude::{ElementExt, ElementExtManual, GstBinExt},
};

pub struct Image {
    pub width: u32,
    pub height: u32,
    pub bytes: Vec<u8>,
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

    pub fn generate(&self, seconds: f64) -> Image {
        let position = gstreamer::ClockTime::from_seconds_f64(seconds);
        let width = self.width;
        let height = self.height;
        let downscale = self.downscale;

        let sample = self.sample(position);

        let (frame, stride) = self.frame(&sample);

        Image {
            width: width as u32 / downscale,
            height: height as u32 / downscale,
            bytes: yuv_to_rgba(frame.as_slice(), width as _, height as _, downscale, stride),
        }
    }

    pub fn generate_with_poster(&self, seconds: f64) -> (Image, Image) {
        let position = gstreamer::ClockTime::from_seconds_f64(seconds);
        let width = self.width;
        let height = self.height;
        let downscale = self.downscale;

        let sample = self.sample(position);

        let (frame, stride) = self.frame(&sample);

        (
            Image {
                width: width as u32 / downscale,
                height: height as u32 / downscale,
                bytes: yuv_to_rgba(frame.as_slice(), width as _, height as _, downscale, stride),
            },
            Image {
                width: width as u32,
                height: height as u32,
                bytes: yuv_to_rgba(frame.as_slice(), width as _, height as _, 1, stride),
            },
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
