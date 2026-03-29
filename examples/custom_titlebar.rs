use iced::alignment;
use iced::mouse;
use iced::widget::{Space, button, column, container, mouse_area, row, scrollable, stack, text};
use iced::{
    Background, Border, Color, Element, Length, Shadow, Subscription, Task, Vector, application,
    window,
};

use iced_window_chrome::{
    CaptionButtons, ChromeSettings, Event, MacosTitlebarSeparatorStyle, WindowCornerPreference,
    WindowsBackdrop, current_windows_capabilities,
};

const TITLEBAR_HEIGHT: f32 = 68.0;
const TITLEBAR_HEIGHT_F64: f64 = 68.0;
const MACOS_TRAFFIC_LIGHT_OFFSET: f64 = 15.0;
const X11_RESIZE_GUTTER: f32 = 8.0;
const MIN_WINDOW_WIDTH: f32 = 760.0;
const MIN_WINDOW_HEIGHT: f32 = 520.0;

fn main() -> iced::Result {
    application(CustomTitlebarDemo::boot, update, view)
        .title(title)
        .window(window_settings())
        .subscription(subscription)
        .run()
}

fn window_settings() -> window::Settings {
    let mut settings = window::Settings {
        size: iced::Size::new(1040.0, 760.0),
        min_size: Some(iced::Size::new(MIN_WINDOW_WIDTH, MIN_WINDOW_HEIGHT)),
        ..window::Settings::default()
    };

    if cfg!(target_os = "windows") {
        settings.decorations = false;
    }

    settings
}

fn title(_: &CustomTitlebarDemo) -> String {
    String::from("custom-titlebar")
}

#[derive(Debug, Clone)]
enum Message {
    Chrome(Event),
    TrackWindow(Option<window::Id>),
    ResizeHover(Option<window::Direction>),
    StartWindowDrag,
    Resize(window::Direction),
    ToggleMaximize,
    Minimize,
    Close,
}

#[derive(Debug, Clone)]
struct CustomTitlebarDemo {
    chrome: ChromeSettings,
    platform: PlatformFlavor,
    window_id: Option<window::Id>,
}

impl CustomTitlebarDemo {
    fn boot() -> (Self, Task<Message>) {
        let platform = PlatformFlavor::detect();
        let chrome = platform.chrome_settings();

        let state = Self {
            chrome: chrome.clone(),
            platform,
            window_id: None,
        };

        (
            state,
            Task::batch([
                iced_window_chrome::apply_to_latest(chrome),
                window::latest().map(Message::TrackWindow),
            ]),
        )
    }
}

fn update(state: &mut CustomTitlebarDemo, message: Message) -> Task<Message> {
    match message {
        Message::Chrome(event) => iced_window_chrome::handle(event),
        Message::TrackWindow(id) => {
            state.window_id = id;
            Task::none()
        }
        Message::ResizeHover(direction) => state
            .window_id
            .filter(|_| state.platform.uses_custom_resize_handles())
            .map(|id| set_x11_resize_cursor(id, direction))
            .unwrap_or_else(Task::none),
        Message::StartWindowDrag => state
            .window_id
            .filter(|_| state.platform.can_drag_titlebar())
            .map(window::drag)
            .unwrap_or_else(Task::none),
        Message::Resize(direction) => state
            .window_id
            .filter(|_| state.platform.uses_custom_resize_handles())
            .map(|id| window::drag_resize(id, direction))
            .unwrap_or_else(Task::none),
        Message::ToggleMaximize => state
            .window_id
            .filter(|_| state.platform.can_drag_titlebar())
            .map(window::toggle_maximize)
            .unwrap_or_else(Task::none),
        Message::Minimize => state
            .window_id
            .filter(|_| state.platform.shows_custom_caption_buttons())
            .map(|id| window::minimize(id, true))
            .unwrap_or_else(Task::none),
        Message::Close => state
            .window_id
            .filter(|_| state.platform.shows_custom_caption_buttons())
            .map(window::close)
            .unwrap_or_else(Task::none),
    }
}

fn subscription(state: &CustomTitlebarDemo) -> Subscription<Message> {
    Subscription::batch([
        iced_window_chrome::subscription(state.chrome.clone()).map(Message::Chrome),
        window::open_events().map(|id| Message::TrackWindow(Some(id))),
    ])
}

fn view(state: &CustomTitlebarDemo) -> Element<'_, Message> {
    let shell: Element<'_, Message> = if state.platform.uses_custom_resize_handles() {
        custom_resize_shell(state)
    } else {
        column![titlebar(state), content(state)]
            .width(Length::Fill)
            .height(Length::Fill)
            .into()
    };

    container(shell)
        .width(Length::Fill)
        .height(Length::Fill)
        .style(|_| app_shell_style())
        .into()
}

