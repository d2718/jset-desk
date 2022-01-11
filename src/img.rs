/*!
module for generating the image window and getting image size/zoom parameters
*/

use std::cell::RefCell;
use std::default::Default;
use std::fs::File;
use std::io::{BufWriter, Write};
use std::rc::Rc;
use std::process::{Command, Stdio};

use fltk::{
    prelude::*,
    button::{Button, RadioRoundButton},
    dialog,
    enums::{Align, Color, Event, Key},
    frame::Frame,
    group::{Flex, Pack, PackType, Scroll},
    image::RgbImage,
    input::IntInput,
    valuator::ValueInput,
    window::DoubleWindow,
};

use crate::fun;
use crate::iter;
use crate::rgb;
use crate::rgb::RGB;

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageParams {
    pub xpix: usize,
    pub ypix: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64,
}

impl Default for ImageParams {
    fn default() -> Self {
        Self {
            xpix: 800,
            ypix: 600,
            x: -2.0,
            y: 1.0,
            width: 3.0,
        }
    }
}

pub struct Pane {
    win: DoubleWindow,
    colors: Rc<RefCell<rgb::Pane>>,
    fun_pane: fun::Pane,
    img_frame: Frame,
    width_ipt: IntInput,
    height_ipt: IntInput,
    nudge_ipt: ValueInput,
    img_zoom_1: RadioRoundButton,
    img_zoom_2: RadioRoundButton,
    img_zoom_3: RadioRoundButton,
    img_zoom_4: RadioRoundButton,
    image_data: Vec<u8>,
    frgb_data: rgb::FImageData,
    current_params: ImageParams,
}

const ROW_HEIGHT: i32 = 24;
const CTRL_COLUMN_WIDTH: i32 = 72;
const CTRL_COLUMN_HEIGHT: i32 = ROW_HEIGHT * 17;
const HALF_BUTTON: i32 = CTRL_COLUMN_WIDTH / 2;
const WINDOW_SIZE_KLUDGE: i32 = 24;

const MIN_IMAGE_DIMENSION: usize = 16;

