//! Native-only window chrome patches for [`iced`].
//!
//! This crate keeps the runtime integration small:
//! - call [`apply`] or [`apply_to_latest`] when you want to patch a live window
//! - install [`subscription`] to watch later-opened windows
//! - forward emitted [`Event`] values back into [`handle`] in your update loop
//!
//! The extra `subscription` + `handle` hop is a real `iced` runtime constraint:
//! subscriptions can observe window openings, but they cannot directly execute
//! `window::run` side effects on their own.

mod settings;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

use iced::{Subscription, Task, window};
use std::fmt;

pub use settings::{
    CaptionButtons, ChromeSettings, LinuxChromeSettings, MacosChromeSettings,
    MacosTitlebarSeparatorStyle, WindowCornerPreference, WindowsChromeSettings,
};

/// The current Windows runtime version.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowsVersion {
    pub major: u32,
    pub minor: u32,
    pub build: u32,
}

impl WindowsVersion {
    /// Returns `true` for Windows 11 builds and newer.
    pub fn is_windows_11_or_newer(self) -> bool {
        self.major > 10 || (self.major == 10 && self.build >= 22_000)
    }
}

impl fmt::Display for WindowsVersion {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}.{}.{}", self.major, self.minor, self.build)
    }
}

/// Windows runtime support flags for version-dependent chrome features.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WindowsCapabilities {
    pub version: WindowsVersion,
    pub corner_preference: bool,
    pub border_color: bool,
    pub title_background_color: bool,
    pub title_text_color: bool,
}

impl WindowsCapabilities {
    /// Returns `true` when the Windows 11 DWM visual chrome APIs are available.
    pub fn supports_dwm_visuals(self) -> bool {
        self.corner_preference
            && self.border_color
            && self.title_background_color
            && self.title_text_color
    }
}

/// Convenience result type for native patch operations.
pub type Result<T> = std::result::Result<T, Error>;

/// Errors produced while trying to patch a live native window.
#[derive(Debug)]
pub enum Error {
    NoWindowAvailable,
    WindowHandle(raw_window_handle::HandleError),
    UnsupportedWindowHandle(&'static str),
    Windows(&'static str),
    Macos(&'static str),
    Linux(&'static str),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWindowAvailable => {
                write!(f, "no live iced window was available to patch")
            }
            Self::WindowHandle(error) => {
                write!(f, "failed to access the native window handle: {error}")
            }
            Self::UnsupportedWindowHandle(kind) => {
                write!(f, "unsupported native window handle: {kind}")
            }
            Self::Windows(message) => write!(f, "Windows API error: {message}"),
            Self::Macos(message) => write!(f, "macOS AppKit error: {message}"),
            Self::Linux(message) => write!(f, "Linux/X11 error: {message}"),
        }
    }
}

impl std::error::Error for Error {}

/// Subscription payloads that should be fed back into [`handle`].
#[derive(Debug, Clone, PartialEq, Hash)]
pub enum Event {
    ApplyToWindow {
        id: window::Id,
        settings: ChromeSettings,
    },
}

/// Apply a chrome patch to the given live `iced` window.
pub fn apply<Message>(id: window::Id, settings: ChromeSettings) -> Task<Message>
where
    Message: Send + 'static,
{
    apply_result(id, settings).discard()
}

/// Apply a chrome patch to the given live `iced` window and return the native result.
pub fn apply_result(id: window::Id, settings: ChromeSettings) -> Task<Result<()>> {
    window::run(id, move |native| apply_native(native, &settings))
}

/// Apply a chrome patch to the most recently opened live `iced` window.
pub fn apply_to_latest<Message>(settings: ChromeSettings) -> Task<Message>
where
    Message: Send + 'static,
{
    apply_to_latest_result(settings).discard()
}

/// Apply a chrome patch to the most recently opened live `iced` window and return the native result.
pub fn apply_to_latest_result(settings: ChromeSettings) -> Task<Result<()>> {
    window::latest().then(move |id| match id {
        Some(id) => apply_result(id, settings.clone()),
        None => Task::done(Err(Error::NoWindowAvailable)),
    })
}

/// Subscribe to later-opened windows so they can be patched in your update loop.
pub fn subscription(settings: ChromeSettings) -> Subscription<Event> {
    window::open_events().with(settings).map(map_open_event)
}

/// Handle a crate [`Event`] emitted by [`subscription`].
pub fn handle<Message>(event: Event) -> Task<Message>
where
    Message: Send + 'static,
{
    handle_result(event).discard()
}

/// Handle a crate [`Event`] emitted by [`subscription`] and return the native result.
pub fn handle_result(event: Event) -> Task<Result<()>> {
    match event {
        Event::ApplyToWindow { id, settings } => apply_result(id, settings),
    }
}

/// Returns Windows runtime capabilities when running on Windows.
#[cfg(target_os = "windows")]
pub fn current_windows_capabilities() -> Option<WindowsCapabilities> {
    windows::current_capabilities()
}

/// Returns Windows runtime capabilities when running on Windows.
#[cfg(not(target_os = "windows"))]
pub fn current_windows_capabilities() -> Option<WindowsCapabilities> {
    None
}

#[cfg(target_os = "windows")]
fn apply_native(window: &dyn iced::window::Window, settings: &ChromeSettings) -> Result<()> {
    use raw_window_handle::RawWindowHandle;

    let handle = window.window_handle().map_err(Error::WindowHandle)?;

    match handle.as_raw() {
        RawWindowHandle::Win32(handle) => windows::apply(handle, settings),
        _ => Err(Error::UnsupportedWindowHandle("non-Win32")),
    }
}

#[cfg(target_os = "macos")]
fn apply_native(window: &dyn iced::window::Window, settings: &ChromeSettings) -> Result<()> {
    use raw_window_handle::RawWindowHandle;

    let handle = window.window_handle().map_err(Error::WindowHandle)?;

    match handle.as_raw() {
        RawWindowHandle::AppKit(handle) => macos::apply(handle, settings),
        _ => Err(Error::UnsupportedWindowHandle("non-AppKit")),
    }
}

#[cfg(target_os = "linux")]
fn apply_native(window: &dyn iced::window::Window, settings: &ChromeSettings) -> Result<()> {
    use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

    let window_handle = window.window_handle().map_err(Error::WindowHandle)?;
    let display_handle = window.display_handle().map_err(Error::WindowHandle)?;

    match (display_handle.as_raw(), window_handle.as_raw()) {
        (RawDisplayHandle::Xlib(display), RawWindowHandle::Xlib(handle)) => {
            linux::apply_xlib(display, handle, settings)
        }
        (RawDisplayHandle::Wayland(_), RawWindowHandle::Wayland(_)) => {
            linux::apply_wayland(settings)
        }
        _ => Err(Error::UnsupportedWindowHandle("non-Xlib Linux")),
    }
}

#[cfg(not(any(target_os = "windows", target_os = "macos", target_os = "linux")))]
fn apply_native(_window: &dyn iced::window::Window, _settings: &ChromeSettings) -> Result<()> {
    Ok(())
}

fn map_open_event((settings, id): (ChromeSettings, window::Id)) -> Event {
    Event::ApplyToWindow { id, settings }
}
