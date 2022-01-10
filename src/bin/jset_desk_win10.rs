/*!
Windows executable.
*/
#![windows_subsystem = "windows"]

use jset_desk::*;

fn main() {
    let a = fltk::app::App::default();
    
    let p = img::Pane::new(
        img::ImageParams::default(),
        "v0.1"
    );
    p.borrow_mut().reiterate();
    
    a.run().unwrap();
}