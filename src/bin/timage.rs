/*!
Testing RGB image generation.

*/

use std::borrow::BorrowMut;
use std::cell::Cell;
use std::rc::Rc;
use std::sync::mpsc;
//~ use std::time::Duration,

use fltk::{
    prelude::*,
    app::App,
    button::Button,
    enums::ColorDepth,
    frame::Frame,
    image::RgbImage,
    valuator::{HorNiceSlider, ValueInput},
    window::{DoubleWindow, Window}
};

struct Evt;
impl Evt {
    const CHANGED: i32 = 40;
}

//~ const TICK = Duration::from_millis(100);

#[derive(Debug, Clone, Copy)]
struct RGB { r: f64, g: f64, b: f64, }

impl RGB {
    fn constrain(x: f64) -> f64 {
        if x < 0.0 { 0.0 }
        else if x > 255.0 { 255.0 }
        else { x }
    }
    
    pub fn new(newr: f64, newg: f64, newb: f64) -> RGB {
        RGB {
            r: RGB::constrain(newr),
            g: RGB::constrain(newg),
            b: RGB::constrain(newb),
        }
    }
    
    pub fn from_color(col: fltk::enums::Color) -> RGB {
        let (rbyte, gbyte, bbyte) = col.to_rgb();
        RGB::new(rbyte as f64, gbyte as f64, bbyte as f64)
    }
    
    pub fn to_color(&self) -> fltk::enums::Color {
        let rbyte = self.r as u8;
        let gbyte = self.g as u8;
        let bbyte = self.b as u8;
        
        fltk::enums::Color::from_rgb(rbyte, gbyte, bbyte)
    }
}

#[derive(Debug, Clone, Copy)]
struct ColorMessage {
    id: usize,
    color: RGB,
}

fn make_picker_row(ypos: i32, height: i32, lab: &'static str)
-> (Frame, HorNiceSlider, ValueInput) {
    let lab = Frame::new(0, ypos, 16, height, lab);
    let mut slider = HorNiceSlider::new(16, ypos, 256, height, None);
    let mut vinput = ValueInput::new(272, ypos, 64, height, None);
    
    slider.set_range(0.0, 255.0); //vinput.set_range(0.0, 255.0);
    vinput.set_bounds(0.0, 255.0);
    slider.set_step(1.0, 1); //vinput.set_step(1.0, 1);
    
    (lab, slider, vinput)
}

fn pick_color(col: RGB) -> Option<RGB> {
    let mut w = DoubleWindow::default()
        .with_size(400, 96)
        .with_label("specify a color");
        
    let (rlab, mut rslider, mut rvinput) = make_picker_row(0, 24, "R");
    let (glab, mut gslider, mut gvinput) = make_picker_row(24, 24, "G");
    let (blab, mut bslider, mut bvinput) = make_picker_row(48, 24, "B");
    rslider.set_value(col.r); rvinput.set_value(col.r);
    gslider.set_value(col.g); gvinput.set_value(col.g);
    bslider.set_value(col.b); bvinput.set_value(col.b);
    
    let mut prev = DoubleWindow::new(336, 0, 64, 96, None);
    prev.end();
    
    let mut ok = Button::new(0, 72, 168, 24, "Set");
    let mut no = Button::new(168, 72, 168, 24, "Cancel");
    
    w.end();
    w.make_modal(true);
    w.show();
    
    let mut get_rgb = {
        let rv = rslider.clone();
        let gv = gslider.clone();
        let bv = bslider.clone();
        move || {
            RGB::new(
                rv.value(),
                gv.value(),
                bv.value(),
            )
        }
    };
    
    let mut pc = prev.clone();
    let mut set_prev = {
        let mut grgb = get_rgb.clone();
        move || {
            let col = grgb();
            let fltkcol = col.to_color();
            pc.set_color(col.to_color());
            pc.redraw();
        }
    };
    
    set_prev();
    
    rslider.set_callback({
        let mut i = rvinput.clone(); let mut f = set_prev.clone();
        move |x| { i.set_value(x.value()); f(); }
    });
    rvinput.set_callback({
        let mut i = rslider.clone(); let mut f = set_prev.clone();
        move |x| { i.set_value(x.value()); f(); }
    });
    gslider.set_callback({
        let (mut i, mut f) = (gvinput.clone(), set_prev.clone());
        move |x| { i.set_value(x.value()); f(); }
    });
    gvinput.set_callback({
        let (mut i, mut f) = (gslider.clone(), set_prev.clone());
        move |x| { i.set_value(x.value()); f(); }
    });
    bslider.set_callback({
        let (mut i, mut f) = (bvinput.clone(), set_prev.clone());
        move |x| { i.set_value(x.value()); f(); }
    });
    bvinput.set_callback({
        let (mut i, mut f) = (bslider.clone(), set_prev.clone());
        move |x| { i.set_value(x.value()); f(); }
    });
    
    let mut picking = Rc::new(Cell::new(true));
    let mut rvalue: Rc<Cell<Option<RGB>>> = Rc::new(Cell::new(None));
    
    ok.set_callback({
        let mut p = picking.clone();
        let mut r = rvalue.clone();
        let mut grgb = get_rgb.clone();
        move |_| {
            p.set(false); 
            r.set(Some(grgb()));
        }
    });
    no.set_callback({
        let mut p = picking.clone();
        move |_| { p.set(false); }
    });
    
    while picking.get() { fltk::app::wait(); }
    
    DoubleWindow::delete(w);
    rvalue.get()
}


fn generate_image_data(v: &mut Vec<u8>) {
    v.clear();
    v.reserve_exact(256*256);
    for y in 0u8..=255 {
        let r = y;
        let b_half = y / 2;
        for x in 0u8..=255 {
            let g = x;
            let b = b_half + (x / 2);
            v.push(r);
            v.push(g);
            v.push(b);
        }
    }
}

fn main() {
    let a = App::default();
    let mut w = Window::default().with_size(256, 300);
    let mut frame = Frame::new(0, 0, 256, 256, None);
    let mut butt  = Button::new(0, 256, 64, 44, "Δ");
    let mut col_w = Window::new(64, 256, 92, 44, None);
    col_w.end();
    w.end();
    w.show();
    
    let mut v: Vec<u8> = Vec::new();
    generate_image_data(&mut v);
    let img = RgbImage::new(&v, 256, 256, ColorDepth::Rgb8).unwrap();
    frame.set_image(Some(img));
    
    butt.set_callback({
        let mut cw = col_w.clone();
        move |_| {
            let c = pick_color(RGB::from_color(cw.color()));
            if let Some(rgb) = c {
                cw.set_color(rgb.to_color());
                cw.redraw();
            }
        }
    });
    
    fltk::app::run();
}