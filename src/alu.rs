use std::{mem, ops::BitAnd, ops::BitOr, ops::BitXor, ops::Shl};

use bitflags::bitflags;
use num::{NumCast, Unsigned};

enum OperationType {
    ADD,
    INC,
    AAA,
    //DAA,
    SUB,
    DEC,
    NEG,
    CMP,
    AAS,
    //DAS,
}

bitflags! {
    #[derive(Default)]
    pub struct Flags : u16 {
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

// pub fn mul16(op1: u16, op2: u16) -> (u32, Flags) {}

// pub fn mul8(op1: u8, op2: u8) -> (u16, Flags) {
//     let result = (op1 as u16) * (op2 as u16);
// }

pub fn das(op1: u8, flags: Flags) -> (u8, Flags) {
    // base on https://pdos.csail.mit.edu/6.828/2018/readings/i386/DAS.htm
    // IF (AL AND 0FH) > 9 OR AF = 1
    // THEN
    //    AL := AL - 6;
    //    AF := 1;
    // ELSE
    //    AF := 0;
    // FI;
    // IF (AL > 9FH) OR (CF = 1)
    // THEN
    //    AL := AL - 60H;
    //    CF := 1;
    // ELSE CF := 0;
    // FI;
    //
    let mut result = op1;
    let mut temp_flags = Flags::empty();

    if op1 & 0x0000F > 9 || flags & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG {
        result -= 6;
        temp_flags |= Flags::AUXILIARY_CARRY_FLAG;
    }
    if op1 > 0x009F || flags & Flags::CARRY_FLAG == Flags::CARRY_FLAG {
        result -= 0x0060;
        temp_flags |= Flags::CARRY_FLAG;
    }
    let mut r_flags = compute_flags_gen(op1, op1, result, None, OperationType::SUB);
    r_flags = (r_flags - Flags::AUXILIARY_CARRY_FLAG) | (temp_flags & Flags::AUXILIARY_CARRY_FLAG);
    r_flags = (r_flags - Flags::CARRY_FLAG) | (temp_flags & Flags::CARRY_FLAG);

    return (result, r_flags);
}

pub fn aas(op1: u16, flags: Flags) -> (u16, Flags) {
    // based on https://stackoverflow.com/questions/51710279/assembly-instructions-aaa
    let mut result = op1;
    if op1 & 0x000F > 9 || flags & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG {
        result -= 262;
    }
    result &= 0xFF0F;
    let r_flags = compute_flags_gen(op1, op1, result, Some(flags), OperationType::AAS);
    (result, r_flags)
}

pub fn cmp16(op1: u16, op2: u16) -> Flags {
    let diff = op1 - op2;
    compute_flags_gen(op1, op2, diff, None, OperationType::CMP)
}

pub fn cmp8(op1: u8, op2: u8) -> Flags {
    let diff = op1 - op2;
    compute_flags_gen(op1, op2, diff, None, OperationType::CMP)
}

pub fn neg16(op1: u16) -> (u16, Flags) {
    let result = 0 - op1;
    let flags = compute_flags_gen(0, op1, result, None, OperationType::NEG);
    (result, flags)
}

pub fn neg8(op1: u8) -> (u8, Flags) {
    let result = 0 - op1;
    let flags = compute_flags_gen(0, op1, result, None, OperationType::NEG);
    (result, flags)
}

pub fn dec16(op1: u16, flags: Flags) -> (u16, Flags) {
    let result = op1 - 1;
    let r_flags = compute_flags_gen(op1, 1, result, Some(flags), OperationType::DEC);
    (result, r_flags)
}

pub fn dec8(op1: u8, flags: Flags) -> (u8, Flags) {
    let result = op1 - 1;
    let r_flags = compute_flags_gen(op1, 1, result, Some(flags), OperationType::DEC);
    (result, r_flags)
}

pub fn sbb16(op1: u16, op2: u16, carry: u16) -> (u16, Flags) {
    let result = op1 - op2 - carry;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::SUB);
    (result, r_flags)
}

pub fn sbb8(op1: u8, op2: u8, carry: u8) -> (u8, Flags) {
    let result = op1 - op2 - carry;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::SUB);
    (result, r_flags)
}

