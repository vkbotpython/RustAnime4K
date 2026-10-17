//! Windows Mica/Acrylic window backdrop.
//!
//! The DWM material is drawn *behind* the window, so three pieces are
//! needed for it to be visible:
//! 1. the window must be created with an alpha channel (`with_transparent`
//!    in `main.rs` — winit transparency is fixed at window creation) and
//!    the app must leave the shell translucent where the material should
//!    shine through: `theme::apply` alpha-scales the surface fills and
//!    `Anime4KPlayerApp::clear_color` clears to full transparency;
//! 2. DWM must be told to draw the material behind the window
//!    (`DwmSetWindowAttribute`, below);
//! 3. the window's legacy blur-behind state must be repaired: winit's
//!    `with_transparent(true)` enables blur-behind with an *empty* region,
//!    which on Windows 11 makes the system backdrop sample nothing (the
//!    material then renders as a flat tint). `enable_glass` restores the
//!    classic no-region "glass", after which the backdrop shows the desktop
//!    content behind the window as expected. It has to run a beat *after*
//!    the attribute change (the app schedules it via
//!    `Anime4KPlayerApp::sync_backdrop_glass`): setting both in one go
//!    races DWM's attribute commit and the material stays flat.
//!
//! Only the documented attribute (`DWMWA_SYSTEMBACKDROP_TYPE`) is used: it
//! exists on Windows 11 22H2 (build 22621) and newer. On older systems the
//! call fails, the window simply stays opaque, and the error is reported
//! once (see `Anime4KPlayerApp::sync_backdrop`).

use crate::theme::Backdrop;

/// Pushes `backdrop` (and the light/dark variant, `dark`) onto the window
/// of `window` — the eframe/winit window, anything implementing
/// `HasWindowHandle`.
#[cfg(windows)]
pub fn apply(
    window: &impl raw_window_handle::HasWindowHandle,
    backdrop: Backdrop,
    dark: bool,
) -> anyhow::Result<()> {
    use anyhow::{Context as _, bail};
    use raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::Foundation::HWND;
    use windows_sys::Win32::Graphics::Dwm::{
        DWM_BB_BLURREGION, DWM_BB_ENABLE, DWM_BLURBEHIND, DWMSBT_MAINWINDOW, DWMSBT_NONE,
        DWMSBT_TABBEDWINDOW, DWMSBT_TRANSIENTWINDOW, DWMWA_SYSTEMBACKDROP_TYPE,
        DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWINDOWATTRIBUTE, DwmEnableBlurBehindWindow,
        DwmSetWindowAttribute,
    };
    use windows_sys::Win32::Graphics::Gdi::{CreateRectRgn, DeleteObject, HGDIOBJ};

    /// `DwmSetWindowAttribute` with an `i32` payload.
    fn set(hwnd: HWND, attribute: DWMWINDOWATTRIBUTE, value: i32) -> anyhow::Result<()> {
        let hr = unsafe {
            DwmSetWindowAttribute(
                hwnd,
                attribute as u32,
                std::ptr::from_ref(&value).cast(),
                size_of::<i32>() as u32,
            )
        };
        if hr < 0 {
            // E_INVALIDARG on Windows 10 / pre-22H2 Windows 11: unknown
            // attribute. The caller keeps the window opaque.
            bail!("DwmSetWindowAttribute({attribute}) failed: HRESULT {hr:#010X}");
        }
        Ok(())
    }

    /// Restores the exact state winit gives a transparent window: blur
    /// behind enabled with an empty region.
    fn restore_winit_blur(hwnd: HWND) -> anyhow::Result<()> {
        let region = unsafe { CreateRectRgn(0, 0, -1, -1) };
        if region.is_null() {
            bail!("CreateRectRgn failed");
        }
        let blur = DWM_BLURBEHIND {
            dwFlags: DWM_BB_ENABLE | DWM_BB_BLURREGION,
            fEnable: 1, // BOOL: TRUE
            hRgnBlur: region,
            fTransitionOnMaximized: 0, // BOOL: FALSE
        };
        let hr = unsafe { DwmEnableBlurBehindWindow(hwnd, &blur) };
        unsafe { DeleteObject(region as HGDIOBJ) };
        if hr < 0 {
            bail!("DwmEnableBlurBehindWindow(empty region) failed: HRESULT {hr:#010X}");
        }
        Ok(())
    }

    let handle = window
        .window_handle()
        .context("window has no raw handle")?
        .as_raw();
    let RawWindowHandle::Win32(win32) = handle else {
        bail!("not a Win32 window: {handle:?}");
    };
    let hwnd: HWND = win32.hwnd.get() as HWND;

    if backdrop == Backdrop::Off {
        // Dropping the material must not touch the caption colors: a user
        // who never enabled the backdrop keeps the plain system title bar.
        // Unsupported systems fail here as well — nothing to report, the
        // window is opaque either way.
        let _ = set(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, DWMSBT_NONE);
        return restore_winit_blur(hwnd);
    }
    // The material's light/dark variant follows the window's dark-mode
    // flag, and the same flag colors the caption to match the app theme.
    set(hwnd, DWMWA_USE_IMMERSIVE_DARK_MODE, i32::from(dark))?;
    let material = match backdrop {
        Backdrop::Mica => DWMSBT_MAINWINDOW,
        Backdrop::MicaAlt => DWMSBT_TABBEDWINDOW,
        Backdrop::Acrylic => DWMSBT_TRANSIENTWINDOW,
        Backdrop::Off => unreachable!("handled above"),
    };
    set(hwnd, DWMWA_SYSTEMBACKDROP_TYPE, material)?;
    // The glass pass is *not* done here: enabling blur-behind in the same
    // breath races DWM's attribute commit and the material stays flat
    // (verified on Windows 11). The caller schedules `enable_glass` a
    // moment later, once a frame has been composed in between.
    Ok(())
}