fn custom_resize_shell(state: &CustomTitlebarDemo) -> Element<'_, Message> {
    let base = column![titlebar(state), content(state)]
        .width(Length::Fill)
        .height(Length::Fill);

    stack![base, custom_resize_overlay()]
        .width(Length::Fill)
        .height(Length::Fill)
        .into()
}

fn custom_resize_overlay() -> Element<'static, Message> {
    let gutter = Length::Fixed(X11_RESIZE_GUTTER);

    column![
        row![
            resize_handle(window::Direction::NorthWest, gutter, gutter),
            resize_handle(window::Direction::North, Length::Fill, gutter),
            resize_handle(window::Direction::NorthEast, gutter, gutter),
        ]
        .width(Length::Fill)
        .height(gutter),
        row![
            resize_handle(window::Direction::West, gutter, Length::Fill),
            Space::new().width(Length::Fill).height(Length::Fill),
            resize_handle(window::Direction::East, gutter, Length::Fill),
        ]
        .width(Length::Fill)
        .height(Length::Fill),
        row![
            resize_handle(window::Direction::SouthWest, gutter, gutter),
            resize_handle(window::Direction::South, Length::Fill, gutter),
            resize_handle(window::Direction::SouthEast, gutter, gutter),
        ]
        .width(Length::Fill)
        .height(gutter),
    ]
    .width(Length::Fill)
    .height(Length::Fill)
    .into()
}

fn resize_handle(
    direction: window::Direction,
    width: impl Into<Length>,
    height: impl Into<Length>,
) -> Element<'static, Message> {
    mouse_area(Space::new().width(width.into()).height(height.into()))
        .interaction(resize_interaction(direction))
        .on_enter(Message::ResizeHover(Some(direction)))
        .on_exit(Message::ResizeHover(None))
        .on_press(Message::Resize(direction))
        .into()
}

fn resize_interaction(direction: window::Direction) -> mouse::Interaction {
    match direction {
        window::Direction::North | window::Direction::South => {
            mouse::Interaction::ResizingVertically
        }
        window::Direction::East | window::Direction::West => {
            mouse::Interaction::ResizingHorizontally
        }
        window::Direction::NorthEast | window::Direction::SouthWest => {
            mouse::Interaction::ResizingDiagonallyUp
        }
        window::Direction::NorthWest | window::Direction::SouthEast => {
            mouse::Interaction::ResizingDiagonallyDown
        }
    }
}

#[cfg(target_os = "linux")]
fn set_x11_resize_cursor<Message>(
    id: window::Id,
    direction: Option<window::Direction>,
) -> Task<Message>
where
    Message: Send + 'static,
{
    window::run(id, move |native| {
        let _ = apply_x11_resize_cursor(native, direction);
    })
    .discard()
}

#[cfg(not(target_os = "linux"))]
fn set_x11_resize_cursor<Message>(
    _id: window::Id,
    _direction: Option<window::Direction>,
) -> Task<Message>
where
    Message: Send + 'static,
{
    Task::none()
}

#[cfg(target_os = "linux")]
fn apply_x11_resize_cursor(
    native: &dyn iced::window::Window,
    direction: Option<window::Direction>,
) -> Result<(), ()> {
    use raw_window_handle::RawWindowHandle;
    use x11rb::NONE;
    use x11rb::connection::Connection;
    use x11rb::protocol::xproto::{self, ConnectionExt as _, FontWrapper, Window};

    const XC_BOTTOM_LEFT_CORNER: u16 = 12;
    const XC_BOTTOM_RIGHT_CORNER: u16 = 14;
    const XC_BOTTOM_SIDE: u16 = 16;
    const XC_LEFT_SIDE: u16 = 70;
    const XC_RIGHT_SIDE: u16 = 96;
    const XC_TOP_LEFT_CORNER: u16 = 134;
    const XC_TOP_RIGHT_CORNER: u16 = 136;
    const XC_TOP_SIDE: u16 = 138;

    let raw_window = native.window_handle().map_err(|_| ())?;
    let window = match raw_window.as_raw() {
        RawWindowHandle::Xlib(handle) => handle.window as Window,
        RawWindowHandle::Xcb(handle) => handle.window.get() as Window,
        _ => return Ok(()),
    };

    let (conn, _) = x11rb::connect(None).map_err(|_| ())?;

    if let Some(direction) = direction {
        let glyph = match direction {
            window::Direction::North => XC_TOP_SIDE,
            window::Direction::South => XC_BOTTOM_SIDE,
            window::Direction::East => XC_RIGHT_SIDE,
            window::Direction::West => XC_LEFT_SIDE,
            window::Direction::NorthEast => XC_TOP_RIGHT_CORNER,
            window::Direction::NorthWest => XC_TOP_LEFT_CORNER,
            window::Direction::SouthEast => XC_BOTTOM_RIGHT_CORNER,
            window::Direction::SouthWest => XC_BOTTOM_LEFT_CORNER,
        };

        let cursor = conn.generate_id().map_err(|_| ())?;
        let font = FontWrapper::open_font(&conn, b"cursor").map_err(|_| ())?;

        conn.create_glyph_cursor(
            cursor,
            font.font(),
            font.font(),
            glyph,
            glyph + 1,
            0,
            0,
            0,
            u16::MAX,
            u16::MAX,
            u16::MAX,
        )
        .map_err(|_| ())?;

        conn.change_window_attributes(
            window,
            &xproto::ChangeWindowAttributesAux::default().cursor(cursor),
        )
        .map_err(|_| ())?;

        conn.free_cursor(cursor).map_err(|_| ())?;
    } else {
        conn.change_window_attributes(
            window,
            &xproto::ChangeWindowAttributesAux::default().cursor(NONE),
        )
        .map_err(|_| ())?;
    }

    conn.flush().map_err(|_| ())?;
    Ok(())
}