pub fn sub16(op1: u16, op2: u16) -> (u16, Flags) {
    let result = op1 - op2;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::SUB);
    (result, r_flags)
}

pub fn sub8(op1: u8, op2: u8) -> (u8, Flags) {
    let result = op1 - op2;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::SUB);
    (result, r_flags)
}

pub fn daa(op1: u8, flags: Flags) -> (u8, Flags) {
    // note: op1 is ax
    // based on https://stackoverflow.com/questions/18945247/how-does-aaa-work-in-8086-instruction-set
    // based on https://www.felixcloutier.com/x86/daa
    // if ( (AL and 0Fh) > 9 or (AuxC = 1)) then
    //     al := al + 6
    //     AuxC := 1               ;Set Auxilliary carry.
    // endif
    // if ( (al > 9Fh) or (Carry = 1)) then
    //     al := al + 60h
    //     Carry := 1;             ;Set carry flag.
    // endif
    let mut temp_flags = Flags::empty();
    let mut result = op1;
    if op1 & 0x000F > 9 || flags & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG {
        result += 6;
        temp_flags |= Flags::AUXILIARY_CARRY_FLAG;
    }
    if result > 0x009F || flags & Flags::CARRY_FLAG == Flags::CARRY_FLAG {
        result += 0x0060;
        temp_flags |= Flags::CARRY_FLAG;
    }
    let mut r_flags = compute_flags_gen(op1, op1, result, None, OperationType::ADD);
    r_flags = (r_flags - Flags::AUXILIARY_CARRY_FLAG) | (temp_flags & Flags::AUXILIARY_CARRY_FLAG);
    r_flags = (r_flags - Flags::CARRY_FLAG) | (temp_flags & Flags::CARRY_FLAG);

    (result, r_flags)
}

pub fn aaa(op1: u16, flags: Flags) -> (u16, Flags) {
    // based on https://asm.inightmare.org/opcodelst/index.php?op=AAA
    // https://stackoverflow.com/questions/51710279/assembly-instructions-aaa
    let mut result = op1;
    if op1 & 0x000F > 9 || flags & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG {
        result += 262;
    }
    result &= 0xFF0F;
    let r_flags = compute_flags_gen(op1, op1, result, Some(flags), OperationType::AAA);
    (result, r_flags)
}

pub fn inc16(op1: u16, flags: Flags) -> (u16, Flags) {
    let result = op1 + 1;
    let r_flags = compute_flags_gen(op1, 1, result, Some(flags), OperationType::INC);
    (result, r_flags)
}

pub fn inc8(op1: u8, flags: Flags) -> (u8, Flags) {
    let result = op1 + 1;
    let r_flags = compute_flags_gen(op1, 1, result, Some(flags), OperationType::INC);
    (result, r_flags)
}

pub fn adc16(op1: u16, op2: u16, carry: u16) -> (u16, Flags) {
    let result = op1 + op2 + carry;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::ADD);
    (result, r_flags)
}

pub fn adc8(op1: u8, op2: u8, carry: u8) -> (u8, Flags) {
    let result = op1 + op2 + carry;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::ADD);
    (result, r_flags)
}

pub fn add16(op1: u16, op2: u16) -> (u16, Flags) {
    let result = op1 + op2;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::ADD);
    (result, r_flags)
}

pub fn add8(op1: u8, op2: u8) -> (u8, Flags) {
    let result = op1 + op2;
    let r_flags = compute_flags_gen(op1, op2, result, None, OperationType::ADD);
    (result, r_flags)
}

fn compute_flags_gen<
    T: Unsigned
        + PartialOrd
        + NumCast
        + BitAnd<Output = T>
        + BitOr<Output = T>
        + BitXor<Output = T>
        + Shl<Output = T>
        + Copy,
