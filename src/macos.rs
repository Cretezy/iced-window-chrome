use crate::{ChromeSettings, Error, MacosChromeSettings, MacosTitlebarSeparatorStyle, Result};

use objc2::rc::Retained;
use objc2::{DefinedClass, MainThreadMarker, MainThreadOnly, define_class, msg_send, sel};
use objc2_app_kit::{
    NSButton, NSLayoutAttribute, NSTitlebarAccessoryViewController, NSTitlebarSeparatorStyle,
    NSView, NSViewDidUpdateTrackingAreasNotification, NSViewFrameDidChangeNotification, NSWindow,
    NSWindowButton, NSWindowDidUpdateNotification, NSWindowStyleMask, NSWindowTitleVisibility,
};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSNotification, NSNotificationCenter, NSObject, NSPoint, NSSize};
use raw_window_handle::AppKitWindowHandle;
use std::cell::{Cell, RefCell};
use std::collections::HashMap;

const OFFSET_EPSILON: CGFloat = 0.001;

thread_local! {
    static TRAFFIC_LIGHT_OFFSETS: RefCell<HashMap<usize, TrafficLightOffsetState>> =
        RefCell::new(HashMap::new());
    static TRAFFIC_LIGHT_OBSERVERS:
        RefCell<HashMap<usize, Retained<TrafficLightOffsetObserver>>> = RefCell::new(HashMap::new());
}

struct TrafficLightOffsetObserverIvars {
    button: Retained<NSButton>,
    expected_x: Cell<CGFloat>,
    expected_y: Cell<CGFloat>,
    adjusting: Cell<bool>,
    posts_frame_changed_notifications: bool,
}

define_class!(
    #[unsafe(super = NSObject)]
    #[thread_kind = MainThreadOnly]
    #[ivars = TrafficLightOffsetObserverIvars]
    struct TrafficLightOffsetObserver;

    impl TrafficLightOffsetObserver {
        #[unsafe(method(trafficLightWindowDidUpdate:))]
        fn traffic_light_window_did_update(&self, _notification: &NSNotification) {
            self.restore_expected_position();
        }
    }
);

impl TrafficLightOffsetObserver {
    fn new(
        button: Retained<NSButton>,
        expected_x: CGFloat,
        expected_y: CGFloat,
        mtm: MainThreadMarker,
    ) -> Retained<Self> {
        let posts_frame_changed_notifications = button.postsFrameChangedNotifications();
        button.setPostsFrameChangedNotifications(true);

        let this = Self::alloc(mtm).set_ivars(TrafficLightOffsetObserverIvars {
            button,
            expected_x: Cell::new(expected_x),
            expected_y: Cell::new(expected_y),
            adjusting: Cell::new(false),
            posts_frame_changed_notifications,
        });
        // SAFETY: NSObject's `init` method has the expected signature.
        unsafe { msg_send![super(this), init] }
    }

    fn restore_expected_position(&self) {
        if self.ivars().adjusting.replace(true) {
            return;
        }

        let frame = self.ivars().button.frame();
        let expected_x = self.ivars().expected_x.get();
        let expected_y = self.ivars().expected_y.get();
        if (frame.origin.x - expected_x).abs() > OFFSET_EPSILON
            || (frame.origin.y - expected_y).abs() > OFFSET_EPSILON
        {
            self.ivars()
                .button
                .setFrameOrigin(NSPoint::new(expected_x, expected_y));
        }
        self.ivars().adjusting.set(false);
    }
}

#[derive(Debug, Clone, Copy)]
struct TrafficLightOffsetState {
    baseline_x: CGFloat,
    baseline_y: CGFloat,
    last_offset_x: CGFloat,
    last_offset_y: CGFloat,
}

pub fn apply(handle: AppKitWindowHandle, settings: &ChromeSettings) -> Result<()> {
    let mtm = MainThreadMarker::new()
        .ok_or(Error::Macos("window patching must run on the main thread"))?;

    let ns_view: Retained<NSView> = unsafe { Retained::retain(handle.ns_view.as_ptr().cast()) }
        .ok_or(Error::Macos("invalid NSView pointer"))?;
    let ns_window = ns_view
        .window()
        .ok_or(Error::Macos("NSView was not installed in an NSWindow"))?;

    apply_style_mask(&ns_window, settings);
    apply_title_visibility(&ns_window, settings);
    apply_titlebar_accessory(&ns_window, settings, mtm);
    apply_traffic_lights(&ns_window, settings, mtm);

    Ok(())
}

