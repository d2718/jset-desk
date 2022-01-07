/*!
Doing the iteration.
*/

use std::iter::Iterator;
use std::sync::mpsc;
use std::thread;

use crate::cx::Cx;
use crate::img::ImageParams;
use crate::rgb;

const SQ_MOD_LIMIT: f64 = 1.0e100;

#[derive(Clone, Debug, PartialEq)]
pub enum IterParams {
    Mandlebrot,
    PseudoMandlebrot(Cx, Cx),
    Polynomial(Vec<Cx>)
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

pub struct IterMap {
    pub width:  usize,
    pub height: usize,
    chunks: Vec<IterChunk>,
}

impl IterMap {
    pub fn color(&self, &map: ColorMap) -> rgb::FImageData {
        let mut v: Vec<RGB> = Vec::with_capacity(self.width * self.height);
        for chunk in self.chunks.iter() {
            for n in chunk.iter() { v.push(map.get(*n)) }
        }
        rgb::FImageData::new(self.width, self.height, v)
    }
}

fn mandlebrot_iterator(c: Cx, limit: usize) -> usize {
    let mut z = Cx { re: 0.0, im: 0.0 };
    
    for n in 0..limit {
        z = (z * z) + c;
        if z.sqmod() > SQ_MOD_LIMIT { return n; }
    }
    limit
}

fn iterate_chunk<F>(mut chunk: IterChunk, f: F, limit: usize) -> IterChunk
    where F: Fn(Cx, usize) -> usize
{
    chunk.data = Vec::with_capacity(chunk.width * chunk.n_rows);
    let f_width  = chunk.width as f64;
    let f_height = chunk.height as f64;
    
    for yp in chunk.y_start..(chunk.y_start + chunk.n_rows) {
        let y_frac = (yp as f64) / f_height;
        let y = chunk.y - (y_frac * chunk.plane_height);
        for xp in 0..chunk.width {
            let x_frac = (xp as f64) / f_width;
            let x = chunk.x + (x_frac * chunk.plane_width);
            let n = f(Cx { re: x, im: y }, limit);
            chunk.data.push(n);
        }
    }
    
    chunk
}

fn make_iter_map(
    img_params: ImageParams,
    iter_params: IterParams,
    iter_limit: usize,
    n_threads: usize,
) -> Vec::<IterChunk> {
    let n_chunks = n_threads * 2;
    let chunk_height = img_params.ypix / n_chunks;
    let last_chunk_height = img_params.ypix % n_chunks;
    let img_height: f64 = img_params.width * (img_params.ypix as f64) / (img_params.xpix as f64);
    let iter_func = mandlebrot_iterator;
    
    let mut to_process: Vec<IterChunk> = Vec::new();
    let mut start_y: usize = 0;
    for n in 0..n_chunks {
        let ic = IterChunk {
            chunk_order: n,
            width: img_params.xpix,
            height: img_params.ypix,
            x: img_params.x,
            y: img_params.y,
            plane_width: img_params.width,
            plane_height: img_height,
            y_start: start_y,
            n_rows: chunk_height,
            data: Vec::new(),
        };
        to_process.push(ic);
        start_y += chunk_height;
    }
    if last_chunk_height > 0 {
        let ic = IterChunk {
            chunk_order: n_chunks,
            width: img_params.xpix,
            height: img_params.ypix,
            x: img_params.x,
            y: img_params.y,
            plane_width: img_params.width,
            plane_height: img_height,
            y_start: start_y,
            n_rows: last_chunk_height,
            data: Vec::new(),
        };
        to_process.push(ic);
    }
    
    let mut done_chunks: Vec<IterChunk> = Vec::new();
    let n_chunks = to_process.len();
    let mut active_threads: usize = 0;
    let (tx, rx) = mpsc::channel::<IterChunk>();
    while done_chunks.len() < n_chunks {
        if active_threads < n_threads {
            if let Some(ic) = to_process.pop() {
                #[cfg(test)]
                println!("chunk {} ->", ic.chunk_order);
                let txc = tx.clone();
                thread::spawn(move || {
                    let nic = iterate_chunk(ic, iter_func, iter_limit);
                    txc.send(nic);
                });
                active_threads += 1;
            }
        }
        if active_threads == n_threads || to_process.len() == 0 {
            let nic = rx.recv().unwrap();
            #[cfg(test)]
            println!("-> chunk {}", nic.chunk_order);
            active_threads -= 1;
            done_chunks.push(nic);
        }
    }
    
    done_chunks.sort_by_key(|x| x.chunk_order);
    
    done_chunks
}

#[cfg(test)]
mod test {
    use super::*;
    
    #[test]
    fn iter_test() {
        let mut imgp = ImageParams::default();
        imgp.ypix = imgp.ypix + 4;
        let iterp = IterParams::Mandlebrot;
        
        let test_chunks = make_iter_map(imgp, iterp, 256, 3);
        for chunk in &test_chunks {
            println!("chunk {}: starts {}, {} lines, {} values",
                chunk.chunk_order, chunk.y_start, chunk.n_rows,
                chunk.data.len());
        }
        
    }
}