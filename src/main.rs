
use std::fs::File;
use std::io::{BufWriter, Write};
use std::process::{Command, Stdio};
use std::sync::mpsc;

use fltk::dialog;

use jset_desk::image::*;
use jset_desk::rw;
use jset_desk::ui;
use jset_desk::ui::img::Msg;

const VERSION: &str = "0.2.1 beta";
const X_CLASS: &str = "JSet-Desktop";

struct Globs {
    iter_pane: ui::iter::IterPane,
    colr_pane: ui::color::ColorPane,
    main_pane: ui::img::ImgPane,
    
    cur_dims: ImageDims,
    cur_iter: IterType,
    cur_spec: ColorSpec,
    cur_cmap: ColorMap,
    cur_imap: IterMap,
    cur_fimg: FImage32,
    
    cur_scale: usize,
}

impl Globs {
    pub fn recheck_and_redraw(&mut self, new_dims: ImageDims) {
        
        let mut should_redraw = false;
        let mut should_reiterate = false;
        let mut should_recolor = false;
        
        if new_dims != self.cur_dims {
            should_redraw = true;
            self.cur_dims = new_dims;
        }
        
        let new_iter = self.iter_pane.get_itertype();
        if new_iter != self.cur_iter {
            should_redraw = true;
            self.cur_iter = new_iter;
        }
        
        let new_spec = self.colr_pane.get_spec();
        if new_spec != self.cur_spec {
            let new_cmap = ColorMap::make(new_spec.clone());
            if new_cmap.len() > self.cur_cmap.len() {
                should_reiterate = true;
            }
            self.cur_spec = new_spec;
            self.cur_cmap = new_cmap;
            should_recolor = true;
        }
        
        if should_redraw {
            self.cur_imap = IterMap::new(
                self.cur_dims,
                self.cur_iter.clone(),
                self.cur_cmap.len()
            );
            should_recolor = true;
        } else if should_reiterate {
            self.cur_imap.reiterate(self.cur_cmap.len());
            should_recolor = true;
        }
        
        if should_recolor {
            self.cur_fimg = self.cur_imap.color(&self.cur_cmap);
        }
        
        let (x, y, data) = self.cur_fimg.to_rgb8(self.cur_scale);
        
        self.main_pane.set_image(x, y, data);
    }
}

#[cfg(target_family="unix")]
fn save_file(xpix: usize, ypix: usize, data: &[u8]) -> std::io::Result<()> {
    let im_cmd = match Command::new("magick").arg("-version").output() {
        Ok(_) => Some("magick"),
        Err(_) => match Command::new("convert").arg("-version").output() {
            Ok(_) => Some("convert"),
            Err(_) => None,
        }
    };
    
    match im_cmd {
        Some(cmd) => {
            let fname = match ui::pick_a_file(".png") {
                Some(fname) => fname,
                None => { return Ok(()); }
            };
            let mut cmd = Command::new(cmd)
                .args(["-", "-define", "png:compression-filter=2",
                       "-define", "png:compression-level=9",
                       "-define", "png:compression-strategy=1",
                       &fname])
                .stdin(Stdio::piped())
                .spawn()?;
            let mut cmd_in = cmd.stdin.as_mut().unwrap();
            write!(&mut cmd_in, "P6 {} {} 255\n", xpix, ypix)?;
            cmd_in.write_all(data)?;
            cmd.wait().unwrap();
        },
        None => {
            let fname = match ui::pick_a_file(".ppm") {
                Some(fname) => fname,
                None => { return Ok(()); }
            };
            let file = File::create(&fname)?;
            let mut w = BufWriter::new(file);
            write!(&mut w, "P6 {} {} 255\n", xpix, ypix)?;
            w.write_all(data)?;
            w.flush()?;
        },
    }
    
    Ok(())
}

#[cfg(target_family="windows")]
fn save_file(xpix: usize, ypix: usize, data: &[u8]) -> std::io::Result<()> {
    let im_cmd = match Command::new("magick").arg("-version").output() {
        Ok(_) => Some("magick"),
        Err(_) => None,
    };
    
    match im_cmd {
        Some(cmd) => {
            let fname = match ui::pick_a_file(".png") {
                Some(fname) => fname,
                None => { return Ok(()); }
            };
            let mut cmd = Command::new(cmd)
                .args(["-", "-define", "png:compression-filter=2",
                       "-define", "png:compression-level=9",
                       "-define", "png:compression-strategy=1",
                       &fname])
                .stdin(Stdio::piped())
                .spawn()?;
            let mut cmd_in = cmd.stdin.as_mut().unwrap();
            write!(&mut cmd_in, "P6 {} {} 255\n", xpix, ypix)?;
            cmd_in.write_all(data)?;
            cmd.wait().unwrap();
        },
        None => {
            let fname = match ui::pick_a_file(".ppm") {
                Some(fname) => fname,
                None => { return Ok(()); }
            };
            let file = File::create(&fname)?;
            let mut w = BufWriter::new(file);
            write!(&mut w, "P6 {} {} 255\n", xpix, ypix)?;
            w.write_all(data)?;
            w.flush()?;
        },
    }
    
    Ok(())
}

