use iced::widget::{button, checkbox, column, container, pick_list, row, text};
use iced::{Color, Element, Length, Size, Subscription, Task, application, window};

use iced_window_chrome::{ChromeSettings, WindowCornerPreference};

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
}

#[derive(Debug, Clone)]
struct ChromeLab {
    chrome: ChromeSettings,
}

impl ChromeLab {
    fn boot() -> (Self, Task<Message>) {
        let state = Self {
            chrome: ChromeSettings::default(),
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
    }
}

fn subscription(state: &ChromeLab) -> Subscription<Message> {
    iced_window_chrome::subscription(state.chrome.clone()).map(Message::Chrome)
}

fn view(state: &ChromeLab) -> Element<'_, Message> {
    let windows = column![
        text("Windows").size(24),
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
        .spacing(12),
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
        .spacing(12),
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
        .spacing(12),
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
        .spacing(12),
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
