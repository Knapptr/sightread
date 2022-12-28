use std::{
    fs::{read_to_string, File},
    io::{self, Read},
    process,
};

use nom::{
    bits,
    bytes::complete::{tag, take, take_until},
    combinator::{map, value},
    error,
    error::ErrorKind,
    error_position,
    number::complete::{be_u16, be_u32},
    sequence::tuple,
    IResult, Parser,
};

#[derive(Clone, Debug)]
enum Format {
    SingleMultiChannel,
    MultiTrackSimultaneous,
    MultiTrackIndependant,
}

#[derive(Debug)]
struct Header {
    format_type: Format,
    tracks: u16,
    division: DivFormat,
}

fn track_chars(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("MTrk")(input)
}
fn header_chars(input: &[u8]) -> IResult<&[u8], &[u8]> {
    tag("MThd")(input)
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
        _ => return Err(nom::Err::Failure(error::Error::new(input, ErrorKind::Fail))),
    };
    Ok((remainder, format_type))
}
fn ntrks(input: &[u8]) -> IResult<&[u8], u16> {
    be_u16(input)
}
#[derive(Debug, Clone)]
enum DivFormat {
    PPQ(usize),
    NegSMPTE,
}
fn division(input: &[u8]) -> IResult<&[u8], DivFormat> {
    // let (remainder, bytes) = take(2usize)(input)?;
    bits(div_format)(input)
}

fn parse_header(input: &[u8]) -> IResult<&[u8], Header> {
    let (remainder, _) = header_chars(input)?;
    let (remainder, (length, format_type, tracks, division)) =
        tuple((length, format_type, ntrks, division))(remainder)?;
    println!("division: {:?}", division);
    Ok((
        remainder,
        Header {
            format_type,
            tracks,
            division,
        },
    ))
}

fn take_track(input: &[u8], length: usize) -> IResult<&[u8], &[u8]> {
    take(length)(input)
}
fn div_format(input: (&[u8], usize)) -> IResult<(&[u8], usize), DivFormat> {
    let (remainder, first): ((&[u8], usize), usize) = nom::bits::complete::take(15usize)(input)?;
    let (remainder, key_bit): ((&[u8], usize), usize) =
        nom::bits::complete::take(1usize)(remainder)?;
    match key_bit {
        0 => Ok((remainder, DivFormat::PPQ(first))),
        1 => {
            eprintln!("Cannot parse files with Negative SMPTE division format. Yet.");
            Err(nom::Err::Failure(error::Error::new(
                input,
                error::ErrorKind::Fail,
            )))
        }
        _ => unreachable!(),
    }
}
fn track_delta_time(input: &[u8]) -> IResult<&[u8], usize> {
    // get the first byte value, and if it is the last byte in the event time
    let (mut remainder, (mut delta_time, mut is_finished)) = bits(track_variable_length)(input)?;
    while !is_finished {
        let (new_remainder, (add_delta_time, new_is_finished)) =
            bits(track_variable_length)(remainder)?;
        remainder = new_remainder;
        is_finished = new_is_finished;
        println!("is finished: {:?}", is_finished);
        delta_time += add_delta_time
    }
    Ok((remainder, delta_time))
}
fn track_variable_length(input: (&[u8], usize)) -> IResult<(&[u8], usize), (usize, bool)> {
    let (remainder, key_bit): ((&[u8], usize), usize) = nom::bits::complete::take(1usize)(input)?;
    println!("Key Bit: {:?}\nRemainder: {:?}", key_bit, remainder);
    let (remainder, value_bits): ((&[u8], usize), usize) =
        nom::bits::complete::take(7usize)(remainder)?;
    println!("value_bits {:?}\nRemainder: {:?}", value_bits, remainder);
    match key_bit {
        0 => Ok((remainder, (value_bits, true))),
        1 => Ok((remainder, (value_bits, false))),
        _ => unreachable!(),
    }
}

fn main() {
    let file_path = "test_files/lechuck.mid";
    let buffer = read_file(file_path).unwrap_or_else(|err| {
        eprintln!("{}", err);
        process::exit(1);
    });
    let (remainder, header) = parse_header(&buffer).unwrap();
    let (remainder, tg) = track_chars(remainder).unwrap();
    let (remainder, track_len) = length(remainder).unwrap();
    println!("{:?}", track_len);
    let (remainder, track_events) = take_track(remainder, track_len as usize).unwrap();
    println!("{:?}", track_delta_time(track_events));
    println!("{:x?}", track_events);
    // for ch in track_chars(remainder).unwrap().1 {
    //     println!("{}", *ch as char);
    // }

    // println!("{:?}", &buffer[..4]);
    // println!("Header chars");
    // let (remainder, header_chars) = header_chars(&buffer).unwrap();
    // println!("{:?}", header_chars);
    // println!("Header Length:");
    // let (remainder, length) = length(remainder).unwrap();
    // println!("{:?}", length);
    // println!("Type:");
    // let (remainder, track_type) = format_type(remainder).unwrap();
    // println!("{:?}", track_type);

    // for ch in &buffer[4..8] {
    //     println!("{}", ch)
    // }
    // println!("Header Chunk:");
    // for ch in &buffer[8..14] {
    //     println!("{}", ch)
    // }
}

fn read_file(path: &str) -> Result<Vec<u8>, io::Error> {
    let mut f = File::open(path)?;
    let mut buffer = Vec::new();
    f.read_to_end(&mut buffer)?;
    Ok(buffer)
}
