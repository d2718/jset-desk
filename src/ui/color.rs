/*!
The pane for specifying the `ColorMap`, attendant functionality, and
some functions to facilitate choosing colors.
*/

use std::cell::{Cell, RefCell};
use std::rc::Rc;
use std::sync::mpsc;

use fltk::{
    app::add_timeout3,
    button::Button,
    enums::{Event, Shortcut},
    frame::Frame,
    input::IntInput,
    prelude::*,
    valuator::{HorNiceSlider, ValueInput},
    window::DoubleWindow,
};

use super::*;
use crate::image::*;

// The following constants all express dimensions of elements of the color
// picker window created by `pick_color()`.
const PICKER_LABEL_WIDTH: i32 = 24;
const PICKER_SLIDER_WIDTH: i32 = 256;
const PICKER_INPUT_WIDTH: i32 = 48;
const PICKER_ROW_HEIGHT: i32 = 32;
const PICKER_OUTPUT_WIDTH: i32 = 4 * PICKER_ROW_HEIGHT;

const PICKER_ROW_WIDTH: i32 = PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH + PICKER_INPUT_WIDTH;
const PICKER_WINDOW_WIDTH: i32 = PICKER_ROW_WIDTH + PICKER_OUTPUT_WIDTH;
const PICKER_WINDOW_HEIGHT: i32 = PICKER_ROW_HEIGHT * 4;
const PICKER_BUTTON_WIDTH: i32 = PICKER_ROW_WIDTH / 2;

// This function only exists to save typing in the implementation of
// `pick_color()`. There are three nearly-identical rows of widgets in the
// color picker window; this abstracts creating them.
fn make_picker_row(
    ypos: i32,
    label: &'static str,
    initial_value: f64,
    mut prev: DoubleWindow,
    rvalue: Rc<Cell<RGB>>,
) -> (Frame, HorNiceSlider, ValueInput) {
    let lab = Frame::default()
        .with_label(label)
        .with_pos(0, ypos)
        .with_size(PICKER_LABEL_WIDTH, PICKER_ROW_HEIGHT);
    let mut slider = HorNiceSlider::default()
        .with_pos(PICKER_LABEL_WIDTH, ypos)
        .with_size(PICKER_SLIDER_WIDTH, PICKER_ROW_HEIGHT);
    slider.set_value(initial_value);
    let mut vinput = ValueInput::new(
        PICKER_LABEL_WIDTH + PICKER_SLIDER_WIDTH,
        ypos,
        PICKER_INPUT_WIDTH,
        PICKER_ROW_HEIGHT,
        None,
    );
    vinput.set_value(initial_value);

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
                "R" => {
                    rv.set_r(x as f32);
                }
                "G" => {
                    rv.set_g(x as f32);
                }
                "B" => {
                    rv.set_b(x as f32);
                }
                s => {
                    panic!("ui::make_picker_row(): bad picker row label: {}", s);
                }
            }
            rvalue.set(rv);
            let c = rgb_to_fltk(rv);
            prev.set_color(c);
            prev.redraw();
        }
    });

    vinput.set_callback({
        let mut slider = slider.clone();
        move |v| {
            let x = v.value();
            slider.set_value(x);
            let mut rv = rvalue.get();
            match label {
                "R" => {
                    rv.set_r(x as f32);
                }
                "G" => {
                    rv.set_g(x as f32);
                }
                "B" => {
                    rv.set_b(x as f32);
                }
                s => {
                    panic!("ui::make_picker_row(): bad picker row label: {}", s);
                }
            }
            rvalue.set(rv);
            let c = rgb_to_fltk(rv);
            prev.set_color(c);
            prev.redraw();
        }
    });

    (lab, slider, vinput)
}

