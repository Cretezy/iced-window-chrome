# `iced-window-chrome`

Native-only window chrome patches for [`iced`](https://github.com/iced-rs/iced).

This crate stays intentionally small: one library crate, one example app, and a direct runtime integration surface for patching already-created native windows on Windows and macOS while remaining a no-op on Linux.

## Highlights

- Native Windows chrome patching for caption, border, buttons, DWM corner preference, and title colors.
- Native macOS titlebar patching for title visibility, traffic lights, transparency, full-size content view, accessory-driven titlebar height, and traffic-light offsets.
- Linux support is explicit best-effort no-op behavior.
- `iced`-friendly API with `Task` helpers for live windows and a small subscription loop for later-opened windows.

## Install

```toml
[dependencies]
iced = "0.14.0"
iced-window-chrome = { path = "." }
```

## Quick Start

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

For one-off patching during boot or after startup, use:

```rust
iced_window_chrome::apply_to_latest::<Message>(ChromeSettings::default())
```

## Why `subscription` + `handle`?

`iced` subscriptions can observe new windows, but they cannot directly execute `window::run` side effects themselves. This crate keeps the integration honest and small by emitting its own event type from `subscription(settings)` and expecting callers to feed that event back into `handle(event)` from their update loop.

## Example App

Run the included demo:

```bash
cargo run --example chrome-lab
```

The demo lets you toggle Windows and macOS settings, patch the latest live window, and open extra windows that are patched through the subscription flow.

## Platform Support

| Platform | Status | Notes |
| --- | --- | --- |
| Windows | Supported | Native style-bit and DWM patching |
| macOS | Supported | AppKit titlebar and traffic-light patching |
| Linux | Best effort | Unsupported settings are no-ops |

## Development

```bash
cargo fmt --all
cargo check --all-targets
cargo clippy --all-targets -- -D warnings
```

The GitHub Actions workflow runs checks on Linux, Windows, and macOS.