#[cfg(target_family="wasm")]
fn save_file(xpix: usize, ypix: usize, data: &[u8]) -> std::io::Result<()> {
    unimplemented!()
}

fn main() {
    fltk::window::DoubleWindow::set_default_xclass(X_CLASS);
    
    let (sndr, rcvr) = mpsc::channel::<ui::img::Msg>();
    let dims = ImageDims {
        xpix: 900,
        ypix: 600,
        x: -2.0,
        y: 1.0,
        width: 3.0,
    };
    
    let a = fltk::app::App::default();

    let mut main_pane = ui::img::ImgPane::new(sndr, VERSION, dims);
    let initial_spec = ColorSpec::new(vec![Gradient::default()], RGB::WHITE);
    let colr_pane = ui::color::ColorPane::new(initial_spec);
    let iter_pane = ui::iter::IterPane::new(IterType::Mandlebrot);
    
    let color_spec = colr_pane.get_spec();
    let color_map  = ColorMap::make(color_spec.clone());
    let iter_type  = iter_pane.get_itertype();
    let iter_map   = IterMap::new(dims, iter_type.clone(), color_map.len());
    
    let fp_image = iter_map.color(&color_map);
    
    let (xpix, ypix, rgb_data) = fp_image.to_rgb8(1);
    main_pane.set_image(xpix, ypix, rgb_data);
    
    let mut globs = Globs {
        iter_pane,
        colr_pane,
        main_pane,
        
        cur_dims: dims,
        cur_iter: iter_type,
        cur_spec: color_spec,
        cur_cmap: color_map,
        cur_imap: iter_map,
        cur_fimg: fp_image,
        
        cur_scale: 1,
    }; 
    
    while a.wait() {
        if let Ok(message) = rcvr.try_recv() {
            #[cfg(debug_assertions)]
            println!("{:?}", &message);
            match message {
                Msg::Load => {
                    let fname = match ui::pick_a_file(".toml") {
                        Some(f) => f,
                        None => { continue; }
                    };
                    match rw::load(fname) {
                        Err(e) => dialog::message_default(
                                    &format!("Error loading file: {}", &e)
                                  ),
                        Ok((dims, cspec, itype)) => {
                            globs.colr_pane.respec(cspec);
                            globs.iter_pane = ui::iter::IterPane::new(itype);
                            globs.recheck_and_redraw(dims);
                        },
                    }
                },
                Msg::Nudge(fxpix, fypix) => {
                    let mut dims = globs.cur_dims;
                    let xfrac = fxpix/(dims.xpix as f64);
                    let yfrac = fypix/(dims.ypix as f64);
                    let (dx, dy) = (xfrac * dims.width, yfrac * dims.height());
                    dims.x = dims.x + dx;
                    dims.y = dims.y - dy;
                    
                    globs.recheck_and_redraw(dims);
                },
                Msg::Recenter(xfrac, yfrac) => {
                    let dims = globs.cur_dims.recenter(xfrac, yfrac);
                    globs.recheck_and_redraw(dims);
                },
                Msg::Redraw(owidth, oheight) => {
                    let dims = globs.cur_dims;
                    let new_xpix = match owidth {
                        Some(x) => x,
                        None => dims.xpix,
                    };
                    let new_ypix = match oheight {
                        Some(y) => y,
                        None => dims.ypix,
                    };
                    let new_dims = dims.resize(new_xpix, new_ypix);
                    globs.recheck_and_redraw(new_dims);
                },
                Msg::SaveImage => {
                    let (xpix, ypix, data) = globs.main_pane.get_image();
                    if let Err(e) = save_file(xpix, ypix, &data) {
                        dialog::message_default(
                            &format!("Error saving file: {}", &e)
                        );
                    }
                },
                Msg::SaveValues => {
                    let fname = match ui::pick_a_file(".toml") {
                        Some(f) => f,
                        None => { continue; },
                    };
                    if let Err(estr) = rw::save(
                        &globs.cur_dims,
                        &globs.cur_spec,
                        &globs.cur_iter,
                        &fname
                    ) {
                        dialog::message_default(&estr);
                    }
                },
                Msg::Scale(n) => {
                    globs.cur_scale = n;
                    globs.recheck_and_redraw(globs.cur_dims);
                },
                Msg::Zoom(r) => {
                    let dims = globs.cur_dims.zoom(r);
                    globs.recheck_and_redraw(dims);
                },
            }
        }
        
    }
}