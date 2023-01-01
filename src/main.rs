mod bytes;
mod file;
mod header;
mod meta;
mod track;

use std::process;

use bytes::variable_length_7;
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
pub struct MidiMessage {
    channel: u8,
    message: VoiceMessage,
}
impl MidiMessage {
    fn create(channel: u8, message: VoiceMessage) -> Self {
        Self { channel, message }
    }
}
#[derive(Debug)]
pub enum VoiceMessage {
    NoteOn(u8, u8),
    NoteOff(u8, u8),
    ControlChange(u8, u8),
    ProgramChange(u8),
    PolyAftertouch(u8, u8),
    MonoAftertouch,
    Other,
    PitchBend,
}
#[derive(Debug)]
pub enum SysexMessage<'a> {
    Placeholder(&'a [u8]),
}
fn get_status_with_tag(
    spec: u8,
) -> impl FnMut((&[u8], usize)) -> IResult<(&[u8], usize), (u8, u8)> {
    move |input| {
        let (remainder, (status_tag, channel)) = tuple((
            nom::bits::complete::tag(spec, 4usize),
            nom::bits::complete::take(4usize),
        ))(input)?;

        Ok((remainder, (status_tag, channel)))
    }
}
fn status_tag(spec: u8) -> impl FnMut(&[u8]) -> IResult<&[u8], (u8, u8)> {
    move |input| {
        let (remainder, (status, channel)) = nom::bits(get_status_with_tag(spec))(input)?;
        println!(
            "Status Tag: Tag: {:x?} - Status - {:x?}- Channel {:x?}",
            spec, status, channel
        );
        Ok((remainder, (status, channel)))
    }
}
fn get_status(input: (&[u8], usize)) -> IResult<(&[u8], usize), (u8, u8)> {
    let (remainder, status_code): (_, u8) = nom::bits::complete::take(4usize)(input)?;
    let (remainder, channel): (_, u8) = nom::bits::complete::take(4usize)(input)?;
    Ok((remainder, (status_code, channel)))
}
fn msg_note_on(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    println!("MSG NOTE ON");
    let (remainder, (note, velocity)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::NoteOn(note, velocity)),
    ))
}
fn msg_note_off(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    println!("Message Note Off");
    let (remainder, (note, velocity)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::NoteOff(note, velocity)),
    ))
}
fn msg_after_poly(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    println!("Message Poly After");
    let (remainder, (note, pressure)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::PolyAftertouch(note, pressure)),
    ))
}
fn msg_cc(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    println!("Message CC");
    let (remainder, (control_number, value)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::ControlChange(control_number, value)),
    ))
}
fn msg_program_change(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    println!("MSG PG");
    let (remainder, program) = be_u8(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::ProgramChange(program)),
    ))
}
pub fn parse_sysex(input: &[u8]) -> IResult<&[u8], MidiEvent> {
    let (remainder, time) = variable_length_7(input)?;
    let (remainder, _) = tag(&[0xF0])(remainder)?;
    println!("Sysex");
    let (remainder, length) = variable_length_7(remainder)?;
    map(map(take(length), SysexMessage::Placeholder), move |msg| {
        MidiEvent::SysEx(time, msg)
    })(remainder)
}
pub fn parse_message_running_status<'a>(
    input: &[u8],
    last_event: Option<MidiMessage>,
) -> IResult<&[u8], MidiEvent> {
    let (remainder, time) = variable_length_7(input)?;
    if remainder[0..1] < [0x80][0..1] {
        match last_event{
            Some(event) => {
                match event.message{
                    VoiceMessage::NoteOn(_,_) => msg_note_on()
                }
            },
            None() Err(nom::Err::Failure(nom::error::Error::new(input, nom::error::ErrorKind::Fail)))
        }
    } else {
        parse_message_event(input, time)
    }
}
pub fn parse_message_event<'a>(input: &'a [u8], time: u32) -> IResult<&[u8], MidiEvent> {
    let (remainder, (status, channel)) = nom::bits(get_status)(input)?;
    let (remainder, event) = match status {
        0x8 =>  msg_note_off(input,channel)
        0x9 =>  msg_note_on(input,channel)
        0xA =>  msg_after_poly(input,channel)// Poly After
        0xB =>  msg_cc(input,channel)// CC
        0xC =>  msg_program_change(input,channel)// PG
        _ => unreachable!(),
    }?;
    Ok((remainder, MidiEvent::Message(time, event)))
}
fn take_with_tag_len<'a>(spec: &'a [u8]) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], &'a [u8]> {
    move |input| {
        let (remainder, _) = tag(spec.clone())(input)?;
        let (remainder, track) = length_data(be_u8)(remainder)?;

        Ok((remainder, track))
    }
}

#[cfg(test)]
#[test]
fn parse_noteon() {
    let notemsg = &[0x90, 0x40, 0x40];
    assert!(matches!(
        msg_note_on(notemsg).unwrap().1,
        VoiceMessage::NoteOn((0, 64, 64))
    ))
}
#[test]
fn parse_noteoff() {
    let notemsg = &[0x80, 0x40, 0x40];
    assert!(matches!(
        msg_note_off(notemsg).unwrap().1,
        VoiceMessage::NoteOff((0, 64, 64))
    ))
}
#[test]
fn parse_polyafter() {
    let notemsg = &[0xA1, 0x40, 0x40];
    assert!(matches!(
        msg_after_poly(notemsg).unwrap().1,
        VoiceMessage::PolyAftertouch((1, 64, 64))
    ))
}
#[test]
fn parse_cc() {
    let control_msg = &[0xB1, 0x04, 0x80];
    assert!(matches!(
        msg_cc(control_msg).unwrap().1,
        VoiceMessage::ControlChange((1, 4, 128))
    ))
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
    while !remainder.is_empty() {
        match track::parse_track(remainder) {
            Ok((new_remainder, track)) => {
                println!("Track Parsed.");
                remainder = new_remainder
            }
            Err(err) => {
                println!("ERROR: {:x?}", err);
                process::exit(1);
            }
        }
        // println!("Track: {:x?}", track)
    }
}
