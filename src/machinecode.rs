use std::io;
use std::io::prelude::*;
use std::mem;
use crate::bit_utils::read_bytes;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Display)]
pub enum Opcode {
    ADD = 0x00,
    OR = 0x08,
    ADC = 0x10,
    SBB = 0x18,
    AND = 0x20,
    ES = 0x26,
    DAA = 0x27,
    SUB = 0x28,
    CS = 0x2E,
    DAS = 0x2F,
    XOR = 0x30,
    SS = 0x36,
    AAA = 0x37,
    CMP = 0x38,
    DS = 0x3E,
    AAS = 0x3F,
    INC = 0x40,
    DEC = 0x48,
    PUSH = 0x50,
    POP = 0x58,
    JO = 0x70,
    JNO = 0x71,
    JB = 0x72, // JB JC JNAE
    JNB = 0x73, // JNB JNC JAE
    JE = 0x74, // JE JZ
    JNE = 0x75, // JNE JNZ
    JBE = 0x76, // JBE JNA
    JNBE = 0x77, // JNBE JA
    JS = 0x78,
    JNS = 0x79,
    JP = 0x7A, // JP JPE
    JNP = 0x7B, // JNP JPO
    JL = 0x7C, // JL JNGE
    JNL = 0x7D, // JNL JGE
    JLE = 0x7E, // JLE JNG
    JNLE = 0x7F, // JNLE JG
    TEST = 0x84,
    XCHG = 0x86,
    LEA = 0x8D,
    MOV = 0x8E,
    NOP = 0x90,
    CBW = 0x98,
    CWD = 0x99,
    WAIT = 0x9B,
    PUSHF = 0x9C,
    POPF = 0x9D,
    SAHF = 0x9E,
    LAHF = 0x9F,
    MOVSB = 0xA4,
    MOVSW = 0xA5,
    CMPSB = 0xA6,
    CMPSW = 0xA7,
    STOSB = 0xAA,
    STOSW = 0xAB,
    LODSB = 0xAC,
    LODSW = 0xAD,
    SCASB = 0xAE,
    SCASW = 0xAF,
    ROL = 0xB8,
    ROR = 0xB9,
    RCL = 0xBA,
    RCR = 0xBB,
    SHL = 0xBC,
    SHR = 0xBD,
    SAL = 0xBE,
    SAR = 0xBF,
    RET = 0xC3,
    LES = 0xC4,
    LDS = 0xC5,
    RETF = 0xCB,
    INT = 0xCD,
    INTO = 0xCE,
    IRET = 0xCF,
    AAM = 0xD4,
    AAD = 0xD5,
    BAD = 0xD6,
    XLAT = 0xD7,
    LOOPNZ = 0xE0, // LOOPNZ LOOPNE
    LOOPZ = 0xE1, // LOOPZ LOOPE
    LOOP = 0xE2,
    JCXZ = 0xE3,
    IN = 0xE4,
    OUT = 0xE6,
    CALL = 0xE8,
    JMP = 0xE9,
    NOT = 0xEA,
    NEG = 0xEB,
    MUL = 0xEC,
    IMUL = 0xED,
    DIV = 0xEE,
    IDIV = 0xEF,
    LOCK = 0xF0,
    REP = 0xF1,
    REPNZ = 0xF2, // REPNZ REPNE
    REPZ = 0xF3, // REPZ REPE
    HLT = 0xF4,
    CMC = 0xF5,
    CLC = 0xF8,
    STC = 0xF9,
    CLI = 0xFA,
    STI = 0xFB,
    CLD = 0xFC,
    STD = 0xFD,
    LCALL = 0xFE,
    LJMP = 0xFF
}
// FREE: 0x01..=0x07 | 0x09..=0x0F | 0x11..=0x17 | 0x19..=0x1F | 0x21..=0x25 | 0x29..=0x2D | 0x31..=0x35 | 0x39..=0x3D | 0x41..=0x47 | 0x49..=0x4F | 0x51..=0x57 | 0x59..=0x6F | 0x80..=0x83 | 0x85 | 0x87..=0x8C | 0x8F | 0x91..=0x97 | 0x9A | 0xA0..=0xA3 | 0xA8..=0xA9 | 0xB0..=0xB7 | 0xC0..=0xC2 | 0xC6..=0xCA | 0xCC | 0xD0..=0xD3 | 0xD8..=0xDF | 0xE5 | 0xE7 | 0xF6..=0xF7

impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Hash, Display)]
pub enum Operand {
    AX = 0,
    CX = 1,
    DX = 2,
    BX = 3,
    SP = 4,
    BP = 5,
    SI = 6,
    DI = 7,
    ES = 8,
    CS = 9,
    SS = 10,
    DS = 11,
    AL = 12,
    CL = 13,
    DL = 14,
    BL = 15,
    AH = 16,
    CH = 17,
    DH = 18,
    BH = 19,
    DisplacementBXSI = 20,
    DisplacementBXDI = 21,
    DisplacementBPSI = 22,
    DisplacementBPDI = 23,
    DisplacementSI = 24,
    DisplacementDI = 25,
    DisplacementBP = 26,
    DisplacementBX = 27,
    Displacement = 28,
    None = 29
}

impl From<u8> for Operand {
    fn from(value: u8) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(value) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct Instruction {
    pub position: u16,
    pub buffer: [u8; 8],
    pub length: u8,
    pub prefix: Opcode,
    pub opcode: Opcode,
    pub data_width: u8,
    pub first_operand: Operand,
    pub second_operand: Operand,
    pub segment_override: Operand,
    pub immediate: u32,
    pub displacement: i32
}

fn decode_mod_rm<R>(stream: &mut R, instruction: &mut Instruction, is_grp: bool, swap_operands: bool) -> io::Result<()> where R: Read {
    let mut field: u32 = 0;
    read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut field)?;
    let mut mode: u8 = ((field as u8)>>6)&0x3;
    let mut first_operand = ((field as u8)>>3)&0x7;
    if !is_grp && instruction.data_width == 8 {
        first_operand += Operand::AL as u8;
    }
    let mut second_operand = (field as u8)&0x7;
    if mode == 3 {
        if instruction.data_width == 8 {
            second_operand += Operand::AL as u8;
        }
    } else if mode == 0 && second_operand == 6 {
        mode = 2;
        second_operand = Operand::Displacement as u8;
    } else {
        second_operand += Operand::DisplacementBXSI as u8;
    }
    let mut displacement: u32 = 0;
    match mode {
        1 => {
            read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut displacement)?;
            instruction.displacement = displacement as i8 as i32;
        },
        2 => {
            read_bytes(stream, 2, &mut instruction.buffer, &mut instruction.length, &mut displacement)?;
            instruction.displacement = displacement as i32;
        }
        _ => {}
    };
    if swap_operands {
        mem::swap(&mut first_operand, &mut second_operand);
    }
    instruction.first_operand = Operand::from(first_operand);
    instruction.second_operand = Operand::from(second_operand);
    Ok(())
}