impl Pane {
    pub fn new(params: ImageParams, version: &str) -> Rc<RefCell<Pane>> {
        let (fxpix, fypix) = (params.xpix as i32, params.ypix as i32);
        let mut w = DoubleWindow::default()
            .with_size(
                fxpix + CTRL_COLUMN_WIDTH + WINDOW_SIZE_KLUDGE,
                fypix + WINDOW_SIZE_KLUDGE
            );
        w.set_label(&format!("JSet-Desktop {}", version));
        w.set_border(true);
        w.make_resizable(true);
        
        let scroll_region = Scroll::default().with_pos(0, 0)
            .with_size(
                fxpix + CTRL_COLUMN_WIDTH + WINDOW_SIZE_KLUDGE,
                fypix + WINDOW_SIZE_KLUDGE
            )
            .with_type(fltk::group::ScrollType::BothAlways);
        let mut imgf = Frame::default().with_pos(CTRL_COLUMN_WIDTH, 0)
            .with_size(fxpix, fypix);
        imgf.set_color(Color::Black);
        
        let ctrl_col = Pack::default().with_pos(0, 0)
            .with_size(CTRL_COLUMN_WIDTH, CTRL_COLUMN_HEIGHT);
        
        let _ = Frame::default().with_label("width").with_size(0, ROW_HEIGHT);
        let mut width_pix_ipt = IntInput::default().with_size(0, ROW_HEIGHT);
        width_pix_ipt.set_value(&params.xpix.to_string());
        let _ = Frame::default().with_label("height").with_size(0, ROW_HEIGHT);
        let mut height_pix_ipt = IntInput::default().with_size(0, ROW_HEIGHT);
        height_pix_ipt.set_value(&params.ypix.to_string());
        
        let _ = Frame::default().with_label("zoom").with_size(0, ROW_HEIGHT);
        let mut zoom_amt_ipt = ValueInput::default().with_size(0, ROW_HEIGHT);
        zoom_amt_ipt.set_value(2.0);
        let zoom_butt_pack = Pack::default().with_size(0, ROW_HEIGHT)
            .with_type(PackType::Horizontal);
        let mut zoom_in_butt = Button::default().with_label("@+")
            .with_size(HALF_BUTTON, 0);
        let mut zoom_out_butt = Button::default().with_label("@line")
            .with_size(HALF_BUTTON, 0);
        zoom_butt_pack.end();
        
        let _ = Frame::default().with_label("nudge").with_size(0, ROW_HEIGHT);
        let mut nudge_amt_ipt = ValueInput::default().with_size(0, ROW_HEIGHT);
        nudge_amt_ipt.set_value(10.0);
        let nudge_top_pack = Pack::default().with_size(0, ROW_HEIGHT)
            .with_type(PackType::Horizontal);
        let mut nudge_up_butt = Button::default().with_label("@#00090->")
            .with_size(HALF_BUTTON, 0);
        let mut nudge_right_butt = Button::default().with_label("@->")
            .with_size(HALF_BUTTON, 0);
        nudge_top_pack.end();
        let nudge_bottom_pack = Pack::default().with_size(0, ROW_HEIGHT)
            .with_type(PackType::Horizontal);
        let mut nudge_left_butt = Button::default().with_label("@<-")
            .with_size(HALF_BUTTON, 0);
        let mut nudge_down_butt = Button::default().with_label("@#00090<-")
            .with_size(HALF_BUTTON, 0);
        nudge_bottom_pack.end();
        
        let _ = Frame::default().with_label("image").with_size(0, ROW_HEIGHT);
        let zoom_flex = Flex::default().column().with_align(Align::Center)
            .with_size(CTRL_COLUMN_WIDTH, 4 * ROW_HEIGHT);
        let mut ab1 = RadioRoundButton::default().with_label("1:1");
        ab1.toggle(true);
        let mut ab2 = RadioRoundButton::default().with_label("2:1");
        let mut ab3 = RadioRoundButton::default().with_label("3:1");
        let mut ab4 = RadioRoundButton::default().with_label("4:1");
        zoom_flex.end();
        
        let mut save_butt = Button::default().with_label("save")
            .with_size(0, ROW_HEIGHT);
        
        ctrl_col.end();
        scroll_region.end();
        w.end();
        w.show();
        
        let p = Pane {
            win: w.clone(),
            img_frame: imgf.clone(),
            colors: rgb::Pane::default(),
            fun_pane: fun::Pane::new(),
            width_ipt: width_pix_ipt.clone(),
            height_ipt: height_pix_ipt.clone(),
            current_params: params,
            nudge_ipt: nudge_amt_ipt.clone(),
            image_data: Vec::new(),
            frgb_data: rgb::FImageData::new(0, 0, Vec::new()),
            img_zoom_1: ab1.clone(),
            img_zoom_2: ab2.clone(),
            img_zoom_3: ab3.clone(),
            img_zoom_4: ab4.clone(),
        };
        
        let p = Rc::new(RefCell::new(p));
        
        w.handle({
            let p = p.clone();
            move |_, evt| {
                match evt {
                    Event::KeyDown => match fltk::app::event_key() {
                        Key::Enter => {
                            p.borrow_mut().reiterate();
                            true
                        },
                        Key::Escape => {
                            // Pretend like we're handling this, so the app
                            // won't quit.
                            true
                        },
                        _ => false,
                    },
                    _ => false,
                }
            }
        });
        
        // But then do quit when we close the window.
        w.set_callback(|_| { fltk::app::quit(); });
        
        imgf.handle({
            let p = p.clone();
            move |f, evt| {
                if evt != Event::Released { return false; }
                
                let mut parms = p.borrow().current_params;
                
                let fxpix = parms.xpix as f64;
                let fypix = parms.ypix as f64;
                
                let (px, py) = fltk::app::event_coords();
                let (px, py) = (px - f.x(), py - f.y());
                let zoom_factor = p.borrow()
                    .get_img_zoom_state()
                    .unwrap() as f64;
                let x_frac = zoom_factor * (px as f64) / fxpix;
                let y_frac = zoom_factor * (py as f64) / fypix;
                let x_frac = x_frac - 0.5;
                let y_frac = y_frac - 0.5;
                
                let height = parms.width * fypix / fxpix;
                let new_x = parms.x + (x_frac * parms.width);
                let new_y = parms.y - (y_frac * height);
                
                parms.x = new_x;
                parms.y = new_y;
                p.borrow_mut().current_params = parms;
                
                p.borrow_mut().reiterate();
                true
            }
        });
        
        nudge_up_butt.set_callback({
            let p = p.clone();
            move |_| {
                let mut p = p.borrow_mut();
                let d = p.get_nudge_distance();
                p.current_params.y = p.current_params.y + d;
                p.reiterate();
            }
        });
        nudge_down_butt.set_callback({
            let p = p.clone();
            move |_| {
                let mut p = p.borrow_mut();
                let d = p.get_nudge_distance();
                p.current_params.y = p.current_params.y - d;
                p.reiterate();
            }
        });
        nudge_left_butt.set_callback({
            let p = p.clone();
            move |_| {
                let mut p = p.borrow_mut();
                let d = p.get_nudge_distance();
                p.current_params.x = p.current_params.x - d;
                p.reiterate();
            }
        });
        nudge_right_butt.set_callback({
            let p = p.clone();
            move |_| {
                let mut p = p.borrow_mut();
                let d = p.get_nudge_distance();
                p.current_params.x = p.current_params.x + d;
                p.reiterate();
            }
        });
        
        zoom_in_butt.set_callback({
            let p = p.clone();
            let zipt = zoom_amt_ipt.clone();
            move |_| {
                let mut p = p.borrow_mut();
                let mut params = p.current_params;
                let z = zipt.value();
                let z_factor =  0.5 * (1.0 - (1.0 / z));
                let height = params.width * (params.ypix as f64) / (params.xpix as f64);
                params.x = params.x + (params.width * z_factor);
                params.y = params.y - (height * z_factor);
                params.width = params.width / z;
                p.current_params = params;
                p.reiterate();
            }
        });
        
        zoom_out_butt.set_callback({
            let p = p.clone();
            let zipt = zoom_amt_ipt.clone();
            move |_| {
                let mut p = p.borrow_mut();
                let mut params = p.current_params;
                let z = zipt.value();
                let z_factor = 0.5 * (z - 1.0);
                let height = params.width * (params.ypix as f64) / (params.xpix as f64);
                params.x = params.x - (params.width * z_factor);
                params.y = params.y + (height * z_factor);
                params.width = params.width * z;
                p.current_params = params;
                p.reiterate();
            }
        });


        ab1.set_callback({
            let p = p.clone();
            move |_| { p.borrow_mut().redraw_image(); }
        });
        ab2.set_callback({
            let p = p.clone();
            move |_| { p.borrow_mut().redraw_image(); }
        });
        ab3.set_callback({
            let p = p.clone();
            move |_| { p.borrow_mut().redraw_image(); }
        });
        ab4.set_callback({
            let p = p.clone();
            move |_| { p.borrow_mut().redraw_image(); }
        });
        
        save_butt.set_callback({
            let p = p.clone();
            move |_| {
                if let Err(e) = p.borrow().save_image_data() {
                    fltk::dialog::message_default(
                        &format!("Error saving file: {}", &e)
                    );
                }
            }
        });
        
        p
    }
    
