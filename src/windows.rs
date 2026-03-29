use crate::{ChromeSettings, Error, Result, WindowCornerPreference, WindowsChromeSettings};

use iced::Color;
use raw_window_handle::Win32WindowHandle;

use std::ffi::c_void;
use std::mem::size_of;
use std::ptr::null_mut;

use windows_sys::Win32::Foundation::{GetLastError, HWND, SetLastError};
use windows_sys::Win32::Graphics::Dwm::{
    DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR, DWMWA_WINDOW_CORNER_PREFERENCE,
    DwmSetWindowAttribute,
};
use windows_sys::Win32::UI::WindowsAndMessaging::{
    DrawMenuBar, EnableMenuItem, GWL_STYLE, GetSystemMenu, GetWindowLongPtrW, MF_BYCOMMAND,
    MF_ENABLED, MF_GRAYED, SC_CLOSE, SWP_FRAMECHANGED, SWP_NOMOVE, SWP_NOSIZE, SWP_NOZORDER,
    SetWindowLongPtrW, SetWindowPos, WS_CAPTION, WS_MAXIMIZEBOX, WS_MINIMIZEBOX, WS_SYSMENU,
    WS_THICKFRAME,
};

const DWMWA_COLOR_DEFAULT: u32 = 0xFFFF_FFFF;
const DWMWA_COLOR_NONE: u32 = 0xFFFF_FFFE;
const DWMWCP_DEFAULT: u32 = 0;
const DWMWCP_DONOTROUND: u32 = 1;
const DWMWCP_ROUND: u32 = 2;
const DWMWCP_ROUNDSMALL: u32 = 3;

pub fn apply(handle: Win32WindowHandle, settings: &ChromeSettings) -> Result<()> {
    let hwnd = handle.hwnd.get() as HWND;

    unsafe {
        apply_style_bits(hwnd, &settings.windows)?;
        apply_system_menu(hwnd, &settings.windows)?;
        apply_dwm_attributes(hwnd, &settings.windows)?;
    }

    Ok(())
}

unsafe fn apply_style_bits(hwnd: HWND, settings: &WindowsChromeSettings) -> Result<()> {
    let mut style = unsafe { GetWindowLongPtrW(hwnd, GWL_STYLE) as u32 };

    style = with_flag(style, WS_CAPTION, settings.caption);
    style = with_flag(style, WS_THICKFRAME, settings.border);
    style = with_flag(style, WS_MINIMIZEBOX, settings.buttons.minimize);
    style = with_flag(style, WS_MAXIMIZEBOX, settings.buttons.maximize);

    let wants_system_menu =
        settings.buttons.close || settings.buttons.minimize || settings.buttons.maximize;
    style = with_flag(style, WS_SYSMENU, wants_system_menu);

    unsafe { SetLastError(0) };
    let previous = unsafe { SetWindowLongPtrW(hwnd, GWL_STYLE, style as isize) };
    if previous == 0 && unsafe { GetLastError() } != 0 {
        return Err(Error::Windows("SetWindowLongPtrW"));
    }

    let ok = unsafe {
        SetWindowPos(
            hwnd,
            null_mut(),
            0,
            0,
            0,
            0,
            SWP_FRAMECHANGED | SWP_NOMOVE | SWP_NOSIZE | SWP_NOZORDER,
        )
    };
    if ok == 0 {
        return Err(Error::Windows("SetWindowPos"));
    }

    Ok(())
}

unsafe fn apply_system_menu(hwnd: HWND, settings: &WindowsChromeSettings) -> Result<()> {
    let wants_system_menu =
        settings.buttons.close || settings.buttons.minimize || settings.buttons.maximize;
    if !wants_system_menu {
        return Ok(());
    }

    let system_menu = unsafe { GetSystemMenu(hwnd, 0) };
    if system_menu.is_null() {
        return Err(Error::Windows("GetSystemMenu"));
    }

    let menu_state = if settings.buttons.close {
        MF_BYCOMMAND | MF_ENABLED
    } else {
        MF_BYCOMMAND | MF_GRAYED
    };

    unsafe { EnableMenuItem(system_menu, SC_CLOSE, menu_state) };

    if unsafe { DrawMenuBar(hwnd) } == 0 {
        return Err(Error::Windows("DrawMenuBar"));
    }

    Ok(())
}

unsafe fn apply_dwm_attributes(hwnd: HWND, settings: &WindowsChromeSettings) -> Result<()> {
    let corner_preference = settings
        .corner_preference
        .unwrap_or(WindowCornerPreference::Default);
    let corner_value = match corner_preference {
        WindowCornerPreference::Default => DWMWCP_DEFAULT,
        WindowCornerPreference::DoNotRound => DWMWCP_DONOTROUND,
        WindowCornerPreference::Round => DWMWCP_ROUND,
        WindowCornerPreference::RoundSmall => DWMWCP_ROUNDSMALL,
    };
    unsafe { set_dwm_attribute(hwnd, DWMWA_WINDOW_CORNER_PREFERENCE as u32, &corner_value)? };

    let border_value = if !settings.border {
        DWMWA_COLOR_NONE
    } else {
        settings
            .border_color
            .map(colorref)
            .unwrap_or(DWMWA_COLOR_DEFAULT)
    };
    unsafe { set_dwm_attribute(hwnd, DWMWA_BORDER_COLOR as u32, &border_value)? };

    let title_background = settings
        .title_background_color
        .map(colorref)
        .unwrap_or(DWMWA_COLOR_DEFAULT);
    unsafe { set_dwm_attribute(hwnd, DWMWA_CAPTION_COLOR as u32, &title_background)? };

    let title_text = settings
        .title_text_color
        .map(colorref)
        .unwrap_or(DWMWA_COLOR_DEFAULT);
    unsafe { set_dwm_attribute(hwnd, DWMWA_TEXT_COLOR as u32, &title_text)? };

    Ok(())
}

unsafe fn set_dwm_attribute<T>(hwnd: HWND, attribute: u32, value: &T) -> Result<()> {
    let result = unsafe {
        DwmSetWindowAttribute(
            hwnd,
            attribute,
            value as *const T as *const c_void,
            size_of::<T>() as u32,
        )
    };

    if result != 0 {
        return Err(Error::Windows("DwmSetWindowAttribute"));
    }

    Ok(())
}

fn with_flag(style: u32, flag: u32, enabled: bool) -> u32 {
    if enabled { style | flag } else { style & !flag }
}

fn colorref(color: Color) -> u32 {
    let [red, green, blue, _] = color.into_rgba8();
    u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16)
}
