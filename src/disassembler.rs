use std::fmt;

use crate::machinecode::{Instruction, Opcode};

static OPERAND_NAMES: &'static [&str] = &[
    "AX", "CX", "DX", "BX",
    "SP", "BP", "SI", "DI",
    "ES", "CS", "SS", "DS",
    "AL", "CL", "DL", "BL",
    "AH", "CH", "DH", "BH",
    "BX+SI", "BX+DI", "BP+SI", "BP+DI",
    "SI", "DI", "BP", "BX",
    "", ""
];

impl Instruction {
    fn fmt_operand(&self, formatter: &mut fmt::Formatter, is_string_operation: bool, operand: crate::machinecode::Operand) -> fmt::Result {
        if operand as u8 >= crate::machinecode::Operand::DisplacementBXSI as u8 {
            formatter.write_str(match self.data_width {
                8 => "byte ptr ",
                16 => "word ptr ",
                32 => "dword ptr ",
                _ => unreachable!()
            })?;
            let segment_override = if is_string_operation && operand == crate::machinecode::Operand::DisplacementDI { crate::machinecode::Operand::ES } else { self.segment_override };
            if segment_override != crate::machinecode::Operand::None {
                formatter.write_str(OPERAND_NAMES[segment_override as usize])?;
                formatter.write_str(":")?;
            }
            formatter.write_str("[")?;
            formatter.write_str(OPERAND_NAMES[operand as usize])?;
            if self.displacement != 0 {
                if self.displacement > 0 {
                    if operand != crate::machinecode::Operand::Displacement {
                        formatter.write_str("+")?;
                    }
                } else {
                    formatter.write_str("-")?;
                }
                formatter.write_fmt(format_args!("{:#06X}", self.displacement.abs()))?;
            }
            formatter.write_str("]")?;
        } else {
            formatter.write_str(OPERAND_NAMES[operand as usize])?;
        }
        Ok(())
    }
}

impl fmt::Display for Instruction {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        if self.prefix != crate::machinecode::Opcode::BAD {
            formatter.write_fmt(format_args!("{}", self.prefix.to_string()))?;
            formatter.write_str(" ")?;
        }
        formatter.write_fmt(format_args!("{}", self.opcode.to_string()))?;
        let is_string_operation = self.opcode as u8 >= 0xA4 && self.opcode as u8<= 0xAF;
        if self.first_operand != crate::machinecode::Operand::None {
            formatter.write_str(" ")?;
            self.fmt_operand(formatter, is_string_operation, self.first_operand)?;
        }
        if self.second_operand != crate::machinecode::Operand::None {
            formatter.write_str(", ")?;
            self.fmt_operand(formatter, is_string_operation, self.second_operand)?;
        }
        if self.immediate != 0xFFFFFFFF {
            if self.first_operand != crate::machinecode::Operand::None {
                formatter.write_str(",")?;
            }
            formatter.write_fmt(format_args!(" {:#X}", self.immediate))?;
        }
        Ok(())
    }
}

pub struct BytesAsHexDec<'a> {
    slice: &'a [u8]
}

impl<'a> BytesAsHexDec<'a> {
    pub fn from_instruction(instruction: &'a Instruction) -> Self {
        Self {
            slice: &instruction.buffer[0..instruction.length as usize]
        }
    }
}

impl fmt::Display for BytesAsHexDec<'_> {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..self.slice.len() {
            formatter.write_fmt(format_args!("{:02X}", self.slice[i]))?;
        }
        if let Some(width) = formatter.width() {
            let fill = formatter.fill();
            for _ in self.slice.len()*2..width {
                write!(formatter, "{}", fill)?;
            }
        }
        Ok(())
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Display)]
pub enum LabelType {
    Entry,
    Function,
    Other,
    None
}