fn apply_style_mask(window: &NSWindow, settings: &ChromeSettings) {
    let chrome = &settings.macos;
    let mut style = window.styleMask();

    // AppKit is happier if live windows stay titled; we hide the titlebar
    // visually instead of stripping the style bit after creation. Borderless
    // windows do not include `Closable` in their initial style mask, so add it
    // back when restoring the native titlebar buttons.
    style.insert(NSWindowStyleMask::Titled | NSWindowStyleMask::Closable);

    if uses_fullsize_content_view(chrome) {
        style.insert(NSWindowStyleMask::FullSizeContentView);
    } else {
        style.remove(NSWindowStyleMask::FullSizeContentView);
    }

    window.setStyleMask(style);
    window.setTitlebarAppearsTransparent(shows_transparent_titlebar(chrome));
    window.setTitlebarSeparatorStyle(titlebar_separator_style(chrome));
}

fn apply_title_visibility(window: &NSWindow, settings: &ChromeSettings) {
    let visibility = if settings.macos.titlebar && settings.macos.title {
        NSWindowTitleVisibility::Visible
    } else {
        NSWindowTitleVisibility::Hidden
    };

    window.setTitleVisibility(visibility);
}

fn apply_traffic_lights(window: &NSWindow, settings: &ChromeSettings, mtm: MainThreadMarker) {
    let chrome = &settings.macos;

    if let Some(content_view) = window.contentView() {
        content_view.layoutSubtreeIfNeeded();
    }

    let buttons = [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ];

    for button_kind in buttons {
        if let Some(button) = window.standardWindowButton(button_kind) {
            button.setHidden(!chrome.traffic_lights);
            let (offset_x, offset_y) = if chrome.traffic_lights {
                (chrome.traffic_light_offset_x, chrome.traffic_light_offset_y)
            } else {
                (None, None)
            };

            apply_traffic_light_offset(&button, offset_x, offset_y, window, mtm);
        }
    }
}

fn shows_transparent_titlebar(chrome: &MacosChromeSettings) -> bool {
    !chrome.titlebar || chrome.titlebar_transparent
}

fn uses_fullsize_content_view(chrome: &MacosChromeSettings) -> bool {
    !chrome.titlebar || chrome.fullsize_content_view || chrome.titlebar_transparent
}

fn titlebar_separator_style(chrome: &MacosChromeSettings) -> NSTitlebarSeparatorStyle {
    match chrome.titlebar_separator_style {
        Some(MacosTitlebarSeparatorStyle::Automatic) => NSTitlebarSeparatorStyle::Automatic,
        Some(MacosTitlebarSeparatorStyle::None) => NSTitlebarSeparatorStyle::None,
        Some(MacosTitlebarSeparatorStyle::Line) => NSTitlebarSeparatorStyle::Line,
        Some(MacosTitlebarSeparatorStyle::Shadow) => NSTitlebarSeparatorStyle::Shadow,
        None if chrome.titlebar_transparent || !chrome.titlebar => NSTitlebarSeparatorStyle::None,
        None => NSTitlebarSeparatorStyle::Automatic,
    }
}

fn apply_titlebar_accessory(window: &NSWindow, settings: &ChromeSettings, mtm: MainThreadMarker) {
    let chrome = &settings.macos;
    let count = window.titlebarAccessoryViewControllers().count();

    for index in (0..count).rev() {
        window.removeTitlebarAccessoryViewControllerAtIndex(index as isize);
    }

    if !(chrome.titlebar || chrome.traffic_lights) {
        return;
    }

    let Some(height) = chrome.titlebar_height else {
        return;
    };

    let controller = NSTitlebarAccessoryViewController::new(mtm);
    controller.setLayoutAttribute(NSLayoutAttribute::Bottom);
    controller.setAutomaticallyAdjustsSize(false);
    controller.setFullScreenMinHeight(height as CGFloat);

    let view = NSView::new(mtm);
    view.setFrameSize(NSSize::new(1.0, height as CGFloat));
    view.setFrameOrigin(NSPoint::new(0.0, 0.0));

    controller.setView(&view);
    window.addTitlebarAccessoryViewController(&controller);
}

fn apply_traffic_light_offset(
    button: &Retained<NSButton>,
    offset_x: Option<f64>,
    offset_y: Option<f64>,
    window: &NSWindow,
    mtm: MainThreadMarker,
) {
    let button_key = Retained::as_ptr(button) as usize;
    let current_origin = button.frame().origin;
    let target_offset_x = offset_x.unwrap_or_default() as CGFloat;
    let target_offset_y = offset_y.unwrap_or_default() as CGFloat;
    let baseline_x = tracked_baseline_x(button_key, current_origin.x);
    let baseline_y = tracked_baseline(button_key, current_origin.y);
    let expected_x = baseline_x + target_offset_x;
    let expected_y = baseline_y + target_offset_y;

    update_traffic_light_observer(
        button,
        offset_x.or(offset_y).map(|_| (expected_x, expected_y)),
        window,
        mtm,
    );
    button.setFrameOrigin(NSPoint::new(expected_x, expected_y));

    store_tracked_offset(
        button_key,
        baseline_x,
        baseline_y,
        target_offset_x,
        target_offset_y,
    );
}

