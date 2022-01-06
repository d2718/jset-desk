
use fltk::app::App;

use jset_desk::{fun, rgb};

fn main() {
    let a = App::default();
    
    let cp = rgb::Pane::default();
    let fp = fun::Pane::new();
    
    a.run().unwrap();
}
