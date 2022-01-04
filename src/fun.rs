/*!
Specifying the iteration function and parameters.
*/

use fltk::{
    input::IntInput,
    window::DoubleWindow,
}

const ROW_HEIGHT: i32 = 32;

pub struct Pane {
    win:    DoubleWindow,
    width:  IntInput,
    height: IntInput,
    
}