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
pub fn variable_length_7(input: &[u8]) -> IResult<&[u8], u32> {
    let mut total: u32 = 0;
    let (mut remainder, (add_total, mut finished)) = bits(handle_7_bit)(input)?;
    total += add_total as u32;
    while !finished {
        let (new_remainder, (add_total, new_finished)) = bits(handle_7_bit)(remainder)?;
        total = total << 7usize;
        total = total | add_total as u32;
        remainder = new_remainder;
        finished = new_finished;
    }
    Ok((remainder, total))
}

#[cfg(test)]
#[test]
fn var_7() {
    let var_bytes = [0x81, 0x00];
    let expected = 0x00000080;
    assert_eq!(expected, variable_length_7(&var_bytes).unwrap().1)
}