/// Second half of `apply` for a material: turns the classic no-region
/// "glass" on (see the module docs) after the window has composed at least
/// one frame since the attribute change. No-op when no material is active.
#[cfg(windows)]
pub fn enable_glass(window: &impl raw_window_handle::HasWindowHandle) -> anyhow::Result<()> {
    use anyhow::{Context as _, bail};
    use raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::Graphics::Dwm::{
        DWM_BB_ENABLE, DWM_BLURBEHIND, DwmEnableBlurBehindWindow,
    };

    let handle = window
        .window_handle()
        .context("window has no raw handle")?
        .as_raw();
    let RawWindowHandle::Win32(win32) = handle else {
        bail!("not a Win32 window: {handle:?}");
    };
    let hwnd = win32.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    let blur = DWM_BLURBEHIND {
        dwFlags: DWM_BB_ENABLE,
        fEnable: 1, // BOOL: TRUE
        hRgnBlur: std::ptr::null_mut(),
        fTransitionOnMaximized: 0, // BOOL: FALSE
    };
    let hr = unsafe { DwmEnableBlurBehindWindow(hwnd, &blur) };
    if hr < 0 {
        bail!("DwmEnableBlurBehindWindow(glass) failed: HRESULT {hr:#010X}");
    }
    Ok(())
}

/// No-op elsewhere (see the `apply` stub above).
#[cfg(not(windows))]
pub fn enable_glass(_window: &impl raw_window_handle::HasWindowHandle) -> anyhow::Result<()> {
    Ok(())
}

/// Whether the window is currently minimized (`IsIconic`). DWM may drop the
/// blur-behind half of the material on minimize, so `sync_backdrop` uses this
/// to re-run the apply sequence once the window comes back.
#[cfg(windows)]
pub fn is_minimized(window: &impl raw_window_handle::HasWindowHandle) -> Option<bool> {
    use raw_window_handle::RawWindowHandle;
    use windows_sys::Win32::UI::WindowsAndMessaging::IsIconic;

    let handle = window.window_handle().ok()?.as_raw();
    let RawWindowHandle::Win32(win32) = handle else {
        return None;
    };
    let hwnd = win32.hwnd.get() as windows_sys::Win32::Foundation::HWND;
    // SAFETY: `hwnd` belongs to a live window (the handle was just obtained
    // from it); `IsIconic` only reads window state.
    Some(unsafe { IsIconic(hwnd) != 0 })
}

/// Always `None`: nothing to repair outside Windows.
#[cfg(not(windows))]
pub fn is_minimized(_window: &impl raw_window_handle::HasWindowHandle) -> Option<bool> {
    None
}

/// No-op elsewhere: Mica/Acrylic are Windows materials and the setting is
/// hidden in the settings dialog on other platforms.
#[cfg(not(windows))]
pub fn apply(
    _window: &impl raw_window_handle::HasWindowHandle,
    _backdrop: Backdrop,
    _dark: bool,
) -> anyhow::Result<()> {
    Ok(())
}
