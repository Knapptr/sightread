use nom::{
    bits,
    bytes::complete::tag,
    number::complete::{be_u16, be_u32},
    sequence::tuple,
    IResult,
};

#[derive(Clone, Debug)]
pub enum Format {
    SingleMultiChannel,
    MultiTrackSimultaneous,
    MultiTrackIndependant,
}

#[derive(Debug)]
pub struct Header {
    format_type: Format,
    tracks: u16,
    division: DivFormat,
}

fn header_chars(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("MThd")(input)
}
#[derive(Debug, Clone)]
pub enum DivFormat {
    PPQ(usize),
    NegSMPTE,
}
fn division(input: &[u8]) -> IResult<&[u8], DivFormat> {
    // let (remainder, bytes) = take(2usize)(input)?;
    bits(div_format)(input)
}

pub fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (remainder, _) = header_chars(input)?;
    let (remainder, (length, format_type, tracks, division)) =
        tuple((length, format_type, ntrks, division))(remainder)?;
    Ok((
        remainder,
        Header {
            format_type,
            tracks,
            division,
        },
    ))
}
fn length(input: &[u8]) -> IResult<&[u8], u32> {
    be_u32(input)
}
fn format_type(input: &[u8]) -> IResult<&[u8], Format> {
    let (remainder, format_number) = be_u16(input)?;
    let format_type = match format_number {
        0 => Format::SingleMultiChannel,
        1 => Format::MultiTrackSimultaneous,
        2 => Format::MultiTrackIndependant,
        _ => {
            return Err(nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Fail,
            )))
        }
    };
    Ok((remainder, format_type))
}
fn ntrks(input: &[u8]) -> IResult<&[u8], u16> {
    be_u16(input)
}
fn div_format(input: (&[u8], usize)) -> IResult<(&[u8], usize), DivFormat> {
    let (remainder, key_bit): ((&[u8], usize), usize) = nom::bits::complete::take(1usize)(input)?;
    let (remainder, first): ((&[u8], usize), usize) =
        nom::bits::complete::take(15usize)(remainder)?;
    match key_bit {
        0 => Ok((remainder, DivFormat::PPQ(first))),
        1 => {
            eprintln!("Cannot parse files with Negative SMPTE division format. Yet.");
            Err(nom::Err::Failure(nom::error::Error::new(
                input,
                nom::error::ErrorKind::Fail,
            )))
        }
        _ => unreachable!(),
    }
}
