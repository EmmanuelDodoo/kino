use rusqlite::{Result, Row};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Image {
    pub path: PathBuf,
    pub main: Option<String>,
    pub accent: Option<String>,
}

impl Image {
    pub fn from_row(row: &Row<'_>, prefix: &str) -> Result<Self> {
        let path = row.get::<_, String>(format!("{prefix}path").as_str())?;
        let path = PathBuf::from(path);

        let main = row.get::<_, Option<String>>(format!("{prefix}main").as_str())?;
        let accent = row.get::<_, Option<String>>(format!("{prefix}accent").as_str())?;

        Ok(Self { path, main, accent })
    }

    pub fn set_main(&mut self, r: u8, g: u8, b: u8, a: f32) {
        self.main = Some(format_color(r, g, b, a));
    }

    pub fn set_accent(&mut self, r: u8, g: u8, b: u8, a: f32) {
        self.accent = Some(format_color(r, g, b, a));
    }

    pub fn get_main(&self) -> Option<(u8, u8, u8, f32)> {
        self.main.as_deref().map(parse_color)
    }

    pub fn get_accent(&self) -> Option<(u8, u8, u8, f32)> {
        self.accent.as_deref().map(parse_color)
    }
}

/// Format: rr-gg-bb-aa in hex.
///
/// Reason: Converting to a single u32 means rgb(0, 255, 255, 1.0) is later
/// interpreted as rgb(255, 255, 255) instead
fn format_color(r: u8, g: u8, b: u8, a: f32) -> String {
    format!("{:0x}-{:0x}-{:0x}-{:0x}", r, g, b, (a * 255.0) as u8)
}

fn parse_color(color: &str) -> (u8, u8, u8, f32) {
    let mut parts = color
        .split("-")
        .map(|part| u8::from_str_radix(part, 16).unwrap());

    let r = parts.next().unwrap();
    let g = parts.next().unwrap();
    let b = parts.next().unwrap();
    let a = (parts.next().unwrap() as f32) / (255.0);

    (r, g, b, a)
}
