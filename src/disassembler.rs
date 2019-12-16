use std::fmt;

const OPERAND_NAMES: [&str; 30] = [
    "AX", "CX", "DX", "BX",
    "SP", "BP", "SI", "DI",
    "ES", "CS", "SS", "DS",
    "AL", "CL", "DL", "BL",
    "AH", "CH", "DH", "BH",
    "BX+SI", "BX+DI", "BP+SI", "BP+DI",
    "SI", "DI", "BP", "BX",
    "", ""
];

impl crate::machinecode::Instruction {
    fn fmt_operand(&self, formatter: &mut fmt::Formatter, is_string_operation: bool, operand: crate::machinecode::Operand) -> fmt::Result {
        if operand as u8 >= crate::machinecode::Operand::DisplacementBXSI as u8 {
            formatter.write_str(match self.data_width {
                8 => "byte ",
                16 => "word ",
                32 => "dword ",
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

impl fmt::Display for crate::machinecode::Instruction {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        for i in 0..(self.length as usize) {
            formatter.write_fmt(format_args!("{:02X}", self.buffer[i]))?;
        }
        for _ in (self.length as usize)..(self.buffer.len()) {
            formatter.write_str("  ")?;
        }
        formatter.write_str("   ")?;
        if self.prefix != crate::machinecode::Opcode::BAD {
            formatter.write_str(" ")?;
            formatter.write_fmt(format_args!("{}", self.prefix.to_string()))?;
        }
        formatter.write_str(" ")?;
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