/**
Pops up a modal window for selecting a color.
*/
pub fn pick_color(start: RGB) -> Option<RGB> {
    let rvalue: Rc<Cell<RGB>> = Rc::new(Cell::new(start));

    let mut w = DoubleWindow::default()
        .with_label("Specify a Color")
        .with_size(PICKER_WINDOW_WIDTH, PICKER_WINDOW_HEIGHT);

    let mut prev = DoubleWindow::default()
        .with_size(PICKER_OUTPUT_WIDTH, PICKER_WINDOW_HEIGHT)
        .with_pos(PICKER_ROW_WIDTH, 0);
    prev.end();
    prev.set_color(rgb_to_fltk(start));

    let (_, _, _) = make_picker_row(0, "R", start.r() as f64, prev.clone(), rvalue.clone());
    let (_, _, _) = make_picker_row(
        PICKER_ROW_HEIGHT,
        "G",
        start.g() as f64,
        prev.clone(),
        rvalue.clone(),
    );
    let (_, _, _) = make_picker_row(
        2 * PICKER_ROW_HEIGHT,
        "B",
        start.b() as f64,
        prev.clone(),
        rvalue.clone(),
    );

    let mut ok = Button::default()
        .with_label("Set @returnarrow")
        .with_size(PICKER_BUTTON_WIDTH, PICKER_ROW_HEIGHT)
        .with_pos(0, 3 * PICKER_ROW_HEIGHT);
    ok.set_shortcut(Shortcut::from_key(Key::Enter));
    let mut no = Button::default()
        .with_label("Cancel (Esc)")
        .with_size(PICKER_BUTTON_WIDTH, PICKER_ROW_HEIGHT)
        .with_pos(PICKER_BUTTON_WIDTH, 3 * PICKER_ROW_HEIGHT);
    no.set_shortcut(Shortcut::from_key(Key::Escape));

    w.end();
    w.make_modal(true);
    w.show();

    let (tx, rx) = mpsc::channel::<Option<RGB>>();

    ok.set_callback({
        let tx = tx.clone();
        move |_| {
            tx.send(Some(rvalue.get())).unwrap();
        }
    });
    no.set_callback({
        move |_| {
            tx.send(None).unwrap();
        }
    });

    while match rx.try_recv() {
        Err(_) => true,
        Ok(c) => {
            DoubleWindow::delete(w);
            return c;
        }
    } {
        fltk::app::wait();
    }
    None
}

// The following constants all specify dimensions of the `GradientChooser`
// widget wrapper's UI elements.
const GRADIENT_BUTTON_WIDTH: i32 = 32;
const GRADIENT_ROW_HEIGHT: i32 = 32;
const GRADIENT_STEPS_WIDTH: i32 = 64;
const GRADIENT_ROW_WIDTH: i32 = (2 * GRADIENT_BUTTON_WIDTH) + GRADIENT_STEPS_WIDTH;

// Wraps some UI elements for specifying a `Gradient`.
struct GradientChooser {
    win: DoubleWindow,
    start_color: Rc<Cell<RGB>>,
    end_color: Rc<Cell<RGB>>,
    steps_n: Rc<Cell<usize>>,
}

impl GradientChooser {
    // Create a new `GradientChooser` that initially displays parameters
    // for the supplied `Gradient`.
    fn new(g: Gradient, drag_color: Rc<Cell<Option<RGB>>>) -> GradientChooser {
        let w = DoubleWindow::default().with_size(GRADIENT_ROW_WIDTH, GRADIENT_ROW_HEIGHT);
        let mut sbutt = Button::default()
            .with_size(GRADIENT_BUTTON_WIDTH, GRADIENT_ROW_HEIGHT)
            .with_pos(0, 0);
        sbutt.set_tooltip("set start color");
        sbutt.set_color(rgb_to_fltk(g.start));
        let mut ebutt = Button::default()
            .with_size(GRADIENT_BUTTON_WIDTH, GRADIENT_ROW_HEIGHT)
            .with_pos(GRADIENT_BUTTON_WIDTH + GRADIENT_STEPS_WIDTH, 0);
        ebutt.set_tooltip("set end color");
        ebutt.set_color(rgb_to_fltk(g.end));
        let mut stepsi = IntInput::default()
            .with_size(GRADIENT_STEPS_WIDTH, GRADIENT_ROW_HEIGHT)
            .with_pos(GRADIENT_BUTTON_WIDTH, 0);
        stepsi.set_tooltip("number of steps");
        stepsi.set_value(&format!("{}", g.steps));
        w.end();

        let sc_cell = Rc::new(Cell::new(g.start));
        let ec_cell = Rc::new(Cell::new(g.end));
        let sn_cell = Rc::new(Cell::new(g.steps));

        sbutt.set_callback({
            let sc_cell = sc_cell.clone();
            move |b| {
                if let Some(c) = pick_color(sc_cell.get()) {
                    b.set_color(rgb_to_fltk(c));
                    b.redraw();
                    sc_cell.set(c);
                }
            }
        });
        ebutt.set_callback({
            let ec_cell = ec_cell.clone();
            move |b| {
                if let Some(c) = pick_color(ec_cell.get()) {
                    b.set_color(rgb_to_fltk(c));
                    b.redraw();
                    ec_cell.set(c);
                }
            }
        });

        stepsi.set_callback({
            let sn_cell = sn_cell.clone();
            move |i| {
                if let Ok(n) = i.value().parse::<usize>() {
                    sn_cell.set(n);
                } else {
                    i.set_value(&format!("{}", sn_cell.get()));
                }
            }
        });
        
        sbutt.handle({
            let sc_cell = sc_cell.clone();
            let drag_color = drag_color.clone();
            move |b, evt| {
                match evt {
                    Event::Enter => {
                        if let Some(c) = drag_color.get() {
                            b.set_color(rgb_to_fltk(c));
                            b.redraw();
                            sc_cell.set(c);
                            true
                        } else {
                            false
                        }
                    },
                    Event::Released => {
                        drag_color.set(Some(sc_cell.get()));
                        add_timeout3(0.0, {
                            let drag_color = drag_color.clone();
                            move |_| { drag_color.set(None); }
                        });
                        true
                    },
                    _ => false,
                }
            }
        });
        ebutt.handle({
            let ec_cell = ec_cell.clone();
            let drag_color = drag_color.clone();
            move |b, evt| {
                match evt {
                    Event::Enter => {
                        if let Some(c) = drag_color.get() {
                            b.set_color(rgb_to_fltk(c));
                            b.redraw();
                            ec_cell.set(c);
                            true
                        } else {
                            false
                        }
                    },
                    Event::Released => {
                        drag_color.set(Some(ec_cell.get()));
                        add_timeout3(0.0, {
                            let drag_color = drag_color.clone();
                            move |_| { drag_color.set(None); }
                        });
                        true
                    },
                    _ => false,
                }
            }
        });


        GradientChooser {
            win: w,
            start_color: sc_cell,
            end_color: ec_cell,
            steps_n: sn_cell,
        }
    }

