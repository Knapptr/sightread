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
    multi::length_data,
    number::complete::be_u8,
    sequence::{pair, tuple, Tuple},
    IResult,
};

use crate::meta::MetaEvent;

#[derive(Debug)]
pub enum MidiEvent<'a> {
    Meta((usize, meta::MetaEvent<'a>)),
    Message((usize, VoiceMessage)),
    SysEx,
}
#[derive(Debug)]
pub enum VoiceMessage {
    NoteOn((u8, u8, u8)),
    NoteOff((u8, u8, u8)),
    ControlChange((u8, u8, u8)),
    ProgramChange,
    PolyAftertouch((u8, u8, u8)),
    MonoAftertouch,
    Other,
    PitchBend,
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
        Ok((remainder, (status, channel)))
    }
}
fn get_status(input: (&[u8], usize)) -> IResult<(&[u8], usize), (u8, u8)> {
    let (remainder, status_code): (_, u8) = nom::bits::complete::take(4usize)(input)?;
    let (remainder, channel): (_, u8) = nom::bits::complete::take(4usize)(input)?;
    Ok((remainder, (status_code, channel)))
}
fn msg_note_on(input: &[u8]) -> IResult<&[u8], VoiceMessage> {
    let (remainder, (status, channel)) = status_tag(0x9)(input)?;
    let (remainder, (note, velocity)) = pair(be_u8, be_u8)(remainder)?;
    Ok((remainder, VoiceMessage::NoteOn((channel, note, velocity))))
}
fn msg_note_off(input: &[u8]) -> IResult<&[u8], VoiceMessage> {
    let (remainder, (status, channel)) = status_tag(0x8)(input)?;
    let (remainder, (note, velocity)) = pair(be_u8, be_u8)(remainder)?;
    Ok((remainder, VoiceMessage::NoteOff((channel, note, velocity))))
}
fn msg_after_poly(input: &[u8]) -> IResult<&[u8], VoiceMessage> {
    let (remainder, (status, channel)) = status_tag(0xA)(input)?;
    let (remainder, (note, pressure)) = pair(be_u8, be_u8)(remainder)?;
    Ok((
        remainder,
        VoiceMessage::PolyAftertouch((channel, note, pressure)),
    ))
}
fn msg_cc(input: &[u8]) -> IResult<&[u8], VoiceMessage> {
    let (remainder, (status, channel)) = status_tag(0xB)(input)?;
    let (remainder, (control_number, value)) = pair(be_u8, be_u8)(remainder)?;
    Ok((
        remainder,
        VoiceMessage::ControlChange((channel, control_number, value)),
    ))
}
fn msg_sysex(input: &[u8]) -> IResult<&[u8], VoiceMessage> {
    let (remainder, _) = tag(&[0xF0])(input)?;
    let (remainder, length) = variable_length_7(remainder)?;
    let (remainder, _sysex) = take(length)(remainder)?;
    Ok((remainder, VoiceMessage::Other))
}
pub fn parse_message_event<'a>(input: &'a [u8]) -> IResult<&[u8], MidiEvent> {
    let (remainder, time) = variable_length_7(input)?;
    let (remainder, event) =
        alt((msg_note_on, msg_note_off, msg_after_poly, msg_cc, msg_sysex))(remainder)?;
    Ok((remainder, MidiEvent::Message((time, event))))
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
    let file_path = "test_files/lechuck.mid";
    let buffer = file::read_file(file_path).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    let (remainder, _header) = header::parse_header(&buffer).unwrap();
    let (remainder, track) = track::parse_track(remainder).unwrap();
    let (remainder, track) = track::parse_track(remainder).unwrap();
    println!("Remainder: {:x?}", remainder);
    let track = track::parse_track(remainder);
    println!("{:x?}", track);
}
