use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::PathBuf;

use consts::*;
use errors::FontCreationError;

use glob::{glob, Paths};

extern crate png;

use regex::Regex;

extern crate sega_cmp;

pub fn list_tiles(input_dir : &PathBuf) -> Result<Paths, FontCreationError> {
    if !input_dir.exists() {
        return Err(FontCreationError::new(format!("Directory does not exist: {}", input_dir.to_string_lossy())));
    }
    match glob(&input_dir.join("*.png").to_string_lossy()) {
        Ok(glob) => return Ok(glob),
        Err(e) => return Err(FontCreationError::new(format!("Error listing files: {}", e))),
    }
}

fn reverse_chunk(input : &[u8]) -> Vec<u8> {
    let mut v = vec![];
    v.extend(input);
    v.reverse();

    return v;
}

fn collapse_bits(bytes : &[u8]) -> Result<u8, FontCreationError> {
    if !bytes.len() == 8 {
        return Err(FontCreationError::new(format!("Input must be 8 bytes long ({} elements provided)", bytes.len())));
    }
    let mut result = 0;
    for (i, byte) in bytes.iter().enumerate() {
        let mask = (1 as u8) << i;

        // Are we setting this bit to 0 or 1?
        match *byte {
            0 => result |= mask,
            1 => result &= !mask,
            _ => {
                return Err(FontCreationError::new(format!("Bits must be either 0 or 1 (value was {})", *byte)));
            }
        }
    }
    return Ok(result);
}

fn rgb_to_2bit(bytes : &[u8]) -> Vec<u8> {
    let bytes_vec = bytes.to_vec();
    if bytes_vec == vec![217, 217, 217] || bytes_vec == vec![216, 216, 216] {
        return Vec::from(BITS_GREY.iter().as_slice().clone());
    } else if bytes_vec == vec![0, 16, 64] {
        return Vec::from(BITS_DARK_BLUE.iter().as_slice().clone());
    } else if bytes_vec == vec![128, 128, 176] {
        return Vec::from(BITS_LIGHT_BLUE.iter().as_slice().clone());
    } else {
        return Vec::from(BITS_TRANSPARENT.iter().as_slice().clone());
    }
}

pub fn decode_png(input : &PathBuf) -> Result<Vec<u8>, io::Error> {
    let decoder = png::Decoder::new(File::open(&input)?);
    let (info, mut reader) = decoder.read_info()?;
    if info.height != 16 || !(info.width == 8 || info.width == 16) {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("Incorrect tile size {}x{} (expected 8x16 or 16x16)", info.width, info.height)));
    }
    let mut buf = vec![0; info.buffer_size()];
    reader.next_frame(&mut buf)?;

    match info.color_type {
        // RGB is fine as-is
        png::ColorType::RGB => {},
        // Drop the alpha channel
        png::ColorType::RGBA => {
            buf = buf
                // In every set of four bytes, the fourth is the alpha
                .chunks(4).flat_map(|a| vec![a[0], a[1], a[2]])
                .collect::<Vec<u8>>();
        },
        _ => {
            return Err(io::Error::new(io::ErrorKind::InvalidData,
                format!("Invalid colour format - only RGB is supported")));
        }
    }

    // Match R,G,B pixel values to the 2-bit values that Lunar uses
    let bit_values : Vec<u8> = buf.chunks(3)
       .flat_map(rgb_to_2bit)
       .collect::<Vec<u8>>()
       // Flip each quarter of the image
       .chunks(128).rev().flat_map(|a| a).cloned()
       .collect::<Vec<u8>>()
       // Flip the image vertically; a row of 32 bits is 16 pixels
       .chunks(32).rev().flat_map(|a| a).cloned()
       .collect::<Vec<u8>>()
       // Flip each line horizontally
       .chunks(32).flat_map(|slice| slice.chunks(2).rev().flat_map(|a| a).cloned())
       .collect();
    // Take our big Vec of bit values and collapse that down into bytes
    let mut output_bytes : Vec<u8> = bit_values.chunks(8)
       .flat_map(collapse_bits)
       .collect();

    // Reverse the horizontal order of pixels.
    // The order returned by a given PNG is the opposite of what Lunar expects.
    output_bytes = output_bytes.chunks(info.width as usize)
        .flat_map(reverse_chunk)
        .collect::<Vec<u8>>();

    // 256 pixels per 16x16 glyph, with four pixels per byte, should be 64
    debug_assert!(output_bytes.len() == 64);

    return Ok(output_bytes);
}