>(
    op1: T,
    op2: T,
    result: T,
    input_flags: Option<Flags>,
    op_type: OperationType,
) -> Flags {
    let mut flags = Flags::empty();

    match op_type {
        OperationType::ADD => {
            if result < op1 {
                flags |= Flags::CARRY_FLAG;
            }

            let mut bits_set = 0;
            for pos in 0..8 {
                if result & (T::one() << T::from(pos).unwrap()) != T::zero() {
                    bits_set += 1;
                }
            }
            if bits_set & 1 == 0 {
                flags |= Flags::PARITY_FLAG;
            }

            let bit_result = result & (T::one() << T::from(3).unwrap());
            let bit_op1 = op1 & (T::one() << T::from(3).unwrap());
            let bit_op2 = op2 & (T::one() << T::from(3).unwrap());

            if (bit_result & bit_op1 & bit_op2 == T::one() << T::from(3).unwrap())
                || (bit_result == T::zero() && bit_op1 | bit_op2 == T::one() << T::from(3).unwrap())
            {
                flags |= Flags::AUXILIARY_CARRY_FLAG;
            }

            if result == T::zero() {
                flags |= Flags::ZERO_FLAG;
            }

            let msb_bit = (mem::size_of::<T>() - 1) * 8 + 7;
            let msb_result = result & (T::one() << T::from(msb_bit).unwrap());

            if msb_result == (T::one() << T::from(msb_bit).unwrap()) {
                flags |= Flags::SIGN_FLAG;
            }

            let msb_op1 = op1 & (T::one() << T::from(msb_bit).unwrap());
            let msb_op2 = op2 & (T::one() << T::from(msb_bit).unwrap());

            if msb_op1 == msb_op2
                && (msb_op1 ^ msb_result == (T::one() << T::from(msb_bit).unwrap())
                    && msb_op2 ^ msb_result == (T::one() << T::from(msb_bit).unwrap()))
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        }
        OperationType::INC => {
            flags |= input_flags.unwrap() & Flags::CARRY_FLAG;
            let mut bits_set = 0;
            for pos in 0..8 {
                if result & (T::one() << T::from(pos).unwrap()) != T::zero() {
                    bits_set += 1;
                }
            }
            if bits_set & 1 == 0 {
                flags |= Flags::PARITY_FLAG;
            }

            let bit_result = result & (T::one() << T::from(3).unwrap());
            let bit_op1 = op1 & (T::one() << T::from(3).unwrap());
            let bit_op2 = op2 & (T::one() << T::from(3).unwrap());

            if (bit_result & bit_op1 & bit_op2 == T::one() << T::from(3).unwrap())
                || (bit_result == T::zero() && bit_op1 | bit_op2 == T::one() << T::from(3).unwrap())
            {
                flags |= Flags::AUXILIARY_CARRY_FLAG;
            }

            if result == T::zero() {
                flags |= Flags::ZERO_FLAG;
            }

            let msb_bit = (mem::size_of::<T>() - 1) * 8 + 7;
            let msb_result = result & (T::one() << T::from(msb_bit).unwrap());

            if msb_result == (T::one() << T::from(msb_bit).unwrap()) {
                flags |= Flags::SIGN_FLAG;
            }

            let msb_op1 = op1 & (T::one() << T::from(msb_bit).unwrap());
            let msb_op2 = op2 & (T::one() << T::from(msb_bit).unwrap());

            if msb_op1 == msb_op2
                && (msb_op1 ^ msb_result == (T::one() << T::from(msb_bit).unwrap())
                    && msb_op2 ^ msb_result == (T::one() << T::from(msb_bit).unwrap()))
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        }
        OperationType::AAA => {
            if op1 & T::from(0x000F).unwrap() > T::from(9).unwrap()
                || input_flags.unwrap() & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG
            {
                flags |= Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG;
            }
        }
        OperationType::SUB | OperationType::CMP => {
            if op2 > op1 {
                flags |= Flags::CARRY_FLAG;
            }

            let mut bits_set = 0;
            for pos in 0..8 {
                if result & (T::one() << T::from(pos).unwrap()) != T::zero() {
                    bits_set += 1;
                }
            }
            if bits_set & 1 == 0 {
                flags |= Flags::PARITY_FLAG;
            }

            if op2 & T::from(0x0F).unwrap() > op1 & T::from(0x0F).unwrap() {
                flags |= Flags::AUXILIARY_CARRY_FLAG;
            }

            if result == T::zero() {
                flags |= Flags::ZERO_FLAG;
            }

            let msb_bit = (mem::size_of::<T>() - 1) * 8 + 7;
            let msb_result = result & (T::one() << T::from(msb_bit).unwrap());

            if msb_result == (T::one() << T::from(msb_bit).unwrap()) {
                flags |= Flags::SIGN_FLAG;
            }

            let msb_op1 = op1 & (T::one() << T::from(msb_bit).unwrap());
            let msb_op2 = op2 & (T::one() << T::from(msb_bit).unwrap());

            if msb_op1 ^ msb_op2 == (T::one() << T::from(msb_bit).unwrap()) && msb_result == msb_op2
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        }
        OperationType::DEC => {
            flags |= input_flags.unwrap() & Flags::CARRY_FLAG;

            let mut bits_set = 0;
            for pos in 0..8 {
                if result & (T::one() << T::from(pos).unwrap()) != T::zero() {
                    bits_set += 1;
                }
            }
            if bits_set & 1 == 0 {
                flags |= Flags::PARITY_FLAG;
            }

            if op2 & T::from(0x0F).unwrap() > op1 & T::from(0x0F).unwrap() {
                flags |= Flags::AUXILIARY_CARRY_FLAG;
            }

            if result == T::zero() {
                flags |= Flags::ZERO_FLAG;
            }

            let msb_bit = (mem::size_of::<T>() - 1) * 8 + 7;
            let msb_result = result & (T::one() << T::from(msb_bit).unwrap());

            if msb_result == (T::one() << T::from(msb_bit).unwrap()) {
                flags |= Flags::SIGN_FLAG;
            }

            let msb_op1 = op1 & (T::one() << T::from(msb_bit).unwrap());
            let msb_op2 = op2 & (T::one() << T::from(msb_bit).unwrap());

            if msb_op1 ^ msb_op2 == (T::one() << T::from(msb_bit).unwrap()) && msb_result == msb_op2
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        }
        OperationType::NEG => {
            if op2 != T::zero() {
                flags |= Flags::CARRY_FLAG;
            }

            let mut bits_set = 0;
            for pos in 0..8 {
                if result & (T::one() << T::from(pos).unwrap()) != T::zero() {
                    bits_set += 1;
                }
            }
            if bits_set & 1 == 0 {
                flags |= Flags::PARITY_FLAG;
            }

            if op2 & T::from(0x0F).unwrap() > op1 & T::from(0x0F).unwrap() {
                flags |= Flags::AUXILIARY_CARRY_FLAG;
            }

            if result == T::zero() {
                flags |= Flags::ZERO_FLAG;
            }

            let msb_bit = (mem::size_of::<T>() - 1) * 8 + 7;
            let msb_result = result & (T::one() << T::from(msb_bit).unwrap());

            if msb_result == (T::one() << T::from(msb_bit).unwrap()) {
                flags |= Flags::SIGN_FLAG;
            }

            let msb_op1 = op1 & (T::one() << T::from(msb_bit).unwrap());
            let msb_op2 = op2 & (T::one() << T::from(msb_bit).unwrap());

            if msb_op1 ^ msb_op2 == (T::one() << T::from(msb_bit).unwrap()) && msb_result == msb_op2
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        }
        OperationType::AAS => {
            if op1 & T::from(0x000F).unwrap() > T::from(9).unwrap()
                || input_flags.unwrap() & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG
            {
                flags |= Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG;
            }
        }
    }

    flags
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_das() {
        assert_eq!(
            (
                0x88,
                Flags::SIGN_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::CARRY_FLAG
            ),
            das(0xEE, Flags::empty())
        );
    }

    #[test]
    fn test_aas() {
        assert_eq!(
            (0x507, Flags::empty()),
            aas(0x5C7, Flags::CARRY_FLAG | Flags::SIGN_FLAG)
        );
    }

    #[test]
    fn test_daa() {
        assert_eq!(
            (
                0x14,
                Flags::AUXILIARY_CARRY_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::CARRY_FLAG
                    | Flags::OVERFLOW_FLAG
            ),
            daa(0xAE, Flags::SIGN_FLAG)
        );
        //assert_eq!((0x34, Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG), daa(0x2E, Flags::OVERFLOW_FLAG | Flags::SIGN_FLAG));
    }

    #[test]
    fn test_aaa() {
        // it doesn't cover all the branches
        assert_eq!(
            (257, Flags::CARRY_FLAG | Flags::AUXILIARY_CARRY_FLAG),
            aaa(11, Flags::empty())
        )
    }

    #[test]
    fn test_cmp16() {
        assert_eq!(Flags::ZERO_FLAG | Flags::PARITY_FLAG, cmp16(0x55FF, 0x55FF));
        assert_eq!(Flags::empty(), cmp16(0xFF01, 0xFF00));
        assert_eq!(Flags::AUXILIARY_CARRY_FLAG, cmp16(281, 267));
        assert_eq!(
            Flags::CARRY_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG | Flags::SIGN_FLAG,
            cmp16(294, 375)
        );
        assert_eq!(
            Flags::AUXILIARY_CARRY_FLAG | Flags::OVERFLOW_FLAG,
            cmp16(32768, 32767)
        )
    }

    #[test]
    fn test_cmp8() {
        assert_eq!(Flags::ZERO_FLAG | Flags::PARITY_FLAG, cmp8(0x55, 0x55));
        assert_eq!(Flags::empty(), cmp8(3, 2));
        assert_eq!(Flags::AUXILIARY_CARRY_FLAG, cmp8(25, 11));
        assert_eq!(
            Flags::CARRY_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG | Flags::SIGN_FLAG,
            cmp8(38, 119)
        );
        assert_eq!(
            Flags::AUXILIARY_CARRY_FLAG | Flags::OVERFLOW_FLAG,
            cmp8(128, 127)
        )
    }

    #[test]
    fn test_neg16() {
        assert_eq!(
            (
                0xFFF5,
                Flags::CARRY_FLAG
                    | Flags::SIGN_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
            ),
            neg16(11)
        );
        assert_eq!(
            (0x7FF0, Flags::CARRY_FLAG | Flags::PARITY_FLAG),
            neg16(0x8010)
        );
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), neg16(0));
        assert_eq!(
            (
                0x8000,
                Flags::CARRY_FLAG | Flags::PARITY_FLAG | Flags::SIGN_FLAG | Flags::OVERFLOW_FLAG
            ),
            neg16(0x8000)
        );
    }

    #[test]
    fn test_neg8() {
        assert_eq!(
            (
                245,
                Flags::CARRY_FLAG
                    | Flags::SIGN_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
            ),
            neg8(11)
        );
        assert_eq!(
            (56, Flags::CARRY_FLAG | Flags::AUXILIARY_CARRY_FLAG),
            neg8(200)
        );
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), neg8(0));
        assert_eq!(
            (
                0x80,
                Flags::CARRY_FLAG | Flags::SIGN_FLAG | Flags::OVERFLOW_FLAG
            ),
            neg8(0x80)
        );
    }

    #[test]
    fn test_dec16() {
        assert_eq!(
            (
                0xFFFF,
                Flags::SIGN_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG
            ),
            dec16(0, Flags::empty())
        );
    }

    #[test]
    fn test_dec8() {
        assert_eq!(
            (
                0xFF,
                Flags::SIGN_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG
            ),
            dec8(0, Flags::empty())
        );
    }

    #[test]
    fn test_sbb16() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), sbb16(0, 0, 0));
        assert_eq!((1, Flags::empty()), sbb16(1, 0, 0));
        assert_eq!((1, Flags::empty()), sbb16(3, 1, 1));
    }

    #[test]
    fn test_sbb8() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), sbb8(0, 0, 0));
        assert_eq!((1, Flags::empty()), sbb8(1, 0, 0));
        assert_eq!((1, Flags::empty()), sbb8(3, 1, 1));
    }

    #[test]
    fn test_sub16() {
        assert_eq!(
            (0, Flags::ZERO_FLAG | Flags::PARITY_FLAG),
            sub16(0x55FF, 0x55FF)
        );
        assert_eq!((1, Flags::empty()), sub16(0xFF01, 0xFF00));
        assert_eq!((14, Flags::AUXILIARY_CARRY_FLAG), sub16(281, 267));
        assert_eq!(
            (
                65455,
                Flags::CARRY_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
                    | Flags::SIGN_FLAG
            ),
            sub16(294, 375)
        );
        assert_eq!(
            (1, Flags::AUXILIARY_CARRY_FLAG | Flags::OVERFLOW_FLAG),
            sub16(32768, 32767)
        )
    }

    #[test]
    fn test_sub8() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), sub8(0x55, 0x55));
        assert_eq!((1, Flags::empty()), sub8(3, 2));
        assert_eq!((14, Flags::AUXILIARY_CARRY_FLAG), sub8(25, 11));
        assert_eq!(
            (
                175,
                Flags::CARRY_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
                    | Flags::SIGN_FLAG
            ),
            sub8(38, 119)
        );
        assert_eq!(
            (1, Flags::AUXILIARY_CARRY_FLAG | Flags::OVERFLOW_FLAG),
            sub8(128, 127)
        )
    }

    #[test]
    fn test_inc16() {
        assert_eq!(
            (
                0,
                Flags::ZERO_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG
            ),
            inc16(0xFFFF, Flags::empty())
        );
    }

    #[test]
    fn test_inc8() {
        assert_eq!(
            (
                0,
                Flags::ZERO_FLAG | Flags::PARITY_FLAG | Flags::AUXILIARY_CARRY_FLAG
            ),
            inc8(0xFF, Flags::empty())
        );
    }

    #[test]
    fn test_adc16() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), adc16(0, 0, 0));
        assert_eq!((1, Flags::empty()), adc16(0, 0, 1));
        assert_eq!((2, Flags::empty()), adc16(1, 0, 1));
    }

    #[test]
    fn test_adc8() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), adc8(0, 0, 0));
        assert_eq!((1, Flags::empty()), adc8(0, 0, 1));
        assert_eq!((2, Flags::empty()), adc8(1, 0, 1));
    }

    #[test]
    fn test_add16() {
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), add16(0, 0));
        assert_eq!((1, Flags::empty()), add16(1, 0));
        assert_eq!(
            (
                0,
                Flags::CARRY_FLAG
                    | Flags::ZERO_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
            ),
            add16(0xFFFF, 1)
        );
        assert_eq!(
            (0xFFE0, Flags::SIGN_FLAG | Flags::CARRY_FLAG),
            add16(0xFFF0, 0xFFF0)
        );
        assert_eq!(
            (
                0x8000,
                Flags::SIGN_FLAG
                    | Flags::OVERFLOW_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
                    | Flags::PARITY_FLAG
            ),
            add16(0x7FFF, 1)
        );
    }

    #[test]
    fn test_add8() {
        assert_eq!(
            (
                137,
                Flags::AUXILIARY_CARRY_FLAG | Flags::SIGN_FLAG | Flags::OVERFLOW_FLAG
            ),
            add8(43, 94)
        );
        assert_eq!((0, Flags::ZERO_FLAG | Flags::PARITY_FLAG), add8(0, 0));
        assert_eq!((1, Flags::empty()), add8(1, 0));
        assert_eq!(
            (
                0,
                Flags::CARRY_FLAG
                    | Flags::ZERO_FLAG
                    | Flags::PARITY_FLAG
                    | Flags::AUXILIARY_CARRY_FLAG
            ),
            add8(0xFF, 1)
        );
        assert_eq!(
            (0xE0, Flags::SIGN_FLAG | Flags::CARRY_FLAG),
            add8(0xF0, 0xF0)
        );
        assert_eq!(
            (
                0x80,
                Flags::SIGN_FLAG | Flags::OVERFLOW_FLAG | Flags::AUXILIARY_CARRY_FLAG
            ),
            add8(0x7F, 1)
        );
    }
}
