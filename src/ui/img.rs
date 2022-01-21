/*!
This module contains the structs and methods required for the pane that
displays the image and controls navigation and zooming.
*/
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::mpsc;

use fltk::{
    button::{Button, RadioRoundButton},
    enums::{Color, ColorDepth},
    frame::Frame,
    group::{Pack, PackType, Scroll, ScrollType},
    image::RgbImage,
    input::IntInput,
    valuator::ValueInput,
    window::DoubleWindow,
};

use super::*;

/**
the ImgPane (or one of its elemnts) will emit a `Msg` whenever a user action
would cause some aspect of the image or its color map to be recalculated and
redisplayed.
*/
#[derive(Clone, Copy, Debug)]
pub enum Msg {
    /// When the user clicks the "Load" button.
    Load,
    /// The user pushes one of the "Nudge" buttons. The values emitted are
    /// horzontal and vertical distance in pixels to nudge the image. This
    /// will get translated to a distance on the complex plane, which is
    /// why floats are okay.
    Nudge(f64, f64),
    /// The user clicks on the image in order to recenter it. The values
    /// emitted are the horizontal/vertical locations of the click as
    /// fractions of the width/height of the image.
    Recenter(f64, f64),
    /// The user just hits the return key. Values emited are values from
    /// the "Width" and "Height" inputs, if valid.
    Redraw(Option<usize>, Option<usize>),
    /// The user clicks the "save image" button.
    SaveImage,
    /// The user clicks the "save values" button.
    SaveValues,
    /// The user clicks one of the scale radio butons; the value emitted
    /// is the scale ratio selected.
    Scale(usize),
    /// The user zooms in/out. The value emitted is the value in the "Zoom"
    /// input (if a zoom in) or its reciprocal (if a zoom out).
    Zoom(f64),
}

const COL_WIDTH:   i32 = 72;
const ROW_HEIGHT:  i32 = 24;
const COL_HEIGHT:  i32 = ROW_HEIGHT * 22;
const HALF_BUTTON: i32 = COL_WIDTH / 2;
const N_SCALERS: usize = 5;
const MIN_DIMENSION: usize = 16;

const DEFAULT_ZOOM:   f64 = 2.0;
const DEFAULT_NUDGE:  f64 = 10.0;

/**
The `ImgPane` is the main window of the application. It displays the actual
image and features the controlls for navigation/zooming.
*/
pub struct ImgPane {
    win: DoubleWindow,
    im_frame: Frame,
    image_data: Vec<u8>,
}

