use nom::{
    branch::alt, bytes::complete::tag, multi::length_data, number::complete::be_u32, IResult,
};

use crate::{meta, MidiEvent};

fn take_track(input: &[u8]) -> IResult<&[u8], &[u8]> {
    let (remainder, _) = tag("MTrk")(input)?;
    let (remainder, track) = length_data(be_u32)(remainder)?;
    Ok((remainder, track))
}

pub fn parse_track(input: &[u8]) -> IResult<&[u8], Vec<MidiEvent>> {
    let (remainder, mut track) = take_track(input)?;
    let mut track_events = Vec::new();
    while !track.is_empty() {
        let (new_track, event) = alt((meta::parse_meta_event,))(track)?;
        track_events.push(event);
        track = new_track;
    }
    Ok((remainder, track_events))
}
