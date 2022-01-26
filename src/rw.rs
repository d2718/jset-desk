/*!
Struct/methods for saving images, and also saving/recalling image
specifications.
*/

use std::fs::File;
use std::io::{BufWriter, Read, Seek, Write};
use std::path::Path;

//use lodepng::{ColorType, Encoder, FilterStrategy};
use serde_derive::{Deserialize, Serialize};

use crate::image::*;

// Maximum size/amount of a file to be read when attempting to decode a
// .toml file.
const READ_LIMIT: usize = 16 * 1024;

/// A container for all the information required to recreate an image.
#[derive(Deserialize, Serialize)]
pub struct ImageParameters {
    iterator: IterType,
    dimensions: ImageDims,
    color_spec: ColorSpec,
}

impl ImageParameters {
    pub fn toml(dims: &ImageDims, cspec: &ColorSpec, iter: &IterType) -> Result<String, String> {
        let ips = ImageParameters {
            dimensions: *dims,
            color_spec: cspec.clone(),
            iterator: iter.clone(),
        };

        match toml::to_string(&ips) {
            Ok(s) => Ok(s),
            Err(e) => Err(format!("Error serializing data: {}", &e)),
        }
    }
}

enum LoadResult {
    Success(ImageParameters),
    GiveUp(String),
    TryOtherType,
}

/// Save the given image information.
pub fn save<P: AsRef<Path>>(
    dims: &ImageDims,
    cspec: &ColorSpec,
    iter: &IterType,
    fname: &P,
) -> Result<(), String> {
    let toml_string = ImageParameters::toml(dims, cspec, iter)?;

    let mut f = match File::create(fname) {
        Ok(f) => f,
        Err(e) => {
            let estr = format!("Error creating output file: {}", &e);
            return Err(estr);
        }
    };

    if let Err(e) = f.write_all(toml_string.as_bytes()) {
        let estr = format!("Error writing to output file: {}", &e);
        return Err(estr);
    }

    if let Err(e) = f.flush() {
        let estr = format!("Error flushing output file: {}", &e);
        return Err(estr);
    }

    Ok(())
}

/// Save the given _image_. Uses maximum zlib compression.
/*
pub fn save_as_png<P: AsRef<Path>>(
    fname: P,
    xpix: usize,
    ypix: usize,
    data: &[u8]
) -> Result<(), String> {
    let mut enc = Encoder::new();
    enc.set_auto_convert(true);
    enc.set_filter_strategy(FilterStrategy::MINSUM, false);
    {
        let mode = enc.info_raw_mut();
        mode.set_colortype(ColorType::RGB);
        mode.set_bitdepth(8);
    }
    {
        let mut nfo = enc.info_png_mut();
        nfo.color.set_colortype(ColorType::RGB);
        nfo.color.set_bitdepth(8);
        nfo.background_defined = false;
        nfo.phys_unit = 0;
    }
    enc.settings_mut().zlibsettings.set_level(9);

    if let Err(e) = enc.encode_file(&fname, data, xpix, ypix) {
        let estr = format!("Error saving file {}: {}",
                            fname.as_ref().display(), &e);
        Err(estr)
    } else {
        Ok(())
    }
}
*/

pub fn save_with_metadata<P: AsRef<Path>>(
    fname: P,
    xpix: usize,
    ypix: usize,
    data: &[u8],
    dims: &ImageDims,
    cspec: &ColorSpec,
    iter: &IterType,
) -> Result<(), String> {
    let fname = fname.as_ref();
    let metadata = ImageParameters::toml(dims, cspec, iter)?;
    let f = match File::create(fname) {
        Ok(f) => f,
        Err(e) => {
            let estr = format!("Error opening {} for writing: {}", fname.display(), &e);
            return Err(estr);
        }
    };
    let mut w = BufWriter::new(f);

    let mut enc = png::Encoder::new(&mut w, xpix as u32, ypix as u32);
    enc.set_color(png::ColorType::Rgb);
    enc.set_depth(png::BitDepth::Eight);
    enc.set_filter(png::FilterType::Paeth);
    enc.set_compression(png::Compression::Best);
    if let Err(e) = enc.add_itxt_chunk("jset_desk parameters".to_string(), metadata) {
        let estr = format!("Error writing metadata: {}", &e);
        return Err(estr);
    }
    let mut writer = match enc.write_header() {
        Err(e) => {
            let estr = format!("Error writing PNG header: {}", &e);
            return Err(estr);
        }
        Ok(x) => x,
    };
    if let Err(e) = writer.write_image_data(data) {
        let estr = format!("Error writing image data: {}", &e);
        return Err(estr);
    }

    Ok(())
}

fn try_to_fill<R: Read>(r: &mut R, buff: &mut [u8]) -> Result<usize, std::io::Error> {
    let mut total_read: usize = 0;

    loop {
        match r.read(&mut buff[total_read..])? {
            0 => {
                return Ok(total_read);
            }
            n => {
                total_read += n;
            }
        }
    }
}

