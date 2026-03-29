use crate::{ChromeSettings, Error, Result};

use objc2::MainThreadMarker;
use objc2::rc::Retained;
use objc2_app_kit::{
    NSLayoutAttribute, NSTitlebarAccessoryViewController, NSView, NSWindow, NSWindowButton,
    NSWindowStyleMask, NSWindowTitleVisibility,
};
use objc2_core_foundation::CGFloat;
use objc2_foundation::{NSPoint, NSSize};
use raw_window_handle::AppKitWindowHandle;

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
    apply_traffic_lights(&ns_window, settings);
    apply_titlebar_accessory(&ns_window, settings, mtm);

    Ok(())
}

fn apply_style_mask(window: &NSWindow, settings: &ChromeSettings) {
    let chrome = &settings.macos;
    let mut style = window.styleMask();

    if chrome.titlebar {
        style.insert(NSWindowStyleMask::Titled);
    } else {
        style.remove(NSWindowStyleMask::Titled);
    }

    if chrome.fullsize_content_view {
        style.insert(NSWindowStyleMask::FullSizeContentView);
    } else {
        style.remove(NSWindowStyleMask::FullSizeContentView);
    }

    window.setStyleMask(style);
    window.setTitlebarAppearsTransparent(chrome.titlebar && chrome.titlebar_transparent);
}

fn apply_title_visibility(window: &NSWindow, settings: &ChromeSettings) {
    let visibility = if settings.macos.title {
        NSWindowTitleVisibility::Visible
    } else {
        NSWindowTitleVisibility::Hidden
    };

    window.setTitleVisibility(visibility);
}

fn apply_traffic_lights(window: &NSWindow, settings: &ChromeSettings) {
    let chrome = &settings.macos;
    let buttons = [
        NSWindowButton::CloseButton,
        NSWindowButton::MiniaturizeButton,
        NSWindowButton::ZoomButton,
    ];

    for button_kind in buttons {
        if let Some(button) = window.standardWindowButton(button_kind) {
            button.setHidden(!chrome.titlebar || !chrome.traffic_lights);

            if chrome.titlebar
                && chrome.traffic_lights
                && let Some(offset_y) = chrome.traffic_light_offset_y
            {
                let mut frame = button.frame();
                frame.origin.y += offset_y as CGFloat;
                button.setFrameOrigin(frame.origin);
            }
        }
    }
}

fn apply_titlebar_accessory(window: &NSWindow, settings: &ChromeSettings, mtm: MainThreadMarker) {
    let chrome = &settings.macos;
    let count = window.titlebarAccessoryViewControllers().count();

    for index in (0..count).rev() {
        window.removeTitlebarAccessoryViewControllerAtIndex(index as isize);
    }

    if !chrome.titlebar {
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
