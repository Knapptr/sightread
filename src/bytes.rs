use nom::{bits, IResult};

fn handle_7_bit(input: (&[u8], usize)) -> IResult<(&[u8], usize), (u8, bool)> {
    let (remainder, key_bit): (_, u8) = nom::bits::complete::take(1u8)(input)?;
    let (remainder, value_bits): (_, u8) = nom::bits::complete::take(7u8)(remainder)?;
    match key_bit {
        0 => Ok((remainder, (value_bits, true))),
        1 => Ok((remainder, (value_bits, false))),
        _ => unreachable!(),
    }
}
pub fn variable_length_7(input: &[u8]) -> IResult<&[u8], usize> {
    let mut total = 0usize;
    let (mut remainder, (add_total, mut finished)) = bits(handle_7_bit)(input)?;
    total += add_total as usize;
    while !finished {
        let (new_remainder, (add_total, new_finished)) = bits(handle_7_bit)(remainder)?;
        total += add_total as usize;
        remainder = new_remainder;
        finished = new_finished;
    }
    Ok((remainder, total))
}
