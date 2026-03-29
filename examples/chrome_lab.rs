use iced::widget::{button, checkbox, column, container, row, text};
use iced::{Element, Length, Size, Subscription, Task, application, window};

use iced_window_chrome::ChromeSettings;

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
