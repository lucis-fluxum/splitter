extern crate ffmpeg;
extern crate hex_slice;

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};

use ffmpeg::format::context::Input;
use hex_slice::AsHex;

// Beginning of AAX file checksum
const FILE_CHECKSUM_START: u64 = 653;
const CHECKSUM_BUFFER_SIZE: usize = 20;

// TODO: Use clap and add extra configuration like output file pattern, bitrate, etc.
fn main() {
    ffmpeg::init().unwrap();
    let path = env::args().nth(1).expect("missing input file name");
    let input = ffmpeg::format::input(&path).expect("unable to create input context");
    let mut file = File::open(path).unwrap();
    print_metadata(&input);
    print_chapters(&input);
    println!("checksum: {}", extract_checksum(&mut file));
}

fn print_metadata(input: &Input) {
    for meta in input.metadata().iter() {
        println!("{}: {}", meta.0, meta.1);
    }
}

fn print_chapters(input: &Input) {
    for chapter in input.chapters() {
        let denominator = chapter.time_base().denominator() as f64;
        let start = (chapter.start() as f64) / denominator;
        let end = (chapter.end() as f64) / denominator;
        println!(
            "{}: {} - {}",
            chapter
                .metadata()
                .get("title")
                .unwrap_or(&chapter.id().to_string()),
            start,
            end
        );
    }
}

fn extract_checksum(file: &mut File) -> String {
    let mut buffer = [0; CHECKSUM_BUFFER_SIZE];
    file.seek(SeekFrom::Start(FILE_CHECKSUM_START)).unwrap();
    file.read_exact(&mut buffer).unwrap();
    format!("{:x}", buffer.plain_hex(false))
}
