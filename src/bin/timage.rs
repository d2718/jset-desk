/*!
Testing RGB image generation.

*/

use fltk::{
    prelude::*,
    app::App,
    enums::ColorDepth,
    frame::Frame,
    image::RgbImage,
    window::Window
};

fn generate_image_data(v: &mut Vec<u8>) {
    v.clear();
    v.reserve_exact(256*256);
    for y in 0u8..=255 {
        let r = y;
        let b_half = y / 2;
        for x in 0u8..=255 {
            let g = x;
            let b = b_half + (x / 2);
            v.push(r);
            v.push(g);
            v.push(b);
        }
    }
}

fn main() {
    let a = App::default();
    let mut w = Window::default().with_size(256, 300);
    let mut frame = Frame::new(0, 0, 256, 256, None);
    w.end();
    w.show();
    
    let mut v: Vec<u8> = Vec::new();
    generate_image_data(&mut v);
    let img = RgbImage::new(&v, 256, 256, ColorDepth::Rgb8).unwrap();
    frame.set_image(Some(img));
    
    a.run().unwrap();
}