fn update_traffic_light_observer(
    button: &Retained<NSButton>,
    expected_position: Option<(CGFloat, CGFloat)>,
    window: &NSWindow,
    mtm: MainThreadMarker,
) {
    let button_key = Retained::as_ptr(button) as usize;

    TRAFFIC_LIGHT_OBSERVERS.with(|observers| {
        let mut observers = observers.borrow_mut();

        if let Some((expected_x, expected_y)) = expected_position {
            if let Some(observer) = observers.get(&button_key) {
                observer.ivars().expected_x.set(expected_x);
                observer.ivars().expected_y.set(expected_y);
                return;
            }

            let observer =
                TrafficLightOffsetObserver::new(button.clone(), expected_x, expected_y, mtm);
            // SAFETY: The selector is implemented by TrafficLightOffsetObserver and the
            // observed object is the NSWindow passed by the live AppKit window.
            unsafe {
                let notification_center = NSNotificationCenter::defaultCenter();
                notification_center.addObserver_selector_name_object(
                    &observer,
                    sel!(trafficLightWindowDidUpdate:),
                    Some(NSWindowDidUpdateNotification),
                    Some(window),
                );
                notification_center.addObserver_selector_name_object(
                    &observer,
                    sel!(trafficLightWindowDidUpdate:),
                    Some(NSViewDidUpdateTrackingAreasNotification),
                    Some(button),
                );
                notification_center.addObserver_selector_name_object(
                    &observer,
                    sel!(trafficLightWindowDidUpdate:),
                    Some(NSViewFrameDidChangeNotification),
                    Some(button),
                );
            }
            observers.insert(button_key, observer);
        } else if let Some(observer) = observers.remove(&button_key) {
            // SAFETY: This observer was registered with this notification center above.
            unsafe {
                NSNotificationCenter::defaultCenter().removeObserver(&observer);
            }
            if !observer.ivars().posts_frame_changed_notifications {
                observer
                    .ivars()
                    .button
                    .setPostsFrameChangedNotifications(false);
            }
        }
    });
}

fn tracked_baseline(view_key: usize, current_y: CGFloat) -> CGFloat {
    tracked_baseline_axis(view_key, current_y, |state| {
        (state.baseline_y, state.last_offset_y)
    })
}

fn tracked_baseline_x(view_key: usize, current_x: CGFloat) -> CGFloat {
    tracked_baseline_axis(view_key, current_x, |state| {
        (state.baseline_x, state.last_offset_x)
    })
}

fn tracked_baseline_axis(
    view_key: usize,
    current: CGFloat,
    axis: impl Fn(TrafficLightOffsetState) -> (CGFloat, CGFloat),
) -> CGFloat {
    TRAFFIC_LIGHT_OFFSETS.with(|states| {
        let states = states.borrow();

        match states.get(&view_key).copied() {
            Some(previous) => {
                let (baseline, last_offset) = axis(previous);
                infer_baseline(current, baseline, last_offset)
            }
            None => current,
        }
    })
}

fn infer_baseline(current: CGFloat, baseline: CGFloat, last_offset: CGFloat) -> CGFloat {
    let preserved = baseline + last_offset;
    let preserved_distance = (current - preserved).abs();
    let baseline_distance = (current - baseline).abs();

    if preserved_distance <= baseline_distance {
        current - last_offset
    } else {
        current
    }
}

fn store_tracked_offset(
    view_key: usize,
    baseline_x: CGFloat,
    baseline_y: CGFloat,
    target_offset_x: CGFloat,
    target_offset_y: CGFloat,
) {
    TRAFFIC_LIGHT_OFFSETS.with(|states| {
        let mut states = states.borrow_mut();

        if target_offset_x.abs() <= OFFSET_EPSILON && target_offset_y.abs() <= OFFSET_EPSILON {
            states.remove(&view_key);
        } else {
            states.insert(
                view_key,
                TrafficLightOffsetState {
                    baseline_x,
                    baseline_y,
                    last_offset_x: target_offset_x,
                    last_offset_y: target_offset_y,
                },
            );
        }
    });
}
