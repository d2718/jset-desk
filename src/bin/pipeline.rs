/*!
testing the iteration->coloration->display pipeline
*/

use jset_desk::*;

fn main() {
    let a = fltk::app::App::default();
    
    let mut p = img::Pane::new(img::ImageParams::default());
    p.borrow_mut().iter_with_current_parameters_and_update();
    
    a.run().unwrap();
}