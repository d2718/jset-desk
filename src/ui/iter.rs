/*!
The pane for specifying the `image::IterType` and attendant functionality.
*/

use std::cell::RefCell;
use std::rc::Rc;

use fltk::{
    prelude::*,
    button::Button,
    enums::Font,
    frame::Frame,
    group::{Pack, PackType},
    menu::Choice,
    valuator::ValueInput,
    window::DoubleWindow,
};

use crate::cx::Cx;
use crate::image::*;
use super::*;

// Labels that are mathematical variable symbols get typeset in this.
const MATH_FONT: Font = Font::HelveticaItalic;

// The following constants express dimensions of elements of the
// `CoefSpecifier`
const COEF_ROW_HEIGHT:   i32 = 32;
const COEF_DEGREE_WIDTH: i32 = 48;
const COEF_VAR_WIDTH:    i32 = 16;
const COEF_INPUT_WIDTH:  i32 = 72;
const COEF_ROW_WIDTH:    i32 = COEF_DEGREE_WIDTH + (4 * COEF_VAR_WIDTH)
    + (2 * COEF_INPUT_WIDTH);

/*
A wrapped collection of UI elements for specifying a complex coefficient in
polar form.
*/
struct CoefSpecifier {
    row: Pack,
    rinput: ValueInput,
    tinput: ValueInput,
}

impl CoefSpecifier {
    // Construct a new `CoefSpecifier` with the given term label and initial
    // values of `r` and `t`heta.
    pub fn new(term: &str, r: f64, t: f64) -> CoefSpecifier {
        let mut rw = Pack::default().with_size(COEF_ROW_WIDTH, COEF_ROW_HEIGHT);
        rw.set_type(PackType::Horizontal);
        rw.end();
        
        let mut deg_lab = Frame::default()
            .with_size(COEF_DEGREE_WIDTH, COEF_ROW_HEIGHT);
        deg_lab.set_label_font(MATH_FONT);
        deg_lab.set_label(term);
        
        let mut rlab = Frame::default().with_label("r:")
            .with_size(COEF_VAR_WIDTH, COEF_ROW_HEIGHT);
        rlab.set_label_font(MATH_FONT);
        
        let mut r_input = ValueInput::default()
            .with_size(COEF_INPUT_WIDTH, COEF_ROW_HEIGHT);
        r_input.set_tooltip(&format!("modulus of {} coefficient", term));
        r_input.set_value(r);
        
        let spacer = Frame::default().with_size(COEF_VAR_WIDTH, COEF_ROW_HEIGHT);
        
        let tlab = Frame::default().with_label("𝜃:")
            .with_size(COEF_VAR_WIDTH, COEF_ROW_HEIGHT);
        
        let mut t_input = ValueInput::default()
            .with_size(COEF_INPUT_WIDTH, COEF_ROW_HEIGHT);
        t_input.set_tooltip(&format!("phase of {} coefficient", term));
        t_input.set_value(t);
        
        let pilab = Frame::default().with_label("𝜋")
            .with_size(COEF_VAR_WIDTH, COEF_ROW_HEIGHT);
        
        rw.add(&deg_lab);
        rw.add(&rlab);
        rw.add(&r_input);
        rw.add(&spacer);
        rw.add(&tlab);
        rw.add(&t_input);
        rw.add(&pilab);
        
        CoefSpecifier {
            row: rw.clone(),
            rinput: r_input.clone(),
            tinput: t_input.clone(),
        }
    }
    
    // Expose a reference to the underlying UI element group so that it
    // can be added to other collections of elements.
    pub fn get_row(&self) -> &Pack { &self.row }
    pub fn get_mut_row(&mut self) -> &mut Pack { &mut self.row }
    
    // Get the complex coefficient specified by.
    pub fn get_value(&self) -> Cx {
        let r = self.rinput.value();
        let t = self.tinput.value() * std::f64::consts::PI;
        Cx::polar(r, t)
    }
    
    // An associated function for generating names for terms of a complex
    // polynomial based on term degree.
    pub fn term_label(degree: usize) -> String {
        match degree {
            0 => "c".to_string(),
            1 => "z".to_string(),
            n @ _ => format!("z^{}", n),
        }
    }
}

