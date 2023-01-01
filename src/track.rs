use nom::{
    branch::alt, bytes::complete::tag, multi::length_data, number::complete::be_u32, IResult,
};

use crate::{bytes::variable_length_7, message::parse_message, meta, parse_sysex, MidiEvent};

fn take_track(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (remainder, _) = tag("MTrk")(input)?;
    let (remainder, track) = length_data(be_u32)(remainder)?;
    Ok((remainder, track))
}

pub fn parse_track<'a>(input: &'a [u8]) -> IResult<&'a [u8], Vec<MidiEvent>> {
    let (remainder, mut track) = take_track(input)?;
    let mut track_events = Vec::new();
    while !track.is_empty() {
        let (remainder, time) = variable_length_7(track)?;
        let last_event = track_events.last();
        let (new_track, event) = alt((
            meta::parse_meta_event(time),
            parse_sysex(time),
            parse_message(time, last_event),
        ))(remainder)?;
        track_events.push(event);
        track = new_track;
    }
    Ok((remainder, track_events))
}
