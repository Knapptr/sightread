use nom::{
    branch::alt,
    bytes::complete::{tag, take},
    combinator::{map, value},
    number::complete::{be_u24, be_u8},
    sequence::tuple,
    IResult,
};

use crate::{bytes::variable_length_7, take_with_tag_len, MidiEvent};

#[derive(Clone, Debug)]
pub enum MetaEvent<'a> {
    SequenceNumber,
    Text(&'a [u8]),
    Copyright(&'a [u8]),
    Name(&'a [u8]),
    InstrumentName(&'a [u8]),
    Lyric(&'a [u8]),
    Marker(&'a [u8]),
    CuePoint(&'a [u8]),
    ChannelPrefix(&'a [u8]),
    EndOfTrack,
    SetTempo(u32),
    SMPTEOffset,
    TimeSignature((u8, u8, u8, u8)),
    KeySignature((u8, u8)),
    SequencerSpecific(&'a [u8]),
    Port(&'a [u8]),
}
pub type MetaResult<'a> = IResult<&'a [u8], MetaEvent<'a>>;

fn meta_seq_number<'a>(input: &'a [u8]) -> MetaResult<'a> {
    let (remainder, _) = tag(&[0x00])(input)?;
    let (remainder, num) = take(1usize)(remainder)?;
    Ok((remainder, MetaEvent::SequenceNumber))
}
fn meta_port(input: &[u8]) -> MetaResult {
    let (remainder, _) = tag([0x21, 0x01])(input)?;
    map(take(1usize), MetaEvent::Port)(remainder)
}
fn meta_text(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x01]), MetaEvent::Text)(input)
}
fn meta_copyright(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x02]), MetaEvent::Copyright)(input)
}
fn meta_name(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x03]), MetaEvent::Name)(input)
}
fn meta_instrument(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x04]), MetaEvent::InstrumentName)(input)
}
fn meta_lyric(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x05]), MetaEvent::Lyric)(input)
}
fn meta_marker(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x06]), MetaEvent::Marker)(input)
}
fn meta_cue_point(input: &[u8]) -> MetaResult {
    map(take_with_tag_len(&[0x07]), MetaEvent::CuePoint)(input)
}
fn meta_tempo(input: &[u8]) -> MetaResult {
    let (remainder, _) = tag(&[0x51, 0x03])(input)?;
    map(be_u24, MetaEvent::SetTempo)(remainder)
}
fn meta_timesig(input: &[u8]) -> MetaResult {
    let (remainder, _) = tag(&[0x58, 0x04])(input)?;
    map(
        tuple((be_u8, be_u8, be_u8, be_u8)),
        MetaEvent::TimeSignature,
    )(remainder)
}
fn meta_keysig(input: &[u8]) -> MetaResult {
    let (remainder, _) = tag(&[0x59, 0x02])(input)?;
    map(tuple((be_u8, be_u8)), MetaEvent::KeySignature)(remainder)
}
fn meta_end_of_track(input: &[u8]) -> MetaResult {
    value(MetaEvent::EndOfTrack, tag(&[0x2f, 0x00]))(input)
}

pub fn parse_meta_event<'a>(time: u32) -> impl FnMut(&'a [u8]) -> IResult<&[u8], MidiEvent> {
    move |input| {
        let (remainder, tag) = tag([0xff])(input)?;
        let (remainder, event) = alt((
            meta_seq_number,
            meta_text,
            meta_copyright,
            meta_name,
            meta_seq_number,
            meta_cue_point,
            meta_lyric,
            meta_marker,
            meta_instrument,
            meta_tempo,
            meta_timesig,
            meta_keysig,
            meta_end_of_track,
            meta_port,
        ))(remainder)?;
        Ok((remainder, MidiEvent::Meta((time, event))))
    }
}
