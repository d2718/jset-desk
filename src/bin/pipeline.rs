/*!
testing the iteration->coloration->display pipeline
*/

use jset_desk::*;

fn main() {
    let start_color = rgb::RGB::BLACK;
    let stop_color  = rgb::RGB::MAGENTA;

    let a = fltk::app::App::default();

    let mut imgp = img::ImageParams::default();
    imgp.xpix = 900; imgp.ypix = 900; imgp.y = 1.5;
    let iterp = iter::IterParams::Mandlebrot;
    let g = rgb::Gradient::new(start_color, stop_color, 256);
    let cmap = rgb::ColorMap::make(&[g], start_color);
    
    let imap = iter::make_iter_map(imgp, iterp, cmap.len(), 3);
    let rgbimg = imap.color(&cmap);
    let mut p = img::Pane::new(imgp);
    p.borrow_mut().set_image(rgbimg);
    
    a.run().unwrap();
}