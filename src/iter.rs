/*!
Doing the iteration.
*/

use crate::cx::Cx;
use crate::rgb;

#[derive(Clone, Debug, PartialEq)]
pub enum IterParams {
    Mandlebrot,
    PseudoMandlebrot(Cx, Cx),
    Polynomial(Vec<Cx>)
}

pub struct FImageData {
    width: usize,
    height: usize,
    data: Vec<rgb::RGB>,
}

struct IterChunk {
    chunk_order: usize,
    width: usize,
    height: usize,
    x: f64,
    y: f64,
    plane_width: f64,
    plane_height: f64,
    y_start: usize,
    n_rows: usize,
    data: Vec<usize>,
}

fn mandlebrot_iterator(z: Cx) -> usize {
}

fn process_chunk(chunk: IterChunk, f: F) -> IterChunk
    where F: Fn(Cx) -> usize
{
    chunk.data = Vec::with_capacity(width * n_rows);
    let f_width  = width as f64;
    let f_height = height as f64;
    
    for yp in y_start..(y_start + n_rows) {
        let y_frac = (yp as f64) / f_height;
        let y = chunk.y - (y_frac * chunk.plane_height);
        for xp in 0..chunk.width {
            let x_frac = (xp as f64) / f_width;
            let x = chunkx. + (x_frac * chunk.plane_width);
            
    }
}