    // Return a reference to the wrapped group of UI elements, so they
    // can be added to groups of widgets.
    pub fn get_win(&self) -> &DoubleWindow {
        &self.win
    }
    // Set the position of the underlying UI group in its containing group.
    pub fn set_pos(&mut self, x: i32, y: i32) {
        self.win.set_pos(x, y);
    }
    // Show the underlying group. I'm honestly not totally sure why this
    // call is necessary, and its necessity seems to be platform-dependent.
    pub fn show(&mut self) {
        self.win.show();
    }

    // Return the specified gradient.
    pub fn get_gradient(&self) -> Gradient {
        Gradient {
            start: self.start_color.get(),
            end: self.end_color.get(),
            steps: self.steps_n.get(),
        }
    }
}

// The calculated width of the `ColorPane`'s window.
const COLOR_PANE_WIDTH: i32 = (4 * GRADIENT_BUTTON_WIDTH) + GRADIENT_STEPS_WIDTH;

// The `ColorPaneGuts` holds the `ColorPane`'s window and other UI
// elements. It also must hold a reference to itself, which is a little
// wonky and probably an anti-pattern. It only exists so that the constructor
// for the public interface to this stuff, the `ColorPane` can just return a
// plain struct instead of an `Rc<RefCell<Self>>`.
struct ColorPaneGuts {
    choosers: Vec<GradientChooser>,
    win: DoubleWindow,
    default_color: RGB,
    drag_color: Rc<Cell<Option<RGB>>>,
    me: Option<Rc<RefCell<ColorPaneGuts>>>,
}

impl ColorPaneGuts {
    fn new(
        new_gradients: Vec<Gradient>,
        default_color: RGB,
        pipe: mpsc::Sender<Msg>,
    ) -> Rc<RefCell<ColorPaneGuts>> {
        let (scrn_w, scrn_h) = fltk::app::screen_size();
        let (scrn_w, scrn_h) = (scrn_w as i32, scrn_h as i32);
        let mut w = DoubleWindow::default().with_pos(scrn_w - COLOR_PANE_WIDTH, scrn_h / 2);
        w.set_border(false);
        w.end();

        setup_subwindow_behavior(&mut w, pipe);
        
        let drag_color: Rc<Cell<Option<RGB>>> = Rc::new(Cell::new(None));

        let pg = Rc::new(RefCell::new(ColorPaneGuts {
            choosers: new_gradients
                .iter()
                .map(|g| GradientChooser::new(*g, drag_color.clone()))
                .collect(),
            win: w.clone(),
            default_color,
            drag_color,
            me: None,
        }));

        pg.borrow_mut().me = Some(pg.clone());

        pg
    }

