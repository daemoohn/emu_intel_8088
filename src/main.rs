fn main() {
    println!("Hello, world!");
}

use bitflags::bitflags;

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
    let mut r_flags = compute_flags(op1 as u16, op1 as u16, true, false, result as u16);
    r_flags = (r_flags - Flags::AUXILIARY_CARRY_FLAG) | (temp_flags & Flags::AUXILIARY_CARRY_FLAG);
    r_flags = (r_flags - Flags::CARRY_FLAG) | (temp_flags & Flags::CARRY_FLAG);

    return (result, r_flags);
}

pub fn aas(op1: u16, flags: Flags) -> (u16, Flags) {
    // based on https://stackoverflow.com/questions/51710279/assembly-instructions-aaa
    // note: op1 is ax
    // IF ((( AL and 0FH ) > 9 ) or (AF==1)
    //     IF CPU<286 THEN
    //         AL = AL-6
    //     ELSE
    //         AX = AX-6
    //     ENDIF
    //     AH = AH-1
    //     CF = 1
    //     AF = 1
    // ELSE
    //     CF = 0
    //     AF = 0
    // ENDIF
    // AL = AL and 0Fh
    let mut r_flags = Flags::empty();
    let mut result = op1;
    if op1 & 0x000F > 9 || flags & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG {
        result -= 262;
        r_flags |= Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG;
    } else {
        r_flags -= Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG;
    }
    result &= 0xFF0F;

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
    let mut r_flags = compute_flags(op1 as u16, op1 as u16, true, false, result as u16);
    r_flags = (r_flags - Flags::AUXILIARY_CARRY_FLAG) | (temp_flags & Flags::AUXILIARY_CARRY_FLAG);
    r_flags = (r_flags - Flags::CARRY_FLAG) | (temp_flags & Flags::CARRY_FLAG);

    (result, r_flags)
}

pub fn aaa(op1: u16, flags: Flags) -> (u16, Flags) {
    // based on https://asm.inightmare.org/opcodelst/index.php?op=AAA
    // https://stackoverflow.com/questions/51710279/assembly-instructions-aaa
    // note: op1 is ax
    // IF ((( AL and 0FH ) > 9 ) or (AF==1)
    //     IF CPU<286 THEN
    //         AL = AL+6
    //     ELSE
    //         AX = AX+6
    //     ENDIF
    //     AH = AH+1
    //     CF = 1
    //     AF = 1
    // ELSE
    //     CF = 0
    //     AF = 0
    // ENDIF
    // AL = AL and 0Fh
    let mut r_flags = Flags::empty();
    let mut result = op1;
    if op1 & 0x000F > 9 || flags & Flags::AUXILIARY_CARRY_FLAG == Flags::AUXILIARY_CARRY_FLAG {
        result += 262;
        r_flags |= Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG;
    } else {
        r_flags -= Flags::AUXILIARY_CARRY_FLAG | Flags::CARRY_FLAG;
    }
    result &= 0xFF0F;

    (result, r_flags)
}

pub fn cmp16(op1: u16, op2: u16) -> Flags {
    let diff = op1 - op2;
    compute_flags(op1, op2, false, true, diff)
}

pub fn cmp8(op1: u8, op2: u8) -> Flags {
    let diff = op1 - op2;
    compute_flags(op1 as u16, op2 as u16, false, false, diff as u16)
}

pub fn neg16(op1: u16) -> (u16, Flags) {
    let (result, mut r_flags) = sub16(0, op1);
    if op1 == 0 {
        r_flags -= Flags::CARRY_FLAG;
    }

    (result, r_flags)
}

pub fn neg8(op1: u8) -> (u8, Flags) {
    let (result, mut r_flags) = sub8(0, op1);
    if op1 == 0 {
        r_flags -= Flags::CARRY_FLAG;
    }
    (result, r_flags)
}

pub fn dec16(op1: u16, flags: Flags) -> (u16, Flags) {
    let (result, mut r_flags) = sub16(op1, 1);
    // we remove carry flag from result flags and add it only if the initial flags contained it
    r_flags -= Flags::CARRY_FLAG | (flags & Flags::CARRY_FLAG);
    (result, r_flags)
}

pub fn dec8(op1: u8, flags: Flags) -> (u8, Flags) {
    let (result, mut r_flags) = sub8(op1, 1);
    // we remove carry flag from result flags and add it only if the initial flags contained it
    r_flags -= Flags::CARRY_FLAG | (flags & Flags::CARRY_FLAG);
    (result, r_flags)
}

pub fn sbb16(op1: u16, op2: u16, carry: u16) -> (u16, Flags) {
    let result = op1 - op2 - carry;
    let r_flags = compute_flags(op1, op2, false, true, result);
    (result, r_flags)
}

