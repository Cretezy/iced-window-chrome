use iced::widget::{button, checkbox, column, container, pick_list, row, text};
use iced::{Color, Element, Length, Size, Subscription, Task, application, window};

use iced_window_chrome::{ChromeSettings, WindowCornerPreference, WindowsCapabilities};

const WINDOW_CORNER_CHOICES: [WindowCornerChoice; 4] = [
    WindowCornerChoice::SystemDefault,
    WindowCornerChoice::Square,
    WindowCornerChoice::Round,
    WindowCornerChoice::RoundSmall,
];

const WINDOWS_COLOR_CHOICES: [WindowsColorChoice; 6] = [
    WindowsColorChoice::SystemDefault,
    WindowsColorChoice::Crimson,
    WindowsColorChoice::Emerald,
    WindowsColorChoice::Indigo,
    WindowsColorChoice::Amber,
    WindowsColorChoice::White,
];

const MACOS_TITLEBAR_HEIGHT_CHOICES: [MacosTitlebarHeightChoice; 6] = [
    MacosTitlebarHeightChoice::SystemDefault,
    MacosTitlebarHeightChoice::Compact,
    MacosTitlebarHeightChoice::Regular,
    MacosTitlebarHeightChoice::Tall,
    MacosTitlebarHeightChoice::Hero,
    MacosTitlebarHeightChoice::Huge,
];

const MACOS_TRAFFIC_LIGHT_OFFSET_CHOICES: [MacosTrafficLightOffsetChoice; 7] = [
    MacosTrafficLightOffsetChoice::SystemDefault,
    MacosTrafficLightOffsetChoice::LiftLarge,
    MacosTrafficLightOffsetChoice::LiftSmall,
    MacosTrafficLightOffsetChoice::Aligned,
    MacosTrafficLightOffsetChoice::DropSmall,
    MacosTrafficLightOffsetChoice::DropMedium,
    MacosTrafficLightOffsetChoice::DropLarge,
];

fn main() -> iced::Result {
    application(ChromeLab::boot, update, view)
        .title(title)
        .window(window::Settings {
            size: Size::new(900.0, 760.0),
            ..window::Settings::default()
        })
        .subscription(subscription)
        .run()
}

fn title(_: &ChromeLab) -> String {
    String::from("chrome-lab")
}

#[derive(Debug, Clone)]
enum Message {
    Chrome(iced_window_chrome::Event),
    ApplyNow,
    OpenWindow,
    Ignore,
    WindowsCaption(bool),
    WindowsBorder(bool),
    WindowsClose(bool),
    WindowsMinimize(bool),
    WindowsMaximize(bool),
    WindowsCorner(WindowCornerChoice),
    WindowsBorderColor(WindowsColorChoice),
    WindowsTitleBackgroundColor(WindowsColorChoice),
    WindowsTitleTextColor(WindowsColorChoice),
    MacosTitlebar(bool),
    MacosTitle(bool),
    MacosTrafficLights(bool),
    MacosTransparent(bool),
    MacosFullsize(bool),
    MacosTitlebarHeight(MacosTitlebarHeightChoice),
    MacosTrafficLightOffset(MacosTrafficLightOffsetChoice),
}

#[derive(Debug, Clone)]
struct ChromeLab {
    chrome: ChromeSettings,
    windows_capabilities: Option<WindowsCapabilities>,
}

impl ChromeLab {
    fn boot() -> (Self, Task<Message>) {
        let state = Self {
            chrome: ChromeSettings::default(),
            windows_capabilities: iced_window_chrome::current_windows_capabilities(),
        };

        (
            state.clone(),
            iced_window_chrome::apply_to_latest(state.chrome.clone()),
        )
    }
}

