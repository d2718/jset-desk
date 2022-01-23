/*!
Struct/methods for saving images, and also saving/recalling image
specifications.
*/

use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;

use lodepng::{ColorType, Encoder, FilterStrategy};
use serde_derive::{Serialize, Deserialize};

use crate::image::*;

/// A container for all the information required to recreate an image.
#[derive(Deserialize, Serialize)]
pub struct ImageParameters {
    iterator:   IterType,
    dimensions: ImageDims,
    color_spec: ColorSpec,
}

/// Save the given image information.
pub fn save<P: AsRef<Path>>(
    dims: &ImageDims,
    cspec: &ColorSpec,
    iter: &IterType,
    fname: &P
) -> Result<(), String> {
    let ips = ImageParameters {
        dimensions: dims.clone(),
        color_spec: cspec.clone(),
        iterator:   iter.clone(),
    };
    
    let toml_string = match toml::to_string(&ips) {
        Ok(s) => s,
        Err(e) => {
            let estr = format!("Error serializing data: {}", &e);
            return Err(estr);
        },
    };
    
    let mut f = match File::create(fname) {
        Ok(f) => f,
        Err(e) => {
            let estr = format!("Error creating output file: {}", &e);
            return Err(estr);
        },
    };
    
    if let Err(e) = f.write_all(&toml_string.as_bytes()) {
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

/// Load the given image information.
pub fn load<P: AsRef<Path>>(fname: P)
-> Result<(ImageDims, ColorSpec, IterType), String> {
    let fname = fname.as_ref();
    let toml_string = match read_to_string(fname) {
        Ok(s) => s,
        Err(e) => {
            let estr = format!("Error reading file {}: {}",
                                fname.display(), &e);
            return Err(estr);
        }
    };
    
    let ips: ImageParameters = match toml::from_str(&toml_string) {
        Ok(x) => x,
        Err(e) => {
            let estr = format!("Error parsing file {}: {}",
                                fname.display(), &e);
            return Err(estr);
        }
    };
    
    Ok((ips.dimensions, ips.color_spec, ips.iterator))
}