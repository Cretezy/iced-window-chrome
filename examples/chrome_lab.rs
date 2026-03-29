use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text};
use iced::{Color, Element, Length, Size, Subscription, Task, application, window};

use iced_window_chrome::{
    ChromeSettings, MacosTitlebarSeparatorStyle, WindowCornerPreference, WindowsBackdrop,
    WindowsCapabilities,
};

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

const WINDOWS_BACKDROP_CHOICES: [WindowsBackdropChoice; 5] = [
    WindowsBackdropChoice::SystemDefault,
    WindowsBackdropChoice::Off,
    WindowsBackdropChoice::Mica,
    WindowsBackdropChoice::Acrylic,
    WindowsBackdropChoice::MicaAlt,
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

const MACOS_SEPARATOR_STYLE_CHOICES: [MacosSeparatorStyleChoice; 4] = [
    MacosSeparatorStyleChoice::SystemDefault,
    MacosSeparatorStyleChoice::Hidden,
    MacosSeparatorStyleChoice::Line,
    MacosSeparatorStyleChoice::Shadow,
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
    WindowsBackdrop(WindowsBackdropChoice),
    MacosTitlebar(bool),
    MacosTitle(bool),
    MacosTrafficLights(bool),
    MacosTransparent(bool),
    MacosFullsize(bool),
    MacosTitlebarHeight(MacosTitlebarHeightChoice),
    MacosTrafficLightOffset(MacosTrafficLightOffsetChoice),
    MacosSeparatorStyle(MacosSeparatorStyleChoice),
    LinuxDecorations(bool),
    LinuxClose(bool),
    LinuxMinimize(bool),
    LinuxMaximize(bool),
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
        Message::WindowsBackdrop(value) => {
            state.chrome.windows.backdrop = value.into_setting();
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
        Message::MacosSeparatorStyle(value) => {
            state.chrome.macos.titlebar_separator_style = value.into_setting();
            reapply(state)
        }
        Message::LinuxDecorations(value) => {
            state.chrome.linux.decorations = value;
            reapply(state)
        }
        Message::LinuxClose(value) => {
            state.chrome.linux.buttons.close = value;
            reapply(state)
        }
        Message::LinuxMinimize(value) => {
            state.chrome.linux.buttons.minimize = value;
            reapply(state)
        }
        Message::LinuxMaximize(value) => {
            state.chrome.linux.buttons.maximize = value;
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
    let windows_backdrop_supported = state
        .windows_capabilities
        .map(WindowsCapabilities::supports_system_backdrop)
        .unwrap_or(false);

    let corner_row: Element<'_, Message> = if windows_visuals_supported {
        picker_row(
            "Corner rounding",
            pick_list(
                WINDOW_CORNER_CHOICES,
                Some(WindowCornerChoice::from_setting(
                    state.chrome.windows.corner_preference,
                )),
                Message::WindowsCorner,
            )
            .width(180)
            .into(),
        )
    } else {
        picker_row(
            "Corner rounding (Win11)",
            locked_picker(WindowCornerChoice::from_setting(
                state.chrome.windows.corner_preference,
            )),
        )
    };

    let border_color_row: Element<'_, Message> = if windows_visuals_supported {
        picker_row(
            "Border color",
            pick_list(
                WINDOWS_COLOR_CHOICES,
                Some(WindowsColorChoice::from_setting(
                    state.chrome.windows.border_color,
                )),
                Message::WindowsBorderColor,
            )
            .width(180)
            .into(),
        )
    } else {
        picker_row(
            "Border color (Win11)",
            locked_picker(WindowsColorChoice::from_setting(
                state.chrome.windows.border_color,
            )),
        )
    };

    let title_background_row: Element<'_, Message> = if windows_visuals_supported {
        picker_row(
            "Title background",
            pick_list(
                WINDOWS_COLOR_CHOICES,
                Some(WindowsColorChoice::from_setting(
                    state.chrome.windows.title_background_color,
                )),
                Message::WindowsTitleBackgroundColor,
            )
            .width(180)
            .into(),
        )
    } else {
        picker_row(
            "Title background (Win11)",
            locked_picker(WindowsColorChoice::from_setting(
                state.chrome.windows.title_background_color,
            )),
        )
    };

    let title_text_row: Element<'_, Message> = if windows_visuals_supported {
        picker_row(
            "Title text",
            pick_list(
                WINDOWS_COLOR_CHOICES,
                Some(WindowsColorChoice::from_setting(
                    state.chrome.windows.title_text_color,
                )),
                Message::WindowsTitleTextColor,
            )
            .width(180)
            .into(),
        )
    } else {
        picker_row(
            "Title text (Win11)",
            locked_picker(WindowsColorChoice::from_setting(
                state.chrome.windows.title_text_color,
            )),
        )
    };

    let backdrop_row: Element<'_, Message> = if windows_backdrop_supported {
        picker_row(
            "Backdrop material",
            pick_list(
                WINDOWS_BACKDROP_CHOICES,
                Some(WindowsBackdropChoice::from_setting(
                    state.chrome.windows.backdrop,
                )),
                Message::WindowsBackdrop,
            )
            .width(180)
            .into(),
        )
    } else {
        picker_row(
            "Backdrop material (22621+)",
            locked_picker(WindowsBackdropChoice::from_setting(
                state.chrome.windows.backdrop,
            )),
        )
    };

    let windows = column![
        text("Windows").size(24),
        text(windows_version_label(state.windows_capabilities)),
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
        backdrop_row,
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
        picker_row(
            "Separator",
            pick_list(
                MACOS_SEPARATOR_STYLE_CHOICES,
                Some(MacosSeparatorStyleChoice::from_setting(
                    state.chrome.macos.titlebar_separator_style
                )),
                Message::MacosSeparatorStyle,
            )
            .width(180)
            .into(),
        ),
        if state.chrome.macos.titlebar || state.chrome.macos.traffic_lights {
            picker_row(
                "Titlebar height",
                pick_list(
                    MACOS_TITLEBAR_HEIGHT_CHOICES,
                    Some(MacosTitlebarHeightChoice::from_setting(
                        state.chrome.macos.titlebar_height,
                    )),
                    Message::MacosTitlebarHeight,
                )
                .width(180)
                .into(),
            )
        } else {
            picker_row(
                "Titlebar height",
                locked_picker(MacosTitlebarHeightChoice::from_setting(
                    state.chrome.macos.titlebar_height,
                )),
            )
        },
        if state.chrome.macos.titlebar || state.chrome.macos.traffic_lights {
            picker_row(
                "Traffic light offset",
                pick_list(
                    MACOS_TRAFFIC_LIGHT_OFFSET_CHOICES,
                    Some(MacosTrafficLightOffsetChoice::from_setting(
                        state.chrome.macos.traffic_light_offset_y,
                    )),
                    Message::MacosTrafficLightOffset,
                )
                .width(180)
                .into(),
            )
        } else {
            picker_row(
                "Traffic light offset",
                locked_picker(MacosTrafficLightOffsetChoice::from_setting(
                    state.chrome.macos.traffic_light_offset_y,
                )),
            )
        },
    ]
    .spacing(12);

    let linux = column![
        text("Linux").size(24),
        text("X11 only"),
        checkbox(state.chrome.linux.decorations)
            .label("Decorations")
            .on_toggle(Message::LinuxDecorations),
        checkbox(state.chrome.linux.buttons.close)
            .label("Close button")
            .on_toggle(Message::LinuxClose),
        checkbox(state.chrome.linux.buttons.minimize)
            .label("Minimize button")
            .on_toggle(Message::LinuxMinimize),
        checkbox(state.chrome.linux.buttons.maximize)
            .label("Maximize button")
            .on_toggle(Message::LinuxMaximize),
    ]
    .spacing(12);

    let controls = row![
        button("Apply to latest window").on_press(Message::ApplyNow),
        button("Open extra window").on_press(Message::OpenWindow),
    ]
    .spacing(12);

    let platform_section: Element<'_, Message> = if cfg!(target_os = "windows") {
        windows.width(Length::Fill).into()
    } else if cfg!(target_os = "macos") {
        macos.width(Length::Fill).into()
    } else if cfg!(target_os = "linux") {
        linux.width(Length::Fill).into()
    } else {
        column![
            text("Unsupported platform").size(24),
            text("This example currently targets Windows, macOS, and Linux."),
        ]
        .spacing(12)
        .width(Length::Fill)
        .into()
    };

    let content = column![
        text("iced-window-chrome").size(34),
        controls,
        platform_section,
    ]
    .spacing(24)
    .padding(24);

    container(scrollable(content).width(Length::Fill).height(Length::Fill))
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn reapply(state: &ChromeLab) -> Task<Message> {
    iced_window_chrome::apply_to_latest(state.chrome.clone())
}

