/*!
Various user interface elements and functionality.

This module is further split up into submodules that govern the behavior
of each of the application's three windows.
*/

use fltk::{
    prelude::*,
    dialog,
    enums::{Color, Event, Key},
    window::DoubleWindow,
};

use crate::image::RGB;

/** Convert an `RGB` struct to an `fltk::enums::Color` value. */
pub fn rgb_to_fltk(c: RGB) -> Color {
    let v = c.to_rgb8();
    Color::from_rgb(v[0], v[1], v[2])
}

/**
Makes some changes to the way an `fltk::window::DoubleWindow` behaves in
order to conform more closely to desired UI behavior.

It removes the "borders", but then sets the window so it can still be
dragged around by clicking on inactive parts. It also prevents the window
from closing when the user hits `esc` when the window is focused.
*/
pub fn setup_subwindow_behavior(w: &mut DoubleWindow) {
    w.handle({
        let (mut wx, mut wy) : (i32, i32) = (w.x(), w.y());
        let (mut x, mut y)   : (i32, i32) = (0, 0);
        move |w, evt| {
            match evt {
                Event::Push => {
                    wx = w.x();
                    wy = w.y();
                    x = fltk::app::event_x();
                    y = fltk::app::event_y();
                    true
                },
                Event::Drag => {
                    let dx = fltk::app::event_x() - x;
                    let dy = fltk::app::event_y() - y;
                    wx = wx + dx;
                    wy = wy + dy;
                    w.set_pos(wx, wy);
                    true
                },
                Event::KeyDown => {
                    match fltk::app::event_key() {
                        Key::Escape => {
                            // Pretend like we handled it to prevent
                            // the default behavior.
                            true
                        },
                        Key::Enter => { false },
                        _ => { false },
                    }
                },
                _ => false,
            }
        }
    });
}

/**
Pops up an `fltk` file chooser dialog to specify a file name and ensures
it ends with the supplied `extension`.
*/
pub fn pick_a_file(extension: &str) -> Option<String> {
    let lc_ext = extension.to_ascii_lowercase();
    let filter = format!("*{}\t*{}", &lc_ext, &extension.to_ascii_uppercase());
    
    let mut fname = match dialog::file_chooser(
        "Name your image file:", &filter, ".", true
    ) {
        None => { return None; },
        Some(f) => f,
    };
    
    if fname.to_ascii_lowercase().ends_with(&lc_ext) {
        return Some(fname);
    }
    
    fname.push_str(extension);
    Some(fname)
}

pub mod color;
pub mod iter;
pub mod img;