impl ImgPane {
    /**
    Instantiates a new `ImgPane` with the initial supplied `ImageDims`.
    The `version` will be shown in the title bar of the window, and the
    `pipe` is the sending end of the channel down which emittied messages
    are to be sent.
    */
    pub fn new(
        pipe: mpsc::Sender<Msg>,
        version: &str,
        dims: crate::image::ImageDims
    ) -> ImgPane {
        let image_xpix = dims.xpix as i32;
        let image_ypix = dims.ypix as i32;
        let mut w = DoubleWindow::default()
            .with_size(image_xpix + COL_WIDTH, image_ypix);
        w.set_label(&format!("JSet-Desktop {}", version));
        w.set_border(true);
        w.make_resizable(true);
        
        let ctrl = Pack::default().with_size(COL_WIDTH, COL_HEIGHT)
            .with_pos(0, 0);
        
        let _ = Frame::default().with_label("Width")
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut width_input = IntInput::default().with_size(COL_WIDTH, ROW_HEIGHT);
        width_input.set_tooltip("set image width in pixels");
        width_input.set_value(&format!("{}", dims.xpix));
        let _ = Frame::default().with_label("Height")
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut height_input = IntInput::default().with_size(COL_WIDTH, ROW_HEIGHT);
        height_input.set_tooltip("set image height in pixels");
        height_input.set_value(&format!("{}", dims.ypix));
        
        let _ = Frame::default().with_label("Zoom")
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut zoom_input = ValueInput::default()
            .with_size(COL_WIDTH, ROW_HEIGHT);
        zoom_input.set_tooltip("set_zoom_ratio");
        zoom_input.set_minimum(1.0);
        zoom_input.set_value(DEFAULT_ZOOM);
        let zoom_butt_pack = Pack::default().with_type(PackType::Horizontal)
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut zoom_in = Button::default().with_label("@+")
            .with_size(HALF_BUTTON, ROW_HEIGHT);
        let mut zoom_out = Button::default().with_label("@line")
            .with_size(HALF_BUTTON, ROW_HEIGHT);
        zoom_butt_pack.end();
        
        let _ = Frame::default().with_label("Nudge")
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut nudge_input = ValueInput::default()
            .with_size(COL_WIDTH, ROW_HEIGHT);
        nudge_input.set_minimum(0.0);
        nudge_input.set_value(DEFAULT_NUDGE);
        nudge_input.set_step(1.0, 10);
        let nudge_top_pack = Pack::default().with_type(PackType::Horizontal)
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut nudge_up_butt = Button::default()
            .with_size(HALF_BUTTON, ROW_HEIGHT)
            .with_label("@#00090->");
        let mut nudge_right_butt = Button::default()
            .with_size(HALF_BUTTON, ROW_HEIGHT)
            .with_label("@->");
        nudge_top_pack.end();
        let nudge_bottom_pack = Pack::default().with_type(PackType::Horizontal)
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let mut nudge_left_butt = Button::default()
            .with_size(HALF_BUTTON, ROW_HEIGHT)
            .with_label("@<-");
        let mut nudge_down_butt = Button::default()
            .with_size(HALF_BUTTON, ROW_HEIGHT)
            .with_label("@#00090<-");
        nudge_bottom_pack.end();
        
        let mut scalers: Vec<RadioRoundButton> = Vec::new();
        
        let _ = Frame::default().with_label("Scale")
            .with_size(COL_WIDTH, ROW_HEIGHT);
        let scale_pack = Pack::default().with_size(COL_WIDTH, 5 * ROW_HEIGHT);
        for n in 0..N_SCALERS {
            let mut sb = RadioRoundButton::default()
                .with_size(COL_WIDTH, ROW_HEIGHT);
            sb.set_label(&format!("{}:1", n+1));
            scalers.push(sb);
        }
        scalers[0].toggle(true);
        scale_pack.end();
        
        let mut save_butt = Button::default().with_label("save\nimage")
            .with_size(COL_WIDTH, 2 * ROW_HEIGHT);
        let mut remember_butt = Button::default().with_label("save\nvalues")
            .with_size(COL_WIDTH, 2 * ROW_HEIGHT);
        let _ = Frame::default().with_size(COL_WIDTH, ROW_HEIGHT); // spacer
        let mut load_butt = Button::default().with_label("load")
            .with_size(COL_WIDTH, ROW_HEIGHT);
        
        ctrl.end();
        
        let scroll_region = Scroll::default().with_pos(COL_WIDTH, 0)
            .with_size(image_xpix, image_ypix)
            .with_type(ScrollType::Both);
        let mut image_frame = Frame::default().with_pos(COL_WIDTH, 0);
        image_frame.set_color(Color::Black);
        scroll_region.end();
        
        w.end();
        w.show();
        
        let ip = ImgPane {
            win: w.clone(),
            im_frame: image_frame.clone(),
            image_data: Vec::new(),
        };
        
        let scalers = Rc::new(RefCell::new(scalers));
        
        let get_scale = {
            let scalers = scalers.clone();
            move || {
                for (n, b) in scalers.borrow().iter().enumerate() {
                    if b.is_toggled() { return n+1; }
                }
                eprintln!("ImgPane closure get_scale(): no scalers toggled.");
                return 1;
            }
        };
        
        let get_nudge_distance = {
            let nudge_input = nudge_input.clone();
            move || {
                let v = nudge_input.value();
                if v < 0.0f64 {
                    eprintln!("Illegal nudge amount: {}", &v);
                    return 0.0f64;
                } else {
                    return v;
                }
            }
        };
        
        let get_zoom_factor = {
            let zoom_input = zoom_input.clone();
            move || {
                let v = zoom_input.value();
                if v < 1.0 {
                    eprintln!("Illegal zoom value (< 1.0): {}", &v);
                    return 1.0f64;
                } else {
                    return v;
                }
            }
        };
        
        w.handle({
            let pipe = pipe.clone();
            let width_input = width_input.clone();
            let height_input = height_input.clone();
            move |_, evt| {
                match evt {
                    Event::KeyDown => match fltk::app::event_key() {
                        Key::Enter => {
                            let xpix = match width_input.value().parse::<usize>() {
                                Err(e) => {
                                    eprintln!("Unable to parse image height: {}", &e);
                                    None
                                },
                                Ok(n) => if n < MIN_DIMENSION {
                                    eprintln!("{} pixels is just too small.", &n);
                                    None
                                } else { Some(n) },
                            };
                            let ypix = match height_input.value().parse::<usize>() {
                                Err(e) => {
                                    eprintln!("Unable to parse image width: {}", &e);
                                    None
                                },
                                Ok(n) => if n < MIN_DIMENSION {
                                    eprintln!("{} pixels is just too small.", &n);
                                    None
                                } else { Some(n) },
                            };
                            pipe.send(Msg::Redraw(xpix, ypix)).unwrap();
                            true
                        },
                        Key::Escape => {
                            // Pretend like we've handled it so the app
                            // won't quit.
                            true
                        },
                        _ => false,
                    },
                    _ => false,
                }
            }
        });
        
        // Quit when the main window is closed.
        w.set_callback(|_| { fltk::app::quit(); });
        
        image_frame.handle({
            let pipe = pipe.clone();
            move |f, evt| {
                if evt != Event::Released { return false; }
                
                let (fxpix, fypix) = (f.w() as f64, f.h() as f64);
                let (px, py) = fltk::app::event_coords();
                let (px, py) = (px - f.x(), py - f.y());
                let x_frac = (px as f64) / fxpix;
                let y_frac = (py as f64) / fypix;
                
                pipe.send(Msg::Recenter(x_frac, y_frac)).unwrap();
                true
            }
        });
        
        zoom_in.set_callback({
            let get_zoom = get_zoom_factor.clone();
            let pipe = pipe.clone();
            move |_| {
                let zf = get_zoom();
                pipe.send(Msg::Zoom(zf)).unwrap();
            }
        });
        zoom_out.set_callback({
            let get_zoom = get_zoom_factor.clone();
            let pipe = pipe.clone();
            move |_| {
                let zf = 1.0 / get_zoom();
                pipe.send(Msg::Zoom(zf)).unwrap();
            }
        });
        
        nudge_up_butt.set_callback({
            let dist = get_nudge_distance.clone();
            let pipe = pipe.clone();
            move |_| {
                let d = dist();
                pipe.send(Msg::Nudge(0.0, -d)).unwrap();
            }
        });
        nudge_down_butt.set_callback({
            let dist = get_nudge_distance.clone();
            let pipe = pipe.clone();
            move |_| {
                let d = dist();
                pipe.send(Msg::Nudge(0.0, d)).unwrap();
            }
        });
        nudge_left_butt.set_callback({
            let dist = get_nudge_distance.clone();
            let pipe = pipe.clone();
            move |_| {
                let d = dist();
                pipe.send(Msg::Nudge(-d, 0.0)).unwrap();
            }
        });
        nudge_right_butt.set_callback({
            let dist = get_nudge_distance.clone();
            let pipe = pipe.clone();
            move |_| {
                let d = dist();
                pipe.send(Msg::Nudge(d, 0.0)).unwrap();
            }
        });
        
        let send_scale = {
            let get_scale = get_scale.clone();
            let pipe = pipe.clone();
            move |_: &mut RadioRoundButton| {
                let s = get_scale();
                pipe.send(Msg::Scale(s)).unwrap();
            }
        };
        
        for b in scalers.borrow_mut().iter_mut() {
            let cb = send_scale.clone();
            b.set_callback(cb);
        }
        
        save_butt.set_callback({
            let pipe = pipe.clone();
            move |_| { pipe.send(Msg::SaveImage).unwrap(); }
        });
        remember_butt.set_callback({
            let pipe = pipe.clone();
            move |_| { pipe.send(Msg::SaveValues).unwrap(); }
        });
        load_butt.set_callback({
            let pipe = pipe.clone();
            move |_| { pipe.send(Msg::Load).unwrap(); }
        });
        
        ip
    }
    