fn update(state: &mut ChromeLab, message: Message) -> Task<Message> {
    match message {
        Message::Chrome(event) => iced_window_chrome::handle(event),
        Message::ApplyNow => reapply(state),
        Message::OpenWindow => {
            let (_, task) = window::open(window::Settings {
                size: Size::new(640.0, 420.0),
                ..window::Settings::default()
            });

            task.map(|_| Message::Ignore)
        }
        Message::Ignore => Task::none(),
        Message::WindowsCaption(value) => {
            state.chrome.windows.caption = value;
            reapply(state)
        }
        Message::WindowsBorder(value) => {
            state.chrome.windows.border = value;
            reapply(state)
        }
        Message::WindowsClose(value) => {
            state.chrome.windows.buttons.close = value;
            reapply(state)
        }
        Message::WindowsMinimize(value) => {
            state.chrome.windows.buttons.minimize = value;
            reapply(state)
        }
        Message::WindowsMaximize(value) => {
            state.chrome.windows.buttons.maximize = value;
            reapply(state)
        }
        Message::WindowsCorner(value) => {
            state.chrome.windows.corner_preference = value.into_setting();
            reapply(state)
        }
        Message::WindowsBorderColor(value) => {
            state.chrome.windows.border_color = value.into_setting();
            reapply(state)
        }
        Message::WindowsTitleBackgroundColor(value) => {
            state.chrome.windows.title_background_color = value.into_setting();
            reapply(state)
        }
        Message::WindowsTitleTextColor(value) => {
            state.chrome.windows.title_text_color = value.into_setting();
            reapply(state)
        }
        Message::MacosTitlebar(value) => {
            state.chrome.macos.titlebar = value;
            reapply(state)
        }
        Message::MacosTitle(value) => {
            state.chrome.macos.title = value;
            reapply(state)
        }
        Message::MacosTrafficLights(value) => {
            state.chrome.macos.traffic_lights = value;
            reapply(state)
        }
        Message::MacosTransparent(value) => {
            state.chrome.macos.titlebar_transparent = value;
            reapply(state)
        }
        Message::MacosFullsize(value) => {
            state.chrome.macos.fullsize_content_view = value;
            reapply(state)
        }
        Message::MacosTitlebarHeight(value) => {
            state.chrome.macos.titlebar_height = value.into_setting();
            reapply(state)
        }
        Message::MacosTrafficLightOffset(value) => {
            state.chrome.macos.traffic_light_offset_y = value.into_setting();
            reapply(state)
        }
    }
}

fn subscription(state: &ChromeLab) -> Subscription<Message> {
    iced_window_chrome::subscription(state.chrome.clone()).map(Message::Chrome)
}