fn picker_row<'a, Message: 'a>(
    label: &'a str,
    control: Element<'a, Message>,
) -> Element<'a, Message> {
    row![text(label).width(Length::Fill), control]
        .spacing(12)
        .into()
}

fn locked_picker<'a, Message: Clone + 'a>(value: impl ToString) -> Element<'a, Message> {
    button(
        row![text(value.to_string()).width(Length::Fill), text("v")]
            .spacing(8)
            .width(Length::Fill),
    )
    .width(180)
    .on_press_maybe(None)
    .into()
}

fn windows_version_label(capabilities: Option<WindowsCapabilities>) -> String {
    capabilities
        .map(|capabilities| format!("Windows {}", capabilities.version))
        .unwrap_or_else(|| String::from("Windows version unavailable"))
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
enum WindowsBackdropChoice {
    SystemDefault,
    Off,
    Mica,
    Acrylic,
    MicaAlt,
}

impl WindowsBackdropChoice {
    fn from_setting(value: Option<WindowsBackdrop>) -> Self {
        match value {
            Some(WindowsBackdrop::None) => Self::Off,
            Some(WindowsBackdrop::Mica) => Self::Mica,
            Some(WindowsBackdrop::Acrylic) => Self::Acrylic,
            Some(WindowsBackdrop::MicaAlt) => Self::MicaAlt,
            None => Self::SystemDefault,
        }
    }

    fn into_setting(self) -> Option<WindowsBackdrop> {
        match self {
            Self::SystemDefault => None,
            Self::Off => Some(WindowsBackdrop::None),
            Self::Mica => Some(WindowsBackdrop::Mica),
            Self::Acrylic => Some(WindowsBackdrop::Acrylic),
            Self::MicaAlt => Some(WindowsBackdrop::MicaAlt),
        }
    }
}

impl std::fmt::Display for WindowsBackdropChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SystemDefault => "System default",
            Self::Off => "Off",
            Self::Mica => "Mica",
            Self::Acrylic => "Acrylic",
            Self::MicaAlt => "Mica Alt",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MacosSeparatorStyleChoice {
    SystemDefault,
    Hidden,
    Line,
    Shadow,
}

impl MacosSeparatorStyleChoice {
    fn from_setting(value: Option<MacosTitlebarSeparatorStyle>) -> Self {
        match value {
            Some(MacosTitlebarSeparatorStyle::None) => Self::Hidden,
            Some(MacosTitlebarSeparatorStyle::Line) => Self::Line,
            Some(MacosTitlebarSeparatorStyle::Shadow) => Self::Shadow,
            Some(MacosTitlebarSeparatorStyle::Automatic) | None => Self::SystemDefault,
        }
    }

    fn into_setting(self) -> Option<MacosTitlebarSeparatorStyle> {
        match self {
            Self::SystemDefault => None,
            Self::Hidden => Some(MacosTitlebarSeparatorStyle::None),
            Self::Line => Some(MacosTitlebarSeparatorStyle::Line),
            Self::Shadow => Some(MacosTitlebarSeparatorStyle::Shadow),
        }
    }
}

impl std::fmt::Display for MacosSeparatorStyleChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(match self {
            Self::SystemDefault => "System default",
            Self::Hidden => "Hidden",
            Self::Line => "Line",
            Self::Shadow => "Shadow",
        })
    }
}

fn approx_eq(left: f64, right: f64) -> bool {
    (left - right).abs() < 0.001
}
