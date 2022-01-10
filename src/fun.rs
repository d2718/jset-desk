/*!
Specifying the iteration function and parameters.
*/

use std::cell::RefCell;
use std::rc::Rc;

use fltk::{
    prelude::*,
    button::Button,
    enums::{Align, Font},
    frame::Frame,
    group::Pack,
    menu::Choice,
    valuator::ValueInput,
    window::DoubleWindow,
};

use crate::cx::Cx;
use crate::iter::IterParams;

const ROW_HEIGHT: i32 = 32;
const DEGREE_LABEL_WIDTH: i32 = 48;
const VAR_LABEL_WIDTH: i32 = 16;
const VAR_INPUT_WIDTH: i32 = 72;
pub const ROW_WIDTH: i32 = DEGREE_LABEL_WIDTH + (4 * VAR_LABEL_WIDTH) + (2 * VAR_INPUT_WIDTH);

const MATH_FONT: Font = Font::HelveticaItalic;

/**
Instantiates and wraps a chunk of UI elements for specifying a complex
polynomial coefficient in polar form. The `new()` constructor takes the
degree of the coefficient (for displaying the appropriate label) and
initial values of _r_ and _𝜃_.
*/
struct Coef {
    row: Pack,
    rinput: ValueInput,
    tinput: ValueInput,
}

impl Coef {
    /** Create a new `fltk::group::Pack` for specifying the modulus and
    phase of a complex polynomial coefficient. `term` sets the label, 
    `r` and `t` supply initial values of _r_ and _𝜃_. */
    pub fn new(term: &str, r: f64, t: f64) -> Coef {
        
        let mut rw = Pack::default().with_size(ROW_WIDTH, ROW_HEIGHT);
        rw.set_type(fltk::group::PackType::Horizontal);
        rw.end();
        
        let mut deg_lab = Frame::default()
            .with_size(DEGREE_LABEL_WIDTH, ROW_HEIGHT);
        deg_lab.set_label_font(MATH_FONT);
        deg_lab.set_label(term);
        
        let mut rlab = Frame::default().with_label("r:")
            .with_size(VAR_LABEL_WIDTH, ROW_HEIGHT);
        rlab.set_label_font(MATH_FONT);
        
        let mut r_input = ValueInput::default()
            .with_size(VAR_INPUT_WIDTH, ROW_HEIGHT);
        r_input.set_tooltip(&format!("modulus of {} coefficient", term));
        r_input.set_value(r);
        
        let spacer = Frame::default().with_size(VAR_LABEL_WIDTH, ROW_HEIGHT); 
        
        let tlab = Frame::default().with_label("𝜃:")
            .with_size(VAR_LABEL_WIDTH, ROW_HEIGHT);
        
        let mut t_input = ValueInput::default()
            .with_size(VAR_INPUT_WIDTH, ROW_HEIGHT);
        t_input.set_tooltip(&format!("phase of {} coefficient", term));
        t_input.set_value(t);
        
        let pilab = Frame::default().with_label("𝜋")
            .with_size(VAR_LABEL_WIDTH, ROW_HEIGHT);
        
        rw.add(&deg_lab);
        rw.add(&rlab);
        rw.add(&r_input);
        rw.add(&spacer);
        rw.add(&tlab);
        rw.add(&t_input);
        rw.add(&pilab);
        
        Coef {
            row: rw.clone(),
            rinput: r_input.clone(),
            tinput: t_input.clone(),
        }
    }
    
    /** Get a reference to the underlying `Pack` so it can be inserted into
    a larger UI element. */
    pub fn get_row(&self) -> &Pack { &self.row }
    pub fn get_mut_row(&mut self) -> &mut Pack { &mut self.row }
    
    /** Get the specified coefficient value. */
    pub fn get_value(&self) -> Cx {
        let r = self.rinput.value();
        let t = self.tinput.value() * std::f64::consts::PI;
        Cx::polar(r, t)
    }
    
    /** Helper function for generating a label for the coefficient of
    the `degree` term. */
    pub fn term_label(degree: usize) -> String {
        match degree {
            0 => "c".to_string(),
            1 => "z".to_string(),
            n @ _ => format!("z^{}", n),
        }
    }
}
 
const DEFAULT_PANE_HEIGHT: i32 = ROW_HEIGHT * 11;
const SELECTOR_WIDTH: i32 = 192;
 
pub struct Pane {
    selector: Choice,
    pm_a:     Coef,
    pm_b:     Coef,
    coefs:    Rc<RefCell<Vec<Coef>>>,
}

