mod bytes;
mod file;
mod header;
mod meta;
mod track;

use std::process;

use nom::{bytes::complete::tag, multi::length_data, number::complete::be_u8, IResult};

use crate::meta::MetaEvent;

#[derive(Debug)]
pub enum MidiEvent<'a> {
    Meta((usize, meta::MetaEvent<'a>)),
    Message,
    SysEx,
}
fn take_with_tag_len<'a>(spec: &'a [u8]) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    move |input| {
        let (remainder, _) = tag(spec.clone())(input)?;
        let (remainder, track) = length_data(be_u8)(remainder)?;

        Ok((remainder, track))
    }
}

fn main() {
    let file_path = "test_files/lechuck.mid";
    let buffer = file::read_file(file_path).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    let (remainder, _header) = header::parse_header(&buffer).unwrap();
    let (remainder, track) = track::parse_track(remainder).unwrap();
    println!("{:?}", track);
    for event in track {
        match event {
            MidiEvent::Meta((_time, MetaEvent::Text(txt))) => {
                println!("{:?}", std::str::from_utf8(txt))
            }
            _ => {}
        }
    }
}