// Specifying the sizes of the UI elements of the `IterPane`'s window.
const COEF_BUTTON_WIDTH:        i32 = 32;
const INITIAL_ITER_PANE_HEIGHT: i32 = COEF_ROW_HEIGHT * 12;
const ITER_SELECTOR_WIDTH:      i32 = 192;

static DEFAULT_COEFS: [[f64; 2]; 3] = [ 
    [0.7, 0.63],
    [0.0, 0.0],
    [1.0, 0.0],
];

/**
This struct holds and manages the UI elements for specifying an image's
`image::IterType`.
*/
pub struct IterPane {
    selector: Choice,
    pm_a:     CoefSpecifier,
    pm_b:     CoefSpecifier,
    coefs:    Rc<RefCell<Vec<CoefSpecifier>>>,
}

impl IterPane {
    /**
    Instantiate a new `IterPane`. By default these have `IterType::Mandlebrot`
    selected.
    */
    pub fn new() -> IterPane {
        let mut w = DoubleWindow::default()
            .with_size(COEF_ROW_WIDTH, INITIAL_ITER_PANE_HEIGHT);
        w.set_border(false);
        
        let _lab = Frame::default().with_label("Iterator Options")
            .with_size(COEF_ROW_WIDTH, COEF_ROW_HEIGHT).with_pos(0, 0);
        
        let mut sel = Choice::default().with_label("Iterator")
            .with_size(ITER_SELECTOR_WIDTH, COEF_ROW_HEIGHT)
            .with_pos(COEF_ROW_WIDTH - ITER_SELECTOR_WIDTH, COEF_ROW_HEIGHT);
        sel.add_choice("Mandlebrot|Pseudo-Mandlebrot|Polynomial");
        sel.set_value(0);
        
        let mut pw = DoubleWindow::default()
            .with_size(COEF_ROW_WIDTH, 3 * COEF_ROW_HEIGHT)
            .with_pos(0, 2 * COEF_ROW_HEIGHT);
        let mut pw_label = Frame::default().with_pos(0, 0)
            .with_size(COEF_ROW_WIDTH, COEF_ROW_HEIGHT).with_label("az^2 + bc");
        pw_label.set_label_font(MATH_FONT);
        let mut a = CoefSpecifier::new("a", 1.0, 0.0);
        a.get_mut_row().set_pos(0, COEF_ROW_HEIGHT);
        let mut b = CoefSpecifier::new("b", 1.0, 0.0);
        b.get_mut_row().set_pos(0, COEF_ROW_HEIGHT * 2);
        pw.end();
        pw.deactivate();
        
        let mut cs: Vec<CoefSpecifier> = Vec::new();
        
        let mut pyw = DoubleWindow::default()
            .with_size(COEF_ROW_WIDTH, 7 * COEF_ROW_HEIGHT)
            .with_pos(0, 5 * COEF_ROW_HEIGHT);
        let _ = Frame::default().with_size(COEF_ROW_WIDTH, COEF_ROW_HEIGHT)
            .with_label("Polynomial Coefficients").with_pos(0, 0);
        let _ = Frame::default().with_pos(0, COEF_ROW_HEIGHT)
            .with_size(COEF_ROW_WIDTH - COEF_BUTTON_WIDTH, COEF_ROW_HEIGHT)
            .with_label("decrease degree");
        let _ = Frame::default().with_pos(COEF_BUTTON_WIDTH, 2*COEF_ROW_HEIGHT)
            .with_size(COEF_ROW_WIDTH - COEF_BUTTON_WIDTH, COEF_ROW_HEIGHT)
            .with_label("increase degree");
        
        let mut coef_add = Button::default().with_label("@+")
            .with_size(COEF_BUTTON_WIDTH, COEF_ROW_HEIGHT)
            .with_pos(0, 2 * COEF_ROW_HEIGHT);
        coef_add.set_tooltip("add a z^3 coefficient");
        let mut coef_del = Button::default().with_label("@line")
            .with_pos(COEF_ROW_WIDTH - COEF_BUTTON_WIDTH, COEF_ROW_HEIGHT)
            .with_size(COEF_BUTTON_WIDTH, COEF_ROW_HEIGHT);
        coef_del.set_tooltip("remove the z^2 coefficient");
        
        for n in 0usize..3 {
            let mut c = CoefSpecifier::new(
                &CoefSpecifier::term_label(n),
                DEFAULT_COEFS[n][0],
                DEFAULT_COEFS[n][1]
            );
            c.get_mut_row().set_pos(0, (n as i32 + 3) * COEF_ROW_HEIGHT);
            cs.push(c);
        }
        pyw.end();
        pyw.deactivate();
        
        w.end();
        w.show();
        
        setup_subwindow_behavior(&mut w);
        
        let cs = Rc::new(RefCell::new(cs));
        
        sel.set_callback({
            let mut pw = pw.clone();
            let mut pyw = pyw.clone();
            move |s| match s.value() {
                0 => { pw.deactivate(); pyw.deactivate(); },
                1 => { pw.activate();   pyw.deactivate(); },
                2 => { pw.deactivate(); pyw.activate();   },
                n @ _ => { eprintln!("IterPane::selector callback illegal value: {}", n); },
            }
        });
        
        coef_del.set_callback({
            let mut win = w.clone();
            let mut pyw = pyw.clone();
            let mut ob  = coef_add.clone();
            let cs = cs.clone();
            move |b| {
                if cs.borrow().len() > 1 {
                    let old_spec = cs.borrow_mut().pop().unwrap();
                    pyw.remove(old_spec.get_row());
                    let (w, h) = (pyw.w(), pyw.h());
                    pyw.set_size(w, h - COEF_ROW_HEIGHT);
                    let h = win.h();
                    win.set_size(w, h - COEF_ROW_HEIGHT);
                    Pack::delete(old_spec.row);
                }
                
                let n = cs.borrow().len();
                if n > 1 {
                    b.set_tooltip(&format!("remove the {} coefficient",
                                           CoefSpecifier::term_label(n-1)));
                } else {
                    b.set_tooltip("LOL that'd be dumb");
                    b.deactivate();
                }
                ob.set_tooltip(&format!("add a {} coefficient",
                               CoefSpecifier::term_label(n)));
            }
        });
        
        coef_add.set_callback({
            let mut win = w.clone();
            let mut pyw = pyw.clone();
            let mut ob  = coef_del.clone();
            let cs = cs.clone();
            move |b| {
                let (w, h) = (win.w(), win.h());
                win.set_size(w, h + COEF_ROW_HEIGHT);
                let h = pyw.h();
                pyw.set_size(w, h + COEF_ROW_HEIGHT);
                let n = cs.borrow().len();
                let y_pos = (3 + n as i32) * COEF_ROW_HEIGHT;
                let mut new_coef = CoefSpecifier::new(
                    &CoefSpecifier::term_label(n), 0.0, 0.0);
                pyw.add(new_coef.get_row());
                new_coef.get_mut_row().set_pos(0, y_pos);
                cs.borrow_mut().push(new_coef);
                
                b.set_tooltip(&format!("add a {} coefficient",
                              CoefSpecifier::term_label(n+1)));
                ob.set_tooltip(&format!("remove the {} coefficient",
                               CoefSpecifier::term_label(n)));
            }
        });
        
        IterPane {
            //win: w,
            selector: sel,
            pm_a: a,
            pm_b: b,
            coefs: cs,
        }
    }
    
    /**Return the `image::IterType` currently specified by the `IterPane`.*/
    pub fn get_itertype(&self) -> IterType {
        match self.selector.value() {
            0 => IterType::Mandlebrot,
            1 => IterType::PseudoMandlebrot(
                self.pm_a.get_value(),
                self.pm_b.get_value()
            ),
            2 => IterType::Polynomial(
                self.coefs.borrow().iter().map(|c| c.get_value()).collect()
            ),
            n @ _ => {
                eprintln!("IterPane::get_itertype(): illegal selector value: {}", &n);
                IterType::Mandlebrot
            },
        }
    }
}


#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn iter_pane() {
        let a = fltk::app::App::default();
        let p = IterPane::new();

        let mut w = DoubleWindow::default().with_size(100, 100);
        let mut b = Button::default().with_label("spek")
            .with_size(96, 24).with_pos(2, 38);
        w.end();
        w.show();
        
        b.set_callback(move |_| {
            println!("{:?}", p.get_itertype());
        });
        
        a.run().unwrap();
    }
}