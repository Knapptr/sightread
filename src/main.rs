mod bytes;
mod file;
mod header;
mod message;
mod meta;
mod track;

use std::process;

use bytes::variable_length_7;
use message::MidiMessage;
use nom::{
    bits,
    branch::alt,
    bytes::complete::{tag, take},
    combinator::map,
    multi::length_data,
    number::complete::be_u8,
    sequence::{pair, tuple, Tuple},
    IResult,
};

use crate::meta::MetaEvent;

#[derive(Debug)]
pub enum MidiEvent<'a> {
    Meta((u32, meta::MetaEvent<'a>)),
    Message(u32, MidiMessage),
    SysEx(u32, SysexMessage<'a>),
}
#[derive(Debug)]
pub enum SysexMessage<'a> {
    Placeholder(&'a [u8]),
}
pub fn parse_sysex(time: u32) -> impl FnMut(&[u8]) -> IResult<&[u8], MidiEvent> {
    move |input| {
        let (remainder, _tag) = tag(&[0xF0])(input)?;
        let (remainder, length) = variable_length_7(remainder)?;
        map(map(take(length), SysexMessage::Placeholder), move |msg| {
            MidiEvent::SysEx(time, msg)
        })(remainder)
    }
}
fn take_with_tag_len<'a>(spec: &'a [u8]) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    move |input| {
        let (remainder, _) = tag(spec.clone())(input)?;
        let (remainder, track) = length_data(be_u8)(remainder)?;

        Ok((remainder, track))
    }
}

fn main() {
    let file_path = "test_files/title.mid";
    let buffer = file::read_file(file_path).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    let (remainder, header) = header::parse_header(&buffer).unwrap();
    println!("Header: {:x?}", header);
    let (mut remainder, track) = track::parse_track(remainder).unwrap();
    let mut tracks = Vec::new();
    while !remainder.is_empty() {
        match track::parse_track(remainder) {
            Ok((new_remainder, track)) => {
                remainder = new_remainder;
                tracks.push(track)
            }
            Err(err) => {
                println!("ERROR: {:x?}", err);
                process::exit(1);
            }
        }
    }
    println!("Tracks: {:x?}", tracks.len());
}
