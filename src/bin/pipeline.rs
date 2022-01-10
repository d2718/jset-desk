/*!
testing the iteration->coloration->display pipeline
*/
use jset_desk::*;

fn main() {
    let a = fltk::app::App::default();
    
    let p = img::Pane::new(img::ImageParams::default(), "internal");
    p.borrow_mut().reiterate();
    
    a.run().unwrap();
}