fn view(state: &ChromeLab) -> Element<'_, Message> {
    let windows_visuals_supported = state
        .windows_capabilities
        .map(WindowsCapabilities::supports_dwm_visuals)
        .unwrap_or(false);

    let windows_visuals_note = state
        .windows_capabilities
        .map(windows_support_note)
        .unwrap_or("Windows-only runtime detection unavailable on this host".to_string());

    let corner_row: Element<'_, Message> = if windows_visuals_supported {
        row![
            text("Corner rounding").width(Length::Fill),
            pick_list(
                WINDOW_CORNER_CHOICES,
                Some(WindowCornerChoice::from_setting(
                    state.chrome.windows.corner_preference
                )),
                Message::WindowsCorner,
            )
            .width(180),
        ]
        .spacing(12)
        .into()
    } else {
        unsupported_setting_row(
            "Corner rounding",
            "Windows 11 DWM visual chrome APIs are required",
        )
    };

    let border_color_row: Element<'_, Message> = if windows_visuals_supported {
        row![
            text("Border color").width(Length::Fill),
            pick_list(
                WINDOWS_COLOR_CHOICES,
                Some(WindowsColorChoice::from_setting(
                    state.chrome.windows.border_color
                )),
                Message::WindowsBorderColor,
            )
            .width(180),
        ]
        .spacing(12)
        .into()
    } else {
        unsupported_setting_row(
            "Border color",
            "Windows 11 DWM visual chrome APIs are required",
        )
    };

    let title_background_row: Element<'_, Message> = if windows_visuals_supported {
        row![
            text("Title background").width(Length::Fill),
            pick_list(
                WINDOWS_COLOR_CHOICES,
                Some(WindowsColorChoice::from_setting(
                    state.chrome.windows.title_background_color
                )),
                Message::WindowsTitleBackgroundColor,
            )
            .width(180),
        ]
        .spacing(12)
        .into()
    } else {
        unsupported_setting_row(
            "Title background",
            "Windows 11 DWM visual chrome APIs are required",
        )
    };

    let title_text_row: Element<'_, Message> = if windows_visuals_supported {
        row![
            text("Title text").width(Length::Fill),
            pick_list(
                WINDOWS_COLOR_CHOICES,
                Some(WindowsColorChoice::from_setting(
                    state.chrome.windows.title_text_color
                )),
                Message::WindowsTitleTextColor,
            )
            .width(180),
        ]
        .spacing(12)
        .into()
    } else {
        unsupported_setting_row(
            "Title text",
            "Windows 11 DWM visual chrome APIs are required",
        )
    };

    let windows = column![
        text("Windows").size(24),
        text(windows_visuals_note),
        checkbox(state.chrome.windows.caption)
            .label("Caption")
            .on_toggle(Message::WindowsCaption),
        checkbox(state.chrome.windows.border)
            .label("Border")
            .on_toggle(Message::WindowsBorder),
        checkbox(state.chrome.windows.buttons.close)
            .label("Close button")
            .on_toggle(Message::WindowsClose),
        checkbox(state.chrome.windows.buttons.minimize)
            .label("Minimize button")
            .on_toggle(Message::WindowsMinimize),
        checkbox(state.chrome.windows.buttons.maximize)
            .label("Maximize button")
            .on_toggle(Message::WindowsMaximize),
        corner_row,
        border_color_row,
        title_background_row,
        title_text_row,
    ]
    .spacing(12);

    let macos = column![
        text("macOS").size(24),
        checkbox(state.chrome.macos.titlebar)
            .label("Titlebar")
            .on_toggle(Message::MacosTitlebar),
        checkbox(state.chrome.macos.title)
            .label("Title text")
            .on_toggle(Message::MacosTitle),
        checkbox(state.chrome.macos.traffic_lights)
            .label("Traffic lights")
            .on_toggle(Message::MacosTrafficLights),
        checkbox(state.chrome.macos.titlebar_transparent)
            .label("Transparent titlebar")
            .on_toggle(Message::MacosTransparent),
        checkbox(state.chrome.macos.fullsize_content_view)
            .label("Full-size content view")
            .on_toggle(Message::MacosFullsize),
        if state.chrome.macos.titlebar {
            row![
                text("Titlebar height").width(Length::Fill),
                pick_list(
                    MACOS_TITLEBAR_HEIGHT_CHOICES,
                    Some(MacosTitlebarHeightChoice::from_setting(
                        state.chrome.macos.titlebar_height
                    )),
                    Message::MacosTitlebarHeight,
                )
                .width(180),
            ]
            .spacing(12)
            .into()
        } else {
            unsupported_setting_row("Titlebar height", "Ignored while the titlebar is disabled")
        },
        if state.chrome.macos.titlebar {
            row![
                text("Traffic light offset").width(Length::Fill),
                pick_list(
                    MACOS_TRAFFIC_LIGHT_OFFSET_CHOICES,
                    Some(MacosTrafficLightOffsetChoice::from_setting(
                        state.chrome.macos.traffic_light_offset_y
                    )),
                    Message::MacosTrafficLightOffset,
                )
                .width(180),
            ]
            .spacing(12)
            .into()
        } else {
            unsupported_setting_row(
                "Traffic light offset",
                "Ignored while the titlebar is disabled",
            )
        },
    ]
    .spacing(12);

    let controls = row![
        button("Apply to latest window").on_press(Message::ApplyNow),
        button("Open extra window").on_press(Message::OpenWindow),
    ]
    .spacing(12);

    let content = column![
        text("iced-window-chrome").size(34),
        text(
            "Toggle settings to patch the latest window now, and keep the \
             subscription installed so any later-opened windows are patched too."
        )
        .width(Length::Fill),
        controls,
        row![windows.width(Length::Fill), macos.width(Length::Fill)].spacing(32),
    ]
    .spacing(24)
    .padding(24);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn reapply(state: &ChromeLab) -> Task<Message> {
    iced_window_chrome::apply_to_latest(state.chrome.clone())
}

fn unsupported_setting_row<'a, Message: 'a>(label: &'a str, note: &'a str) -> Element<'a, Message> {
    row![text(label).width(Length::Fill), text(note),]
        .spacing(12)
        .into()
}

