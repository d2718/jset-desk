/*!
Everything required for specifying and creating the bytes of an image.
*/

use std::sync::mpsc;
use std::thread;

use lazy_static::lazy_static;

use crate::cx::Cx;

lazy_static!{
    static ref N_THREADS: usize = num_cpus::get_physical();
}

// When a point's squared modulus exceeds this amount under iteration, it
// will be considered to have "diverged" and will be colored the "default"
// color.
const SQ_MOD_LIMIT: f64 = 1.0e100;

const CHUNKS_PER_THREAD: usize = 2;
const MAX_SCALE_FACTOR: usize = 5;
const SCALE_PALETTE_SIZE: usize = MAX_SCALE_FACTOR * MAX_SCALE_FACTOR;

/**
Represents a color with red, green, and blue components as floating-point
numbers in the range [0.0, 255.0]. This is the form in which it's easiest
to do calculations. Includes a method for converting to hard-byte RGB format.
*/
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RGB { r: f32, g: f32, b: f32 }

// For constraining the arguments to `RGB::new()` to the proper range.
fn constrain_f32(x: f32) -> f32 {
    if x < 0.0 { 0.0 }
    else if x > 255.0 { 255.0 }
    else { x }
}

impl RGB {
    /**
    Instantiate a new `RGB` color representation with the given color
    component values. Values outside the accepted range will be clamped.
    */
    pub fn new(red: f32, green: f32, blue: f32) -> RGB {
        RGB {
            r: constrain_f32(red),
            g: constrain_f32(green),
            b: constrain_f32(blue),
        }
    }

    /** Convert to a three-byte `[R, G, B]` array. */
    pub fn to_rgb8(&self) -> [u8; 3] {
        [
            self.r as u8,
            self.g as u8,
            self.b as u8
        ]
    }
    
    /** Average a slice of color values. */
    pub fn average(colors: &[RGB]) -> RGB {
        let (mut rtot, mut gtot, mut btot) : (f32, f32, f32) = (0.0, 0.0, 0.0);
        
        for px in colors.iter() {
            rtot += px.r; gtot += px.g; btot += px.b;
        }
        
        let nf = colors.len() as f32;
        RGB::new(rtot/nf, gtot/nf, btot)
    }
    
    pub const BLACK:  RGB = RGB { r: 0.0, g: 0.0, b: 0.0 };
    pub const WHITE:  RGB = RGB { r: 255.0, g: 255.0, b: 255.0 };
}

/**
Represents a mapping from the pixels of an image to a view of the
complex plane. `xpix` and `ypix` are the dimensions of the image in pixels,
(`x`, `y`) is the location of the upper-left-hand corner of the image on
the complex plane, and `width` is the horizontal size of the image on the
complex plane.
*/
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ImageDims {
    pub xpix: usize,
    pub ypix: usize,
    pub x: f64,
    pub y: f64,
    pub width: f64
}

impl ImageDims {
    /** Return the vertical size of the image on the complex plane. */
    pub fn height(&self) -> f64 {
        self.width * (self.ypix as f64) / (self.xpix as f64)
    }
    
    /** Return the coordinates of the center of the image. */
    pub fn center(&self) -> (f64, f64) {
        (
            self.x + self.width / 2.0,
            self.y + self.height() / 2.0,
        )
    }
    
    /** Return a new view zoomed in by the given factor. */
    pub fn zoom(&self, factor: f64) -> ImageDims {
        let (c_x, c_y) = self.center();
        let (n_w, n_h) = (self.width / factor, self.height() / factor);
        let (n_x, n_y) = (c_x - n_w / 2.0, c_y + n_h / 2.0);
        
        ImageDims {
            xpix: self.xpix,
            ypix: self.ypix,
            x: n_x,
            y: n_y,
            width: n_w,
        }
    }
    
