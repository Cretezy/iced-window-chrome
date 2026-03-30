# iced-window-chrome

Small native window-chrome patches for [`iced`](https://github.com/iced-rs/iced).

The crate patches already-created windows. It does not replace the runner or try
to build a custom window framework.

## What it does

- Windows: caption, border, caption buttons, corner preference, caption colors, and Windows 11 backdrop material
- macOS: titlebar visibility, title text, traffic lights, transparency, full-size content view, titlebar height, traffic-light offset, separator style
- Linux: X11 Motif WM hints for decorations and close/minimize/maximize buttons

# ATTENTION!!! CAVEATS!!!

- This crate avoids using a custom runner to modify windows before creation. That means a window may appear unpatched for a split second at startup. So far, this has only been observed on macOS. I have not added a custom runner yet because I want to keep the library light and avoid taking over the app runner unless there is a clear reason to do so. I also want to leave room for future `iced` add-ons that may have a better reason to own the runner.

- If you want custom-runner support, please open an issue describing your use case, platform, and why post-creation patching is not enough. If there is already an issue for it, add a thumbs-up reaction instead of opening a duplicate. That is the easiest way to gauge demand on GitHub.

- Originally, I thought Windows would be able to keep a native resize border with the unified custom titlebar. In practice, that did not work out cleanly in the `custom_titlebar` example. The version that preserved drag-to-snap behavior used the frameless `winit` path, but once the window is frameless there is no native resize border left to grab. The earlier approach of patching styles after creation kept more of the native frame, but it did not preserve the Windows snap behavior I wanted. Because of that tradeoff, the only version that delivered both snap and reliable resizing was to draw invisible resize handles in the app. If users want that promoted from example code into a first-class crate feature, that would broaden the crate beyond native-only patching.

- If you want this custom-resize-border approach built into the crate, please open an issue explaining your use case, which platform(s) matter to you, and why the example is not enough. If an issue for it already exists, add a thumbs-up reaction there. That is the clearest way to show demand without splitting discussion across duplicate issues.

- If a minimum window width is not set on Linux, shrinking the window via a side resize handle may destabilize or crash some compositors. This was observed on Manjaro XFCE.

## Install

```toml
[dependencies]
iced = "0.14.0"
iced-window-chrome = { path = "." }
```

## Basic use

```rust
use iced::{Subscription, Task};
use iced_window_chrome::{ChromeSettings, Event};

#[derive(Debug, Clone)]
enum Message {
    Chrome(Event),
}

struct App {
    chrome: ChromeSettings,
}

fn subscription(app: &App) -> Subscription<Message> {
    iced_window_chrome::subscription(app.chrome.clone()).map(Message::Chrome)
}

fn update(app: &mut App, message: Message) -> Task<Message> {
    match message {
        Message::Chrome(event) => iced_window_chrome::handle(event),
    }
}
```

For a one-off patch:

```rust
iced_window_chrome::apply_to_latest::<Message>(ChromeSettings::default())
```

`subscription` + `handle` exists because `iced` subscriptions can observe new
windows, but they cannot directly run `window::run` side effects.

## Example

```bash
cargo run --example chrome-lab
```

The lab only shows controls for the current platform.

```bash
cargo run --example custom_titlebar
```

`custom_titlebar` shows a uniform header across platforms:
Windows uses a frameless window with custom resize handles, macOS keeps native
traffic lights and the native resize border, X11 goes fully custom, and
Wayland treats the header as presentation only.

## Platform notes

| Platform | Status | Notes |
| --- | --- | --- |
| Windows | Supported | Native style bits and DWM attributes |
| macOS | Supported | AppKit titlebar patching |
| Linux | Best effort | X11 only; Wayland is currently a no-op |
