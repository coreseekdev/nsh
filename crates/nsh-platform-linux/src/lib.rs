//! Linux backend for nsh: GTK 4 widgets for views, Pango for text.
//!
//! - Each node is an [`view::NshView`] (a `gtk::Widget` subclass) placed
//!   at the frame the core laid out and drawn with GSK render nodes
//!   (background, borders, shadows, text glyphs, images); text fields,
//!   web views and videos are GTK widgets hosted in their node's view.
//! - The engine runs on its own thread ([`nsh_engine::Engine`]); each
//!   commit is one hop to the GTK main loop ([`surface`]).
//! - Text is shaped and broken into lines by Pango on the engine thread;
//!   the main thread only draws the glyph runs it produced ([`text`]).
//! - Native animations run on the root's frame clock ([`animation`]).
//!
//! [`AppView`] is one surface as a widget (for embedding); [`run_app`]
//! opens one in a window. Off Linux this crate is empty.

#![cfg(target_os = "linux")]

mod access;
#[cfg(feature = "dev")]
mod agent;
mod animation;
mod app;
mod canvas;
mod chrome;
mod decor;
#[cfg(feature = "dev")]
pub mod dev;
mod file_drop;
mod file_requests;
mod fps;
#[cfg(test)]
mod gtk_test;
mod image;
mod input;
mod keymap;
mod menu_style;
mod mount;
mod native;
mod navigation;
mod scroll;
mod selectable_view;
mod shader;
mod surface;
#[cfg(feature = "dev")]
mod synth;
mod system;
mod text;
mod text_input;
mod transition;
mod view;

pub use gtk4;

/// The system clipboard's text, read synchronously for hosts that need an
/// answer in the current turn (Tk's `clipboard get`): the main context runs
/// until GTK's asynchronous read answers (at most one second, then empty).
pub fn clipboard_text_blocking() -> String {
    use gtk4 as gtk;
    use gtk::prelude::*;

    let Some(display) = gtk::gdk::Display::default() else {
        return String::new();
    };
    let clipboard = display.clipboard();
    let (tx, rx) = std::sync::mpsc::channel();
    clipboard.read_text_async(None::<&gtk::gio::Cancellable>, move |result| {
        let text = result
            .ok()
            .flatten()
            .map(|text| text.to_string())
            .unwrap_or_default();
        let _ = tx.send(text);
    });
    let context = gtk::glib::MainContext::default();
    let deadline = std::time::Instant::now() + std::time::Duration::from_millis(1000);
    loop {
        match rx.try_recv() {
            Ok(text) => return text,
            Err(std::sync::mpsc::TryRecvError::Disconnected) => return String::new(),
            Err(std::sync::mpsc::TryRecvError::Empty) => {}
        }
        if std::time::Instant::now() > deadline {
            return String::new();
        }
        context.iteration(false);
    }
}
pub use nsh_core::platform::{Appearance, Locale};
pub use nsh_engine::{
    App, Bundle, ByteSize, CacheLocation, CacheOptions, CachePolicy, Limits, Root,
};

#[cfg(feature = "dev")]
pub use crate::app::set_root;
pub use crate::{
    app::{AppOptions, AppSource, AppView, run_app, set_components, set_ui_sets},
    image::AssetBase,
    native::{ComponentContext, ComponentFactory, Components, NativeComponent},
};