    /**
    Set the image to be displayed.
    
    Won't do anything if the dimensions passed don't match the length of
    the data supplied.
    */
    pub fn set_image(&mut self, xpix: usize, ypix: usize, data: Vec<u8>) {
        let npix = xpix * ypix;
        if npix *3 != data.len() {
            eprintln!("Image dimensions don't match data dimenison.");
            return;
        }
        
        self.image_data = data;
        let (w, h) = (xpix as i32, ypix as i32);
        let frame_img = unsafe {
            RgbImage::from_data(
                &self.image_data,
                w, h,
                ColorDepth::Rgb8
            ).unwrap()
        };
        
        self.im_frame.set_size(w, h);
        self.im_frame.set_image(Some(frame_img));
        self.win.redraw();
        fltk::app::sleep(0.01);
    }
    
    /**
    Get the data of the image displayed.
    
    This is just used to save the data (I think).
    */
    pub fn get_image(&self) -> (usize, usize, Vec<u8>) {
        let immij = self.im_frame.image().unwrap();
        (immij.w() as usize, immij.h() as usize, immij.to_rgb_data())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    fn generate_image_data() -> (usize, usize, Vec<u8>) {
        let xpix: u16 = 256 * 2;
        let ypix: u16 = 256;
        let width  = xpix as usize;
        let height = ypix as usize;
        let n_pix = width * height;
        let mut data: Vec<u8> = Vec::with_capacity(n_pix);
        
        for yp in 0..ypix {
            let r = yp as u8;
            for xp in 0..xpix {
                let b = (xp / 2) as u8;
                let g = ((yp + xp) / 3) as u8;
                data.push(r);
                data.push(g);
                data.push(b);
            }
        }
        
        (width, height, data)
    }
    
    #[test]
    fn image_pane() {
        let a = fltk::app::App::default();
        let (w, h, data) = generate_image_data();
        let (tx, rx) = mpsc::channel::<Msg>();
        
        let mut p = ImgPane::new(tx, "internal test");
        p.set_image(w, h, data);
        fltk::app::sleep(0.01);
        
        //~ a.run().unwrap();
        while a.wait() {
            match rx.try_recv() {
                Ok(m) => { println!("{:?}", &m); },
                Err(e) => {},
            }
        }
    }
}