pub fn create_font_data(input_dir : &PathBuf) -> Result<Vec<u8>, FontCreationError> {
    let mut codepoints : Vec<u8> = vec![];
    let mut imagedata : Vec<u8> = vec![];

    for file in list_tiles(&input_dir)?.filter_map(Result::ok) {
        let codepoint = parse_codepoint_from_filename(&file.to_string_lossy())?;
        codepoints.push(codepoint);

        match decode_png(&file) {
            Ok(bytes) => imagedata.extend(bytes),
            Err(e) => {
                return Err(FontCreationError::new(format!("Unable to parse image data for file {}!\n{}", &file.to_string_lossy(), e)));
            }
        }
    }

    return Ok(imagedata);
}

pub fn parse_codepoint_from_filename(filename : &str) -> Result<u8, FontCreationError> {
    let filename = String::from(filename);
    let re = Regex::new(r"(\d*)\.png$").unwrap();
    if !re.is_match(&filename) {
        return Err(FontCreationError::new(format!("Unable to parse codepoint from filename: {}", filename)));
    }

    let captures = re.captures(&filename).unwrap();
    return Ok(captures[1].parse().unwrap());
}

pub fn write_compressed(imagedata: Vec<u8>, mut target_file: &File) -> Result<(), io::Error> {
    let header = sega_cmp::create_header(imagedata.len() as i32, sega_cmp::Size::Byte);
    let compressed; 
    match sega_cmp::compress(&imagedata, sega_cmp::Size::Byte) {
        Ok(d) => compressed = d,
        Err(e) => return Err(io::Error::new(io::ErrorKind::Other,
                                                 format!("{}", e))),
    }

    target_file.write_all(&header)?;
    target_file.write_all(&compressed)?;
    return Ok(());
}

pub fn write_uncompressed(imagedata: Vec<u8>, mut target_file: &File) -> Result<(), io::Error> {
    target_file.write_all(&imagedata)?;

    return Ok(());
}

pub fn insert_data_into_file(mut data: Vec<u8>, target_data: Vec<u8>, game: Game) -> Result<Vec<u8>, FontCreationError> {
    assert_eq!(target_data.len(), game.system_dat_size() as usize);

    // Uncompressed size should match the original
    if data.len() > game.font_len_uncompressed() as usize {
        return Err(FontCreationError::new(format!("Requested font is too large for SYSTEM.DAT (provided size {}, max size {})", data.len(), game.font_len_uncompressed())));
    }
    data.resize(game.font_len_uncompressed() as usize, 0);

    let mut compressed;
    match sega_cmp::compress(&data, sega_cmp::Size::Byte) {
        Ok(d) => compressed = d,
        Err(e) => return Err(FontCreationError::new(format!("{}", e))),
    }
    // Compressed size also has to match the original, and almost certainly needs padding
    compressed.resize(game.font_len_compressed() as usize, 0);
    let mut new_data = target_data.clone();
    // We ignore the latter half of the clone entirely
    new_data.split_off(game.font_start_address() as usize);
    new_data.append(&mut compressed);
    let mut antecedent = target_data.clone().split_off((game.font_start_address() + game.font_len_compressed()) as usize);
    new_data.append(&mut antecedent);
    assert_eq!(new_data.len(), game.system_dat_size() as usize);

    return Ok((new_data));
}
