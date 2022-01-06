/*!
module for generating the image window and getting image size/zoom parameters
*/

use fltk::{
    prelude::*,
    button::RadioRoundButton,
    enums::{Align, Color},
    frame::Frame,
    group::{Flex, Pack, Scroll},
    image::RgbImage,
    input::IntInput,
    valuator::ValueInput,
    window::DoubleWindow,
};

use crate::rgb::RGB;

#[derive(Clone, Debug, PartialEq)]
struct ImageParams {
    xpix: usize,
    ypix: usize,
    x: f64,
    y: f64,
    width: f64,
}

fn color_average(dat: &[RGB]) -> RGB {
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

struct Pane {
    win: DoubleWindow,
    img_frame: Frame,
    width_ipt: IntInput,
    height_ipt: IntInput,
    zoom_ipt: ValueInput,
    nudge_ipt: IntInput,
    img_zoom_1: RadioRoundButton,
    img_zoom_2: RadioRoundButton,
    img_zoom_3: RadioRoundButton,
    img_zoom_4: RadioRoundButton,
    image_data: Vec<u8>,
    current_params: ImageParams,
}

const ROW_HEIGHT: i32 = 32;
const CTRL_COLUMN_WIDTH: i32 = 72;
const CTRL_COLUMN_HEIGHT: i32 = ROW_HEIGHT * 10;

impl Pane {
    pub fn new(params: ImageParams) -> Pane {
        let (fxpix, fypix) = (params.xpix as i32, params.ypix as i32);
        let mut w = DoubleWindow::default().with_label("JSet-Desktop")
            .with_size(fxpix + CTRL_COLUMN_WIDTH, fypix);
        w.set_border(true);
        w.make_resizable(true);
        
        let scroll_region = Scroll::default().with_pos(0, 0)
            .with_size(fxpix + CTRL_COLUMN_WIDTH, fypix);
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
        
        let _ = Frame::default().with_label("nudge").with_size(0, ROW_HEIGHT);
        let mut nudge_amt_ipt = IntInput::default().with_size(0, ROW_HEIGHT);
        nudge_amt_ipt.set_value(&(10.to_string()));
        
        let _ = Frame::default().with_label("image").with_size(0, ROW_HEIGHT);
        let zoom_flex = Flex::default().column().with_align(Align::Center)
            .with_size(CTRL_COLUMN_WIDTH, 4 * ROW_HEIGHT);
        let mut ab1 = RadioRoundButton::default().with_label("1:1");
        ab1.toggle(true);
        let mut ab2 = RadioRoundButton::default().with_label("2:1");
        let mut ab3 = RadioRoundButton::default().with_label("3:1");
        let mut ab4 = RadioRoundButton::default().with_label("4:1");
        zoom_flex.end();
        
        ctrl_col.end();
        scroll_region.end();
        w.end();
        w.show();
        
        width_pix_ipt.set_callback({
            let mut f = imgf.clone();
            move |ipt| {
                let w: u16 = match ipt.value().parse() {
                    Ok(n) => n,
                    Err(_) => { println!("Error parsing new image frame width."); return },
                };
                let h = f.h();
                f.set_size(w as i32, h);
                println!("Set image frame size ({}, {})", w, h);
            }
        });
        
        height_pix_ipt.set_callback({
            let mut f = imgf.clone();
            move |ipt| {
                let h: u16 = match ipt.value().parse() {
                    Ok(n) => n,
                    Err(_) => { println!("Error parsing new image frame height."); return },
                };
                let w = f.w();
                f.set_size(w, h as i32);
                println!("Set image frame size ({}, {})", w, h);
            }
        });
        
        let p = Pane {
            win: w.clone(),
            img_frame: imgf.clone(),
            width_ipt: width_pix_ipt.clone(),
            height_ipt: height_pix_ipt.clone(),
            current_params: params,
            zoom_ipt: zoom_amt_ipt.clone(),
            nudge_ipt: nudge_amt_ipt.clone(),
            img_zoom_1: ab1.clone(),
            img_zoom_2: ab2.clone(),
            img_zoom_3: ab3.clone(),
            img_zoom_4: ab4.clone(),
        };
        
        p
    }
    
    fn get_img_zoom_state(&self) -> Option<usize> {
        if self.img_zoom_1.is_toggled() { Some(1) }
        else if self.img_zoom_2.is_toggled() { Some(2) }
        else if self.img_zoom_3.is_toggled() { Some(3) }
        else if self.img_zoom_4.is_toggled() { Some(4) }
        else { None }
    }
    
    pub fn set_image(
        &mut self,
        width: usize,
        height: usize,
        pixvals: &[RGB]
    ) {
        let chunk = match self.get_img_zoom_state() {
            Some(n) => n,
            None => {
                eprintln!("img::Pane::set_image(): illegal im_zoom_state");
                return;
            },
        };
        
        let pixlines = height / chunk;
        let pixcols  = width / chunk;
        let n_pixels = pixlines * pixcols;
        let mut rgba_data: Vec<u8> = Vec::with_capacity(n_pixels * 4);
        let mut palette: [RGB; 16] = [RGB::new(0.0, 0.0, 0.0); 16];
        
        for yi in 0..pixlines {
            let base_offset = yi * width * chunk;
            for xi in 0..pixcols {
                let offs = base_offset + (xi * chunk);
                let mut pp = 0usize;
                for y in 0..chunk {
                    let po = offs + (width * y);
                    for x in 0..chunk {
                        palette[pp] = pixvals[po+x];
                        pp += 1;
                    }
                }
                let avg_p = color_average(&palette[0..pp]);
                for b in avg_p.to_rgba().iter() { rgba_data.push(*b); }
            }
        }
        
        self.image_data = rgb_data;
        let frame_image = unsafe {
            RgbImage::from_data(
                &self.image_data,
                pixcols as i32,
                pixlines as i32,
                fltk::enums::ColorDepth::Rgba8
            ).unwrap();
        }
        
        self.img_frame.set_image(Some(frame_image));
    }
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn make_image_pane() {
        let a = fltk::app::App::default();
        let parms = ImageParams {
            xpix: 800,
            ypix: 600,
            x: -2.0,
            y: 1.0,
            width: 3.0
        };
        let mut p = Pane::new(parms);
        
        a.run().unwrap();
    }
}