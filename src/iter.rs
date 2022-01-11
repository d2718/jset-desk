/*!
Doing the iteration.
*/

use std::sync::mpsc;
use std::thread;

use crate::cx::Cx;
use crate::img::ImageParams;
use crate::rgb;

// When a point's squared modulus exceeds this amount under iteration, it
// will be considered to have "diverged" and will be colored the "default"
// color.
const SQ_MOD_LIMIT: f64 = 1.0e100;

/**
A type to fully describe the type of iteration function to be used.

This combined with an iteration limit (as provided by the length of a
target `rgb::ColorMap`) is all the information required for iterating
a point.

The `fun::Pane` provides an interface for the user to specify this,
its `.get_params()` method returns one of these.
*/
#[derive(Clone, Debug, PartialEq)]
pub enum IterParams {
    Mandlebrot,
    PseudoMandlebrot(Cx, Cx),
    Polynomial(Vec<Cx>)
}

/*
A chunk of image data for parallel processing.

The `IterChunk` starts life with the `data` field empty; all the other
fields contain information about the desired image and what part of it the
`IterChunk` is responsible for. It is then consumed by the function
`iterate_chunk()` and reborn with its `data` field full, describing the
iteration map for its section of the image.
*/
struct IterChunk {
    chunk_order: usize,
    params: IterParams,
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

/**
An image in "iteration map" form. Each pixel is represented by a `usize`
indicating how many iterations the point there took to "diverge". The
actual internal representation is opaque, and its only purpose, really,
is to be combined with an `rgb::ColorMap` in order to produce an
`rgb::FImageData`.
*/
pub struct IterMap {
    pub width:  usize,
    pub height: usize,
    chunks: Vec<IterChunk>,
}

impl IterMap {
    /* Stitch together the given chunks after they've been processed. */
    fn compile(
        xpix: usize,
        ypix: usize,
        mut iterated_chunks: Vec<IterChunk>
    ) -> IterMap {
        iterated_chunks.sort_by_key(|x| x.chunk_order);
        
        IterMap {
            width:  xpix,
            height: ypix,
            chunks: iterated_chunks,
        }
    }
    
    /**
    Combine this `IterMap` with its target `rgb::ColorMap` in order to
    produce a "floating-point image", represented by the `rgb::FImageData`.
    This can then be further processed or turned into an external image
    format.
    */
    pub fn color(&self, map: &rgb::ColorMap) -> rgb::FImageData {
        let mut v: Vec<rgb::RGB> = Vec::with_capacity(self.width * self.height);
        for chunk in self.chunks.iter() {
            for n in chunk.data.iter() { v.push(map.get(*n)) }
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

fn pseudomandle_maker(a: Cx, b: Cx) -> Box<dyn Fn(Cx, usize) -> usize> {
    let f = move |c, limit| {
        let mut z = Cx { re: 0.0, im: 0.0 };
        let pseudo_c = b * c;
        
        for n in 0..limit {
            z = (a * z * z) + pseudo_c;
            if z.sqmod() > SQ_MOD_LIMIT { return n; }
        }
        limit
    };
    Box::new(f)
}

fn polyiter_maker(v: Vec<Cx>) -> Box<dyn Fn(Cx, usize) -> usize> {
    let deg = v.len() - 1;
    let f = move |c, limit| {
        let mut z = c;
        for n in 0..limit {
            let mut tot = Cx { re: 0.0, im: 0.0 };
            let mut w = Cx { re: 1.0, im: 0.0 };
            for a in v[0..deg].iter() {
                tot = tot + (*a) * w;
                w = w * z;
            }
            tot = unsafe { tot + (*v.get_unchecked(deg) * w) };
            z = tot;
            if z.sqmod() > SQ_MOD_LIMIT { return n; }
        }
        limit
    };
    
    Box::new(f)
}

fn iterate_chunk(mut chunk: IterChunk, limit: usize) -> IterChunk {
    chunk.data = Vec::with_capacity(chunk.width * chunk.n_rows);
    let f_width  = chunk.width as f64;
    let f_height = chunk.height as f64;
    let f = match chunk.params {
        IterParams::Mandlebrot => Box::new(mandlebrot_iterator),
        IterParams::PseudoMandlebrot(a, b) => pseudomandle_maker(a, b),
        IterParams::Polynomial(ref v) => polyiter_maker(v.clone()),
    };
    
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

pub fn make_iter_map(
    img_params: ImageParams,
    iter_params: IterParams,
    iter_limit: usize,
    n_threads: usize,
) -> IterMap {
    let n_chunks = n_threads * 2;
    let chunk_height = img_params.ypix / n_chunks;
    let last_chunk_height = img_params.ypix % n_chunks;
    let img_height: f64 = img_params.width * (img_params.ypix as f64) / (img_params.xpix as f64);
    
    let mut to_process: Vec<IterChunk> = Vec::new();
    let mut start_y: usize = 0;
    for n in 0..n_chunks {
        let ic = IterChunk {
            chunk_order: n,
            params: iter_params.clone(),
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
            params: iter_params.clone(),
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
                    let nic = iterate_chunk(ic, iter_limit);
                    txc.send(nic).unwrap();
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
    
    IterMap::compile(img_params.xpix, img_params.ypix, done_chunks)
}

#[cfg(test)]
mod test {
    use super::*;
}