pub fn decode_instruction_part<R>(stream: &mut R, instruction: &mut Instruction) -> io::Result<()> where R: Read {
    let opcode = {
        let mut opcode: u32 = 0;
        read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut opcode)?;
        opcode as u8
    };
    let swap_operands: bool = opcode&2 == 0;
    instruction.data_width = if opcode&1 == 0 { 8 } else { 16 };
    instruction.first_operand = Operand::None;
    instruction.second_operand = Operand::None;
    instruction.immediate = 0xFFFFFFFF;
    instruction.displacement = 0;
    match opcode {
        0x88..=0x8C | 0x8E => { // MOV
            instruction.opcode = Opcode::MOV;
            if opcode > 0x8B {
                instruction.data_width = 16;
            }
            decode_mod_rm(stream, instruction, false, swap_operands)?;
            match opcode {
                0x8C => {
                    instruction.second_operand = Operand::from(instruction.second_operand as u8+Operand::ES as u8);
                },
                0x8E => {
                    instruction.first_operand = Operand::from(instruction.first_operand as u8+Operand::ES as u8);
                },
                _ => {}
            }
        },
        0x8D => { // LEA
            instruction.opcode = Opcode::LEA;
            decode_mod_rm(stream, instruction, false, false)?;
            instruction.displacement = instruction.displacement as i16 as i32;
        },
        0xC4 | 0xC5 => { // LES LDS
            instruction.opcode = Opcode::from(opcode);
            instruction.data_width = 16;
            decode_mod_rm(stream, instruction, false, false)?;
            instruction.displacement = instruction.displacement as i16 as i32;
        },
        0x8F => { // POP
            instruction.opcode = Opcode::POP;
            decode_mod_rm(stream, instruction, false, swap_operands)?;
            instruction.first_operand = instruction.second_operand;
            instruction.second_operand = Operand::None;
        },
        0x84 | 0x85 | 0x86 | 0x87 => { // TEST XCHG
            instruction.opcode = Opcode::from(opcode&0xFE);
            decode_mod_rm(stream, instruction, false, swap_operands)?;
        },
        0xD0..=0xD3 => { // GRP2 : ROL/0 ROR/1 RCL/2 RCR/3 SHL/4 SHR/5 SAL/6 SAR/7
            decode_mod_rm(stream, instruction, true, true)?;
            instruction.opcode = Opcode::from(0xB8+instruction.second_operand as u8);
            match opcode {
                0xD0 | 0xD1 => {
                    instruction.second_operand = Operand::None;
                    instruction.immediate = 1;
                },
                0xD2 | 0xD3 => {
                    instruction.second_operand = Operand::CL;
                },
                _ => unreachable!()
            }
        },
        0xFE => { // GRP4 : INC/0 DEC/1
            decode_mod_rm(stream, instruction, true, true)?;
            instruction.opcode = match instruction.second_operand as u8 {
                0 => Opcode::INC,
                1 => Opcode::DEC,
                _ => Opcode::BAD
            };
            instruction.second_operand = Operand::None;
        },
        0xFF => { // GRP5 : INC/0 DEC/1 CALL/2 LCALL/3 JMP/4 LJMP/5 PUSH/6
            decode_mod_rm(stream, instruction, true, true)?;
            let sub_type = instruction.second_operand as u8;
            instruction.opcode = match sub_type {
                0 => Opcode::INC,
                1 => Opcode::DEC,
                2 => Opcode::CALL,
                3 => Opcode::LCALL,
                4 => Opcode::JMP,
                5 => Opcode::LJMP,
                6 => Opcode::PUSH,
                _ => Opcode::BAD
            };
            if sub_type >= 2 && sub_type <= 5 {
                instruction.displacement = instruction.displacement as i16 as i32;
            }
            instruction.second_operand = Operand::None;
        },
        0x80..=0x83 => { // GRP1 : ADD OR ADC SBB AND SUB XOR CMP
            decode_mod_rm(stream, instruction, true, true)?;
            instruction.opcode = Opcode::from(instruction.second_operand as u8*8);
            instruction.second_operand = Operand::None;
            read_bytes(stream, if opcode != 0x81 { 1 } else { 2 }, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
            if opcode == 0x83 {
                instruction.immediate = instruction.immediate as i8 as u32;
            }
        },
        0xF6 | 0xF7 => { // GRP3 : TEST/0 NOT/2 NEG/3 MUL/4 IMUL/5 DIV/6 IDIV/7
            decode_mod_rm(stream, instruction, true, true)?;
            let sub_type = instruction.second_operand as u8;
            instruction.opcode = match sub_type {
                0 => {
                    read_bytes(stream, if opcode == 0xF6 { 1 } else { 2 }, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
                    Opcode::TEST
                },
                1 => Opcode::BAD,
                _ => Opcode::from(0xE8+sub_type) // NOT/2 NEG/3 MUL/4 IMUL/5 DIV/6 IDIV/7
            };
            instruction.second_operand = Operand::None;
        },
        0x00..=0x05 | 0x08..=0x0D | 0x10..=0x15 | 0x18..=0x1D | 0x20..=0x25 | 0x28..=0x2D | 0x30..=0x35 | 0x38..=0x3D => { // ADD OR ADC SBB AND SUB XOR CMP
            instruction.opcode = Opcode::from(opcode&0xF8);
            match (opcode)&7 {
                0x04 => {
                    instruction.first_operand = Operand::AL;
                    read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
                },
                0x05 => {
                    instruction.first_operand = Operand::AX;
                    read_bytes(stream, 2, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
                },
                _ => {
                    decode_mod_rm(stream, instruction, false, swap_operands)?;
                }
            }
        },
        0xC6 | 0xC7 => { // MOV
            instruction.opcode = Opcode::MOV;
            decode_mod_rm(stream, instruction, false, swap_operands)?;
            instruction.first_operand = instruction.second_operand;
            instruction.second_operand = Operand::None;
            read_bytes(stream, if opcode == 0xC6 { 1 } else { 2 }, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0x70..=0x7F | 0xE0..=0xE3 | 0xEB => { // Jcc LOOPcc JCXZ JMP
            instruction.opcode = if opcode == 0xEB { Opcode::JMP } else { Opcode::from(opcode) };
            instruction.data_width = 8;
            read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
            instruction.immediate = ((instruction.immediate as i8 as i32)+(instruction.length as i32)+(instruction.position as i32)) as u32;
        },
        0xE8 | 0xE9 => { // CALL JMP
            instruction.opcode = Opcode::from(opcode);
            instruction.data_width = 16;
            read_bytes(stream, 2, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
            instruction.immediate = ((instruction.immediate as i16 as i32)+(instruction.length as i32)+(instruction.position as i32)) as u32;
        },
        0xA0..=0xA3 => { // MOV
            instruction.opcode = Opcode::MOV;
            if opcode&2 == 0 {
                instruction.first_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
                instruction.second_operand = Operand::Displacement;
            } else {
                instruction.first_operand = Operand::Displacement;
                instruction.second_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
            }
            let mut displacement: u32 = 0;
            read_bytes(stream, 2, &mut instruction.buffer, &mut instruction.length, &mut displacement)?;
            instruction.displacement = displacement as i16 as i32;
        },
        0xA8 | 0xA9 => { // TEST
            instruction.opcode = Opcode::TEST;
            instruction.first_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
            read_bytes(stream, instruction.data_width/8, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0xB0..=0xBF => { // MOV
            instruction.opcode = Opcode::MOV;
            if opcode < 0xB8 {
                instruction.data_width = 8;
                instruction.first_operand = Operand::from((opcode&7)+(Operand::AL as u8));
            } else {
                instruction.data_width = 16;
                instruction.first_operand = Operand::from(opcode&7);
            }
            read_bytes(stream, instruction.data_width/8, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0x9A | 0xEA => { // LCALL LJMP
            instruction.opcode = match opcode {
                0x9A => Opcode::LCALL,
                0xEA => Opcode::LJMP,
                _ => unreachable!()
            };
            instruction.data_width = 32;
            read_bytes(stream, 4, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0xC2 | 0xCA => { // RET RETF
            instruction.opcode = match opcode {
                0xC2 => Opcode::RET,
                0xCA => Opcode::RETF,
                _ => unreachable!()
            };
            instruction.data_width = 16;
            read_bytes(stream, 2, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0xCD | 0xD4 | 0xD5 => { // INT AAM AAD
            instruction.opcode = Opcode::from(opcode);
            instruction.data_width = 8;
            read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0xE4 | 0xE5 => { // IN
            instruction.opcode = Opcode::from(opcode&0xF6);
            instruction.first_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
            instruction.data_width = 8;
            read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0xE6 | 0xE7 => { // OUT
            instruction.opcode = Opcode::from(opcode&0xF6);
            instruction.second_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
            instruction.data_width = 8;
            read_bytes(stream, 1, &mut instruction.buffer, &mut instruction.length, &mut instruction.immediate)?;
        },
        0xEC | 0xED => { // IN
            instruction.opcode = Opcode::from(opcode&0xF6);
            instruction.first_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
            instruction.second_operand = Operand::DX;
        },
        0xEE | 0xEF => { // OUT
            instruction.opcode = Opcode::from(opcode&0xF6);
            instruction.first_operand = Operand::DX;
            instruction.second_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
        },
        0x06 | 0x0E | 0x16 | 0x1E | 0x07 | 0x0F | 0x17 | 0x1F => { // PUSH POP
            instruction.opcode = if instruction.data_width == 8 { Opcode::PUSH } else { Opcode::POP };
            instruction.data_width = 16;
            instruction.first_operand = match opcode&0xFE {
                0x06 | 0x07 => Operand::ES,
                0x0E | 0x0F => Operand::CS,
                0x16 | 0x17 => Operand::SS,
                0x1E | 0x1F => Operand::DS,
                _ => unreachable!()
            };
        },
        0x40..=0x5F => { // INC DEC PUSH POP
            instruction.opcode = Opcode::from(opcode&0xF8);
            instruction.data_width = 16;
            instruction.first_operand = Operand::from(opcode&7);
        },
        0x91..=0x97 => { // XCHG
            instruction.opcode = Opcode::XCHG;
            instruction.data_width = 16;
            instruction.first_operand = Operand::from(opcode&7);
            instruction.second_operand = Operand::AX;
        },
        0xCC => { // INT
            instruction.opcode = Opcode::INT;
            instruction.immediate = 3;
        },
        0xA4..=0xA7 => { // MOVSB MOVSW CMPSB CMPSW
            instruction.opcode = Opcode::from(opcode);
            instruction.first_operand = Operand::DisplacementDI;
            instruction.second_operand = Operand::DisplacementSI;
        },
        0xAA | 0xAB | 0xAE | 0xAF => { // STOSB STOSW SCASB SCASW
            instruction.opcode = Opcode::from(opcode);
            instruction.first_operand = Operand::DisplacementDI;
            instruction.second_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
        },
        0xAC | 0xAD => { // LODSB LODSW
            instruction.opcode = Opcode::from(opcode);
            instruction.first_operand = if instruction.data_width == 8 { Operand::AL } else { Operand::AX };
            instruction.second_operand = Operand::DisplacementSI;
        },
        0x26 | 0x2E | 0x36 | 0x3E => { // ES: CS: SS: DS:
            instruction.opcode = Opcode::from(opcode);
            instruction.segment_override = Operand::from((Operand::ES as u8)+(opcode-0x26)/8);
            decode_instruction_part(stream, instruction)?;
        },
        0xF2 | 0xF3 => { // REPZ/REPNE REPZ/REPE
            instruction.prefix = Opcode::from(opcode);
            decode_instruction_part(stream, instruction)?;
            match instruction.opcode {
                Opcode::MOVSB | Opcode::MOVSW | Opcode::STOSB | Opcode::STOSW | Opcode::LODSB | Opcode::LODSW => { instruction.prefix = Opcode::REP },
                _ => {}
            }
        },
        0x27 | 0x2F | 0x37 | 0x3F | 0x90 | 0x98 | 0x99 | 0x9B | 0x9C | 0x9D | 0x9E | 0x9F | 0xC3 | 0xCB | 0xCE | 0xCF | 0xD7 | 0xF0 | 0xF4 | 0xF5 | 0xF8 | 0xF9 | 0xFA | 0xFB | 0xFC | 0xFD => {
            // DAA DAS AAA AAS NOP CBW CWD WAIT PUSHF POPF SAHF LAHF RET LEAVE RETF INTO IRET XLAT LOCK HLT CMC CLC STC CLI STI CLD STD
            instruction.opcode = Opcode::from(opcode);
        },
        0x60..=0x6F | 0xC0 | 0xC1 | 0xC8 | 0xC9 | 0xD8..=0xDF | 0xF1 | 0xD6 => { // BAD
            instruction.opcode = Opcode::BAD;
        }
    }
    Ok(())
}

pub fn decode_instruction<R>(stream: &mut R, instruction: &mut Instruction) -> io::Result<()> where R: Read {
    instruction.length = 0;
    instruction.prefix = Opcode::BAD;
    instruction.segment_override = Operand::None;
    decode_instruction_part(stream, instruction)
}
