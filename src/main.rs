use std::io;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::fs::File;
use std::fs::OpenOptions;
use std::path::PathBuf;
use structopt::StructOpt;
use std::num::ParseIntError;
extern crate humansize;
use humansize::{FileSize, file_size_opts as options};

use indicatif::{ProgressBar, ProgressStyle};

fn parse_hex(s: &str) -> Result<u8, ParseIntError> {
    u8::from_str_radix(s, 16)
}

#[derive(Debug, StructOpt)]
#[structopt(name = "cut-trailing-bytes", about = "A tool for cut trailing bytes, default cut trailing NULL bytes(0x00 in hex)")]
struct Opt {
    /// File to cut
    #[structopt(parse(from_os_str))]
    file: PathBuf,

    /// For example, pass 'ff' if want to cut 0xff
    #[structopt(short = "c", long = "cut-byte", default_value="0", parse(try_from_str = parse_hex))]
    byte_in_hex: u8,

    /// Check the file but don't real cut it
    #[structopt(short, long = "dry-run")]
    dry_run: bool,
}


fn main() -> io::Result<()> {

    let opt = Opt::from_args();
    let filename = &opt.file;
    let mut f = File::open(filename)?;
    let mut buffer = [0; 4096];
    let mut valid_len = f.seek(SeekFrom::End(0)).unwrap();
    let total = valid_len;
    let mut tmp_len = 0;
    let mut n;

    let pb = ProgressBar::new(valid_len);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:.cyan/blue}] {bytes}/{total_bytes} {msg} ({eta})"));

    loop {
        if valid_len >= 4096 {
            valid_len = f.seek(SeekFrom::Current(-4096))?;
        } else {
            valid_len = f.seek(SeekFrom::Start(0))?;
        }
        n = f.read(&mut buffer[..])?;
        f.seek(SeekFrom::Current(0))?;
        if n == 0 { break; }

        for byte in buffer.bytes() {
            match byte.unwrap() {
                byte if byte == opt.byte_in_hex => { tmp_len += 1; }
                _ => {
                    valid_len += tmp_len;
                    tmp_len = 0;
                    valid_len += 1;
                }
            }
            n -= 1;
            if n == 0 { break; }
        }
        pb.inc(4096);
        // exit if 1st buffer passed
        tmp_len = 0;
        if valid_len < 4096 { break; }
        // exit if at least one char doesnt match
        if valid_len > f.seek(SeekFrom::Current(-4096))? { break; }
    }
    pb.finish_at_current_pos();
    // pb.finish_with_message("done");
    if !opt.dry_run {
        let f = OpenOptions::new().write(true).open(filename);
        f.unwrap().set_len(valid_len)?;
    }
    println!("cut {} from {} to {}", filename.display(), total.file_size(options::BINARY).unwrap(), valid_len.file_size(options::BINARY).unwrap());

    Ok(())
}