fn windows_support_note(capabilities: WindowsCapabilities) -> String {
    if capabilities.supports_dwm_visuals() {
        format!(
            "Detected Windows {}. Windows 11 DWM visual chrome controls are enabled.",
            capabilities.version
        )
    } else {
        format!(
            "Detected Windows {}. Corner rounding and DWM title/border colors are Windows 11-only, so those controls are disabled.",
            capabilities.version
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowCornerChoice {
    SystemDefault,
    Square,
    Round,
    RoundSmall,
}

impl WindowCornerChoice {
    fn from_setting(value: Option<WindowCornerPreference>) -> Self {
        match value {
            Some(WindowCornerPreference::DoNotRound) => Self::Square,
            Some(WindowCornerPreference::Round) => Self::Round,
            Some(WindowCornerPreference::RoundSmall) => Self::RoundSmall,
            Some(WindowCornerPreference::Default) | None => Self::SystemDefault,
        }
    }

    fn into_setting(self) -> Option<WindowCornerPreference> {
        match self {
            Self::SystemDefault => None,
            Self::Square => Some(WindowCornerPreference::DoNotRound),
            Self::Round => Some(WindowCornerPreference::Round),
            Self::RoundSmall => Some(WindowCornerPreference::RoundSmall),
        }
    }
}

impl std::fmt::Display for WindowCornerChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SystemDefault => "System default",
            Self::Square => "Square",
            Self::Round => "Round",
            Self::RoundSmall => "Round small",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum WindowsColorChoice {
    SystemDefault,
    Crimson,
    Emerald,
    Indigo,
    Amber,
    White,
}

impl WindowsColorChoice {
    fn from_setting(value: Option<Color>) -> Self {
        match value.map(Color::into_rgba8) {
            Some([184, 50, 88, 255]) => Self::Crimson,
            Some([20, 184, 166, 255]) => Self::Emerald,
            Some([99, 102, 241, 255]) => Self::Indigo,
            Some([245, 158, 11, 255]) => Self::Amber,
            Some([255, 255, 255, 255]) => Self::White,
            _ => Self::SystemDefault,
        }
    }

    fn into_setting(self) -> Option<Color> {
        match self {
            Self::SystemDefault => None,
            Self::Crimson => Some(Color::from_rgb8(184, 50, 88)),
            Self::Emerald => Some(Color::from_rgb8(20, 184, 166)),
            Self::Indigo => Some(Color::from_rgb8(99, 102, 241)),
            Self::Amber => Some(Color::from_rgb8(245, 158, 11)),
            Self::White => Some(Color::from_rgb8(255, 255, 255)),
        }
    }
}

impl std::fmt::Display for WindowsColorChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SystemDefault => "System default",
            Self::Crimson => "Crimson",
            Self::Emerald => "Emerald",
            Self::Indigo => "Indigo",
            Self::Amber => "Amber",
            Self::White => "White",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MacosTitlebarHeightChoice {
    SystemDefault,
    Compact,
    Regular,
    Tall,
    Hero,
    Huge,
}

impl MacosTitlebarHeightChoice {
    fn from_setting(value: Option<f64>) -> Self {
        match value {
            Some(value) if approx_eq(value, 28.0) => Self::Compact,
            Some(value) if approx_eq(value, 36.0) => Self::Regular,
            Some(value) if approx_eq(value, 48.0) => Self::Tall,
            Some(value) if approx_eq(value, 60.0) => Self::Hero,
            Some(value) if approx_eq(value, 72.0) => Self::Huge,
            _ => Self::SystemDefault,
        }
    }

    fn into_setting(self) -> Option<f64> {
        match self {
            Self::SystemDefault => None,
            Self::Compact => Some(28.0),
            Self::Regular => Some(36.0),
            Self::Tall => Some(48.0),
            Self::Hero => Some(60.0),
            Self::Huge => Some(72.0),
        }
    }
}

impl std::fmt::Display for MacosTitlebarHeightChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SystemDefault => "System default",
            Self::Compact => "28 pt",
            Self::Regular => "36 pt",
            Self::Tall => "48 pt",
            Self::Hero => "60 pt",
            Self::Huge => "72 pt",
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MacosTrafficLightOffsetChoice {
    SystemDefault,
    LiftLarge,
    LiftSmall,
    Aligned,
    DropSmall,
    DropMedium,
    DropLarge,
}

impl MacosTrafficLightOffsetChoice {
    fn from_setting(value: Option<f64>) -> Self {
        match value {
            Some(value) if approx_eq(value, -12.0) => Self::LiftLarge,
            Some(value) if approx_eq(value, -6.0) => Self::LiftSmall,
            Some(value) if approx_eq(value, 0.0) => Self::Aligned,
            Some(value) if approx_eq(value, 6.0) => Self::DropSmall,
            Some(value) if approx_eq(value, 12.0) => Self::DropMedium,
            Some(value) if approx_eq(value, 18.0) => Self::DropLarge,
            _ => Self::SystemDefault,
        }
    }

    fn into_setting(self) -> Option<f64> {
        match self {
            Self::SystemDefault => None,
            Self::LiftLarge => Some(-12.0),
            Self::LiftSmall => Some(-6.0),
            Self::Aligned => Some(0.0),
            Self::DropSmall => Some(6.0),
            Self::DropMedium => Some(12.0),
            Self::DropLarge => Some(18.0),
        }
    }
}

impl std::fmt::Display for MacosTrafficLightOffsetChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SystemDefault => "System default",
            Self::LiftLarge => "-12 pt",
            Self::LiftSmall => "-6 pt",
            Self::Aligned => "0 pt",
            Self::DropSmall => "+6 pt",
            Self::DropMedium => "+12 pt",
            Self::DropLarge => "+18 pt",
        })
    }
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.001
}