pub fn get_reference(instruction: &Instruction, position: u32) -> (u32, LabelType) {
    if instruction.first_operand != crate::machinecode::Operand::None {
        return (0, LabelType::None);
    }
    (if instruction.data_width == 32 {
        crate::bus::BUS::physical_address((instruction.immediate>>16) as u16, instruction.immediate as u16) as u32
    } else {
        (position as i32+instruction.immediate as i32) as u32
    }, match instruction.opcode {
        Opcode::JO | Opcode::JNO | Opcode::JB | Opcode::JNB | Opcode::JE | Opcode::JNE | Opcode::JBE | Opcode::JNBE | Opcode::JS | Opcode::JNS | Opcode::JP | Opcode::JNP | Opcode::JL | Opcode::JNL | Opcode::JLE | Opcode::JNLE | Opcode::JCXZ |
        Opcode::JMP | Opcode::LJMP | Opcode::LOOPNZ | Opcode::LOOPZ | Opcode::LOOP => LabelType::Other,
        Opcode::CALL | Opcode::LCALL => LabelType::Function,
        _ => LabelType::None
    })
}

pub struct LabelBlock {
    label_type: LabelType,
    end_position: u32,
    positions: std::vec::Vec<u32>
}

impl LabelBlock {
    pub fn new(label_type: LabelType) -> Self {
        Self {
            label_type: label_type,
            end_position: 0,
            positions: std::vec::Vec::new()
        }
    }

    pub fn decode(&mut self, ram: &[u8], label_pool: &mut std::collections::BTreeMap<u32, LabelType>, mut position: u32) {
        let mut instruction: Instruction = unsafe { std::mem::zeroed() };
        let mut continue_decoding = true;
        while continue_decoding {
            let mut read_buffer = &ram[position as usize..position as usize+8];
            if !crate::machinecode::decode_instruction(&mut read_buffer, &mut instruction).is_ok() {
                panic!();
            }
            self.positions.push(position);
            continue_decoding = match instruction.opcode {
                Opcode::RET | Opcode::RETF | Opcode::IRET | Opcode::JMP | Opcode::LJMP => false,
                _ => true
            };
            let (reference, label_type) = get_reference(&instruction, position);
            if label_type != LabelType::None {
                label_pool.insert(reference, label_type);
            }
            position += instruction.length as u32;
        }
        self.end_position = position;
    }

    pub fn print(&self, ram: &[u8]) {
        print!("label_{:05X}:", self.positions[0]);
        if self.label_type == LabelType::Other {
            println!("");
        } else {
            println!(" ; {}", self.label_type);
        }
        let mut instruction: Instruction = unsafe { std::mem::zeroed() };
        for position in &self.positions {
            let mut read_buffer = &ram[*position as usize..*position as usize+8];
            if !crate::machinecode::decode_instruction(&mut read_buffer, &mut instruction).is_ok() {
                panic!();
            }
            let (reference, label_type) = get_reference(&instruction, *position);
            if label_type == LabelType::None {
                println!("\t{}", instruction);
            } else {
                println!("\t{} label_{:05X}", instruction.opcode.to_string(), reference);
            }
        }
    }
}

pub fn disassemble(ram: &[u8], entry_position: u32) {
    let mut blocks: std::collections::BTreeMap<u32, LabelBlock> = std::collections::BTreeMap::new();
    let mut label_pool: std::collections::BTreeMap<u32, LabelType> = std::collections::BTreeMap::new();
    label_pool.insert(entry_position, LabelType::Entry);
    while !label_pool.is_empty() {
        let label = label_pool.iter().next().unwrap();
        let label = (*label.0, *label.1);
        label_pool.remove(&label.0);
        if let Some((position, closest_block)) = blocks.range_mut(..label.0+1).next_back() {
            if *position == label.0 {
                if closest_block.label_type != label.1 {
                    eprintln!("Warning: Conflicting label types {} and {} at {:08X}", closest_block.label_type, label.1, label.0);
                }
                continue;
            } else if *position < label.0 && label.0 < closest_block.end_position {
                if let Ok(index) = closest_block.positions.binary_search(&label.0) {
                    let mut block = LabelBlock::new(label.1);
                    block.positions = closest_block.positions.drain(index..).collect();
                    block.end_position = closest_block.end_position;
                    closest_block.end_position = label.0;
                    blocks.insert(label.0, block);
                } else {
                    eprintln!("Warning: Jump inmid an instruction at {:08X}", label.0);
                }
                continue;
            }
        }
        let mut block = LabelBlock::new(label.1);
        block.decode(ram, &mut label_pool, label.0);
        blocks.insert(label.0, block);
    }
    for block in blocks.values() {
        block.print(ram);
    }
}
