use crate::{ChromeSettings, Error, Result};

use raw_window_handle::{XlibDisplayHandle, XlibWindowHandle};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{self, ConnectionExt as _, Window};
use x11rb::wrapper::ConnectionExt as _;

x11rb::atom_manager! {
    pub MotifAtoms: MotifAtomsCookie {
        _MOTIF_WM_HINTS,
    }
}

const MWM_HINTS_FUNCTIONS: u32 = 1 << 0;
const MWM_HINTS_DECORATIONS: u32 = 1 << 1;

const MWM_FUNC_ALL: u32 = 1 << 0;
const MWM_FUNC_MINIMIZE: u32 = 1 << 3;
const MWM_FUNC_MAXIMIZE: u32 = 1 << 4;
const MWM_FUNC_CLOSE: u32 = 1 << 5;

#[derive(Debug, Clone, Copy)]
struct MotifHints {
    flags: u32,
    functions: u32,
    decorations: u32,
    input_mode: u32,
    status: u32,
}

impl MotifHints {
    const fn new() -> Self {
        Self {
            flags: 0,
            functions: 0,
            decorations: 0,
            input_mode: 0,
            status: 0,
        }
    }

    fn set_decorations(&mut self, decorations: bool) {
        self.flags |= MWM_HINTS_DECORATIONS;
        self.decorations = decorations.into();
    }

    fn set_close_button(&mut self, enabled: bool) {
        self.set_function(MWM_FUNC_CLOSE, enabled);
    }

    fn set_minimize_button(&mut self, enabled: bool) {
        self.set_function(MWM_FUNC_MINIMIZE, enabled);
    }

    fn set_maximize_button(&mut self, enabled: bool) {
        self.set_function(MWM_FUNC_MAXIMIZE, enabled);
    }

    fn set_function(&mut self, func: u32, enabled: bool) {
        if enabled {
            self.add_func(func);
        } else {
            self.remove_func(func);
        }
    }

    fn add_func(&mut self, func: u32) {
        if self.flags & MWM_HINTS_FUNCTIONS != 0 {
            if self.functions & MWM_FUNC_ALL != 0 {
                self.functions &= !func;
            } else {
                self.functions |= func;
            }
        }
    }

    fn remove_func(&mut self, func: u32) {
        if self.flags & MWM_HINTS_FUNCTIONS == 0 {
            self.flags |= MWM_HINTS_FUNCTIONS;
            self.functions = MWM_FUNC_ALL;
        }

        if self.functions & MWM_FUNC_ALL != 0 {
            self.functions |= func;
        } else {
            self.functions &= !func;
        }
    }

    fn from_property(values: &[u32]) -> Self {
        Self {
            flags: values.first().copied().unwrap_or_default(),
            functions: values.get(1).copied().unwrap_or_default(),
            decorations: values.get(2).copied().unwrap_or_default(),
            input_mode: values.get(3).copied().unwrap_or_default(),
            status: values.get(4).copied().unwrap_or_default(),
        }
    }

    fn as_property(self) -> [u32; 5] {
        [
            self.flags,
            self.functions,
            self.decorations,
            self.input_mode,
            self.status,
        ]
    }
}

pub fn apply_xlib(
    _display_handle: XlibDisplayHandle,
    window_handle: XlibWindowHandle,
    settings: &ChromeSettings,
) -> Result<()> {
    let window = window_handle.window as Window;
    let (conn, _) =
        x11rb::connect(None).map_err(|_| Error::Linux("failed to open X11 connection"))?;
    let atoms = MotifAtoms::new(&conn)
        .map_err(|_| Error::Linux("failed to request X11 atoms"))?
        .reply()
        .map_err(|_| Error::Linux("failed to resolve X11 atoms"))?;
    let mut hints = get_motif_hints(&conn, window, atoms._MOTIF_WM_HINTS)?;
    let chrome = &settings.linux;

    hints.set_decorations(chrome.decorations);
    hints.set_close_button(chrome.buttons.close);
    hints.set_minimize_button(chrome.buttons.minimize);
    hints.set_maximize_button(chrome.buttons.maximize);

    conn.change_property32(
        xproto::PropMode::REPLACE,
        window,
        atoms._MOTIF_WM_HINTS,
        atoms._MOTIF_WM_HINTS,
        &hints.as_property(),
    )
    .map_err(|_| Error::Linux("failed to set Motif WM hints"))?;
    conn.flush()
        .map_err(|_| Error::Linux("failed to flush X11 requests"))?;

    Ok(())
}

pub fn apply_wayland(_settings: &ChromeSettings) -> Result<()> {
    Ok(())
}

fn get_motif_hints(
    conn: &impl Connection,
    window: Window,
    motif_hints: xproto::Atom,
) -> Result<MotifHints> {
    let reply = conn
        .get_property(false, window, motif_hints, motif_hints, 0, 5)
        .map_err(|_| Error::Linux("failed to request Motif WM hints"))?
        .reply()
        .map_err(|_| Error::Linux("failed to read Motif WM hints"))?;

    if reply.format != 32 || reply.type_ != motif_hints {
        return Ok(MotifHints::new());
    }

    let Some(values) = reply.value32() else {
        return Ok(MotifHints::new());
    };

    let values: Vec<u32> = values.take(5).collect();

    if values.len() < 5 {
        Ok(MotifHints::new())
    } else {
        Ok(MotifHints::from_property(&values))
    }
}
