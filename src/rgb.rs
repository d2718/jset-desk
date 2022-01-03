/*!
Dealing with colorspace.
*/

use std::cell::{Cell, RefCell};
use std::default::Default;
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

pub fn pick_color(col: RGB) -> Option<RGB> {
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

pub struct Gradient {
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
        st.set_minimum(0.0);
        // st.set_step(1.0, 1);
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
    
    pub fn set_pos(&mut self, x: i32, y: i32) { self.row.set_pos(x, y); }
    
    pub fn get_row(&self) -> &Pack { &self.row }
    
    pub fn get_from(&self) -> RGB { RGB::from_color(self.from.color()) }
    pub fn get_to(&self)   -> RGB { RGB::from_color(self.to.color()) }
    pub fn get_steps(&self) -> usize { 
        let v = self.steps.value();
        if v < 0.0 { 0usize }
        else { v as usize }
    }
}

impl Default for Gradient {
    fn default() -> Gradient {
        Gradient::new(RGB::black(), RGB::black())
    }
}

pub struct Pane {
    win: DoubleWindow,
    gradients: Vec<Gradient>,
    default_color: Button,
    me: Option<Rc<RefCell<Pane>>>,
}

const PANE_WIDTH: i32 = GRADIENT_TOTAL_WIDTH + 2 * GRADIENT_BUTTON_SIZE;
const REMOVE_BUTTON_XPOS: i32 = GRADIENT_TOTAL_WIDTH + GRADIENT_BUTTON_SIZE;

impl Pane {
    pub fn new() -> Rc<RefCell<Pane>> {
        let mut grads: Vec<Gradient> = Vec::new();
        let mut def_c = Button::default()
            .with_size(2 * GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE);
        def_c.set_color(RGB::black().to_color());
        
        let mut w = DoubleWindow::default().with_label("Gradients")
            .with_size(PANE_WIDTH, 3 * GRADIENT_BUTTON_SIZE);
        w.end();
        w.show();
        
        def_c.set_callback( |b| {
            let cur_col = b.color();
            if let Some(c) = pick_color(RGB::from_color(cur_col)) {
                b.set_color(c.to_color());
            }
        });
        
        let p = Pane {
            win: w,
            gradients: grads,
            default_color: def_c,
            me: None,
        };
        
        p.me = Some(Rc::new(RefCell::new(p)));
        
        p.me.unwrap().clone()
    }
    
    pub fn show(&mut self) {
        for grad in self.gradients.iter() {
            self.win.remove(grad.get_row());
        }
        self.win.remove(&self.default_color);
        self.win.clear();
        
        let pane_height = ((self.gradients.len() as i32) + 2) * GRADIENT_BUTTON_SIZE;
        self.win.set_size(PANE_WIDTH, pane_height);
        
        for (n, grad) in self.gradients.iter_mut().enumerate() {
            let y_pos = (n as i32) * GRADIENT_BUTTON_SIZE;
            let mut ib = Button::default().with_label("@+")
                .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
                .with_pos(0, y_pos);
            self.win.add(&ib);
            
            self.win.add(grad.get_row());
            grad.set_pos(GRADIENT_BUTTON_SIZE, y_pos);
            
            let mut rb = Button::default().with_label("X")
                .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
                .with_pos(REMOVE_BUTTON_XPOS, y_pos);
            self.win.add(&rb);
        }
        
        let add_row_ypos = (self.gradients.len() as i32) * GRADIENT_BUTTON_SIZE;
        let add_butt = Button::default().with_label("@+")
            .with_size(2 * GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
            .with_pos(0, add_row_ypos);
        self.win.add(&add_butt);
        let add_lab = Frame::default().with_label("append gradient")
            .with_size(GRADIENT_TOTAL_WIDTH, GRADIENT_BUTTON_SIZE)
            .with_pos(2 * GRADIENT_BUTTON_SIZE, add_row_ypos);
        self.win.add(&add_lab);
 
        let default_row_ypos = add_row_ypos + GRADIENT_BUTTON_SIZE;
        let def_lab = Frame::default().with_label("default color")
            .with_size(GRADIENT_TOTAL_WIDTH, GRADIENT_BUTTON_SIZE)
            .with_pos(0, default_row_ypos);
        self.win.add(&def_lab);
        self.default_color.set_pos(GRADIENT_TOTAL_WIDTH, default_row_ypos);
        self.win.add(&self.default_color);
        
        self.win.redraw();
    }
    
    pub fn push_gradient(&mut self, g: Gradient) {
        self.gradients.push(g);
        self.show();
    }
    
    pub fn insert_gradient(&mut self, n: usize) {
        if n >= self.gradients.len() {
            match self.gradients.last() {
                None => self.push_gradient(Gradient::default()),
                Some(lg) => self.push_gradient(Gradient::new(lg.get_to(), RGB::black())),
            }
        } else {
            let new_from = if n == 0 {
                RGB::black()
            } else {
                self.gradients[n-1].get_from()
            };
            let new_to = self.gradients[n].get_from();
            
            let ng = Gradient::new(new_from, new_to);
            self.gradients.insert(n, ng);
            self.show();
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn pick_a_color() {
        let a = fltk::app::App::default();
        let c = pick_color(RGB { r:0.0, g:0.0, b:0.0 });
        println!("color picked: {:?}", &c);
        a.quit();
    }
    
    #[test]
    fn make_gradient() {
        let a = fltk::app::App::default();
        let mut g = Gradient::new(RGB::black(), RGB::black());
        let mut w = DoubleWindow::default().with_size(400, 400);
        w.add(g.get_row());
        let mut b = Button::default().with_pos(100, 100)
            .with_size(64, 32).with_label("done");
        w.end();
        w.show();
        
        b.set_callback(move |_| { 
            println!("{:?} to {:?} in {} steps", g.get_from(), g.get_to(),
                                                 g.get_steps());
            a.quit();
        });
        a.run().unwrap();
    }
    
    #[test]
    fn make_pane() {
        let a = fltk::app::App::default();
        let mut p = Pane::new();
        let g = Gradient::new(RGB::black(), RGB::new(255.0, 255.0, 255.0));
        &p.push_gradient(g);
        
        a.run().unwrap();
    }
        
}