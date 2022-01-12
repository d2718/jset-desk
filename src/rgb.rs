/*!
Dealing with colorspace.

The `rgb` module contains types and methods to convert between different
color representations, as well as a collection of GUI elements for
specifying a color map.
*/

use std::cell::{Cell, RefCell};
use std::default::Default;
use std::rc::Rc;

use fltk::{
    prelude::*,
    button::Button,
    enums::{Align, Event, Key, Shortcut},
    frame::Frame,
    valuator::{HorNiceSlider, ValueInput},
    window::DoubleWindow,
};

const PICKER_ROW_HEIGHT:         i32 = 32;
const PICKER_LABEL_WIDTH:        i32 = 16;
const PICKER_SLIDER_WIDTH:       i32 = 256;
const PICKER_INPUT_WIDTH:        i32 = 64;
const PICKER_COLOR_WINDOW_WIDTH: i32 = 4 * PICKER_ROW_HEIGHT;

/**
Represents a color with red, green, and blue components as floating-point
numbers in the range (0.0, 255.0). This is the form in which it's easiest
to do calculations. Includes methods for converting to other useful data
representations.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB { r: f32, g: f32, b: f32 }

// For constraining arguments to `RGB::new()` to the proper range.
fn constrain_f32(x: f32) -> f32 {
    if x < 0.0 { 0.0 }
    else if x > 255.0 { 255.0 }
    else { x }
}

impl RGB {
    /** Instantiate a new `RGB` color representation with the given color
    component values. Values outside the accepted range will be constrained. */
    pub fn new(newr: f32, newg: f32, newb: f32) -> RGB {
        RGB {
            r: constrain_f32(newr),
            g: constrain_f32(newg),
            b: constrain_f32(newb),
        }
    }

    /** Convert from a color value used by `fltk`. */
    pub fn from_color(col: fltk::enums::Color) -> RGB {
        let (rbyte, gbyte, bbyte) = col.to_rgb();
        RGB::new(rbyte as f32, gbyte as f32, bbyte as f32)
    }
    
    /** Convert to a color value used by `fltk`. */
    pub fn to_color(&self) -> fltk::enums::Color {
        let rbyte = self.r as u8;
        let gbyte = self.g as u8;
        let bbyte = self.b as u8;
        
        fltk::enums::Color::from_rgb(rbyte, gbyte, bbyte)
    }
    
    /** Convert to a four-byte array of `[R, G, B, A]`. */
    pub fn to_rgba(&self) -> [u8; 4] {
        let rbyte = self.r as u8;
        let gbyte = self.g as u8;
        let bbyte = self.b as u8;
        
        [rbyte, gbyte, bbyte, 0xFF]
    }
    
    /** Convert to a three-byte array of `[R, G, B]`. */
    pub fn to_rgb8(&self) -> [u8; 3] {
        let rbyte = self.r as u8;
        let gbyte = self.g as u8;
        let bbyte = self.b as u8;
        
        [rbyte, gbyte, bbyte]
    }
    
    pub const BLACK:   RGB = RGB { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE:   RGB = RGB { r: 255.0, g: 255.0, b: 255.0 };
    pub const RED:     RGB = RGB { r: 255.0, g: 0.0, b: 0.0 };
    pub const GREEN:   RGB = RGB { r: 0.0, g: 255.0, b: 0.0 };
    pub const BLUE:    RGB = RGB { r: 0.0, g: 0.0, b: 255.0 };
    pub const YELLOW:  RGB = RGB { r: 255.0, g: 255.0, b: 0.0 };
    pub const CYAN:    RGB = RGB { r: 0.0, g: 255.0, b: 255.0 };
    pub const MAGENTA: RGB = RGB { r: 255.0, g: 0.0, b: 255.0 };
}

/**
Average the values of the colors in the given slice.

This function is used to display scaled-down versions of large images.
*/
pub fn color_average(dat: &[RGB]) -> RGB {
    let mut rtot = 0.0f32;
    let mut gtot = 0.0f32;
    let mut btot = 0.0f32;
    
    for pix in dat.iter() {
        rtot += pix.r;
        gtot += pix.g;
        btot += pix.b;
    }
    
    let n_float = dat.len() as f32;
    
    RGB::new(rtot/n_float, gtot/n_float, btot/n_float)
}

/**
An image, with pixels specified as `RGB` values.

Also contains width & height in number of pixels, and can expose its data
as a slice, or be consumed to give up the data as an owned `Vec`.
*/
pub struct FImageData {
    w: usize,
    h: usize,
    data: Vec<RGB>,
}

