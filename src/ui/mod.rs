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

const A_KEY: Key = Key::from_char('a');
const Z_KEY: Key = Key::from_char('z');

/**
UI elements will emit a `Msg` in order to communicate with the main loop.

Most of these are emitted by the main ImgPane, but messages to focus on the
other windows are emitted by both subwindows.
*/
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    FocusColorPane,
    FocusIterPane,
    FocusMainPane,
    /// Load image parameters previously saved to a TOML file.
    Load,
    /// The user pushes one of the "Nudge" buttons. The values emitted are
    /// horzontal and vertical distance in pixels to nudge the image. This
    /// will get translated to a distance on the complex plane, which is
    /// why floats are okay.
    Nudge(f64, f64),
    /// The user clicks on the image in order to recenter it. The values
    /// emitted are the horizontal/vertical locations of the click as
    /// fractions of the width/height of the image.
    Recenter(f64, f64),
    /// The user just hits the return key. Values emited are values from
    /// the "Width" and "Height" inputs, if valid.
    Redraw(Option<usize>, Option<usize>),
    /// Save current image.
    SaveImage,
    /// Save current image generation parameters to a TOML file.
    SaveValues,
    /// The user clicks one of the scale radio butons; the value emitted
    /// is the scale ratio selected.
    Scale(usize),
    /// The user zooms in/out. The value emitted is the value in the "Zoom"
    /// input (if a zoom in) or its reciprocal (if a zoom out).
    Zoom(f64),
}

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
pub fn setup_subwindow_behavior(
    w: &mut DoubleWindow,
    pipe: std::sync::mpsc::Sender<Msg>
) {
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
                        Key::Enter => {
                            pipe.send(Msg::FocusMainPane).unwrap();
                            true
                        },
                        A_KEY => {
                            pipe.send(Msg::FocusIterPane).unwrap();
                            true
                        },
                        Z_KEY => {
                            pipe.send(Msg::FocusColorPane).unwrap();
                            true
                        },
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
pub fn pick_a_file(extension: &str, force_extension: bool) -> Option<String> {
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
    
    if force_extension {
        fname.push_str(extension);
    }
    Some(fname)
}

pub mod color;
pub mod iter;
pub mod img;