    // Every time a gradient chooser is added or removed, the window
    // needs to be resized/redrawn.
    fn redraw(&mut self) {
        for ch in self.choosers.iter() {
            self.win.remove(ch.get_win());
        }
        self.win.clear();
        let height = (3 + self.choosers.len() as i32) * GRADIENT_ROW_HEIGHT;
        self.win.set_size(COLOR_PANE_WIDTH, height);
        self.win.begin();

        let _ = Frame::default()
            .with_label("Color Map")
            .with_pos(0, 0)
            .with_size(COLOR_PANE_WIDTH, GRADIENT_ROW_HEIGHT);

        for (n, ch) in self.choosers.iter_mut().enumerate() {
            let ypos = (1 + n as i32) * GRADIENT_ROW_HEIGHT;
            let mut insert_butt = Button::default()
                .with_label("@+")
                .with_size(GRADIENT_BUTTON_WIDTH, GRADIENT_ROW_HEIGHT)
                .with_pos(0, ypos);
            insert_butt.set_tooltip("insert gradient before this one");
            self.win.add(ch.get_win());
            ch.set_pos(GRADIENT_BUTTON_WIDTH, ypos);
            //ch.show();
            let mut remove_butt = Button::default()
                .with_label("x")
                .with_size(GRADIENT_BUTTON_WIDTH, GRADIENT_ROW_HEIGHT)
                .with_pos(GRADIENT_BUTTON_WIDTH + GRADIENT_ROW_WIDTH, ypos);
            remove_butt.set_tooltip("remove this gradient");

            insert_butt.set_callback({
                let me = self.me.as_ref().unwrap().clone();
                move |_| {
                    me.borrow_mut().insert(n);
                }
            });

            remove_butt.set_callback({
                let me = self.me.as_ref().unwrap().clone();
                move |_| {
                    me.borrow_mut().remove(n);
                }
            });
        }

        let tail_w_ypos = (1 + self.choosers.len() as i32) * GRADIENT_ROW_HEIGHT;
        let tail_label_w = (2 * GRADIENT_BUTTON_WIDTH) + GRADIENT_STEPS_WIDTH;
        //~ let tail_w = DoubleWindow::default()
        //~ .with_size(COLOR_PANE_WIDTH, 2*GRADIENT_ROW_HEIGHT)
        //~ .with_pos(0, tail_w_ypos);
        let mut append_butt = Button::default()
            .with_label("@+")
            .with_pos(0, tail_w_ypos)
            .with_size(2 * GRADIENT_BUTTON_WIDTH, GRADIENT_ROW_HEIGHT);
        let _ = Frame::default()
            .with_label("append gradient")
            .with_pos(2 * GRADIENT_BUTTON_WIDTH, tail_w_ypos)
            .with_size(tail_label_w, GRADIENT_ROW_HEIGHT);
        let _ = Frame::default()
            .with_label("default color")
            .with_pos(0, tail_w_ypos + GRADIENT_ROW_HEIGHT)
            .with_size(tail_label_w, GRADIENT_ROW_HEIGHT);
        let mut default_select = Button::default()
            .with_pos(tail_label_w, tail_w_ypos + GRADIENT_ROW_HEIGHT)
            .with_size(2 * GRADIENT_BUTTON_WIDTH, GRADIENT_ROW_HEIGHT);
        default_select.set_color(rgb_to_fltk(self.default_color));
        default_select.set_tooltip("set default color");
        //~ tail_w.end();

        self.win.end();
        self.win.show();

        for ch in self.choosers.iter_mut() {
            ch.show();
        }

        append_butt.set_callback({
            let me = self.me.as_ref().unwrap().clone();
            move |_| {
                me.borrow_mut().append();
            }
        });

        default_select.set_callback({
            let me = self.me.as_ref().unwrap().clone();
            move |b| {
                let old_c = me.borrow().default_color;
                if let Some(c) = pick_color(old_c) {
                    me.borrow_mut().default_color = c;
                    b.set_color(rgb_to_fltk(c));
                    b.redraw();
                }
            }
        });
        default_select.handle({
            let drag_color = self.drag_color.clone();
            let me = self.me.as_ref().unwrap().clone();
            move |b, evt| {
                match evt {
                    Event::Enter => {
                        if let Some(c) = drag_color.get() {
                            b.set_color(rgb_to_fltk(c));
                            me.borrow_mut().default_color = c;
                            b.redraw();
                            true
                        } else {
                            false
                        }
                    },
                    Event::Released => {
                        drag_color.set(Some(me.borrow().default_color));
                        add_timeout3(0.0, {
                            let drag_color = drag_color.clone();
                            move |_| { drag_color.set(None); }
                        });
                        true
                    },
                    _ => false,
                }
            }
        });
    }

