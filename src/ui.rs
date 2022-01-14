/*!
The user interface.

All FLTK garbage goes here.
*/

use std::cell::Cell;
use std::rc::Rc;

use fltk::{
    prelude::*,
    app::{event_key, };
    button::Button,
    enums::{Color, Event, Key, Shortcut},
    frame::Frame,
    valuator::{HorizNiceSlider, ValueInput},
};

use crate::image::*;

const SPACE_KEY = Key::from_char(' ');

fn rgb_to_flkt(c: RGB) -> Color {
    let v = c.to_rgb8();
    Color::from_rgb(v[0], v[1], v[2])
}

const PICKER_LABEL_WIDTH: i32 = 16;
const PICKER_SLIDER_WIDTH: i32 = 256;
const PICKER_INPUT_WIDTH: i32 = 64;
const PICKER_ROW_HEIGHT: i32 = 32;
const PICKER_OUTPUT_WIDTH: i32 = 80;

const PICKER_ROW_WIDTH: i32 = PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH
                            + PICKER_INPUT_WIDTH;
const PICKER_WINDOW_WIDTH: i32 = PICKER_ROW_WIDTH + PICKER_OUTPUT_WIDTH;
const PICKER_WINDOW_HEIGHT: i32 = PICKER_ROW_HEIGHT * 4;
const PICKER_BUTTON_WIDTH: i32 = PICKER_ROW_WIDTH / 2;

fn make_picker_row(ypos: i32, lab: &'static str)
-> (Frame, HorNiceSlider, ValueInput) {
    let lab = Frame::default().with_label(lab)
        .with_pos(0, ypos)
        .with_size(PICKER_LABEL_WIDTH, PICKER_ROW_HEIGHT);
    let mut slider = HorNiceSlider::default()
        .with_pos(PICKER_LABEL_WIDTH, ypos)
        .with_size(PICKER_SLIDER_WIDTH, PICKER_ROW_HEIGHT);
    let mut vinput = ValueInput::default()
        .with_pos(PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH, ypos)
        .with_size(PICKER_INPUT_WIDTH, PICKER_ROW_HEIGHT);
    
    slider.set_range(0.0, 255.0);
    vinput.set_bounds(0.0, 255.0);
    slider.set_step(1.0, 1);
    
    (lab, slider, vinput)
}

pub fn pick_color(start: RGB) -> Option<RGB> {
    
    let rvalue: Rc<Cell<RGB>> = Rc::new(Cell::new(start));
    
    let mut w = DoubleWindow::default().with_label("Specify a Color")
        .with_size(PICKER_WINDOW_WIDTH, PICKER_WINDOW_HEIGHT);

    let mut prev = Frame::default()
        .with_size(PICKER_OUTPUT_WIDTH, PICKER_WINDOW_HEIGHT)
        .with_pos(PICKER_ROW_WIDTH, 0);
    prev.set_color(rgb_to_fltk(start));
    
    let (_rlab, mut rslider, mut rvinput) = make_picker_row(0, "R");
    rslider.set_callback({
        let rvalue = rvalue.clone();
        let mut rvinput = rvinput.clone();
        let mut prev = prev.clone();
        move |s| {
            
    let (_glab, mut gslider, mut gvinput) = make_picker_row(PICKER_ROW_HEIGHT, "G");
    let (_blab, mut bslider, mut bvinput) = make_picker_row(2 * PICKER_ROW_HEIGHT, "B");
    
    let mut ok = Button::default().with_label("Set @returnarrow")
        .with_size(PICKER_BUTTON_WIDTH, PICKER_ROW_HEIGHT)
        .with_pos(0, 3 * PICKER_ROW_HEIGHT);
    ok.set_shortcut(Shortcut::from_key(Key::Enter));
    let mut no = Button::default().with_label("Cancel (Esc)")
        .with_size(PICKER_BUTTON_WIDTH, PICKER_ROW_HEIGHT)
        .with_pos(PICKER_BUTTON_WIDTH, 3 * PICKER_ROW_HEIGHT);
    no.set_shortcut(Shortcut::from_key(Key::Escape));
        