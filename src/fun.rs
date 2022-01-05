/*!
Specifying the iteration function and parameters.
*/

#![allow(dead_code)]

use fltk::{
    prelude::*,
    enums::Font,
    frame::Frame,
    group::Pack,
    input::IntInput,
    valuator::ValueInput,
    window::DoubleWindow,
};

use crate::cx::Cx;

const ROW_HEIGHT: i32 = 32;
const DEGREE_LABEL_WIDTH: i32 = 48;
const VAR_LABEL_WIDTH: i32 = 16;
const VAR_INPUT_WIDTH: i32 = 72;
pub const ROW_WIDTH: i32 = DEGREE_LABEL_WIDTH + (4 * VAR_LABEL_WIDTH) + (2 * VAR_INPUT_WIDTH);

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
        deg_lab.set_label_font(Font::HelveticaItalic);
        deg_lab.set_label(term);
        
        let mut rlab = Frame::default().with_label("r:")
            .with_size(VAR_LABEL_WIDTH, ROW_HEIGHT);
        rlab.set_label_font(Font::HelveticaItalic);
        
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
 
//~ pub struct Pane {
    //~ win:    DoubleWindow,
    //~ width:  IntInput,
    //~ height: IntInput,
    
//~ }

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
}