    fn get_nudge_distance(&self) -> f64 {
        let nudge_pix = self.nudge_ipt.value();
        let p = self.current_params;
        let d = p.width * nudge_pix / (p.xpix as f64);
        d
    }
    
    fn get_img_zoom_state(&self) -> Option<usize> {
        if self.img_zoom_1.is_toggled() { Some(1) }
        else if self.img_zoom_2.is_toggled() { Some(2) }
        else if self.img_zoom_3.is_toggled() { Some(3) }
        else if self.img_zoom_4.is_toggled() { Some(4) }
        else { None }
    }
    
    fn make_image_data_native(&self) -> (usize, usize, Vec<u8>) {
        let width  = self.frgb_data.width();
        let height = self.frgb_data.height();
        
        let mut rgb8_data: Vec<u8> = Vec::with_capacity(width * height * 3);
        for p in self.frgb_data.pixels().iter() {
            for b in p.to_rgb8().iter() {
                rgb8_data.push(*b);
            }
        }
      
        (width, height, rgb8_data)
    }
    
    fn make_image_data_scaled(&self, chunk: usize) -> (usize, usize, Vec<u8>) {
        let pixlines = self.current_params.ypix / chunk;
        let pixcols  = self.current_params.xpix / chunk;
        let n_pixels = pixlines * pixcols;
        let mut rgb8_data: Vec<u8> = Vec::with_capacity(n_pixels * 3);
        let mut palette: [RGB; 16] = [RGB::new(0.0, 0.0, 0.0); 16];
        let f_data = self.frgb_data.pixels();
        
        for yi in 0..pixlines {
            let base_offset = yi * self.current_params.xpix * chunk;
            for xi in 0..pixcols {
                let offs = base_offset + (xi * chunk);
                let mut pp = 0usize;
                for y in 0..chunk {
                    let po = offs + (self.current_params.xpix * y);
                    for x in 0..chunk {
                        palette[pp] = f_data[po+x];
                        pp += 1;
                    }
                }
                let avg_p = rgb::color_average(&palette[0..pp]);
                for b in avg_p.to_rgb8().iter() { rgb8_data.push(*b); }
            }
        }
        
        (pixcols, pixlines, rgb8_data)
    }
    
    pub fn redraw_image(&mut self) {
        let (pixcols, pixlines, rgb8_data) = match self.get_img_zoom_state() {
            Some(1) => self.make_image_data_native(),
            Some(n) => self.make_image_data_scaled(n),
            None => {
                eprintln!("img::Pane::set_image(): illegal im_zoom_state");
                return;
            },
        };
        
        self.image_data = rgb8_data;
        let frame_image = unsafe {
            RgbImage::from_data(
                &self.image_data,
                pixcols as i32,
                pixlines as i32,
                fltk::enums::ColorDepth::Rgb8
            ).unwrap()
        };
        
        self.img_frame.set_size(pixcols as i32, pixlines as i32);
        self.img_frame.set_image(Some(frame_image));
        self.win.redraw();
        
    }
    
