/*!
Dealing with colorspace.
*/

use std::cell::Cell;
use std::rc::Rc;

use fltk::{
    prelude::*,
    button::Button,
    frame::Frame,
    group::Pack,
    valuator::{HorNiceSlider, ValueInput},
    window::DoubleWindow,
};

const PICKER_ROW_HEIGHT:         i32 = 32;
const PICKER_LABEL_WIDTH:        i32 = 16;
const PICKER_SLIDER_WIDTH:       i32 = 256;
const PICKER_INPUT_WIDTH:        i32 = 64;
const PICKER_COLOR_WINDOW_WIDTH: i32 = 4 * PICKER_ROW_HEIGHT;

#[derive(Debug, Clone, Copy)]
pub struct RGB { r: f64, g: f64, b: f64 }

fn constrain_f64(x: f64) -> f64 {
    if x < 0.0 { 0.0 }
    else if x > 255.0 { 255.0 }
    else { x }
}

impl RGB {
    pub fn new(newr: f64, newg: f64, newb: f64) -> RGB {
        RGB {
            r: constrain_f64(newr),
            g: constrain_f64(newg),
            b: constrain_f64(newb),
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
    
    pub fn black() -> RGB {
        RGB { r:0.0, g:0.0, b:0.0 }
    }
}

fn make_picker_row(ypos: i32, lab: &'static str)
-> (Frame, HorNiceSlider, ValueInput) {
    let lab = Frame::new(0, ypos, PICKER_LABEL_WIDTH, PICKER_ROW_HEIGHT, lab);
    let mut slider = HorNiceSlider::new(PICKER_LABEL_WIDTH, ypos,
                        PICKER_SLIDER_WIDTH, PICKER_ROW_HEIGHT, None);
    let mut vinput = ValueInput::new(PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH,
                        ypos, PICKER_INPUT_WIDTH, PICKER_ROW_HEIGHT, None);
    
    slider.set_range(0.0, 255.0); //vinput.set_range(0.0, 255.0);
    vinput.set_bounds(0.0, 255.0);
    slider.set_step(1.0, 1); //vinput.set_step(1.0, 1);
    
    (lab, slider, vinput)
}

fn pick_color(col: RGB) -> Option<RGB> {
    let mut w = DoubleWindow::default()
        .with_size(PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH
                   + PICKER_INPUT_WIDTH + PICKER_COLOR_WINDOW_WIDTH,
                   4 * PICKER_ROW_HEIGHT)
        .with_label("specify a color");
        
    let (rlab, mut rslider, mut rvinput) = make_picker_row(0, "R");
    let (glab, mut gslider, mut gvinput) = make_picker_row(PICKER_ROW_HEIGHT, "G");
    let (blab, mut bslider, mut bvinput) = make_picker_row(2*PICKER_ROW_HEIGHT, "B");
    rslider.set_value(col.r); rvinput.set_value(col.r);
    gslider.set_value(col.g); gvinput.set_value(col.g);
    bslider.set_value(col.b); bvinput.set_value(col.b);
    
    let mut prev = DoubleWindow::new(
        PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH + PICKER_INPUT_WIDTH, 0,
        PICKER_COLOR_WINDOW_WIDTH, 4*PICKER_ROW_HEIGHT, None);
    prev.end();
    
    let butt_width = (PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH + PICKER_INPUT_WIDTH) / 2;
    let butt_ypos = 3 * PICKER_ROW_HEIGHT;
    let mut ok = Button::new(0, butt_ypos, butt_width, PICKER_ROW_HEIGHT, "Set");
    let mut no = Button::new(butt_width, butt_ypos, butt_width,
                                PICKER_ROW_HEIGHT, "Cancel");
    
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

pub struct Pane {
    win: DoubleWindow,
}

struct Gradient {
    row: Pack,
    from: Button,
    to: Button,
    steps: ValueInput,
}

const GRADIENT_BUTTON_SIZE: i32 = 32;
const GRADIENT_STEPS_WIDTH: i32 = 64;
const GRADIENT_TOTAL_WIDTH: i32 = (2 * GRADIENT_BUTTON_SIZE) + GRADIENT_STEPS_WIDTH;

impl Gradient {
    pub fn new(from_col: RGB, to_col: RGB) -> Gradient {
        let mut rw = Pack::default()
            .with_size(GRADIENT_TOTAL_WIDTH, GRADIENT_BUTTON_SIZE);
        rw.set_type(fltk::group::PackType::Horizontal);
        rw.end();
        let mut fr = Button::default()
            .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE);
        fr.set_color(from_col.to_color());
        let mut st = ValueInput::default()
            .with_size(GRADIENT_STEPS_WIDTH, GRADIENT_BUTTON_SIZE);
        st.set_value(256.0);
        let mut t  = Button::default()
            .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE);
        t.set_color(to_col.to_color());
        
        rw.add(&fr); rw.add(&st); rw.add(&t);
        
        fr.set_callback(
            move |b| {
                let cur_col = b.color();
                if let Some(c) = pick_color(RGB::from_color(cur_col)) {
                    b.set_color(c.to_color());
                }
            }
        );
        t.set_callback(
            move |b| {
                let cur_col = b.color();
                if let Some(c) = pick_color(RGB::from_color(cur_col)) {
                    b.set_color(c.to_color());
                }
            }
        );
        
        Gradient {
            row: rw.clone(),
            from: fr.clone(),
            to: t.clone(),
            steps: st.clone(),
        }
    }
    
    pub fn get_row(&self) -> &Row { &self.row }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn pick_a_color() {
        let a = fltk::app::App::default();
        let c = pick_color(RGB { r:0.0, g:0.0, b:0.0 });
        println!("color picked: {:?}", &c);
    }
    
    #[test]
    fn make_gradient() {
        let a = fltk::app::App::default();
        let mut g = Gradient::new(RGB::black(), RGB::black());
        let mut w = DoubleWindow::default().with_size(400, 400);
        w.add(g.get_row());
        let mut b = Button::default().with_pos(200, 200).with_label("done");
        w.end();
        
        b.set_callback(move |_| { fltk::app::quit(); });
        a.run().unwrap();
    }
        
}