impl Pane {
    pub fn new() -> Pane {
        let mut w = DoubleWindow::default().with_label("Iterator Options")
            .with_size(ROW_WIDTH, DEFAULT_PANE_HEIGHT);
        
        let mut sel = Choice::default().with_label("Iterator")
            .with_size(SELECTOR_WIDTH, ROW_HEIGHT)
            .with_pos(ROW_WIDTH - SELECTOR_WIDTH, 0);
        sel.add_choice("Mandlebrot|Pseudo-Mandlebrot|Polynomial");
        sel.set_value(0);
        
        let mut pw = DoubleWindow::default()
            .with_size(ROW_WIDTH, 3 * ROW_HEIGHT)
            .with_pos(0, ROW_HEIGHT);
        let mut pw_label = Frame::default().with_size(ROW_WIDTH, ROW_HEIGHT)
            .with_pos(0, 0).with_label("az^2 + bc");
        pw_label.set_label_font(MATH_FONT);
        let mut a = Coef::new("a", 1.0, 0.0);
        a.get_mut_row().set_pos(0, ROW_HEIGHT);
        let mut b = Coef::new("b", 1.0, 0.0);
        b.get_mut_row().set_pos(0, 2 * ROW_HEIGHT);
        pw.end();
        pw.deactivate();
        
        let mut cs: Vec<Coef> = Vec::new();
        
        let mut pyw = DoubleWindow::default()
            .with_size(ROW_WIDTH, 7 * ROW_HEIGHT)
            .with_pos(0, 4 * ROW_HEIGHT);
        let _ = Frame::default().with_size(ROW_WIDTH, ROW_HEIGHT)
            .with_label("Polynomial Coefficients").with_pos(0, 0);
        let mut c = Coef::new(&Coef::term_label(0), 0.7, 0.63);
        c.get_mut_row().set_pos(0, ROW_HEIGHT);
        cs.push(c);
        let mut c = Coef::new(&Coef::term_label(1), 0.0, 0.0);
        c.get_mut_row().set_pos(0, 2 * ROW_HEIGHT);
        cs.push(c);
        let mut c = Coef::new(&Coef::term_label(2), 1.0, 0.0);
        c.get_mut_row().set_pos(0, 3 * ROW_HEIGHT);
        cs.push(c);
        let mut coeff_del = Button::default()
            .with_pos(ROW_HEIGHT, 5 * ROW_HEIGHT)
            .with_size(ROW_HEIGHT, ROW_HEIGHT)
            .with_label("decrease degree").with_align(Align::Right);
        let mut coeff_add = Button::default()
            .with_pos(ROW_WIDTH - (2 * ROW_HEIGHT), 6 * ROW_HEIGHT)
            .with_size(ROW_HEIGHT, ROW_HEIGHT)
            .with_label("increase degree").with_align(Align::Left);
        pyw.end();
        pyw.deactivate();
        
        let cs = Rc::new(RefCell::new(cs));
        
        sel.set_callback({
            let mut pw = pw.clone();
            let mut pyw = pyw.clone();
            move |x| match x.value() {
                0 => { pw.deactivate(); pyw.deactivate(); },
                1 => { pw.activate(); pyw.deactivate(); },
                2 => { pw.deactivate(); pyw.activate(); },
                n @ _ => { eprintln!("Pane::selector callback illegal value: {}", n); },
            }
        });
        
        coeff_del.set_callback({
            let mut win = w.clone();
            let mut pyw = pyw.clone();
            let mut ob  = coeff_add.clone();
            let cs = cs.clone();
            move |b| {
                if cs.borrow().len() > 1 {
                    let dc = cs.borrow_mut().pop().unwrap();
                    pyw.remove(dc.get_row());
                    let (w, h) = (pyw.w(), pyw.h());
                    pyw.set_size(w, h - ROW_HEIGHT);
                    let h = win.h();
                    win.set_size(w, h - ROW_HEIGHT);
                    let (x, y) = (b.x(), b.y());
                    b.set_pos(x, y - ROW_HEIGHT);
                    let (x, y) = (ob.x(), ob.y());
                    ob.set_pos(x, y - ROW_HEIGHT);
                    Pack::delete(dc.row);
                }   
            }
        });
        
        coeff_add.set_callback({
            let mut win = w.clone();
            let mut pyw = pyw.clone();
            let mut ob  = coeff_del.clone();
            let cs = cs.clone();
            move |b| {
                let ncoeffs = cs.borrow().len();
                let y_pos = (ncoeffs + 1) as i32 * ROW_HEIGHT;
                let mut new_coeff = Coef::new(&Coef::term_label(ncoeffs), 0.0, 0.0);
                pyw.add(new_coeff.get_row());
                new_coeff.get_mut_row().set_pos(0, y_pos);
                cs.borrow_mut().push(new_coeff);
                let (w, h) = (win.w(), win.h());
                win.set_size(w, h + ROW_HEIGHT);
                let (w, h) = (pyw.w(), pyw.h());
                pyw.set_size(w, h + ROW_HEIGHT);
                let (x, y) = (b.x(), b.y());
                b.set_pos(x, y + ROW_HEIGHT);
                let (x, y) = (ob.x(), ob.y());
                ob.set_pos(x, y + ROW_HEIGHT);
            }
        });
        
        w.end();
        w.show();
        
        let p = Pane {
            selector: sel.clone(),
            pm_a:     a,
            pm_b:     b,
            coefs:    cs,
        };
        
        p
    }
    
    pub fn get_params(&self) -> IterParams {
        match self.selector.value() {
            0 => IterParams::Mandlebrot,
            1 => IterParams::PseudoMandlebrot(
                self.pm_a.get_value(),
                self.pm_b.get_value()
            ),
            2 => IterParams::Polynomial(
                self.coefs.borrow().iter().map(|c| c.get_value()).collect()
            ),
            n @ _ => {
                eprintln!("Illegal iterator input value: {}", &n);
                IterParams::Mandlebrot
            },
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn make_coef() {
        let a = fltk::app::App::default();
        let mut v: Vec::<Coef> = Vec::new();
        for n in 0usize..3 {
            let lab = Coef::term_label(n);
            let c = Coef::new(&lab, 1.0, 0.0);
            v.push(c);
        }
        let mut w = DoubleWindow::default().with_size(ROW_WIDTH, 400)
            .with_label("struct `Coef` test");
        let mut p = Pack::default().size_of_parent().center_of_parent();
        p.end();
        
        for c in v.iter() { p.add(c.get_row()); }
        
        let mut b = fltk::button::Button::default().with_label("valz")
            .with_size(32, 32);
        
        b.set_callback(move |_| {
            for (n, c) in v.iter().enumerate() {
                println!("{}: {:?}", n, c.get_value());
            }
        });
        p.add(&b);
        w.show();
        a.run().unwrap();
    }
    
    #[test]
    fn make_fun_pane() {
        let a = fltk::app::App::default();
        let p = Pane::new();
        
        a.run().unwrap();
    }
}