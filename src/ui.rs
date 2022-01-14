/*!
The user interface.

All FLTK garbage goes here.
*/

use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;

use fltk::{
    prelude::*,
    app::{event_key, },
    button::Button,
    enums::{Color, Event, Key, Shortcut},
    frame::Frame,
    valuator::{HorNiceSlider, ValueInput},
    window::DoubleWindow,
};

use crate::image::*;

const SPACE_KEY: Key = Key::from_char(' ');

fn rgb_to_fltk(c: RGB) -> Color {
    let v = c.to_rgb8();
    let r = Color::from_rgb(v[0], v[1], v[2]);
    println!("{:?}", r);
    r
}

const PICKER_LABEL_WIDTH: i32 = 16;
const PICKER_SLIDER_WIDTH: i32 = 256;
const PICKER_INPUT_WIDTH: i32 = 64;
const PICKER_ROW_HEIGHT: i32 = 32;
const PICKER_OUTPUT_WIDTH: i32 = 4 * PICKER_ROW_HEIGHT;

const PICKER_ROW_WIDTH: i32 = PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH
                            + PICKER_INPUT_WIDTH;
const PICKER_WINDOW_WIDTH: i32 = PICKER_ROW_WIDTH + PICKER_OUTPUT_WIDTH;
const PICKER_WINDOW_HEIGHT: i32 = PICKER_ROW_HEIGHT * 4;
const PICKER_BUTTON_WIDTH: i32 = PICKER_ROW_WIDTH / 2;

fn make_picker_row(
    ypos: i32,
    label: &'static str,
    prev: DoubleWindow,
    rvalue: Rc<Cell<RGB>>
) -> (Frame, HorNiceSlider, ValueInput) {
    let lab = Frame::default().with_label(label)
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
    
    slider.set_callback({
        let rvalue = rvalue.clone();
        let mut vinput = vinput.clone();
        let mut prev = prev.clone();
        move |s| {
            let x = s.value();
            vinput.set_value(x);
            let mut rv = rvalue.get();
            match label {
                "R" => { rv.set_r(x as f32); }
                "G" => { rv.set_g(x as f32); }
                "B" => { rv.set_b(x as f32); }
                s @ _ => { panic!("ui::make_picker_row(): bad picker row label: {}", s); },
            }
            rvalue.set(rv);
            let c = rgb_to_fltk(rv);
            prev.set_color(c);
            prev.redraw();
        }
    });
    
    vinput.set_callback({
        let rvalue = rvalue.clone();
        let mut slider = slider.clone();
        let mut prev = prev.clone();
        move |v| {
            let x = v.value();
            slider.set_value(x);
            let mut rv = rvalue.get();
            match label {
                "R" => { rv.set_r(x as f32); }
                "G" => { rv.set_g(x as f32); }
                "B" => { rv.set_b(x as f32); }
                s @ _ => { panic!("ui::make_picker_row(): bad picker row label: {}", s); },
            }
            rvalue.set(rv);
            let c = rgb_to_fltk(rv);
            prev.set_color(c);
            prev.redraw();
        }
    });
    
    (lab, slider, vinput)
}

pub fn pick_color(start: RGB) -> Option<RGB> {
    
    let rvalue: Rc<Cell<RGB>> = Rc::new(Cell::new(start));
    
    let mut w = DoubleWindow::default().with_label("Specify a Color")
        .with_size(PICKER_WINDOW_WIDTH, PICKER_WINDOW_HEIGHT);

    let mut prev = DoubleWindow::default()
        .with_size(PICKER_OUTPUT_WIDTH, PICKER_WINDOW_HEIGHT)
        .with_pos(PICKER_ROW_WIDTH, 0);
    prev.end();
    prev.set_color(rgb_to_fltk(start));
    
    let (_, _, _) = make_picker_row(0, "R", prev.clone(), rvalue.clone());
    let (_, _, _) = make_picker_row(PICKER_ROW_HEIGHT, "G", prev.clone(), rvalue.clone());
    let (_, _, _) = make_picker_row(2 * PICKER_ROW_HEIGHT, "B", prev.clone(), rvalue.clone());
    
    let mut ok = Button::default().with_label("Set @returnarrow")
        .with_size(PICKER_BUTTON_WIDTH, PICKER_ROW_HEIGHT)
        .with_pos(0, 3 * PICKER_ROW_HEIGHT);
    ok.set_shortcut(Shortcut::from_key(Key::Enter));
    let mut no = Button::default().with_label("Cancel (Esc)")
        .with_size(PICKER_BUTTON_WIDTH, PICKER_ROW_HEIGHT)
        .with_pos(PICKER_BUTTON_WIDTH, 3 * PICKER_ROW_HEIGHT);
    no.set_shortcut(Shortcut::from_key(Key::Escape));
    
    w.end();
    w.make_modal(true);
    w.show();
    
    let (tx, rx) = mpsc::channel::<Option<RGB>>();
    
    ok.set_callback({
        let r = rvalue.clone();
        let tx = tx.clone();
        move |_| { tx.send(Some(r.get())).unwrap(); }
    });
    no.set_callback({
        let tx = tx.clone();
        move |_| { tx.send(None).unwrap(); }
    });
       
    while {
        match rx.try_recv() {
            Err(_) => true,
            Ok(c) => {
                DoubleWindow::delete(w);
                return c;
                false
            }
        }
    } { fltk::app::wait(); }
    None
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn pick_a_color() {
        let a = fltk::app::App::default();
        let c = pick_color(RGB::BLACK);
        println!("{:?}", c);
        a.quit();
    }
}