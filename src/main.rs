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

extern crate byteorder;
use byteorder::{BigEndian, WriteBytesExt};

extern crate png;

extern crate regex;
use regex::Regex;

fn list_tiles(input_dir : &Path) -> Result<Paths, String> {
    if !input_dir.exists() {
        return Err(format!("Directory does not exist: {}", input_dir.to_string_lossy()));
    }
    match glob(&input_dir.join("*.png").to_string_lossy()) {
        Ok(glob) => return Ok(glob),
        Err(e) => return Err(format!("Error listing files: {}", e)),
    }
}

fn _expand_chunk(input : &[u8]) -> Vec<u8> {
    let mut v = vec![];
    v.extend(input);
    v.resize(16, 255);

    return v
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
        // We're assuming greyscale input with only two colours,
        // so values are expected to be 0 or 255.
        if *byte == 0 {
            result |= mask;
        } else {
            result &= !mask;
        }
    }
    return Ok(result);
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

    if info.color_type != png::ColorType::Indexed
        && info.color_type != png::ColorType::GrayscaleAlpha
        && info.color_type != png::ColorType::Grayscale {
        return Err(io::Error::new(io::ErrorKind::InvalidData,
            format!("Invalid colour format - only greyscale or indexed are supported")));
    }

    // Drop the alpha channel
    if info.color_type == png::ColorType::GrayscaleAlpha {
        buf = buf
            // In every pair of bytes, 0 is alpha, 1 is the pixel.
            .chunks(2).map(|a| a[1])
            // In a transparent image, the colour of the font/background
            // is reverse from what we want. Need to invert it.
            .map(|c| !c)
            .collect::<Vec<u8>>();
    }

    // Reverse the horizontal order of pixels.
    // The order returned by a given PNG is the opposite of what Lunar expects.
    buf = buf.chunks(info.width as usize)
        .flat_map(_reverse_chunk)
        .collect::<Vec<u8>>();

    // Expand from 8x16 to 16x16
    if info.width == 8 {
        buf = buf.chunks(8).flat_map(_expand_chunk).collect::<Vec<u8>>();
    }

    // Convert greyscale to 1bpp
    if info.color_type == png::ColorType::GrayscaleAlpha || info.color_type == png::ColorType::Grayscale {
        buf = buf.chunks(8).flat_map(collapse_bits).collect::<Vec<u8>>();
    }

    return Ok(buf);
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

fn calculate_header_length(number_of_codepoints : usize) -> u16 {
    // Size of the header length portion of the header plus
    // size of the codepoint segment (fixed-width 2-byte pairs)
    return 2 + (2 * number_of_codepoints as u16);
}

// Codepoints are assumed to be two-byte sequences; to support
// single-byte codepoints, we prepend 00 to each of them.
fn generate_codepoint_table(codepoints : Vec<u8>) -> Vec<u8> {
    return codepoints.iter().flat_map(|c| vec![0, *c]).collect::<Vec<u8>>();
}

fn main() {
    let matches = App::new("fontbuild")
                          .version("0.1.0")
                          .author("Misty De Meo")
                          .about("Rebuild font files for Magical School Lunar!")
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
    let header_length = calculate_header_length(codepoints.len());
    let mut header_length_bin : Vec<u8> = vec![];
    header_length_bin.write_u16::<BigEndian>(header_length).unwrap();
    target_file.write_all(&header_length_bin).unwrap();
    target_file.write_all(&generate_codepoint_table(codepoints)).unwrap();
    target_file.write_all(&imagedata).unwrap();
    target_file.write_all(&append_data).unwrap();
}