fn titlebar(state: &CustomTitlebarDemo) -> Element<'_, Message> {
    let lead_inset = Space::new().width(state.platform.leading_inset());

    let drag_content = row![
        title_label(),
        tab_chip("Overview"),
        tab_chip("Native edges"),
        tab_chip("Unified layout"),
        status_pill(state.platform.header_note()),
    ]
    .spacing(10)
    .align_y(alignment::Vertical::Center)
    .width(Length::Fill);

    let drag_strip: Element<'_, Message> = if state.platform.can_drag_titlebar() {
        mouse_area(
            container(drag_content)
                .width(Length::Fill)
                .height(Length::Fill)
                .center_y(Length::Fill),
        )
        .on_press(Message::StartWindowDrag)
        .on_double_click(Message::ToggleMaximize)
        .into()
    } else {
        container(drag_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .center_y(Length::Fill)
            .into()
    };

    let buttons: Element<'_, Message> = if state.platform.shows_custom_caption_buttons() {
        row![
            caption_button("min", Message::Minimize, false),
            caption_button("max", Message::ToggleMaximize, false),
            caption_button("x", Message::Close, true),
        ]
        .spacing(8)
        .align_y(alignment::Vertical::Center)
        .into()
    } else {
        Space::new().width(Length::Shrink).into()
    };

    container(
        row![lead_inset, drag_strip, buttons]
            .spacing(12)
            .align_y(alignment::Vertical::Center)
            .width(Length::Fill)
            .height(Length::Fill),
    )
    .height(TITLEBAR_HEIGHT)
    .padding([0, 18])
    .style(|_| titlebar_style())
    .into()
}

fn content(state: &CustomTitlebarDemo) -> Element<'_, Message> {
    let body = column![
        hero_card(state),
        row![
            info_card(
                "What stays native",
                state.platform.native_edges(),
                Color::from_rgb8(234, 162, 86),
            ),
            info_card(
                "What we draw",
                state.platform.custom_layers(),
                Color::from_rgb8(92, 154, 138),
            ),
        ]
        .spacing(18),
        info_card(
            "Why this example exists",
            "It shows a single visual titlebar that adapts to each platform instead \
             of pretending every window system behaves the same.",
            Color::from_rgb8(108, 124, 168),
        ),
    ]
    .spacing(18)
    .padding([18, 18]);

    scrollable(body).height(Length::Fill).into()
}

fn hero_card(state: &CustomTitlebarDemo) -> Element<'_, Message> {
    let summary = match state.platform {
        PlatformFlavor::Windows => {
            "Windows uses a frameless window, keeps native snap behavior, and adds invisible resize handles around the custom titlebar shell."
        }
        PlatformFlavor::Macos => {
            "macOS keeps native traffic lights, shifts the layout right, and uses a taller transparent titlebar."
        }
        PlatformFlavor::LinuxX11 => {
            "X11 uses an undecorated window, draws its own controls, and forwards resize drags back to the window manager."
        }
        PlatformFlavor::LinuxWayland => {
            "Wayland keeps this as a visual header only. Dragging and caption buttons stay out of the custom bar."
        }
        PlatformFlavor::Other => {
            "Unsupported platforms fall back to the visual shell without native chrome changes."
        }
    };

    container(
        column![
            text("Unified Custom Titlebar")
                .size(36)
                .color(Color::from_rgb8(35, 30, 25)),
            text(summary).size(18).color(Color::from_rgb8(90, 82, 72)),
            row![
                metric_chip("Header height", "68 px"),
                metric_chip("Platform", state.platform.label()),
                metric_chip(
                    "Drag bar",
                    if state.platform.can_drag_titlebar() {
                        "enabled"
                    } else {
                        "visual only"
                    },
                ),
            ]
            .spacing(10),
        ]
        .spacing(14),
    )
    .padding(24)
    .width(Length::Fill)
    .style(|_| card_style(Color::from_rgb8(255, 247, 234)))
    .into()
}