    pub fn set_image(
        &mut self,
        image: rgb::FImageData,
    ) {
        self.img_frame.set_size(image.width() as i32, image.height() as i32);
        self.current_params.xpix = image.width();
        self.current_params.ypix = image.height();
        self.frgb_data = image;
       
        self.redraw_image();
    }
    
    fn get_iter_params(&self) -> iter::IterParams {
        self.fun_pane.get_params()
    }
    
    pub fn reiterate(&mut self) {
        match self.width_ipt.value().parse::<usize>() {
            Ok(n) => if n < MIN_IMAGE_DIMENSION {
                eprintln!("A width of {} pixels is just too small.", n);
            } else {
                self.current_params.xpix = n;
            },
            Err(e) => { eprintln!("Error parsing new image width: {}", &e); }
        }
        match self.height_ipt.value().parse::<usize>() {
            Ok(n) => if n < MIN_IMAGE_DIMENSION {
                eprintln!("A height of {} pixels is just too small.", n);
            } else {
                self.current_params.ypix = n;
            },
            Err(e) => { eprintln!("Error parsing new image height: {}", &e); }
        }
        
        let colormap = self.colors.borrow().generate_color_map();
        let iterparams = self.get_iter_params();
        let itermap = iter::make_iter_map(
            self.current_params,
            iterparams,
            colormap.len(),
            num_cpus::get_physical()
        );
        let new_fimage = itermap.color(&colormap);
        self.set_image(new_fimage);
    }
    
    pub fn save_image_data(&self) -> std::io::Result<()> {
        #[cfg(target_family="unix")]
        let im_command = "convert";
        #[cfg(target_family="windows")]
        let im_command = "magick";
        #[cfg(target_family="wasm")]
        unimplemented!();
        
        let img_zoom = match self.get_img_zoom_state() {
            Some(n) => n,
            None => {
                dialog::alert_default("Illegal image zoom state; unable to save.");
                return Ok(());
            }
        };
        
        let xpix = self.current_params.xpix / img_zoom;
        let ypix = self.current_params.ypix / img_zoom;
        
        match Command::new(im_command).arg("-version").output() {
            Ok(_) => {
                let fname = match pick_a_file(".png") {
                    Some(fname) => fname,
                    None => { return Ok(()); }
                };
                let mut cmd = Command::new(im_command)
                    .args(["-", "-define", "png:compression-filter=2",
                            "-define", "png:compression-level=9",
                            "-define", "png:compression-strategy=1",
                            &fname])
                    .stdin(Stdio::piped())
                    .spawn()?;
                let mut cmd_in = cmd.stdin.as_mut().unwrap();
                write!(&mut cmd_in, "P6 {} {} 255\n", xpix, ypix)?;
                cmd_in.write_all(&self.image_data)?;
                cmd.wait().unwrap();
            },
            Err(_) => {
                let fname = match pick_a_file(".ppm") {
                    Some(fname) => fname,
                    None => { return Ok(()); }
                };
                let file = File::create(&fname)?;
                let mut w = BufWriter::new(file);
                write!(&mut w, "P6 {} {} 255\n", xpix, ypix)?;
                w.write_all(&self.image_data)?;
                w.flush()?;
            },
        }
        
        Ok(())
    }
}

fn pick_a_file(extension: &str) -> Option<String> {
    let lc_ext = extension.to_ascii_lowercase();
    let filter = format!("*{}\t*{}", &lc_ext, &extension.to_ascii_uppercase());
    
    let mut fname = match dialog::file_chooser(
        "Name your image file:", &filter, ".", true
    ) {
        None => { return None; },
        Some(f) => f,
    };
    
    if fname.to_ascii_lowercase().ends_with(&lc_ext) {
        return Some(fname);
    }
    
    fname.push_str(&extension);
    return Some(fname);
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn test_save() {
        let a = fltk::app::App::default();
    
        let mut p = Pane::new(ImageParams::default());
        p.borrow_mut().reiterate();
        let res = p.borrow().save_image_data();
        println!("{:?}", &res);
        fltk::app::quit();
    }
    
    #[test]
    fn test_esc() {
        let a = fltk::app::App::default();
        let mut p = Pane::new(ImageParams::default());
        p.borrow_mut().reiterate();
        
        println!("{:?}", p.borrow().win.trigger());
        
        a.run().unwrap();
    }
}