    /**
    Return a new view centered on the same spot, but with the aspect
    ratio changed.
    
    The new view will cover at least as much of the plane as the current one.
    */
    pub fn resize(&self, new_xpix: usize, new_ypix: usize) -> ImageDims {
        let cur_aspect = (self.xpix as f64) / (self.ypix as f64);
        let new_aspect = (new_xpix as f64) / (new_ypix as f64);
        let (c_x, c_y) = self.center();
        
        if new_aspect > cur_aspect {
            let new_w = self.height() * new_aspect;
            let n_x = c_x - new_w / 2.0;
            ImageDims {
                xpix: new_xpix,
                ypix: new_ypix,
                x: n_x,
                y: self.y,
                width: new_w,
            }
        } else {
            let new_h = self.width / new_aspect;
            let n_y = c_y + new_h / 2.0;
            ImageDims {
                xpix: new_xpix,
                ypix: new_ypix,
                x: self.x,
                y: n_y,
                width: self.width,
            }
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Gradient { pub start: RGB, pub end: RGB, pub steps: usize }

#[derive(Clone, Debug)]
pub struct ColorMap {
    gradients: Vec<Gradient>,
    length: usize,
    default: RGB,
    colors: Vec<RGB>,
}

impl ColorMap {
    pub fn make(gradients: Vec<Gradient>, default: RGB) -> ColorMap {
        let length = gradients.iter().map(|g| g.steps).sum();
        let mut colors: Vec<RGB> = Vec::with_capacity(length);
        
        for grad in gradients.iter() {
            let dr = grad.end.r - grad.start.r;
            let dg = grad.end.g - grad.start.g;
            let db = grad.end.b - grad.start.b;
            let steps_f = grad.steps as f32;
            for n in 0..grad.steps {
                let frac = (n as f32) / steps_f;
                let c = RGB::new(
                    grad.start.r + frac*dr,
                    grad.start.g + frac*dg,
                    grad.start.b + frac*db,
                );
                colors.push(c);
            }
        }
        
        ColorMap { gradients, length, default, colors }
    }
    
    pub fn len(&self) -> usize { self.length }
    
    pub fn get(&self, n: usize) -> RGB {
        match self.colors.get(n) {
            Some(c) => *c,
            None => self.default,
        }
    }
}

impl PartialEq for ColorMap {
    fn eq(&self, other: &Self) -> bool {
        (self.default == other.default)
        && (self.gradients == other.gradients)
    }
}

pub struct FImage32 {
    dims: ImageDims,
    data: Vec<RGB>,
}

impl FImage32 {
    pub fn xpix(&self) -> usize { self.dims.xpix }
    pub fn ypix(&self) -> usize { self.dims.ypix }
    pub fn pixels(&self) -> &[RGB] { &self.data }
    
    fn to_rgb8_full_resolution(&self) -> Vec<u8> {
        let n_pix = self.dims.xpix * self.dims.ypix;
        let mut rgb8_data: Vec<u8> = Vec::with_capacity(n_pix * 3);
        for p in self.data.iter() {
            for b in p.to_rgb8().iter() {
                rgb8_data.push(*b);
            }
        }
    
    rgb8_data
    }
    
    fn to_rgb8_scaled(&self, ratio: usize) -> (usize, usize, Vec<u8>) {
        let pix_lines = self.dims.xpix / ratio;
        let pix_cols  = self.dims.ypix / ratio;
        let n_pix     = pix_lines * pix_cols;
        let mut rgb8_data: Vec<u8> = Vec::with_capacity(n_pix * 3);
        let mut palette: [RGB; SCALE_PALETTE_SIZE]
                = [RGB::BLACK, SCALE_PALETTE_SIZE];
        
        for yi in 0..pix_lines {
            let base_offs = yi * self.dims.xpix * ratio;
            for xi in 0..pixcols {
                let offs = base_offs + (xi * ratio);
                let mut pp = 0usize;
                for y in 0..ratio {
                    let po = offs + (self.dims.xpix * y);
                    for x in 0..ratio {
                        palette[pp] = self.data[po+x];
                        pp += 1;
                    }
                }
                let avg_p = RGB::average(&palette[0..pp]);
                for b in avg_p.to_rgb8().iter {
                    rgb8_data.push(*b);
                }
            }
        }
        
        (pix_cols, pix_lines, rgb8_data)
    }
    
    pub fn to_rgb8(&self, scale_factor: usize) -> (usize, usize, Vec::<u8>) {
        if scale_factor < 2 {
            (
                self.dims.xpix,
                self.dims.ypix,
                self.to_rgb8_full_resolution()
            )
        else if scale_factor > MAX_SCALE_FACTOR {
            self.to_rgb8_scaled(MAX_SCALE_FACTOR)
        } else {
            self.to_rgb8_scaled(scale_factor)
        }
    }
}

/**
A type to fully describe the type of iteration to be used.

This, combined with an iteration limit (the length of a target `ColorMap`)
is all the information required for iterating a point.
*/
#[derive(Clone, Debug, PartialEq)]
pub enum IterType {
    Mandlebrot,
    PseudoMandlebrot(Cx, Cx),
    Polynomial(Vec<Cx>),
}

/* Iterate a point using the Mandlebrot iterator. */
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

/*
A description of a portion of an image to be iterated, suitable to be processed
in parallel with other `IterMapChunk`s. Together with the length of a target
`ColorMap`, this is all the information required to make an iteration map
for the specified portion of an image.

Processing with `.iterate()` will fill the chunk's `.data` member with the
actual iteration map values.

Processing with `.reiterate()` will extend the iteration map to the new
limit for only those points who were already at the last limit. The idea
here is for redrawing an image where the only thing that has changed is
the length of the `ColorMap`.
*/
struct IterMapChunk {
    dims: ImageDims,
    itertype: IterType,
    y_start: usize,
    n_rows: usize,
    last_limit: usize,
    data: Vec<usize>,
}

impl IterMapChunk {
    fn iterate(&mut self, limit: usize) {
        let n_pix = self.dims.xpix * self.n_rows;
        let mut new_data: Vec<usize> = Vec::with_capacity(n_pix);
        let f_xpix = self.dims.xpix as f64;
        let f_ypix = self.dims.ypix as f64;
        let height = self.dims.height();
        let f = match self.itertype.clone() {
            IterType::Mandlebrot => Box::new(mandlebrot_iterator),
            IterType::PseudoMandlebrot(a, b) => pseudomandle_maker(a, b),
            IterType::Polynomial(v) => polyiter_maker(v),
        };
        
        for yp in self.y_start..(self.y_start + self.n_rows) {
            let y_frac = (yp as f64) / f_ypix;
            let y = self.dims.y - (y_frac + height);
            for xp in 0..self.dims.xpix {
                let x_frac = (xp as f64) / f_xpix;
                let x = self.dims.x + (x_frac * self.dims.width);
                let n = f(Cx { re: x, im: y }, limit);
                new_data.push(n);
            }
        }
        
        self.last_limit = limit;
        self.data = new_data;
    }
    
    fn reiterate(&mut self, limit: usize) {
        if limit < self.last_limit { return; }
        
        let f_xpix = self.dims.xpix as f64;
        let f_ypix = self.dims.ypix as f64;
        let height = self.dims.height();
        let f = match self.itertype.clone() {
            IterType::Mandlebrot => Box::new(mandlebrot_iterator),
            IterType::PseudoMandlebrot(a, b) => pseudomandle_maker(a, b),
            IterType::Polynomial(v) => polyiter_maker(v),
        };
        
        let mut idx: usize = 0;
        for yp in self.y_start..(self.y_start + self.n_rows) {
            let y_frac = (yp as f64) / f_ypix;
            let y = self.dims.y - (y_frac * height);
            for xp in 0..self.dims.xpix {
                if self.data[idx] == self.last_limit {
                    let x_frac = (xp as f64) / f_xpix;
                    let x = self.dims.x + (x_frac * self.dims.width);
                    let n = f(Cx { re: x, im: y }, limit);
                    self.data[idx] = n;
                }
                idx += 1;
            }
        }
        
        self.last_limit = limit;
    }
}


pub struct IterMap {
    dims: ImageDims,
    itertype: IterType,
    limit: usize,
    chunks: Vec<IterMapChunk>,
}

impl IterMap {
    pub fn new(
        dims: ImageDims,
        itertype: IterType,
        limit: usize
    ) -> IterMap {
        let n_chunks = CHUNKS_PER_THREAD * *N_THREADS;
        let chunk_height = dims.ypix / n_chunks;
        let last_chunk_height = dims.ypix % n_chunks;
        
        let mut to_process: Vec<IterMapChunk> = Vec::new();
        let mut start_y: usize = 0;
        for _ in 0..n_chunks {
            let imc = IterMapChunk {
                dims: dims,
                itertype: itertype.clone(),
                y_start: start_y,
                n_rows: chunk_height,
                last_limit: 0,
                data: Vec::new(),
            };
            to_process.push(imc);
            start_y += chunk_height;
        }
        if last_chunk_height > 0 {
            let imc = IterMapChunk {
                dims: dims,
                itertype: itertype.clone(),
                y_start: start_y,
                n_rows: last_chunk_height,
                last_limit: 0,
                data: Vec::new(),
            };
            to_process.push(imc);
        }
        
        let n_chunks = to_process.len();
        let mut done_chunks: Vec<IterMapChunk> = Vec::with_capacity(n_chunks);
        let mut active_threads: usize = 0;
        let (tx, rx) = mpsc::channel::<IterMapChunk>();
        while done_chunks.len() < n_chunks {
            if active_threads < *N_THREADS {
                if let Some(mut imc) = to_process.pop() {
                    let txc = tx.clone();
                    thread::spawn(move || {
                        imc.iterate(limit);
                        txc.send(imc).unwrap();
                    });
                    active_threads += 1;
                }
            }
            if active_threads == *N_THREADS || to_process.len() == 0 {
                let imc = rx.recv().unwrap();
                active_threads -= 1;
                done_chunks.push(imc);
            }
        }
        
        done_chunks.sort_by_key(|imc| imc.y_start);
        
        IterMap {
            dims: dims,
            itertype: itertype.clone(),
            limit: limit,
            chunks: done_chunks,
        }
    }
    
    pub fn reiterate(&mut self, limit: usize) {
        if limit <= self.limit { return; }
        
        let n_chunks = self.chunks.len();
        let mut done_chunks: Vec<IterMapChunk> = Vec::with_capacity(n_chunks);
        let mut active_threads: usize = 0;
        let (tx, rx) = mpsc::channel::<IterMapChunk>();
        while done_chunks.len() < n_chunks {
            if active_threads < *N_THREADS {
                if let Some(mut imc) = self.chunks.pop() {
                    let txc = tx.clone();
                    thread::spawn(move || {
                        imc.reiterate(limit);
                        txc.send(imc).unwrap();
                    });
                    active_threads += 1;
                }
            }
            if active_threads == *N_THREADS || self.chunks.len() == 0 {
                let imc = rx.recv().unwrap();
                active_threads -= 1;
                done_chunks.push(imc);
            }
        }
        
        done_chunks.sort_by_key(|imc| imc.y_start);
        
        std::mem::swap(&mut self.chunks, &mut done_chunks);
        self.limit = limit;
    }
    
    pub fn dims(&self) -> ImageDims { self.dims }
    pub fn itertype(&self) -> &IterType { &self.itertype }
    pub fn limit(&self) -> usize { self.limit }
    
    pub fn color(&self, map: &ColorMap) -> FImage32 {
        let n_pix = self.dims.xpix * self.dims.ypix;
        let mut rgb_data: Vec<RGB> = Vec::with_capacity(n_pix);
        
        for chunk in self.chunks.iter() {
            for n in chunk.data.iter() {
                rgb_data.push(map.get(*n));
            }
        }
        
        FImage32 {
            dims: self.dims,
            data: rgb_data,
        }
    }
}

