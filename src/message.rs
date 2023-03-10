use nom::{
    number::complete::be_u8,
    sequence::{pair, tuple},
    IResult,
};

use crate::MidiEvent;

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
fn get_status(input: (&[u8], usize)) -> IResult<(&[u8], usize), (u8, u8)> {
    let (remainder, status_code): (_, u8) = nom::bits::complete::take(4usize)(input)?;
    let (remainder, channel): (_, u8) = nom::bits::complete::take(4usize)(remainder)?;
    Ok((remainder, (status_code, channel)))
}
fn msg_note_on(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    let (remainder, (note, velocity)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::NoteOn(note, velocity)),
    ))
}
fn msg_note_off(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    let (remainder, (note, velocity)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::NoteOff(note, velocity)),
    ))
}
fn msg_after_poly(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    let (remainder, (note, pressure)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::PolyAftertouch(note, pressure)),
    ))
}
fn msg_cc(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    let (remainder, (control_number, value)) = pair(be_u8, be_u8)(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::ControlChange(control_number, value)),
    ))
}
fn msg_program_change(input: &[u8], channel: u8) -> IResult<&[u8], MidiMessage> {
    let (remainder, program) = be_u8(input)?;
    Ok((
        remainder,
        MidiMessage::create(channel, VoiceMessage::ProgramChange(program)),
    ))
}
pub fn parse_message<'a, 'b>(
    time: u32,
    last_event: Option<&'b MidiEvent>,
) -> impl FnMut(&'a [u8]) -> IResult<&'a [u8], MidiEvent<'a>> + 'b {
    move |input| {
        if input[0..1] < [0x80][0..1] {
            match last_event {
                Some(midi_event) => match midi_event {
                    MidiEvent::Message(_, event) => {
                        let (remainder, new_event) = match event.message {
                            VoiceMessage::NoteOn(_, _) => msg_note_on(input, event.channel),
                            VoiceMessage::NoteOff(_, _) => msg_note_off(input, event.channel),
                            VoiceMessage::ControlChange(_, _) => msg_cc(input, event.channel),
                            VoiceMessage::ProgramChange(_) => {
                                msg_program_change(input, event.channel)
                            }
                            _ => todo!(),
                        }?;
                        Ok((remainder, MidiEvent::Message(time, new_event)))
                    }
                    _ => parse_message_event(input, time),
                },
                None => Err(nom::Err::Failure(nom::error::Error::new(
                    input,
                    nom::error::ErrorKind::Fail,
                ))),
            }
        } else {
            parse_message_event(input, time)
        }
    }
}
pub fn parse_message_event<'a>(input: &'a [u8], time: u32) -> IResult<&[u8], MidiEvent> {
    let (remainder, (status, channel)) = nom::bits(get_status)(input)?;
    let (remainder, event) = match status {
        0x8 => msg_note_off(remainder, channel),
        0x9 => msg_note_on(remainder, channel),
        0xA => msg_after_poly(remainder, channel), // Poly After
        0xB => msg_cc(remainder, channel),         // CC
        0xC => msg_program_change(remainder, channel), // PG
        _ => unreachable!(),
    }?;
    Ok((remainder, MidiEvent::Message(time, event)))
}
#[cfg(test)]
#[test]
fn parse_noteon() {
    let notemsg = &[0x40, 0x40];
    assert!(matches!(
        msg_note_on(notemsg, 1).unwrap().1,
        MidiMessage {
            channel: 1,
            message: VoiceMessage::NoteOn(64, 64)
        }
    ))
}
#[test]
fn parse_noteoff() {
    let notemsg = &[0x80, 0x40, 0x40];
    assert!(matches!(
        msg_note_off(notemsg, 1).unwrap().1,
        MidiMessage {
            channel: 1,
            message: VoiceMessage::NoteOff(64, 64)
        }
    ))
}
