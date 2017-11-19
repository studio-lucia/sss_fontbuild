use std::fs::File;
use std::io;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::Path;
use std::process::exit;

extern crate clap;
use clap::{Arg, App};

extern crate glob;
use glob::{glob, Paths};

extern crate png;

extern crate regex;
use regex::Regex;

// Each pixel is two bits; there are four pixels in each byte.
// This provides mappings between the colour value and its definition in bits.
const BITS_TRANSPARENT : [u8; 2] = [1, 1];
const BITS_GREY        : [u8; 2] = [0, 1];
const BITS_DARK_BLUE   : [u8; 2] = [0, 0];
const BITS_LIGHT_BLUE  : [u8; 2] = [1, 0];

fn list_tiles(input_dir : &Path) -> Result<Paths, String> {
    if !input_dir.exists() {
        return Err(format!("Directory does not exist: {}", input_dir.to_string_lossy()));
    }
    match glob(&input_dir.join("*.png").to_string_lossy()) {
        Ok(glob) => return Ok(glob),
        Err(e) => return Err(format!("Error listing files: {}", e)),
    }
}

fn _reverse_chunk(input : &[u8]) -> Vec<u8> {
    let mut v = vec![];
    v.extend(input);
    v.reverse();

    return v;
}

fn collapse_bits(bytes : &[u8]) -> Result<u8, String> {
    if !bytes.len() == 8 {
        return Err(format!("Input must be 8 bytes long ({} elements provided)", bytes.len()));
    }
    let mut result = 0;
    for (i, byte) in bytes.iter().enumerate() {
        let mask = (1 as u8) << i;

        // Are we setting this bit to 0 or 1?
        match *byte {
            0 => result |= mask,
            1 => result &= !mask,
            _ => {
                return Err(format!("Bits must be either 0 or 1 (value was {})", *byte));
            }
        }
    }
    return Ok(result);
}

fn _rgb_to_2bit(bytes : &[u8]) -> Vec<u8> {
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

fn decode_png(input : &Path) -> Result<Vec<u8>, io::Error> {
    let decoder = png::Decoder::new(File::open(input)?);
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
       .flat_map(_rgb_to_2bit)
       .collect();
    // Take our big Vec of bit values and collapse that down into bytes
    let mut output_bytes : Vec<u8> = bit_values.chunks(8)
       .flat_map(collapse_bits)
       .collect();

    // Reverse the horizontal order of pixels.
    // The order returned by a given PNG is the opposite of what Lunar expects.
    output_bytes = output_bytes.chunks(info.width as usize)
        .flat_map(_reverse_chunk)
        .collect::<Vec<u8>>();

    // 256 pixels per 16x16 glyph, with four pixels per byte, should be 64
    debug_assert!(output_bytes.len() == 64);

    return Ok(output_bytes);
}

fn parse_codepoint_from_filename(filename : &str) -> Result<u8, String> {
    let filename = String::from(filename);
    let re = Regex::new(r"(\d*)\.png$").unwrap();
    if !re.is_match(&filename) {
        return Err(format!("Unable to parse codepoint from filename: {}", filename));
    }

    let captures = re.captures(&filename).unwrap();
    return Ok(captures[1].parse().unwrap());
}

fn main() {
    let matches = App::new("fontbuild")
                          .version("0.1.0")
                          .author("Misty De Meo")
                          .about("Rebuild font files for Lunar: Silver Star Story")
                          .arg(Arg::with_name("input_dir")
                              .help("Path to tiles to insert")
                              .required(true)
                              .index(1))
                          .arg(Arg::with_name("target")
                              .help("Font file to write to")
                              .required(true)
                              .index(2))
                          .arg(Arg::with_name("append")
                              .short("a")
                              .long("append")
                              .help("Append extra data to the end of the file")
                              .required(false)
                              .takes_value(true))
                          .get_matches();
    let input_dir = matches.value_of("input_dir").unwrap().to_string();
    let input_path = Path::new(&input_dir);

    let target = matches.value_of("target").unwrap().to_string();
    let mut target_file;
    match File::create(&target) {
        Ok(f) => target_file = f,
        Err(e) => {
            println!("Unable to open target file {}!\n{}", target, e);
            exit(1);
        }
    }

    let mut append_data : Vec<u8>;
    match matches.value_of("append") {
        Some(append) => {
            let append_file;
            match File::open(append) {
                Ok(f) => append_file = f,
                Err(e) => {
                    println!("Unable to open append file {}!\n{}", append, e);
                    exit(1);
                }
            }

            let mut buf_reader = BufReader::new(append_file);
            append_data = vec![];
            buf_reader.read_to_end(&mut append_data).unwrap();
        },
        None => append_data = vec![],
    }

    let mut codepoints : Vec<u8> = vec![];
    let mut imagedata : Vec<u8> = vec![];

    for file in list_tiles(input_path).unwrap().filter_map(Result::ok) {
        let codepoint;
        match parse_codepoint_from_filename(&file.to_string_lossy()) {
            Ok(val) => codepoint = val,
            Err(e) => {
                println!("{}", e);
                exit(1);
            }
        }

        codepoints.push(codepoint);

        match decode_png(&file.as_path()) {
            Ok(bytes) => imagedata.extend(bytes),
            Err(e) => {
                println!("Unable to parse image data for file {}!\n{}", &file.to_string_lossy(), e);
                exit(1);
            }
        }
    }
    target_file.write_all(&imagedata).unwrap();
    target_file.write_all(&append_data).unwrap();
}
