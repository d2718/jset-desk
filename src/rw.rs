/*!
Struct/methods to save/recall an image specification.
*/

use std::fs::{File, read_to_string};
use std::io::Write;
use std::path::Path;

use::serde_derive::{Serialize, Deserialize};

use crate::image::*;

#[derive(Deserialize, Serialize)]
pub struct ImageParameters {
    iterator:   IterType,
    dimensions: ImageDims,
    color_spec: ColorSpec,
}


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

pub fn save_as_png<P: AsRef<Path>>(
    fname: P,
    xpix: usize,
    ypix: usize,
    data: &[u8]
) -> Result<(), String> {
    let fname = fname.as_ref();
    match lodepng::encode_file(fname, data, xpix, ypix,
                                lodepng::ColorType::RGB, 8) {
        Err(e) => {
            let estr = format!("Error writing file {}: {}",
                               fname.display(), &e);
            Err(estr)
        },
        Ok(_) => Ok(()),
    }
}

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