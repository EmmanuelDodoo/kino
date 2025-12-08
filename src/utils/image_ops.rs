use image::{
    DynamicImage, GenericImage, ImageBuffer, ImageReader, Rgba,
    imageops::{self, FilterType},
};

use iced::Color;
use iced::widget::image::Handle;

const DEFAULT_POSTER: &[u8] = include_bytes!("default.png");

fn open(path: &str) -> Option<DynamicImage> {
    ImageReader::open(path)
        .and_then(|reader| reader.with_guessed_format())
        .inspect_err(|error| tracing::error!("Image opening error on {path}. Error\n{error}"))
        .ok()
        .and_then(|reader| {
            reader
                .decode()
                .inspect_err(|error| {
                    tracing::error!("Image opening error on {path}. Error\n{error}")
                })
                .ok()
        })
}

pub fn default_poster() -> Option<Handle> {
    use std::io::Cursor;

    let default = Cursor::new(DEFAULT_POSTER);

    let img = ImageReader::new(default)
        .with_guessed_format()
        .inspect_err(|error| tracing::error!("Default poster opening error. Error\n{error}"))
        .ok()
        .and_then(|reader| {
            reader
                .decode()
                .inspect_err(|error| {
                    tracing::error!("Default poster opening error. Error\n{error}")
                })
                .ok()
        })?;

    let img = img.to_rgba8();

    Some(Handle::from_rgba(
        img.width(),
        img.height(),
        bytes::Bytes::from(img.into_raw()),
    ))
}

pub fn collage<'a>(
    paths: impl Iterator<Item = &'a str>,
    width: u32,
    height: u32,
) -> Option<Handle> {
    let imgs: Vec<DynamicImage> = paths.filter_map(open).take(3).collect();

    if imgs.is_empty() {
        return None;
    }

    let len = imgs.len();
    let mut canvas: ImageBuffer<Rgba<u8>, Vec<_>> = ImageBuffer::new(width, height);

    let mut flip = true;
    let mut img_width = 0;
    let mut img_height = 0;

    for (i, img) in imgs.into_iter().enumerate() {
        let remaining_height = height.saturating_sub(img_height);
        let remaining_width = width.saturating_sub(img_width);
        let last = i == len - 1;

        if flip {
            let width = if last {
                remaining_width
            } else {
                remaining_width / 2
            };
            let height = remaining_height;
            let img = img.resize_to_fill(width, height, FilterType::Triangle);

            if let Err(error) = canvas.copy_from(&img, img_width, img_height) {
                tracing::error!("Collection collage error: Error\n{error}");
                continue;
            };

            img_width += width;
        } else {
            let width = remaining_width;
            let height = if last {
                remaining_height
            } else {
                remaining_height / 2
            };
            let img = img.resize_to_fill(width, height, FilterType::Triangle);

            if let Err(error) = canvas.copy_from(&img, img_width, img_height) {
                tracing::error!("Collection collage error: Error\n{error}");
                continue;
            };

            img_height += height;
        }

        flip = !flip;
    }

    Some(Handle::from_rgba(
        canvas.width(),
        canvas.height(),
        bytes::Bytes::from(canvas.into_raw()),
    ))
}

#[derive(Debug, Clone, Copy)]
struct Hsl {
    h: f32, // 0..360
    s: f32, // 0..1
    l: f32, // 0..1
}

fn rgb_to_hsl(r: u8, g: u8, b: u8) -> Hsl {
    let (r, g, b) = (r as f32 / 255.0, g as f32 / 255.0, b as f32 / 255.0);
    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let mut h = 0.0;
    let l = (max + min) / 2.0;
    let d = max - min;

    let s = if d == 0.0 {
        0.0
    } else {
        d / (1.0 - (2.0 * l - 1.0).abs())
    };

    if d != 0.0 {
        h = if max == r {
            60.0 * (((g - b) / d) % 6.0)
        } else if max == g {
            60.0 * ((b - r) / d + 2.0)
        } else {
            60.0 * ((r - g) / d + 4.0)
        };
    }

    if h < 0.0 {
        h += 360.0;
    }

    Hsl { h, s, l }
}

fn hsl_to_rgb(hsl: Hsl) -> [u8; 3] {
    let (h, s, l) = (hsl.h, hsl.s, hsl.l);
    let c = (1.0 - (2.0 * l - 1.0).abs()) * s;
    let x = c * (1.0 - (((h / 60.0) % 2.0) - 1.0).abs());
    let m = l - c / 2.0;

    let (r, g, b) = match h as u32 {
        0..=59 => (c, x, 0.0),
        60..=119 => (x, c, 0.0),
        120..=179 => (0.0, c, x),
        180..=239 => (0.0, x, c),
        240..=299 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    [
        ((r + m) * 255.0).round() as u8,
        ((g + m) * 255.0).round() as u8,
        ((b + m) * 255.0).round() as u8,
    ]
}

fn adaptive_sample_color_compl(img: &DynamicImage) -> [u8; 3] {
    // Downscale to 1×1 for average color
    let small = imageops::resize(img, 2, 2, imageops::FilterType::Triangle);
    let pixel = small.get_pixel(0, 0);
    let [r, g, b, _] = pixel.0;

    let hsl = rgb_to_hsl(r, g, b);

    // Shift hue by 180 degrees (complementary)
    let mut sample = hsl;
    sample.h = (sample.h + 180.0) % 360.0;

    // Adjust luminance for contrast
    // Dark image → brighter overlay; Bright image → darker overlay
    sample.l = if sample.l < 0.5 { 0.8 } else { 0.2 };

    // overlay.s = overlay.s.clamp(0.5, 0.9);
    sample.s = sample.s.min(0.9);

    hsl_to_rgb(sample)
}

pub fn sample_complement(path: &str) -> Option<Color> {
    let img = open(path);

    img.as_ref().map(|img| {
        let color = adaptive_sample_color_compl(img);
        Color::from_rgb8(color[0], color[1], color[2])
    })
}