impl FImageData {
    pub fn new(width: usize, height: usize, dat: Vec<RGB>) -> FImageData {
        FImageData { w: width, h: height, data: dat }
    }
    
    pub fn width(&self)  -> usize { self.w }
    pub fn height(&self) -> usize { self.h }
    pub fn pixels(&self) -> &[RGB] { &self.data.as_slice() }
    pub fn to_data(self) -> Vec<RGB> { self.data }
}

/* This function is honestly just used to save typing in the body of
   `pick_color()` below. I need three almost-identical rows of widgets,
   and this function creates a row. */
fn make_picker_row(ypos: i32, lab: &'static str)
-> (Frame, HorNiceSlider, ValueInput) {
    let lab = Frame::new(0, ypos, PICKER_LABEL_WIDTH, PICKER_ROW_HEIGHT, lab);
    let mut slider = HorNiceSlider::new(PICKER_LABEL_WIDTH, ypos,
                        PICKER_SLIDER_WIDTH, PICKER_ROW_HEIGHT, None);
    let mut vinput = ValueInput::new(PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH,
                        ypos, PICKER_INPUT_WIDTH, PICKER_ROW_HEIGHT, None);
    
    slider.set_range(0.0, 255.0);
    vinput.set_bounds(0.0, 255.0);
    slider.set_step(1.0, 1);
    
    (lab, slider, vinput)
}

/**
Instantiates a modal RGB color picker window, then returns either
  * `Some(RGB)` if the user selects a color
  * `None` if the user clicks cancel
*/
pub fn pick_color(col: RGB) -> Option<RGB> {
    let mut w = DoubleWindow::default()
        .with_size(PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH
                   + PICKER_INPUT_WIDTH + PICKER_COLOR_WINDOW_WIDTH,
                   4 * PICKER_ROW_HEIGHT)
        .with_label("specify a color");
        
    let (_rlab, mut rslider, mut rvinput) = make_picker_row(0, "R");
    let (_glab, mut gslider, mut gvinput) = make_picker_row(PICKER_ROW_HEIGHT, "G");
    let (_blab, mut bslider, mut bvinput) = make_picker_row(2*PICKER_ROW_HEIGHT, "B");
    rslider.set_value(col.r as f64); rvinput.set_value(col.r as f64);
    gslider.set_value(col.g as f64); gvinput.set_value(col.g as f64);
    bslider.set_value(col.b as f64); bvinput.set_value(col.b as f64);
    
    let prev = DoubleWindow::new(
        PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH + PICKER_INPUT_WIDTH, 0,
        PICKER_COLOR_WINDOW_WIDTH, 4*PICKER_ROW_HEIGHT, None);
    prev.end();
    
    let butt_width = (PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH + PICKER_INPUT_WIDTH) / 2;
    let butt_ypos = 3 * PICKER_ROW_HEIGHT;
    let mut ok = Button::new(0, butt_ypos, butt_width, PICKER_ROW_HEIGHT, "Set @returnarrow");
    let mut no = Button::new(butt_width, butt_ypos, butt_width,
                                PICKER_ROW_HEIGHT, "Cancel (Esc)");
    
    w.end();
    w.make_modal(true);
    w.show();
    
    let get_rgb = {
        let rv = rslider.clone();
        let gv = gslider.clone();
        let bv = bslider.clone();
        move || {
            RGB::new(
                rv.value() as f32,
                gv.value() as f32,
                bv.value() as f32,
            )
        }
    };
    
    let mut pc = prev.clone();
    let mut set_prev = {
        let grgb = get_rgb.clone();
        move || {
            let col = grgb();
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
    
    let picking = Rc::new(Cell::new(true));
    let rvalue: Rc<Cell<Option<RGB>>> = Rc::new(Cell::new(None));
    
    ok.set_callback({
        let p = picking.clone();
        let r = rvalue.clone();
        let grgb = get_rgb.clone();
        move |_| {
            p.set(false); 
            r.set(Some(grgb()));
        }
    });
    ok.set_shortcut(Shortcut::from_key(Key::Enter));
    
    no.set_callback({
        let p = picking.clone();
        move |_| { p.set(false); }
    });
    no.set_shortcut(Shortcut::from_key(Key::Escape));
    
    while picking.get() { fltk::app::wait(); }
    
    DoubleWindow::delete(w);
    rvalue.get()
}

/**
Instantiates and wraps a UI element (rather, a collection of elements--a
`DoubleWindow`) that represents a single gradient in a color map. The "from"
color, the "to" color, and the number of steps between them are all editable.
*/
pub struct Gradient {
    row:   DoubleWindow,
    from:  Button,
    to:    Button,
    steps: ValueInput,
}

// width of a to/from color button; also the height of the entire row
const GRADIENT_BUTTON_SIZE: i32 = 32;
// width of the input for specifying the number of steps
const GRADIENT_STEPS_WIDTH: i32 = 64;
// calculated width of the entire window
const GRADIENT_TOTAL_WIDTH: i32 = (2 * GRADIENT_BUTTON_SIZE) + GRADIENT_STEPS_WIDTH;

impl Gradient {
    /** Instantiate a new `Gradient` element from and to the given colors. */
    pub fn new(from_col: RGB, to_col: RGB, n_steps: usize) -> Gradient {
        let rw = DoubleWindow::default()
            .with_size(GRADIENT_TOTAL_WIDTH, GRADIENT_BUTTON_SIZE);
        
        let mut fr = Button::default()
            .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
            .with_pos(0, 0);
        fr.set_color(from_col.to_color());
        fr.set_tooltip("start color");
        
        let mut st = ValueInput::default()
            .with_size(GRADIENT_STEPS_WIDTH, GRADIENT_BUTTON_SIZE)
            .with_pos(GRADIENT_BUTTON_SIZE, 0);
        st.set_value(n_steps as f64);
        st.set_minimum(0.0);
        st.set_tooltip("number of steps");
        
        let mut t  = Button::default()
            .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
            .with_pos(GRADIENT_BUTTON_SIZE + GRADIENT_STEPS_WIDTH, 0);
        t.set_color(to_col.to_color());
        t.set_tooltip("end color");
        
        rw.end();
        
        fr.set_callback(
            move |b| {
                let cur_col = b.color();
                if let Some(c) = pick_color(RGB::from_color(cur_col)) {
                    b.set_color(c.to_color());
                    b.redraw();
                }
            }
        );
        t.set_callback(
            move |b| {
                let cur_col = b.color();
                if let Some(c) = pick_color(RGB::from_color(cur_col)) {
                    b.set_color(c.to_color());
                    b.redraw();
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
    
    /// Set the position of the element in the containing window.
    pub fn set_pos(&mut self, x: i32, y: i32) { self.row.set_pos(x, y); }
    /// Explicitly show the element.
    ///
    /// This seems to be a kludgy requirement for getting proper behavior
    /// under Windows.
    pub fn show(&mut self) {
        // self.row.set_size(GRADIENT_TOTAL_WIDTH, GRADIENT_BUTTON_SIZE);
        self.row.show();
    }
    
    /// Get a reference to the UI element for adding to other collections.
    pub fn get_row(&self) -> &DoubleWindow { &self.row }
    
    /// Get the starting color.
    pub fn get_from(&self) -> RGB { RGB::from_color(self.from.color()) }
    /// Get the ending color.
    pub fn get_to(&self)   -> RGB { RGB::from_color(self.to.color()) }
    /// Get the number of steps from "start" to "end" color.
    pub fn get_steps(&self) -> usize { 
        let v = self.steps.value();
        if v < 0.0 { 0usize }
        else { v as usize }
    }
}

impl Default for Gradient {
    /// A `Default` `Gradient` goes from black to black.
    fn default() -> Gradient {
        Gradient::new(RGB::BLACK, RGB::BLACK, 256)
    }
}

/** Maps iterations-to-diverge to colors. */
pub struct ColorMap {
    /// mapping from iterations (`usize`) to color `RBG`
    data: Vec<RGB>,
    /// anything that iterates off the end of `data` gets this color
    default: RGB,
}

impl ColorMap {
    /**
    Instantiate a new `ColorMap` from the given slice of `Gradient`s and
    default color.
    */
    pub fn make(gradients: &[Gradient], def: RGB) -> ColorMap {
        let total_steps = gradients.iter().map(|g| g.get_steps()).sum();
        let mut new_data: Vec<RGB> = Vec::with_capacity(total_steps);
        
        for g in gradients.iter() {
            let c0 = g.get_from();
            let c1 = g.get_to();
            let delta_r = c1.r - c0.r;
            let delta_g = c1.g - c0.g;
            let delta_b = c1.b - c0.b;
            let gradient_steps = g.get_steps() as f32;
            for n in 0usize..g.get_steps() {
                let frac = (n as f32) / gradient_steps;
                let c = RGB::new(
                    c0.r + frac*delta_r,
                    c0.g + frac*delta_g,
                    c0.b + frac*delta_b,
                );
                new_data.push(c);
            }
        }
        
        ColorMap { data: new_data, default: def }
    }
    
    /**
    Return the `n`th color in the `ColorMap`, or the default color if there
    aren't that many colors.
    
    This function is meant to answer the question, "What color should a
    point that takes `n` iterations to diverge past the given limit be
    colored?"
    */
    pub fn get(&self, n: usize) -> RGB {
        match self.data.get(n) {
            None => self.default,
            Some(c) => *c,
        }
    }
    
    /** Return the total number of steps (and thus shades) in the map. */
    pub fn len(&self) -> usize { self.data.len() }
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct GradientSpec { from: RGB, to: RGB, steps: usize }
#[derive(Debug, Clone, PartialEq)]
pub struct ColorMapSpec { grads: Vec<GradientSpec> }

impl ColorMapSpec {
    pub fn empty() -> ColorMapSpec { ColorMapSpec { grads: Vec::new() } }
    
    pub fn len(&self) -> usize { self.grads.iter().map(|g| g.steps).sum() }
}

/**
Instantiates and wraps the UI window for specifying the color map. The
internals require references to the struct in order to function properly
(this is undoubtedly an anti-pattern), so the `new()` "constructor" returns
an `Rc<RefCell<Pane>>`.
*/
pub struct Pane {
    win: DoubleWindow,
    gradients: Vec<Gradient>,
    default_color: Button,
    me: Option<Rc<RefCell<Pane>>>,
}

// calculated width of the `Pane` window
const PANE_WIDTH: i32 = GRADIENT_TOTAL_WIDTH + 2 * GRADIENT_BUTTON_SIZE;
// another useful calculated with
const REMOVE_BUTTON_XPOS: i32 = GRADIENT_TOTAL_WIDTH + GRADIENT_BUTTON_SIZE;

impl Pane {
    /** Instantiate the color map UI and return an `Rc<RefCell<Pane>>` to
    the wrapping structure. */
    pub fn new() -> Rc<RefCell<Pane>> {
        let mut def_c = Button::default()
            .with_size(2 * GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE);
        def_c.set_color(RGB::BLACK.to_color());
        def_c.set_tooltip("set default color");
        
        let mut w = DoubleWindow::default().with_label("Color Map")
            .with_size(PANE_WIDTH, 4 * GRADIENT_BUTTON_SIZE);
        w.set_border(false);
        w.end();
        w.show();
        
        def_c.set_callback( |b| {
            let cur_col = b.color();
            if let Some(c) = pick_color(RGB::from_color(cur_col)) {
                b.set_color(c.to_color());
            }
        });
        
        let p = Rc::new(RefCell::new(Pane {
            win: w.clone(),
            gradients: Vec::new(),
            default_color: def_c,
            me: None,
        }));
        
        p.borrow_mut().me = Some(p.clone());
        
        /*
        Because
        
          * We don't want people clicking on the little `x` and closing
            the color map pane.
            
          * There seems to be a Win10 bug (as far as I can tell, it's a bug,
            because this doesn't happen on my Debian system) where dragging
            this particular window around by its title bar causes the window
            to resize, which messes up the size and layout of all the
            contained widgets.
        
        We have removed the "borders" from this window (which includes the
        title bar) and are implementing "click and drag", which seems to
        not manifest the bug.
        
        We are also pretending to handle the user hitting `Escape` because
        we don't want that to close the window, either.
        */
        w.handle({
            let (mut wx, mut wy) : (i32, i32) = (w.x(), w.y());
            let (mut x, mut y)   : (i32, i32) = (0, 0);
            move |w, evt| {
                match evt {
                    Event::Push => {
                        wx = w.x(); wy = w.y();
                        x = fltk::app::event_x(); y = fltk::app::event_y();
                        true
                    },
                    Event::Drag => {
                        let dx = fltk::app::event_x() - x;
                        let dy = fltk::app::event_y() - y;
                        wx = wx + dx;
                        wy = wy + dy;
                        w.set_pos(wx, wy);
                        true
                    },
                    Event::KeyDown => {
                        if fltk::app::event_key() == Key::Escape {
                            // pretend like we handled it
                            true
                        } else {
                            false
                        }
                    },
                    _ => false,
                }
            }
        });
        
        p
    }
    
    /** Instantiate the UI with the "default" color map, which is a single
    gradient of 256 steps from black to white. */
    pub fn default() -> Rc<RefCell<Pane>> {
        let p = Pane::new();
        p.borrow_mut().default_color.set_color(fltk::enums::Color::White);
        p.borrow_mut().insert_gradient(0);
        
        p
    }
    
    /**
    Redraw the UI when necessary.
    
    This is generally when a `Gradient` is added or removed.
    */
    fn show(&mut self) {
        for grad in self.gradients.iter() {
            self.win.remove(grad.get_row());
        }
        self.win.remove(&self.default_color);
        self.win.clear();
        
        let pane_height = ((self.gradients.len() as i32) + 3) * GRADIENT_BUTTON_SIZE;
        self.win.set_size(PANE_WIDTH, pane_height);
        
        let lab = Frame::default().with_label("Color Map")
            .with_size(PANE_WIDTH, GRADIENT_BUTTON_SIZE)
            .with_pos(0, 0);
        self.win.add(&lab);
        
        for (n, grad) in self.gradients.iter_mut().enumerate() {
            let y_pos = (n as i32 + 1) * GRADIENT_BUTTON_SIZE;
            let mut ib = Button::default().with_label("@+")
                .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
                .with_pos(0, y_pos);
            ib.set_tooltip("insert new gradient");
            self.win.add(&ib);
            
            self.win.add(grad.get_row());
            grad.set_pos(GRADIENT_BUTTON_SIZE, y_pos);
            grad.show();
            
            let mut rb = Button::default().with_label("X")
                .with_size(GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
                .with_pos(REMOVE_BUTTON_XPOS, y_pos);
            rb.set_tooltip("remove this gradient");
            self.win.add(&rb);
            
            ib.set_callback({
                let p = self.me.as_ref().unwrap().clone();
                move |_| { p.borrow_mut().insert_gradient(n); }
            });
            rb.set_callback({
                let p = self.me.as_ref().unwrap().clone();
                move |_| {
                    p.borrow_mut().gradients.remove(n);
                    p.borrow_mut().show();
                }
            });
        }
        
        let add_row_ypos = (self.gradients.len() as i32 + 1) * GRADIENT_BUTTON_SIZE;
        let mut add_butt = Button::default().with_label("@+")
            .with_size(2 * GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE)
            .with_pos(0, add_row_ypos);
        add_butt.set_tooltip("append new gradient");
        self.win.add(&add_butt);
        add_butt.set_callback({
            let p = self.me.as_ref().unwrap().clone();
            move |_| {
                let n = p.borrow().gradients.len();
                p.borrow_mut().insert_gradient(n)
            }
        });
            
        let add_lab = Frame::default().with_label("append gradient")
            .with_size(GRADIENT_TOTAL_WIDTH, GRADIENT_BUTTON_SIZE)
            .with_pos(2 * GRADIENT_BUTTON_SIZE, add_row_ypos);
        self.win.add(&add_lab);
 
        let default_row_ypos = add_row_ypos + GRADIENT_BUTTON_SIZE;
        self.default_color.set_pos(GRADIENT_TOTAL_WIDTH, default_row_ypos);
        self.default_color.set_size(2 * GRADIENT_BUTTON_SIZE, GRADIENT_BUTTON_SIZE);
        self.default_color.set_label("default color");
        self.default_color.set_align(Align::Left);
        self.win.add(&self.default_color);
        
        self.win.redraw();
    }
    
    /**
    Inserts a new `Gradient` at position `n`, moving the `Gradient` there and
    all subsequent positions forward. If `n` is past the end of the `Gradient`
    vector, it will be appended to the end. Starting and ending colors of the
    inserted `Gradient` will be automatically set to match up with adjacent
    gradients.
    */
    fn insert_gradient(&mut self, n: usize) {
        if n >= self.gradients.len() {
            let new_to = RGB::from_color(self.default_color.color());
            let new_from = match self.gradients.last() {
                None => RGB::BLACK,
                Some(lg) => lg.get_to(),
            };
            let new_g = Gradient::new(new_from, new_to, 256);
            self.gradients.push(new_g);
        } else {
            let new_from = if n == 0 {
                RGB::BLACK
            } else {
                self.gradients[n-1].get_to()
            };
            let new_to = self.gradients[n].get_from();
            
            let ng = Gradient::new(new_from, new_to, 256);
            self.gradients.insert(n, ng);
        }
        self.show();
    }
    
    /**
    Using the contained `Gradient`s and default color, generate a `ColorMap`
    that can be used to color iteration maps.
    */
    pub fn generate_color_map(&self) -> ColorMap {
        ColorMap::make(
            &self.gradients,
            RGB::from_color(self.default_color.color())
        )
    }
    
    pub fn get_spec(&self) -> ColorMapSpec {
        let gspex: Vec<GradientSpec> = self.gradients.iter().map(
            |g| GradientSpec {
                from: g.get_from(),
                to: g.get_to(),
                steps: g.get_steps(),
            }
        ).collect();
        
        ColorMapSpec { grads: gspex }
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
        let g = Gradient::new(RGB::black(), RGB::black(), 256);
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
        let _ = Pane::default();
        
        a.run().unwrap();
    }
}