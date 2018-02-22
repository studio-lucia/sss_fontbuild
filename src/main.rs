use std::fs::File;
use std::io::BufReader;
use std::io::prelude::*;
use std::path::PathBuf;
use std::process::exit;

#[macro_use] extern crate quicli;
use quicli::prelude::*;
// Import this here to clobber quicli's custom Result with the default one
#[allow(unused_imports)]
use std::result::Result;

extern crate sss_fontbuild;
use sss_fontbuild::consts::*;
use sss_fontbuild::utils;

#[derive(StructOpt, Debug)]
struct Opt {
    #[structopt(help = "Path to tiles to insert", parse(from_os_str))]
    input: PathBuf,
    #[structopt(help = "Font file to write to", parse(from_os_str))]
    target: PathBuf,
    #[structopt(short = "i", long = "insert",
                help = "Insert font into the game's SYSTEM.DAT")]
    insert: bool,
    #[structopt(short = "a", long = "append",
                help = "Append extra data to the end of the file",
                parse(from_os_str))]
    append: Option<PathBuf>,
    #[structopt(short = "c", long = "compress",
                help = "Compress the generated data using Sega's CMP")]
    compress: bool,
}

fn main_create(args: Opt) -> quicli::prelude::Result<()> {
    let mut target_file = File::create(&args.target)?;

    let mut append_data : Vec<u8>;
    match args.append {
        Some(append) => {
            let append_file = File::open(&append)?;
            let mut buf_reader = BufReader::new(append_file);
            append_data = vec![];
            buf_reader.read_to_end(&mut append_data)?;
        },
        None => append_data = vec![],
    }

    let imagedata = utils::create_font_data(&args.input)?;

    if args.compress {
        utils::write_compressed(imagedata, &target_file)?;
    } else {
        utils::write_uncompressed(imagedata, &target_file)?;
    }
    target_file.write_all(&append_data)?;

    return Ok(());
}

fn main_insert(args: Opt) -> quicli::prelude::Result<()> {
    let game;
    let target_size = args.target.metadata()?.len();
    match target_size {
        SSS_SYSTEM_DAT_SIZE  => game = Game::SSS,
        SSSC_SYSTEM_DAT_SIZE => game = Game::SSSC,
        _ => {
            println!("Couldn't recognize provided SYSTEM.DAT file!");
            println!("Provided size, {}, doesn't match known files.", target_size);
            exit(1);
        }
    }

    let imagedata = utils::create_font_data(&args.input)?;

    let mut system_dat_data = vec![];
    File::open(&args.target)?.read_to_end(&mut system_dat_data)?;

    let altered_data = utils::insert_data_into_file(imagedata, system_dat_data, game)?;

    let mut target_file = File::create(&args.target)?;
    target_file.write_all(&altered_data)?;

    return Ok(());
}

main!(|args: Opt| {
    if args.insert {
        main_insert(args)?;
    } else {
        main_create(args)?;
    }
});
