use image::{
    DynamicImage, GenericImage, ImageBuffer, ImageReader, Rgba,
    imageops::{self, FilterType},
};

use super::{Color, Image};
use std::path::PathBuf;

const DEFAULT_POSTER_PATH: &[u8] = include_bytes!("../../resources/images/default_poster.png");

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

pub fn default_poster() -> Option<Image> {
    use std::io::Cursor;

    let default = Cursor::new(DEFAULT_POSTER_PATH);

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

    Some(Image {
        width: img.width(),
        height: img.height(),
        bytes: img.into_raw(),
    })
}

pub fn collage<'a>(paths: impl Iterator<Item = &'a str>, width: u32, height: u32) -> Option<Image> {
    let imgs: Vec<DynamicImage> = paths.filter_map(open).take(4).collect();

    if imgs.is_empty() {
        return None;
    }

    let len = imgs.len();
    let mut canvas: ImageBuffer<Rgba<u8>, Vec<_>> = ImageBuffer::new(width, height);
    let filter = FilterType::Triangle;

    match len {
        0 => return None,
        1 => {
            let img = imgs.first().expect("collage len");
            let img = img.resize_to_fill(width, height, filter);

            if let Err(error) = canvas.copy_from(&img, 0, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };
        }
        2 => {
            let first = imgs.first().expect("collage len");
            let sec = imgs.get(1).expect("collage len");
            let first_width = width / 2;

            let first = first.resize_to_fill(first_width, height, filter);

            if let Err(error) = canvas.copy_from(&first, 0, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };

            let sec_width = width.saturating_sub(first_width);
            let sec = sec.resize_to_fill(sec_width, height, filter);

            if let Err(error) = canvas.copy_from(&sec, first_width, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };
        }
        3 => {
            let first = imgs.first().expect("collage len");
            let sec = imgs.get(1).expect("collage len");
            let third = imgs.get(2).expect("collage len");

            let first_width = width / 2;
            let sec_height = height / 2;

            let first = first.resize_to_fill(first_width, height, filter);

            if let Err(error) = canvas.copy_from(&first, 0, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };

            let sec_width = width.saturating_sub(first_width);
            let sec = sec.resize_to_fill(sec_width, sec_height, filter);

            if let Err(error) = canvas.copy_from(&sec, first_width, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };

            let third_height = height.saturating_sub(sec_height);
            let third = third.resize_to_fill(sec_width, third_height, filter);

            if let Err(error) = canvas.copy_from(&third, first_width, sec_height) {
                tracing::error!("Collection collage error: Error\n{error}");
            };
        }

        _ => {
            let first = imgs.first().expect("collage len");
            let sec = imgs.get(1).expect("collage len");
            let third = imgs.get(2).expect("collage len");
            let fourth = imgs.get(3).expect("collage len");

            let first_width = width / 2;
            let first_height = height / 2;

            let first = first.resize_to_fill(first_width, first_height, filter);
            if let Err(error) = canvas.copy_from(&first, 0, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };

            let sec_width = width.saturating_sub(first_width);
            let sec = sec.resize_to_fill(sec_width, first_height, filter);
            if let Err(error) = canvas.copy_from(&sec, first_width, 0) {
                tracing::error!("Collection collage error: Error\n{error}");
            };

            let third_height = height.saturating_sub(first_height);
            let third = third.resize_to_fill(first_width, third_height, filter);

            if let Err(error) = canvas.copy_from(&third, 0, first_height) {
                tracing::error!("Collection collage error: Error\n{error}");
            };

            let fourth = fourth.resize_to_fill(sec_width, third_height, filter);
            if let Err(error) = canvas.copy_from(&fourth, first_width, first_height) {
                tracing::error!("Collection collage error: Error\n{error}");
            };
        }
    }

    Some(Image {
        width: canvas.width(),
        height: canvas.height(),
        bytes: canvas.into_raw(),
    })
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

fn adaptive_sample_color_compl(img: &DynamicImage) -> (Color, Color) {
    // Downscale to 1×1 for average color
    let small = imageops::resize(img, 2, 2, imageops::FilterType::CatmullRom);
    let pixel = small.get_pixel(0, 0);
    let [r, g, b, _] = pixel.0;

    let target = 7.0;
    let sample = (r, g, b, 1.0);
    let mut hsl = rgb_to_hsl(r, g, b);
    let bg_lum = relative_luminance(sample);
    let direction = if bg_lum < 0.5 { 1.0 } else { -1.0 };

    for _ in 0..20 {
        let [r, g, b] = hsl_to_rgb(hsl);
        let rgb = (r, g, b, 1.0);

        if relative_contrast(sample, rgb) >= target {
            return (sample, rgb);
        }
        hsl.l = (hsl.l + direction * 0.05).clamp(0.0, 1.0);
    }

    // guaranteed fallback
    if bg_lum < 0.5 {
        (sample, (255, 255, 255, 1.0))
    } else {
        (sample, (0, 0, 0, 1.0))
    }
}

pub fn sample_complement(path: &str) -> Option<(Color, Color)> {
    let img = open(path);

    img.as_ref().map(adaptive_sample_color_compl)
}

// fn adaptive_sample_color_compl(img: &DynamicImage) -> Color {
//     // Downscale to 1×1 for average color
//     let small = imageops::resize(img, 2, 2, imageops::FilterType::CatmullRom);
//     let pixel = small.get_pixel(0, 0);
//     let [r, g, b, _] = pixel.0;
//
//     let hsl = rgb_to_hsl(r, g, b);
//
//     // Shift hue by 180 degrees (complementary)
//     let mut sample = hsl;
//     sample.h = (sample.h + 180.0) % 360.0;
//
//     // Adjust luminance for contrast
//     // Dark image → brighter overlay; Bright image → darker overlay
//     sample.l = if sample.l < 0.5 { 0.8 } else { 0.2 };
//
//     // overlay.s = overlay.s.clamp(0.5, 0.9);
//     sample.s = sample.s.min(0.9);
//
//     let [r, g, b] = hsl_to_rgb(sample);
//
//     (r, g, b, 1.0)
// }
//
// pub fn sample_complement(path: &str) -> Option<Color> {
//     let img = open(path);
//
//     img.as_ref().map(adaptive_sample_color_compl)
// }

/// Returns the relative luminance of the [`Color`].
/// <https://www.w3.org/TR/WCAG21/#dfn-relative-luminance>
fn relative_luminance(color: Color) -> f32 {
    // As described in:
    // https://en.wikipedia.org/wiki/SRGB#The_reverse_transformation
    fn linear_component(u: f32) -> f32 {
        if u < 0.04045 {
            u / 12.92
        } else {
            ((u + 0.055) / 1.055).powf(2.4)
        }
    }

    let (r, g, b) = (
        linear_component((color.0 as f32) / 255.0),
        linear_component((color.1 as f32) / 255.0),
        linear_component((color.2 as f32) / 255.0),
    );

    0.2126 * r + 0.7152 * g + 0.0722 * b
}

/// Returns the [relative contrast ratio] of the [`Color`] against another one.
///
/// [relative contrast ratio]: https://www.w3.org/TR/WCAG21/#dfn-contrast-ratio
fn relative_contrast(a: Color, b: Color) -> f32 {
    let lum_a = relative_luminance(a);
    let lum_b = relative_luminance(b);

    (lum_a.max(lum_b) + 0.05) / (lum_a.min(lum_b) + 0.05)
}

pub fn save_generated_poster(
    id: registry::models::VideoId,
    img: Image,
    db: PathBuf,
    path: PathBuf,
) {
    tracing::debug!("Saving generated thumbnail on {id}");
    use image::{ImageBuffer, Rgba, codecs::jpeg::JpegEncoder};
    use registry::{db, models::VideoId};
    use rusqlite::types::ToSqlOutput;

    let path = crate::fetch::poster_path(path, id);

    let Some(img): Option<ImageBuffer<Rgba<u8>, _>> =
        ImageBuffer::from_raw(img.width, img.height, img.bytes)
    else {
        tracing::error!("Error saving generated poster image buffer on {id}");
        return;
    };

    let mut file = match std::fs::File::create(path.clone()) {
        Ok(file) => file,
        Err(error) => {
            tracing::error!("Error saving generated poster on {id}.\n{error}");
            return;
        }
    };

    let mut encoder = JpegEncoder::new(&mut file);

    if let Err(error) = encoder.encode_image(&img) {
        tracing::error!("Error saving generated poster on {id}.\n{error}");
        return;
    }

    let db = match db::Database::open(db) {
        Ok(db) => db,
        Err(error) => {
            tracing::error!("fetcher Db Error \n{error}");
            return;
        }
    };

    let table = match id {
        VideoId::Movie(_) => "movie",
        VideoId::Episode(_) => "episode",
    };

    let sql =
        format!("UPDATE {table} SET poster=:poster, generate_poster=:generate_poster WHERE id=:id");

    let path = path.display().to_string();

    match db.execute(
        &sql,
        &[
            (":id", &ToSqlOutput::from(id)),
            (":generate_poster", &ToSqlOutput::from(false)),
            (":poster", &ToSqlOutput::from(path)),
        ],
    ) {
        Ok(_) => {
            tracing::debug!("Generated poster {id} saved.")
        }
        Err(error) => {
            tracing::error!("Error saving generated poster on {id}.\n{error}");
        }
    }
}
