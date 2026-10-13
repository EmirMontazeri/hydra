// Copyright (C) 2026 Javad Rajabzadeh
// SPDX-License-Identifier: GPL-3.0-or-later

//! Closing a window without aborting the process on macOS.
//!
//! When an `NSWindow` deallocates, AppKit deactivates the text-input context
//! of its still-current view *after* detaching it from the window, and the
//! Text Input UI (cursor accessories, the Globe/emoji picker, dictation)
//! asks that view for `firstRectForCharacterRange:`. winit answers by
//! unwrapping `self.window()`, which is already gone, and the panic aborts
//! the app across the `extern "C"` boundary. Resigning the first responder
//! while the window is still attached lets AppKit finish that deactivation
//! first, so nothing is left to do at dealloc.

use iced::window;
use iced::Task;

/// `window::close`, preceded on macOS by releasing the window's text input.
pub fn close<T: Send + 'static>(id: window::Id) -> Task<T> {
    #[cfg(target_os = "macos")]
    {
        window::run(id, release_text_input)
            .discard()
            .chain(window::close(id))
    }
    #[cfg(not(target_os = "macos"))]
    {
        window::close(id)
    }
}

#[cfg(target_os = "macos")]
fn release_text_input(window: &dyn window::Window) {
    use iced::window::raw_window_handle::RawWindowHandle;
    use objc2::rc::Retained;
    use objc2_app_kit::NSView;

    let Ok(handle) = window.window_handle() else {
        return;
    };
    let RawWindowHandle::AppKit(appkit) = handle.as_raw() else {
        return;
    };
    // SAFETY: iced hands out the handle of a live window and runs this on the
    // main thread, where AppKit requires it.
    let Some(view): Option<Retained<NSView>> =
        (unsafe { Retained::retain(appkit.ns_view.as_ptr().cast()) })
    else {
        return;
    };
    if let Some(ns_window) = view.window() {
        ns_window.makeFirstResponder(None);
    }
}
