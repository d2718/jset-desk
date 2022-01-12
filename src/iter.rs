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

/*
Iterate a point using the Mandlebrot iterator.

This function is called by `iterate_chunk()` below for `IterChunk`s whose
`IterParams` are of type `Mandlebrot`.
*/
fn mandlebrot_iterator(c: Cx, limit: usize) -> usize {
    let mut z = Cx { re: 0.0, im: 0.0 };
    
    for n in 0..limit {
        z = (z * z) + c;
        if z.sqmod() > SQ_MOD_LIMIT { return n; }
    }
    limit
}

/*
Generate and return a function (a closure) to iterate a point using a
Pseudo-Mandlebrot iterator.

I'm not sure if this is a real thing, but it's what I'm calling it. The
_Mandlebrot_ iterator uses the function

   f(z) = z^2 + c

to iterate a given point _c_. A _Pseudo_-Mandlebrot iterator is a parametrized
mapping, such that for a given complex (a, b),

   f(a, b)(z) = az^2 + bc

iterates the given point _c_.

This function is called by `iterate_chunk()` below for `IterChunk`s whose
`IterParams` are of type `PseudoMandlebrot`.
*/
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

/*
Generate and return a function (a closure) to iterate a point using an
arbitrary polynomial iterator.

Given a vector `v` of complex coefficients, this function will generate
the iteration function

    f(z) = v[0]*z + v[1]*z^2 + v[2]*z^3 + ...
    
It is called by `iterate_chunk()` for `IterChunk`s whose `IterParams` are
of type `Polynomial`.
*/
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
A description of a portion of an image to be iterated, sutable to be processed
in parallel with other `ChunkRecipe`s. Together with the length of a target,
`rgb::ColorMap`, this is all the information required to make an iteration
map for a portion of an image.

Processing with the `.iterate()` method will consume this and return an
`IterMapchunk`, which contains the actual portion of the iteration map.
*/
struct ChunkRecipe {
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
}

/*
A chunk of an image, specified by a `ChunkRecipe`, after processing.

A vector of these, together with image dimensions in pixels, specify an
iteration map.
*/
struct IterMapChunk {
    chunk_order: usize,
    data: Vec<usize>,
}

impl ChunkRecipe {
    /* Consume this `ChunkRecipe`, do the iteration, and produce an
    `IterMapChunk` */
    fn iterate(self, limit: usize) -> IterMapChunk {
        let mut data = Vec::with_capacity(self.width * self.n_rows);
        let f_width  = self.width as f64;
        let f_height = self.height as f64;
        let f = match self.params {
            IterParams::Mandlebrot => Box::new(mandlebrot_iterator),
            IterParams::PseudoMandlebrot(a, b) => pseudomandle_maker(a, b),
            IterParams::Polynomial(v) => polyiter_maker(v),
        };
        
        for yp in self.y_start..(self.y_start + self.n_rows) {
            let y_frac = (yp as f64) / f_height;
            let y = self.y - (y_frac * self.plane_height);
            for xp in 0..self.width {
                let x_frac = (xp as f64) / f_width;
                let x = self.x + (x_frac * self.plane_width);
                let n = f(Cx { re: x, im: y }, limit);
                data.push(n);
            }
        }
        
        IterMapChunk {
            chunk_order: self.chunk_order,
            data,
        }
    }
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
    chunks: Vec<IterMapChunk>,
}

impl IterMap {
    /* Stitch together the given chunks after they've been processed. */
    fn compile(
        xpix: usize,
        ypix: usize,
        mut iterated_chunks: Vec<IterMapChunk>
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

/**
This function combines all the image and iteration parameters, and does the
heavy arithmetic to make an `IterMap`.

`iter_limit` should be the length of the target `ColorMap` that will be used
to color the `IterMap`.
*/
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
    
    let mut to_process: Vec<ChunkRecipe> = Vec::new();
    let mut start_y: usize = 0;
    for n in 0..n_chunks {
        let ic = ChunkRecipe {
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
        };
        to_process.push(ic);
        start_y += chunk_height;
    }
    if last_chunk_height > 0 {
        let ic = ChunkRecipe {
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
        };
        to_process.push(ic);
    }
    
    let mut done_chunks: Vec<IterMapChunk> = Vec::new();
    let n_chunks = to_process.len();
    let mut active_threads: usize = 0;
    let (tx, rx) = mpsc::channel::<IterMapChunk>();
    while done_chunks.len() < n_chunks {
        if active_threads < n_threads {
            if let Some(cr) = to_process.pop() {
                #[cfg(test)]
                println!("chunk {} ->", cr.chunk_order);
                let txc = tx.clone();
                thread::spawn(move || {
                    let imc = cr.iterate(iter_limit);
                    txc.send(imc).unwrap();
                });
                active_threads += 1;
            }
        }
        if active_threads == n_threads || to_process.len() == 0 {
            let imc = rx.recv().unwrap();
            #[cfg(test)]
            println!("-> chunk {}", imc.chunk_order);
            active_threads -= 1;
            done_chunks.push(imc);
        }
    }
    
    IterMap::compile(img_params.xpix, img_params.ypix, done_chunks)
}

#[cfg(test)]
mod test {
    use super::*;
}