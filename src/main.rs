#[macro_use]
extern crate clap;
extern crate ffmpeg;
extern crate hex_slice;

mod chapter_info;

use std::fs::File;
use std::io::{self, Read, Seek, SeekFrom, Write};
use std::path::PathBuf;
use std::process::Command;

use clap::{App, Arg};
use ffmpeg::format::context::Input;
use hex_slice::AsHex;

use chapter_info::ChapterInfo;

// Beginning of AAX file checksum
const FILE_CHECKSUM_START: u64 = 653;
const CHECKSUM_BUFFER_SIZE: usize = 20;

// TODO: Add extra configuration like output file pattern
// TODO: Fill those ID3 headers with metadata
// TODO: Save activation_bytes for use in subsequent runs
fn main() {
    let args = App::new(crate_name!())
        .version(crate_version!())
        .author(crate_authors!())
        .about("Split AAX files from Audible into multiple MP3 files by chapter using ffmpeg.")
        .arg(
            Arg::with_name("FILE")
                .help("AAX file to split")
                .required(true)
                .empty_values(false),
        )
        .arg(
            Arg::with_name("bitrate")
                .short("b")
                .long("bitrate")
                .help("Set the desired bitrate of outputted MP3 files")
                .default_value("256k"),
        )
        .get_matches();

    let path = PathBuf::from(args.value_of("FILE").unwrap());
    let desired_bitrate = args.value_of("bitrate").unwrap();

    ffmpeg::init().unwrap();
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
        &desired_bitrate,
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

fn write_chapters(
    input: &Input,
    filename: &str,
    activation_bytes: &str,
    book_title: &str,
    desired_bitrate: &str,
) {
    // TODO: Parallellize
    for ch_info in get_chapter_infos(&input) {
        let output_name = format!("{} - {}.mp3", book_title, ch_info.title);

        print!("Writing {}... ", output_name);
        io::stdout().flush().unwrap();

        let output = Command::new("ffmpeg")
            .args(&["-activation_bytes", &activation_bytes])
            .args(&["-i", filename])
            .args(&["-vn", "-b:a", desired_bitrate])
            .args(&[
                "-ss",
                &ch_info.start.to_string(),
                "-to",
                &ch_info.end.to_string(),
            ])
            .arg(&output_name)
            .output()
            .expect(&format!("couldn't create {}", output_name));
        if output.status.success() {
            println!("Done.");
        }
    }
}
