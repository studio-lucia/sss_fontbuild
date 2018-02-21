use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

#[macro_use] extern crate quicli;
use quicli::prelude::*;
// Import this here to clobber quicli's custom Result with the default one
use std::result::Result;

extern crate sss_fontbuild;
use sss_fontbuild::utils;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(help = "Path to tiles to insert", parse(from_os_str))]
    input: PathBuf,
    #[structopt(help = "Font file to write to", parse(from_os_str))]
    target: PathBuf,
    #[structopt(short = "a", long = "append",
                help = "Append extra data to the end of the file",
                parse(from_os_str))]
    append: Option<PathBuf>,
    #[structopt(short = "c", long = "compress",
                help = "Compress the generated data using Sega's CMP")]
    compress: bool,
}

main!(|args: Opt| {
    let mut target_file;
    match File::create(&args.target) {
        Ok(f) => target_file = f,
        Err(e) => {
            println!("Unable to open target file {}!\n{}", args.target.to_string_lossy(), e);
            exit(1);
        }
    }

    let mut append_data : Vec<u8>;
    match args.append {
        Some(append) => {
            let append_file;
            match File::open(&append) {
                Ok(f) => append_file = f,
                Err(e) => {
                    println!("Unable to open append file {}!\n{}", append.to_string_lossy(), e);
                    exit(1);
                }
            }

            let mut buf_reader = BufReader::new(append_file);
            append_data = vec![];
            buf_reader.read_to_end(&mut append_data)?;
        },
        None => append_data = vec![],
    }

    let mut codepoints : Vec<u8> = vec![];
    let mut imagedata : Vec<u8> = vec![];

    for file in utils::list_tiles(&args.input)?.filter_map(Result::ok) {
        let codepoint = utils::parse_codepoint_from_filename(&file.to_string_lossy())?;
        codepoints.push(codepoint);

        match utils::decode_png(&file) {
            Ok(bytes) => imagedata.extend(bytes),
            Err(e) => {
                println!("Unable to parse image data for file {}!\n{}", &file.to_string_lossy(), e);
                exit(1);
            }
        }
    }

    if args.compress {
        utils::write_compressed(imagedata, &target_file)?;
    } else {
        utils::write_uncompressed(imagedata, &target_file)?;
    }
    target_file.write_all(&append_data)?;
});