fn info_card<'a>(title: &'a str, body: &'a str, accent: Color) -> Element<'a, Message> {
    container(
        column![
            text(title).size(18).color(Color::from_rgb8(38, 34, 29)),
            text(body).size(16).color(Color::from_rgb8(86, 80, 72)),
        ]
        .spacing(10),
    )
    .padding(20)
    .width(Length::Fill)
    .style(move |_| accented_card_style(accent))
    .into()
}

fn title_label<'a>() -> Element<'a, Message> {
    text("Custom titlebar demo")
        .size(18)
        .color(Color::from_rgb8(245, 241, 233))
        .into()
}

fn tab_chip<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(15).color(Color::from_rgb8(229, 222, 211)))
        .padding([8, 12])
        .style(|_| tab_style())
        .into()
}

fn status_pill<'a>(label: &'a str) -> Element<'a, Message> {
    container(text(label).size(14).color(Color::from_rgb8(232, 238, 231)))
        .padding([8, 12])
        .style(|_| status_style())
        .into()
}

fn metric_chip<'a>(label: &'a str, value: &'a str) -> Element<'a, Message> {
    container(
        column![
            text(label).size(12).color(Color::from_rgb8(129, 118, 105)),
            text(value).size(16).color(Color::from_rgb8(40, 35, 30)),
        ]
        .spacing(4),
    )
    .padding([10, 14])
    .style(|_| metric_style())
    .into()
}

fn caption_button<'a>(label: &'a str, message: Message, danger: bool) -> Element<'a, Message> {
    button(text(label).size(14))
        .width(40)
        .height(30)
        .padding(0)
        .style(move |_, status| caption_button_style(status, danger))
        .on_press(message)
        .into()
}

fn app_shell_style() -> iced::widget::container::Style {
    iced::widget::container::Style::default().background(Color::from_rgb8(244, 237, 229))
}

fn titlebar_style() -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(Color::from_rgb8(35, 42, 48))
        .border(
            Border::default()
                .color(Color::from_rgb8(68, 78, 87))
                .width(1),
        )
        .shadow(Shadow {
            color: Color {
                a: 0.18,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 10.0),
            blur_radius: 24.0,
        })
}

fn card_style(background: Color) -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(background)
        .border(
            Border::default()
                .color(Color::from_rgb8(214, 202, 188))
                .width(1),
        )
        .shadow(Shadow {
            color: Color {
                a: 0.08,
                ..Color::BLACK
            },
            offset: Vector::new(0.0, 8.0),
            blur_radius: 20.0,
        })
}

fn accented_card_style(accent: Color) -> iced::widget::container::Style {
    card_style(Color::from_rgb8(252, 249, 243))
        .border(Border::default().color(accent.scale_alpha(0.6)).width(1))
}

fn tab_style() -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(Color::from_rgba8(255, 255, 255, 0.08))
        .border(
            Border::default()
                .color(Color::from_rgba8(255, 255, 255, 0.12))
                .width(1),
        )
}

fn status_style() -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(Color::from_rgb8(76, 103, 88))
        .border(
            Border::default()
                .color(Color::from_rgb8(106, 137, 118))
                .width(1),
        )
}

fn metric_style() -> iced::widget::container::Style {
    iced::widget::container::Style::default()
        .background(Color::from_rgb8(250, 245, 237))
        .border(
            Border::default()
                .color(Color::from_rgb8(223, 214, 203))
                .width(1),
        )
}