fn try_load_toml(f: &mut File) -> LoadResult {
    let mut buff: Vec<u8> = vec![0; READ_LIMIT];

    let str_len = match try_to_fill(f, &mut buff) {
        Ok(n) => n,
        Err(e) => {
            return LoadResult::GiveUp(e.to_string());
        }
    };

    let toml_str = match std::str::from_utf8(&buff[..str_len]) {
        Ok(s) => s,
        Err(_) => {
            return LoadResult::TryOtherType;
        }
    };

    let ips: ImageParameters = match toml::from_str(toml_str) {
        Ok(x) => x,
        Err(_) => {
            return LoadResult::TryOtherType;
        }
    };

    LoadResult::Success(ips)
}

fn try_load_png(f: &mut File) -> LoadResult {
    let dec = png::Decoder::new(f);
    let rdr = match dec.read_info() {
        Ok(r) => r,
        Err(e) => {
            return LoadResult::GiveUp(e.to_string());
        }
    };

    let mut meta_text: Option<String> = None;

    for chunk in rdr.info().utf8_text.iter() {
        if &chunk.keyword == "jset_desk parameters" {
            match chunk.get_text() {
                Ok(s) => {
                    meta_text = Some(s);
                    break;
                }
                Err(e) => {
                    eprintln!("Error decoding metadata text chunk: {}", &e);
                }
            }
        }
    }

    let meta_text = match meta_text {
        Some(s) => s,
        None => {
            return LoadResult::GiveUp(
                "File contains no recognizable metadata parameters.".to_string(),
            );
        }
    };

    let ips: ImageParameters = match toml::from_str(&meta_text) {
        Ok(x) => x,
        Err(e) => {
            let estr = format!("Error decoding metadata chunk: {}", &e);
            return LoadResult::GiveUp(estr);
        }
    };

    LoadResult::Success(ips)
}

pub fn load<P: AsRef<Path>>(fname: P) -> Result<(ImageDims, ColorSpec, IterType), String> {
    let fname = fname.as_ref();
    let mut f = match File::open(fname) {
        Ok(f) => f,
        Err(e) => {
            let estr = format!("Error opening file {}: {}", fname.display(), &e);
            return Err(estr);
        }
    };

    match try_load_toml(&mut f) {
        LoadResult::Success(ips) => {
            return Ok((ips.dimensions, ips.color_spec, ips.iterator));
        }
        LoadResult::GiveUp(e) => {
            return Err(e);
        }
        LoadResult::TryOtherType => { /* continue trying other type! */ }
    }

    if let Err(e) = f.seek(std::io::SeekFrom::Start(0)) {
        return Err(e.to_string());
    }

    match try_load_png(&mut f) {
        LoadResult::Success(ips) => Ok((ips.dimensions, ips.color_spec, ips.iterator)),
        LoadResult::GiveUp(e) => Err(e),
        LoadResult::TryOtherType => Err("Could not load from PNG for some reason.".to_string()),
    }
}

//~ pub fn load_from_metadata<P: AsRef<Path>>(fname: P)
//~ -> Result<(ImageDims, ColorSpec, IterType), String> {
//~ let fname = fname.as_ref();
//~ let f = match File::open(fname) {
//~ Ok(f) => f,
//~ Err(e) => {
//~ let estr = format!("Error opening file {}: {}",
//~ fname.display(), &e);
//~ return Err(estr);
//~ },
//~ };

//~ let dec = png::Decoder::new(f);
//~ let rdr = match dec.read_info() {
//~ Ok(r) => r,
//~ Err(e) => {
//~ let estr = format!("Error reading file {} metadata: {}",
//~ fname.display(), &e);
//~ return Err(estr);
//~ },
//~ };

//~ let mut meta_text: Option<String> = None;

//~ for chunk in rdr.info().utf8_text.iter() {
//~ if &chunk.keyword == "jset_desk parameters" {
//~ match chunk.get_text() {
//~ Ok(s) => {
//~ meta_text = Some(s);
//~ break;
//~ },
//~ Err(e) => {
//~ eprintln!("Error decoding metadata text chunk: {}", &e);
//~ },
//~ }
//~ }
//~ }

//~ let meta_text = match meta_text {
//~ Some(s) => s,
//~ None => {
//~ let estr = format!("Unable to read any image parameters from {}.",
//~ fname.display());
//~ return Err(estr);
//~ },
//~ };

//~ let ips: ImageParameters = match toml::from_str(&meta_text) {
//~ Ok(x) => x,
//~ Err(e) => {
//~ let estr = format!("Error parsing image parameters from {}: {}",
//~ fname.display(), &e);
//~ return Err(estr);
//~ },
//~ };

//~ Ok((ips.dimensions, ips.color_spec, ips.iterator))
//~ }

// Load the given image information.
//~ pub fn load<P: AsRef<Path>>(fname: P)
//~ -> Result<(ImageDims, ColorSpec, IterType), String> {
//~ let fname = fname.as_ref();
//~ let toml_string = match read_to_string(fname) {
//~ Ok(s) => s,
//~ Err(e) => {
//~ let estr = format!("Error reading file {}: {}",
//~ fname.display(), &e);
//~ return Err(estr);
//~ }
//~ };

//~ let ips: ImageParameters = match toml::from_str(&toml_string) {
//~ Ok(x) => x,
//~ Err(e) => {
//~ let estr = format!("Error parsing file {}: {}",
//~ fname.display(), &e);
//~ return Err(estr);
//~ }
//~ };

//~ Ok((ips.dimensions, ips.color_spec, ips.iterator))
//~ }
