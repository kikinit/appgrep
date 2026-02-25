pub mod brew;
pub mod cargo;
pub mod desktop;
pub mod dnf;
pub mod dpkg;
pub mod flatpak;
pub mod npm;
pub mod pacman;
pub mod snap;
pub mod standalone;

use thiserror::Error;

use crate::app::Application;

#[derive(Error, Debug)]
pub enum ProviderError {
    #[error("tool not available: {0}")]
    ToolNotAvailable(String),
    #[error("permission denied: {0}")]
    PermissionDenied(String),
    #[error("parse error: {0}")]
    ParseError(String),
    #[error("io error: {0}")]
    Io(#[from] std::io::Error),
}

pub trait AppProvider: Send + Sync {
    fn name(&self) -> &str;
    fn is_available(&self) -> bool;
    fn discover(&self) -> Result<Vec<Application>, ProviderError>;
}
