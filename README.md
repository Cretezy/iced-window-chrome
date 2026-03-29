# iced-window-chrome

Small native window-chrome patches for [`iced`](https://github.com/iced-rs/iced).

The crate patches already-created windows. It does not replace the runner or try
to build a custom window framework.

## What it does

- Windows: caption, border, caption buttons, corner preference, caption colors, and Windows 11 backdrop material
- macOS: titlebar visibility, title text, traffic lights, transparency, full-size content view, titlebar height, traffic-light offset, separator style
- Linux: X11 Motif WM hints for decorations and close/minimize/maximize buttons

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

`custom_titlebar` shows a single thicker header strategy across platforms:
Windows keeps the native resize border, macOS keeps native traffic lights, X11
goes fully custom, and Wayland treats the header as presentation only.

## Platform notes

| Platform | Status | Notes |
| --- | --- | --- |
| Windows | Supported | Native style bits and DWM attributes |
| macOS | Supported | AppKit titlebar patching |
| Linux | Best effort | X11 only; Wayland is currently a no-op |

Windows shadow color is not supported. The public DWM window-frame APIs do not
expose a standalone shadow-color control.

## Development

```bash
cargo fmt --all
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
```
