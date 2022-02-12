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

pub fn add16(op1: u16, op2: u16) -> (u16, Flags) {
    let result = op1 + op2;

    let mut flags = Flags::EMPTY;

    if result < op1 {
        flags = flags | Flags::CARRY_FLAG;
    }

    let mut bits_set = 0;
    for pos in 0..8 {
        if result & (1 << pos) != 0 {
            bits_set += 1;
        }
    }
    if bits_set & 1 == 0 {
        flags = flags | Flags::PARITY_FLAG;
    }

    let bit_result = result & (1 << 7);
 
    if bit_result == 0 {
        let bit_op1 = op1 & (1 << 7);
        let bit_op2 = op2 & (1 << 7);
    
        if bit_result ^ bit_op1 == (1 << 7) || bit_result ^ bit_op2 == (1 << 7){
            flags = flags | Flags::AUXILIARY_CARRY_FLAG;
        }
    }

    if result == 0 {
        flags = flags | Flags::ZERO_FLAG;
    }

    let msb_result = result & (1 << 15);

    if msb_result == (1 << 15){
        flags = flags | Flags::SIGN_FLAG;
    }

    let msb_op1 = op1 & (1 << 15);
    let msb_op2 = op2 & (1 << 15);

    if msb_op1 ^ msb_result == (1 << 15) && msb_op2 ^ msb_result == (1 << 15){
        flags = flags | Flags::OVERFLOW_FLAG;
    }

    return (result, flags)
}

pub fn add8(op1: u8, op2: u8) -> (u8, Flags) {
    let result = op1 + op2;

    let mut flags = Flags::EMPTY;

    if result < op1 {
        flags = flags | Flags::CARRY_FLAG;
    }

    let mut bits_set = 0;

    for pos in 0..8 {
        if result & (1 << pos) != 0 {
            bits_set += 1;
        }
    }
    if bits_set & 1 == 0 {
        flags = flags | Flags::PARITY_FLAG;
    }

    let bit_result = result & (1 << 3);
 
    if bit_result == 0 {
        let bit_op1 = op1 & (1 << 3);
        let bit_op2 = op2 & (1 << 3);
    
        if bit_result ^ bit_op1 == (1 << 3) || bit_result ^ bit_op2 == (1 << 3){
            flags = flags | Flags::AUXILIARY_CARRY_FLAG;
        }
    }

    if result == 0 {
        flags = flags | Flags::ZERO_FLAG;
    }

    let msb_result = result & (1 << 7);

    if msb_result == (1 << 7){
        flags = flags | Flags::SIGN_FLAG;
    }

    let msb_op1 = op1 & (1 << 7);
    let msb_op2 = op2 & (1 << 7);

    if msb_op1 ^ msb_result == (1 << 7) && msb_op2 ^ msb_result == (1 << 7){
        flags = flags | Flags::OVERFLOW_FLAG;
    }

    return (result, flags)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_add16(){
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), add16(0, 0));
        assert_eq!((1, Flags::EMPTY), add16(1, 0));
        assert_eq!((0, Flags::CARRY_FLAG | Flags::ZERO_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG), add16(0xFFFF, 1));
        assert_eq!((0xFFE0, Flags::SIGN_FLAG | Flags::CARRY_FLAG), add16(0xFFF0, 0xFFF0));
        assert_eq!((0x8000, Flags::SIGN_FLAG | Flags::OVERFLOW_FLAG | Flags::AUXILIARY_CARRY_FLAG | Flags::PARITY_FLAG), add16(0x7FFF, 1));
    }

    #[test]
    fn test_add8() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), add8(0, 0));
        assert_eq!((1, Flags::EMPTY), add8(1, 0));
        assert_eq!((0, Flags::CARRY_FLAG | Flags::ZERO_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG), add8(0xFF, 1));
        assert_eq!((0xE0, Flags::SIGN_FLAG | Flags::CARRY_FLAG), add8(0xF0, 0xF0));
        assert_eq!((0x80, Flags::SIGN_FLAG | Flags::OVERFLOW_FLAG | Flags::AUXILIARY_CARRY_FLAG), add8(0x7F, 1));
    }
}