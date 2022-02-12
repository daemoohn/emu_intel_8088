fn main() {
    println!("Hello, world!");
}

use bitflags::bitflags;

bitflags! {
    pub struct Flags : u16 {
        const EMPTY = 0;
        const CARRY_FLAG = 1 << 0;
        const PARITY_FLAG = 1 << 2;
        const AUXILIARY_CARRY_FLAG = 1 << 4;
        const ZERO_FLAG = 1 << 6;
        const SIGN_FLAG = 1 << 7;
        const TRAP_FLAG = 1 << 8;
        const INTERRUPT_FLAG = 1 << 9;
        const DIRECTION_FLAG = 1 << 10;
        const OVERFLOW_FLAG = 1 << 11;
    }
}

pub fn add8(op1: u8, op2: u8) -> (u8, Flags) {
    // compute the result
    let result = op1 + op2;

    let mut flags = Flags::EMPTY;

    // set carry flag if the MSB of the result is 0 and at least one MSB of the operands is 1
    let msb_result = result & (1 << 7);
    let msb_op1 = op1 & (1 << 7);
    let msb_op2 = op2 & (1 << 7);

    if msb_result == 0 {
        if msb_result ^ msb_op1 == (1 << 7) || msb_result ^ msb_op2 == (1 << 7) {
            flags = flags | Flags::CARRY_FLAG;
        }
    }

    // count the number of bits set in the result
    // if the number is even set the parity flag
    let mut bits_set = 0;

    for pos in 0..7 {
        bits_set += result & (1 << pos);
    }
    if bits_set & 1 == 0 {
        flags = flags | Flags::PARITY_FLAG;
    }

    // set the auxiliary carry if the 4th bit of the result is 0 and at least one of the 4th bits of the operands is 1
    let bit_result = result & (1 << 3);
 
    if bit_result == 0 {
        let bit_op1 = op1 & (1 << 3);
        let bit_op2 = op2 & (1 << 3);
    
        if bit_result ^ bit_op1 == (1 << 3) || bit_result ^ bit_op2 == (1 << 3){
            flags = flags | Flags::AUXILIARY_CARRY_FLAG;
        }
    }

    // Zero flag is set when result == 0
    if result == 0 {
        flags = flags | Flags::ZERO_FLAG;
    }

    // Set sign flag to MSB of result
    if msb_result == 1 {
        flags = flags | Flags::SIGN_FLAG;
    }

    // if two operators with MSB 1 have result with MSB 0 we have overflow
    // if two operators with MSB 0 have result with MSB 1 we have overflow
    if msb_op1 ^ msb_result == (1 << 7) && msb_op2 ^ msb_result == (1 << 7){
        flags = flags | Flags::OVERFLOW_FLAG;
    }

    return (result, flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn internal() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), add8(0, 0));
        assert_eq!((1, Flags::EMPTY), add8(1, 0));
        assert_eq!((0, Flags::CARRY_FLAG | Flags::ZERO_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG), add8(0xFF, 1));
    }
}