fn caption_button_style(
    status: iced::widget::button::Status,
    danger: bool,
) -> iced::widget::button::Style {
    let background = match (danger, status) {
        (true, iced::widget::button::Status::Hovered) => Color::from_rgb8(211, 94, 78),
        (true, _) => Color::from_rgb8(168, 83, 70),
        (false, iced::widget::button::Status::Hovered) => Color::from_rgb8(98, 109, 119),
        (false, _) => Color::from_rgb8(72, 82, 91),
    };

    iced::widget::button::Style {
        background: Some(Background::Color(background)),
        text_color: Color::from_rgb8(245, 241, 233),
        border: Border::default()
            .color(Color::from_rgba8(255, 255, 255, 0.12))
            .width(1),
        shadow: Shadow::default(),
        snap: false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PlatformFlavor {
    Windows,
    Macos,
    LinuxX11,
    LinuxWayland,
    Other,
}

impl PlatformFlavor {
    fn detect() -> Self {
        if cfg!(target_os = "windows") {
            Self::Windows
        } else if cfg!(target_os = "macos") {
            Self::Macos
        } else if cfg!(target_os = "linux") {
            if std::env::var_os("WAYLAND_DISPLAY").is_some() {
                Self::LinuxWayland
            } else {
                Self::LinuxX11
            }
        } else {
            Self::Other
        }
    }

    fn chrome_settings(self) -> ChromeSettings {
        let mut chrome = ChromeSettings::default();

        match self {
            Self::Windows => {
                if let Some(capabilities) = current_windows_capabilities() {
                    if capabilities.supports_dwm_visuals() {
                        chrome.windows.corner_preference = Some(WindowCornerPreference::Round);
                    }

                    if capabilities.supports_system_backdrop() {
                        chrome.windows.backdrop = Some(WindowsBackdrop::Mica);
                    }
                }
            }
            Self::Macos => {
                chrome.macos.titlebar = true;
                chrome.macos.title = false;
                chrome.macos.traffic_lights = true;
                chrome.macos.titlebar_transparent = true;
                chrome.macos.fullsize_content_view = true;
                chrome.macos.titlebar_height = Some(TITLEBAR_HEIGHT_F64);
                chrome.macos.traffic_light_offset_y = Some(MACOS_TRAFFIC_LIGHT_OFFSET);
                chrome.macos.titlebar_separator_style = Some(MacosTitlebarSeparatorStyle::None);
            }
            Self::LinuxX11 => {
                chrome.linux.decorations = false;
                chrome.linux.buttons = CaptionButtons {
                    close: false,
                    minimize: false,
                    maximize: false,
                };
            }
            Self::LinuxWayland | Self::Other => {}
        }

        chrome
    }

    fn label(self) -> &'static str {
        match self {
            Self::Windows => "Windows",
            Self::Macos => "macOS",
            Self::LinuxX11 => "Linux X11",
            Self::LinuxWayland => "Linux Wayland",
            Self::Other => "Other",
        }
    }

    fn header_note(self) -> &'static str {
        match self {
            Self::Windows => "custom drag bar + resize overlay",
            Self::Macos => "native traffic lights preserved",
            Self::LinuxX11 => "fully custom header + resize overlay",
            Self::LinuxWayland => "header only",
            Self::Other => "visual shell",
        }
    }

    fn native_edges(self) -> &'static str {
        match self {
            Self::Windows => {
                "Windows still handles the actual move, resize, and snap behavior, but the edge hit regions are provided by the app."
            }
            Self::Macos => {
                "The traffic lights stay native and keep their hover behavior. The content is shifted right so the header does not collide with them."
            }
            Self::LinuxX11 => {
                "The window manager still performs the actual move and resize operations, but the app provides the drag regions, resize edges, and caption buttons."
            }
            Self::LinuxWayland => {
                "Wayland keeps the compositor in charge. The custom bar is just part of the app content."
            }
            Self::Other => "No native chrome patch is applied here.",
        }
    }

    fn custom_layers(self) -> &'static str {
        match self {
            Self::Windows => {
                "Drag surface, app branding, tabs, caption buttons, and invisible resize handles are drawn in iced."
            }
            Self::Macos => {
                "The thick titlebar is drawn in iced while AppKit still owns the traffic lights."
            }
            Self::LinuxX11 => {
                "The header visuals, caption buttons, drag surface, and invisible resize handles are all drawn in iced."
            }
            Self::LinuxWayland => {
                "The layout stays visually unified, but it avoids fake window controls."
            }
            Self::Other => "The demo still renders the shared header UI.",
        }
    }

    fn can_drag_titlebar(self) -> bool {
        matches!(self, Self::Windows | Self::Macos | Self::LinuxX11)
    }

    fn shows_custom_caption_buttons(self) -> bool {
        matches!(self, Self::Windows | Self::LinuxX11)
    }

    fn uses_custom_resize_handles(self) -> bool {
        matches!(self, Self::Windows | Self::LinuxX11)
    }

    fn leading_inset(self) -> Length {
        if matches!(self, Self::Macos) {
            Length::Fixed(112.0)
        } else {
            Length::Shrink
        }
    }
}