    // Insert a new `GradientChooser` at position `n`. If `n` is larger
    // than the current `Vec` of `GradientChooser`s, it will just be
    // appended. This behavor is relied upon.
    fn insert(&mut self, n: usize) {
        let (new_start, new_end): (RGB, RGB);

        if n == 0 {
            new_start = RGB::BLACK;
            if self.choosers.is_empty() {
                new_end = self.default_color;
            } else {
                new_end = self.choosers[0].start_color.get();
            }
        } else if n >= self.choosers.len() {
            new_start = match self.choosers.last() {
                None => RGB::BLACK,
                Some(g) => g.end_color.get(),
            };
            new_end = self.default_color;
        } else {
            new_start = self.choosers[n - 1].end_color.get();
            new_end = self.choosers[n].start_color.get();
        }

        let g = Gradient {
            start: new_start,
            end: new_end,
            steps: 256,
        };
        let gc = GradientChooser::new(g, self.drag_color.clone());
        self.choosers.insert(n, gc);

        self.redraw();
    }

    // Append a `GradientChooser` to the end.
    fn append(&mut self) {
        self.insert(self.choosers.len());
    }

    // Remove the `GradientChooser` at position `n`, if it exists; don't
    // do anything (like crash) if it doesn't.
    fn remove(&mut self, n: usize) {
        if n < self.choosers.len() {
            let ch = self.choosers.remove(n);
            self.win.remove(&ch.win);
            self.redraw();
            DoubleWindow::delete(ch.win);
        }
    }

    // Remove all `GradientChoosers`.
    fn clear(&mut self) {
        loop {
            match self.choosers.pop() {
                Some(ch) => {
                    self.win.remove(&ch.win);
                    self.redraw();
                    DoubleWindow::delete(ch.win);
                }
                None => {
                    return;
                }
            }
        }
    }
}

/**
This struct holds and manages all the UI elements for specifying the
image's `ColorMap`.
*/
pub struct ColorPane {
    guts: Rc<RefCell<ColorPaneGuts>>,
}

impl ColorPane {
    /** Instantiate a new `ColorPane` with the provided specification. */
    pub fn new(spec: ColorSpec, pipe: mpsc::Sender<Msg>) -> ColorPane {
        let def = spec.default();
        let cpg = ColorPaneGuts::new(spec.gradients(), def, pipe);
        cpg.borrow_mut().redraw();
        ColorPane { guts: cpg }
    }

    /** Get the `ColorSpec` currently specified by the `ColorPane`. */
    pub fn get_spec(&self) -> ColorSpec {
        let g = self.guts.borrow();
        ColorSpec::new(
            g.choosers.iter().map(|ch| ch.get_gradient()).collect(),
            g.default_color,
        )
    }

    pub fn respec(&mut self, new_spec: ColorSpec) {
        let new_default = new_spec.default();
        let mut g = self.guts.borrow_mut();
        g.default_color = new_default;
        g.clear();
        for grad in new_spec.gradients().into_iter() {
            let gc = GradientChooser::new(grad, g.drag_color.clone());
            g.choosers.push(gc);
        }
        g.redraw();
    }

    /**
    Raise pane to the top by hiding then showing its window. It seems
    like there should be a more direct way to do this.
    */
    pub fn raise(&mut self) {
        let w = &mut self.guts.borrow_mut().win;
        w.hide();
        w.show();
    }
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

    #[test]
    fn query_gradient() {
        let a = fltk::app::App::default();
        let g = Gradient {
            start: RGB::BLACK,
            end: RGB::WHITE,
            steps: 256,
        };
        let mut gc = GradientChooser::new(g);

        let mut w = DoubleWindow::default().with_size(256, 128);
        w.add(gc.get_win());
        gc.set_pos(16, 16);
        gc.show();
        w.end();
        w.show();

        setup_subwindow_behavior(&mut w);

        const K: Key = Key::from_char(' ');

        while a.wait() {
            match fltk::app::event() {
                Event::KeyDown => {
                    let k = fltk::app::event_key();
                    println!("{:?}", &k);
                }
                _ => {}
            }
        }
    }

    #[test]
    fn color_pane() {
        let a = fltk::app::App::default();
        let gv = vec![Gradient::default()];
        let p = ColorPane::new(gv, RGB::WHITE);

        let mut w = DoubleWindow::default().with_size(100, 100);
        let mut b = Button::default()
            .with_label("spek")
            .with_size(96, 24)
            .with_pos(2, 38);
        w.end();
        w.show();

        b.set_callback(move |_| {
            println!("{:?}", p.get_spec());
        });

        a.run().unwrap();
    }
}