pub fn sbb8(op1: u8, op2: u8, carry: u8) -> (u8, Flags) {
    let result = op1 - op2 - carry;
    let r_flags = compute_flags(op1 as u16, op2 as u16, false, false, result as u16);
    (result, r_flags)
}

pub fn sub16(op1: u16, op2: u16) -> (u16, Flags) {
    let result = op1 - op2;
    let r_flags = compute_flags(op1, op2, false, true, result);
    (result, r_flags)
}

pub fn sub8(op1: u8, op2: u8) -> (u8, Flags) {
    let result = op1 - op2;
    let r_flags = compute_flags(op1 as u16, op2 as u16, false, false, result as u16);
    (result, r_flags)
}

pub fn inc16(op1: u16, flags: Flags) -> (u16, Flags) {
    let (result, mut r_flags) = add16(op1, 1);
    // we remove carry flag from result flags and add it only if the initial flags contained it
    r_flags -= Flags::CARRY_FLAG | (flags & Flags::CARRY_FLAG);
    (result, r_flags)
}

pub fn inc8(op1: u8, flags: Flags) -> (u8, Flags) {
    let (result, mut r_flags) = add8(op1, 1);
    // we remove carry flag from result flags and add it only if the initial flags contained it
    r_flags -= Flags::CARRY_FLAG | (flags & Flags::CARRY_FLAG);
    (result, r_flags)
}

pub fn adc16(op1: u16, op2: u16, carry: u16) -> (u16, Flags) {
    let result = op1 + op2 + carry;
    let r_flags = compute_flags(op1, op2, true, true, result);
    (result, r_flags)
}

pub fn adc8(op1: u8, op2: u8, carry: u8) -> (u8, Flags) {
    let result = op1 + op2 + carry;
    let r_flags = compute_flags(op1 as u16, op2 as u16, true, false, result as u16);

    (result, r_flags)
}

pub fn add16(op1: u16, op2: u16) -> (u16, Flags) {
    let result = op1 + op2;
    let r_flags = compute_flags(op1, op2, true, true, result);

    (result, r_flags)
}

pub fn add8(op1: u8, op2: u8) -> (u8, Flags) {
    let result = op1 + op2;
    let r_flags = compute_flags(op1 as u16, op2 as u16, true, false, result as u16);

    (result, r_flags)
}

fn compute_flags(op1: u16, op2: u16, is_add: bool, is_word: bool, result: u16) -> Flags {
    let mut flags = Flags::empty();

    if is_add {
        if result < op1 {
            flags |= Flags::CARRY_FLAG;
        }
    } else if op2 > op1 {
        flags |= Flags::CARRY_FLAG;
    }

    let mut bits_set = 0;

    for pos in 0..8 {
        if result & (1 << pos) != 0 {
            bits_set += 1;
        }
    }
    if bits_set & 1 == 0 {
        flags |= Flags::PARITY_FLAG;
    }

    if is_add {
        let bit_result = result & (1 << 3);
        let bit_op1 = op1 & (1 << 3);
        let bit_op2 = op2 & (1 << 3);

        if (bit_result & bit_op1 & bit_op2 == 1 << 3)
            || (bit_result == 0 && bit_op1 | bit_op2 == 1 << 3)
        {
            flags |= Flags::AUXILIARY_CARRY_FLAG;
        }
    } else if op2 & 0x0F > op1 & 0x0F {
        flags |= Flags::AUXILIARY_CARRY_FLAG;
    }

    if result == 0 {
        flags |= Flags::ZERO_FLAG;
    }

    let msb_result;
    if is_word {
        msb_result = result & (1 << 15);

        if msb_result == (1 << 15) {
            flags |= Flags::SIGN_FLAG;
        }

        let msb_op1 = op1 & (1 << 15);
        let msb_op2 = op2 & (1 << 15);

        if is_add {
            if msb_op1 == msb_op2
                && (msb_op1 ^ msb_result == (1 << 15) && msb_op2 ^ msb_result == (1 << 15))
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        } else if msb_op1 ^ msb_op2 == 1 << 15 && msb_result == msb_op2 {
            flags |= Flags::OVERFLOW_FLAG;
        }
    } else {
        msb_result = result & (1 << 7);

        if msb_result == (1 << 7) {
            flags |= Flags::SIGN_FLAG;
        }

        let msb_op1 = op1 & (1 << 7);
        let msb_op2 = op2 & (1 << 7);

        if is_add {
            if msb_op1 == msb_op2
                && (msb_op1 ^ msb_result == (1 << 7) && msb_op2 ^ msb_result == (1 << 7))
            {
                flags |= Flags::OVERFLOW_FLAG;
            }
        } else if msb_op1 ^ msb_op2 == 1 << 7 && msb_result == msb_op2 {
            flags |= Flags::OVERFLOW_FLAG;
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
