#![allow(dead_code)]

use std::fmt::{self, Display};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug)]
pub enum Error {
    GStreamerError(GStreamerError),
    ThumbnailEmptyVideo,
    IO(std::io::Error),
}

impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::GStreamerError(error) => error.fmt(f),
            Self::IO(error) => error.fmt(f),
            Self::ThumbnailEmptyVideo => write!(f, "Tried creating a thumbnail for an empty Video"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::GStreamerError(error) => error.source(),
            Self::IO(error) => error.source(),
            Self::ThumbnailEmptyVideo => None,
        }
    }
}

impl From<std::io::Error> for Error {
    fn from(value: std::io::Error) -> Self {
        Self::IO(value)
    }
}

#[derive(Debug, Clone)]
pub enum GStreamerError {
    Glib(glib::Error),
    BoolError(glib::BoolError),
    StateChangeError(gstreamer::StateChangeError),
}

impl From<GStreamerError> for Error {
    fn from(value: GStreamerError) -> Self {
        Self::GStreamerError(value)
    }
}

impl From<glib::Error> for GStreamerError {
    fn from(value: glib::Error) -> Self {
        Self::Glib(value)
    }
}

impl From<glib::BoolError> for GStreamerError {
    fn from(value: glib::BoolError) -> Self {
        Self::BoolError(value)
    }
}

impl From<gstreamer::StateChangeError> for GStreamerError {
    fn from(value: gstreamer::StateChangeError) -> Self {
        Self::StateChangeError(value)
    }
}

impl Display for GStreamerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Glib(error) => error.fmt(f),
            Self::BoolError(error) => error.fmt(f),
            Self::StateChangeError(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for GStreamerError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Glib(error) => error.source(),
            Self::BoolError(error) => error.source(),
            Self::StateChangeError(error) => error.source(),
        }
    }
}
