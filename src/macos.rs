use crate::{ChromeSettings, Error, MacosChromeSettings, Result};

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSLayoutAttribute, NSTitlebarAccessoryViewController, NSTitlebarSeparatorStyle, NSView,
    NSWindow, NSWindowButton, NSWindowStyleMask, NSWindowTitleVisibility,
};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSPoint, NSSize};
use raw_window_handle::AppKitWindowHandle;
use std::cell::RefCell;
use std::collections::HashMap;

const OFFSET_EPSILON: CGFloat = 0.001;

thread_local! {
    static TRAFFIC_LIGHT_OFFSETS: RefCell<HashMap<usize, TrafficLightOffsetState>> =
        RefCell::new(HashMap::new());
}

#[derive(Debug, Clone, Copy)]
struct TrafficLightOffsetState {
    baseline_y: CGFloat,
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
    apply_traffic_lights(&ns_window, settings);

    Ok(())
}

fn apply_style_mask(window: &NSWindow, settings: &ChromeSettings) {
    let chrome = &settings.macos;
    let mut style = window.styleMask();

    // AppKit is happier if live windows stay titled; we hide the titlebar
    // visually instead of stripping the style bit after creation.
    style.insert(NSWindowStyleMask::Titled);

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

fn apply_traffic_lights(window: &NSWindow, settings: &ChromeSettings) {
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
            let offset_y = if chrome.traffic_lights {
                chrome.traffic_light_offset_y
            } else {
                None
            };

            apply_traffic_light_offset(&button, offset_y);
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
    if chrome.titlebar_transparent || !chrome.titlebar {
        NSTitlebarSeparatorStyle::None
    } else {
        NSTitlebarSeparatorStyle::Automatic
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

fn apply_traffic_light_offset(button: &objc2_app_kit::NSButton, offset_y: Option<f64>) {
    let button_key = button as *const objc2_app_kit::NSButton as usize;
    let current_origin = button.frame().origin;
    let target_offset_y = offset_y.unwrap_or_default() as CGFloat;
    let baseline_y = tracked_baseline(button_key, current_origin.y);

    button.setFrameOrigin(NSPoint::new(current_origin.x, baseline_y + target_offset_y));

    store_tracked_offset(button_key, baseline_y, target_offset_y);
}

fn tracked_baseline(view_key: usize, current_y: CGFloat) -> CGFloat {
    TRAFFIC_LIGHT_OFFSETS.with(|states| {
        let states = states.borrow();

        match states.get(&view_key).copied() {
            Some(previous) => infer_baseline_y(current_y, previous),
            None => current_y,
        }
    })
}

fn infer_baseline_y(current_y: CGFloat, previous: TrafficLightOffsetState) -> CGFloat {
    let preserved_y = previous.baseline_y + previous.last_offset_y;
    let preserved_distance = (current_y - preserved_y).abs();
    let baseline_distance = (current_y - previous.baseline_y).abs();

    if preserved_distance <= baseline_distance {
        current_y - previous.last_offset_y
    } else {
        current_y
    }
}

fn store_tracked_offset(view_key: usize, baseline_y: CGFloat, target_offset_y: CGFloat) {
    TRAFFIC_LIGHT_OFFSETS.with(|states| {
        let mut states = states.borrow_mut();

        if target_offset_y.abs() <= OFFSET_EPSILON {
            states.remove(&view_key);
        } else {
            states.insert(
                view_key,
                TrafficLightOffsetState {
                    baseline_y,
                    last_offset_y: target_offset_y,
                },
            );
        }
    });
}
