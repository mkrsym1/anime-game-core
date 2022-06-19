pub mod downloader;
pub mod archives;
pub mod installer;

pub mod prelude {
    pub use super::downloader::Downloader;
    pub use super::archives::Archive;
    pub use super::installer::{
        Installer,
        Update as InstallerUpdate
    };
}