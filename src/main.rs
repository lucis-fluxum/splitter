extern crate ffmpeg;
extern crate hex_slice;

mod chapter_info;

use std::env;
use std::fs::File;
use std::io::{Read, Seek, SeekFrom};
use std::path::PathBuf;
use std::process::Command;

use ffmpeg::format::context::Input;
use hex_slice::AsHex;

use chapter_info::ChapterInfo;

// Beginning of AAX file checksum
const FILE_CHECKSUM_START: u64 = 653;
const CHECKSUM_BUFFER_SIZE: usize = 20;

// TODO: Use clap and add extra configuration like output file pattern, bitrate, etc.
// TODO: Fill those ID3 headers with metadata
// TODO: Save activation_bytes for use in subsequent runs
fn main() {
    ffmpeg::init().unwrap();

    let path = PathBuf::from(env::args().nth(1).expect("missing input file name"));
    let input = ffmpeg::format::input(&path).expect("unable to create input context");
    // TODO: Use input metadata to grab title
    let book_title = path.file_stem().unwrap().to_str().unwrap();

    let mut file = File::open(&path).unwrap();
    let checksum = extract_checksum(&mut file);

    print_metadata(&input);

    println!("\nRunning rcrack for {}...", checksum);
    let activation_bytes = extract_activation_bytes(&checksum);
    println!("activation_bytes: {}", activation_bytes);

    write_chapters(
        &input,
        path.to_str().unwrap(),
        &activation_bytes,
        &book_title,
    );
}

fn print_metadata(input: &Input) {
    for meta in input.metadata().iter() {
        println!("{}: {}", meta.0, meta.1);
    }
}

fn get_chapter_infos(input: &Input) -> Vec<ChapterInfo> {
    input
        .chapters()
        .map(|chapter| ChapterInfo::new(&chapter))
        .collect()
}

fn extract_checksum(file: &mut File) -> String {
    let mut buffer = [0; CHECKSUM_BUFFER_SIZE];
    file.seek(SeekFrom::Start(FILE_CHECKSUM_START)).unwrap();
    file.read_exact(&mut buffer).unwrap();
    format!("{:x}", buffer.plain_hex(false))
}

fn extract_activation_bytes(hash: &str) -> String {
    let rcrack_output = Command::new("sh")
        .args(&["-c", &format!("cd ./rcrack && ./rcrack . -h {}", hash)])
        .output()
        .expect("failed to start rcrack");
    String::from_utf8_lossy(&rcrack_output.stdout)
        .split(':')
        .next_back()
        .expect("couldn't extract activation bytes from hash")
        .trim_right()
        .to_string()
}

fn write_chapters(input: &Input, filename: &str, activation_bytes: &str, book_title: &str) {
    // TODO: Parallellize
    for ch_info in get_chapter_infos(&input) {
        let ffmpeg_output = Command::new("ffmpeg")
            .args(&["-activation_bytes", &activation_bytes])
            .args(&["-i", filename])
            .args(&["-vn", "-b:a", "320k"])
            .args(&[
                "-ss",
                &ch_info.start.to_string(),
                "-to",
                &ch_info.end.to_string(),
            ])
            .arg(&format!("{} - {}.mp3", book_title, ch_info.title))
            .output()
            .expect(&format!("couldn't create {}", ch_info.title));
        println!("{:?}", ffmpeg_output);
    }
}
