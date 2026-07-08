//! Platform-specific clipboard provider

use crate::PlatformError;
use async_trait::async_trait;
use clipboard_core::ClipboardItem;

/// Platform-agnostic clipboard provider trait
#[async_trait]
pub trait ClipboardProvider: Send + Sync {
    /// Get clipboard content
    async fn get_content(&self) -> Result<ClipboardItem, PlatformError>;

    /// Set clipboard content
    async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError>;

    /// Watch for clipboard changes
    async fn watch_changes(&self) -> Result<(), PlatformError>;
}

/// Platform-specific provider enum
#[derive(Clone)]
pub enum PlatformProvider {
    #[cfg(windows)]
    Windows(windows::WindowsProvider),
    #[cfg(target_os = "linux")]
    Linux(linux::LinuxProvider),
    #[cfg(target_os = "macos")]
    Macos(macos::MacosProvider),
}

#[async_trait]
impl ClipboardProvider for PlatformProvider {
    async fn get_content(&self) -> Result<ClipboardItem, PlatformError> {
        match self {
            #[cfg(windows)]
            PlatformProvider::Windows(p) => p.get_content().await,
            #[cfg(target_os = "linux")]
            PlatformProvider::Linux(p) => p.get_content().await,
            #[cfg(target_os = "macos")]
            PlatformProvider::Macos(p) => p.get_content().await,
            #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
            _ => Err(PlatformError::UnsupportedPlatform),
        }
    }

    async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError> {
        match self {
            #[cfg(windows)]
            PlatformProvider::Windows(p) => p.set_content(item).await,
            #[cfg(target_os = "linux")]
            PlatformProvider::Linux(p) => p.set_content(item).await,
            #[cfg(target_os = "macos")]
            PlatformProvider::Macos(p) => p.set_content(item).await,
            #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
            _ => Err(PlatformError::UnsupportedPlatform),
        }
    }

    async fn watch_changes(&self) -> Result<(), PlatformError> {
        match self {
            #[cfg(windows)]
            PlatformProvider::Windows(p) => p.watch_changes().await,
            #[cfg(target_os = "linux")]
            PlatformProvider::Linux(p) => p.watch_changes().await,
            #[cfg(target_os = "macos")]
            PlatformProvider::Macos(p) => p.watch_changes().await,
            #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
            _ => Err(PlatformError::UnsupportedPlatform),
        }
    }
}

/// Get the appropriate clipboard provider for the current platform
pub fn get_provider() -> Result<PlatformProvider, PlatformError> {
    #[cfg(windows)]
    {
        Ok(PlatformProvider::Windows(WindowsProvider::new()))
    }

    #[cfg(target_os = "linux")]
    {
        Ok(PlatformProvider::Linux(LinuxProvider::new()))
    }

    #[cfg(target_os = "macos")]
    {
        Ok(PlatformProvider::Macos(macos::MacosProvider::new()))
    }

    #[cfg(not(any(windows, target_os = "linux", target_os = "macos")))]
    {
        Err(PlatformError::UnsupportedPlatform)
    }
}

#[cfg(windows)]
mod windows {
    use super::*;
    use arboard::Clipboard;
    use clipboard_core::{ClipboardItem, ContentType};

    #[derive(Clone)]
    pub struct WindowsProvider;

    impl WindowsProvider {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ClipboardProvider for WindowsProvider {
        async fn get_content(&self) -> Result<ClipboardItem, PlatformError> {
            let mut ctx =
                Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

            let text = ctx
                .get_text()
                .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

            let content_bytes = text.into_bytes();
            let content_type = ContentType::detect(&content_bytes);

            Ok(ClipboardItem::new(content_type, content_bytes))
        }

        async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError> {
            if item.content_type == ContentType::Text {
                let text = String::from_utf8_lossy(&item.content).to_string();
                let mut ctx =
                    Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

                ctx.set_text(&text)
                    .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;
            }
            // TODO: Handle other content types
            Ok(())
        }

        async fn watch_changes(&self) -> Result<(), PlatformError> {
            // TODO: Implement Windows clipboard watching using clipboard format listeners
            Err(PlatformError::WatchFailed("Not implemented".to_string()))
        }
    }
}

#[cfg(target_os = "linux")]
mod linux {
    use super::*;
    use arboard::Clipboard;
    use clipboard_core::{ClipboardItem, ContentType};

    #[derive(Clone)]
    pub struct LinuxProvider;

    impl LinuxProvider {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ClipboardProvider for LinuxProvider {
        async fn get_content(&self) -> Result<ClipboardItem, PlatformError> {
            let mut ctx =
                Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

            let text = ctx
                .get_text()
                .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

            let content_bytes = text.into_bytes();
            let content_type = ContentType::detect(&content_bytes);

            Ok(ClipboardItem::new(content_type, content_bytes))
        }

        async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError> {
            if item.content_type == ContentType::Text {
                let text = String::from_utf8_lossy(&item.content).to_string();
                let mut ctx =
                    Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

                ctx.set_text(&text)
                    .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;
            }
            // TODO: Handle other content types
            Ok(())
        }

        async fn watch_changes(&self) -> Result<(), PlatformError> {
            // TODO: Implement clipboard watching using wl-clipboard or x11 events
            Err(PlatformError::WatchFailed("Not implemented".to_string()))
        }
    }
}

#[cfg(target_os = "macos")]
mod macos {
    use super::*;
    use arboard::Clipboard;
    use clipboard_core::{ClipboardItem, ContentType};

    #[derive(Clone)]
    pub struct MacosProvider;

    impl MacosProvider {
        pub fn new() -> Self {
            Self
        }
    }

    #[async_trait]
    impl ClipboardProvider for MacosProvider {
        async fn get_content(&self) -> Result<ClipboardItem, PlatformError> {
            let mut ctx =
                Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

            let text = ctx
                .get_text()
                .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

            let content_bytes = text.into_bytes();
            let content_type = ContentType::detect(&content_bytes);

            Ok(ClipboardItem::new(content_type, content_bytes))
        }

        async fn set_content(&self, item: &ClipboardItem) -> Result<(), PlatformError> {
            if item.content_type == ContentType::Text {
                let text = String::from_utf8_lossy(&item.content).to_string();
                let mut ctx =
                    Clipboard::new().map_err(|e| PlatformError::AccessFailed(e.to_string()))?;

                ctx.set_text(&text)
                    .map_err(|e| PlatformError::AccessFailed(e.to_string()))?;
            }
            // TODO: Handle other content types
            Ok(())
        }

        async fn watch_changes(&self) -> Result<(), PlatformError> {
            // TODO: Implement clipboard watching using NSPasteboard notifications
            Err(PlatformError::WatchFailed("Not implemented".to_string()))
        }
    }
}
