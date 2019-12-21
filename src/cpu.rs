use crate::bit_utils::lsb_mask;
use crate::bit_utils::msb_mask;
use crate::machinecode::Opcode;
use crate::machinecode::Operand;

macro_rules! binary_arithmetic_operation {
    ($cpu:ident, $bus:ident, $update_dst:expr, $carry_adjust:expr, $($operation:tt)*) => {
        let (dst, first_value) = $cpu.get_operand($bus, $cpu.instruction.first_operand, $cpu.instruction.data_width);
        $cpu.first_value = first_value;
        $cpu.second_value = $cpu.get_operand($bus, $cpu.instruction.second_operand, $cpu.instruction.data_width).1.wrapping_add($carry_adjust);
        $cpu.result_value = ($cpu.first_value) $($operation)* ($cpu.second_value);
        if $update_dst {
            $bus.write_to_memory($cpu, dst, $cpu.instruction.data_width, $cpu.result_value);
        }
        $cpu.set_arithmetic_flags();
    };
}

macro_rules! unary_arithmetic_operation {
    ($cpu:ident, $bus:ident, $($operation:tt)*) => {
        let (dst, second_value) = $cpu.get_operand($bus, $cpu.instruction.first_operand, $cpu.instruction.data_width);
        $cpu.second_value = second_value;
        $cpu.result_value = ($cpu.first_value) $($operation)* ($cpu.second_value);
        $bus.write_to_memory($cpu, dst, $cpu.instruction.data_width, $cpu.result_value);
        $cpu.set_arithmetic_flags();
    };
}

macro_rules! bit_shift_operation {
    ($cpu:ident, $bus:ident, $T8:ty, $T16:ty, $operation:tt) => {
        let (dst, first_value) = $cpu.get_operand($bus, $cpu.instruction.first_operand, $cpu.instruction.data_width);
        $cpu.first_value = first_value;
        $cpu.second_value = $cpu.get_operand($bus, $cpu.instruction.second_operand, 8).1;
        if $cpu.instruction.data_width == 16 {
            $cpu.result_value = ($cpu.first_value as $T16).$operation($cpu.second_value) as u32;
        } else {
            $cpu.result_value = ($cpu.first_value as $T8).$operation($cpu.second_value) as u32;
        }
        $bus.write_to_memory($cpu, dst, $cpu.instruction.data_width, $cpu.result_value);
        $cpu.set_arithmetic_flags();
    };
}

macro_rules! rotate_with_carry_operation {
    ($cpu:ident, $bus:ident, $carry:ident, $($operation:tt)*) => {
        let (dst, first_value) = $cpu.get_operand($bus, $cpu.instruction.first_operand, $cpu.instruction.data_width);
        $cpu.first_value = first_value;
        $cpu.second_value = $cpu.get_operand($bus, $cpu.instruction.second_operand, 8).1;
        $cpu.result_value = (($cpu.get_flag(Flag::Carry) as u32)<<$cpu.instruction.data_width)|$cpu.first_value;
        $cpu.result_value = $($operation)*;
        $bus.write_to_memory($cpu, dst, $cpu.instruction.data_width, $cpu.result_value);
        let $carry = (($cpu.result_value>>$cpu.instruction.data_width)&1) as u16;
        $cpu.reset_flag(Flag::Carry);
        $cpu.reset_flag(Flag::Overflow);
        $cpu.set_flag(Flag::Carry, $carry);
    };
}

macro_rules! multiplication_operation {
    ($cpu:ident, $bus:ident, $T8:ty, $T16:ty, $T32:ty) => {
        let value: $T32;
        let flag: bool;
        let second_value = $cpu.get_operand($bus, $cpu.instruction.first_operand, $cpu.instruction.data_width).1;
        if $cpu.instruction.data_width == 16 {
            value = ($cpu.get_register(Operand::AX) as $T16 as $T32)*(second_value as $T16 as $T32);
            $cpu.set_register(Operand::DX, ((value>>16)&0xFFFF) as u16);
            $cpu.set_register(Operand::AX, (value&0xFFFF) as u16);
            flag = if std::any::TypeId::of::<$T32>() == std::any::TypeId::of::<u32>() {
                (value>>$cpu.instruction.data_width)&1 == 1
            } else {
                (value as i16 as i32) != (value as i32)
            };
        } else {
            value = ($cpu.get_register(Operand::AL) as $T8 as $T32)*(second_value as $T8 as $T32);
            $cpu.set_register(Operand::AX, value as u16);
            flag = if std::any::TypeId::of::<$T32>() == std::any::TypeId::of::<u32>() {
                (value>>$cpu.instruction.data_width)&1 == 1
            } else {
                (value as i8 as i32) != (value as i32)
            };
        }
        $cpu.reset_flag(Flag::Carry);
        $cpu.reset_flag(Flag::Overflow);
        $cpu.set_flag(Flag::Carry, flag as u16);
        $cpu.set_flag(Flag::Overflow, flag as u16);
    };
}

macro_rules! division_operation {
    ($cpu:ident, $bus:ident, $T8:ty, $T16:ty, $T32:ty) => {
        let divisor = $cpu.get_operand($bus, $cpu.instruction.first_operand, $cpu.instruction.data_width).1;
        if divisor == 0 {
            return $cpu.software_interrupt($bus, 0);
        }
        if $cpu.instruction.data_width == 16 {
            let dividend = (($cpu.get_register(Operand::DX) as $T32)<<16)|($cpu.get_register(Operand::AX) as $T32);
            let quotient = dividend/(divisor as $T16 as $T32);
            if std::any::TypeId::of::<$T32>() == std::any::TypeId::of::<u32>() {
                if quotient > 0xFFFF {
                    return $cpu.software_interrupt($bus, 0);
                }
            } else {
                if quotient > 0x7FFF || -(quotient as i32) > 0x8000 {
                    return $cpu.software_interrupt($bus, 0);
                }
            }
            let remainder = dividend%(divisor as $T16 as $T32);
            $cpu.set_register(Operand::DX, remainder as u16);
            $cpu.set_register(Operand::AX, quotient as u16);
        } else {
            let dividend = $cpu.get_register(Operand::AX) as $T16 as $T32;
            let quotient = dividend/(divisor as $T8 as $T32);
            if std::any::TypeId::of::<$T32>() == std::any::TypeId::of::<u32>() {
                if quotient > 0xFF {
                    return $cpu.software_interrupt($bus, 0);
                }
            } else {
                if quotient > 0x7F || -(quotient as i32) > 0x80 {
                    return $cpu.software_interrupt($bus, 0);
                }
            }
            let remainder = dividend%(divisor as $T8 as $T32);
            $cpu.set_register(Operand::AX, ((remainder as u16)<<8)|(quotient as u16));
        }
    };
}

macro_rules! decimal_adjust_operation {
    ($cpu:ident, $operation:tt) => {
        let auxiliary_carry = $cpu.get_flag(Flag::AuxiliaryCarry);
        let carry = $cpu.get_flag(Flag::Carry);
        let value = $cpu.get_register(Operand::AX);
        $cpu.reset_flag(Flag::AuxiliaryCarry);
        $cpu.reset_flag(Flag::Carry);
        if value&0x0F > 9 || auxiliary_carry {
            $cpu.set_register(Operand::AX, $cpu.get_register(Operand::AX).$operation(0x06)&0x0F);
            $cpu.set_flag(Flag::AuxiliaryCarry, 1);
        }
        if value&0xFF > 99 || carry {
            $cpu.set_register(Operand::AX, $cpu.get_register(Operand::AX).$operation(0x60)&0xFF);
            $cpu.set_flag(Flag::Carry, 1);
        }
    };
}

macro_rules! ascii_adjust_operation {
    ($cpu:ident, $operation:tt) => {
        let auxiliary_carry = $cpu.get_flag(Flag::AuxiliaryCarry);
        $cpu.reset_flag(Flag::AuxiliaryCarry);
        if $cpu.get_register(Operand::AX)&0x0F > 9 || auxiliary_carry {
            $cpu.set_register(Operand::AX, $cpu.get_register(Operand::AX).$operation(0x106)&0x0F);
            $cpu.set_flag(Flag::AuxiliaryCarry, 1);
        }
    };
}

macro_rules! string_operation {
    ($cpu:ident, $bus:ident, $dst:ident, $src:ident, $T:ty, $update_dst:expr, $update_src:expr, $($operation:tt)*) => {
        while $cpu.instruction.prefix == Opcode::BAD || $cpu.get_register(Operand::CX) > 0 {
            let $dst = $cpu.memory_operand($bus, false, Operand::ES, $cpu.get_register(Operand::DI));
            let $src = $cpu.memory_operand($bus, true, Operand::DS, $cpu.get_register(Operand::SI));
            $($operation)*;
            if !$cpu.get_flag(Flag::Direction) {
                $cpu.set_register(Operand::DI, $cpu.get_register(Operand::DI).wrapping_add($update_dst));
                $cpu.set_register(Operand::SI, $cpu.get_register(Operand::SI).wrapping_add($update_src));
            } else {
                $cpu.set_register(Operand::DI, $cpu.get_register(Operand::DI).wrapping_sub($update_dst));
                $cpu.set_register(Operand::SI, $cpu.get_register(Operand::SI).wrapping_sub($update_src));
            }
            if $cpu.instruction.prefix != Opcode::BAD {
                $cpu.set_register(Operand::CX, $cpu.get_register(Operand::CX).wrapping_sub(1));
            }
            if match $cpu.instruction.prefix {
                Opcode::REP => false,
                Opcode::REPNZ => $cpu.get_flag(Flag::Zero),
                Opcode::REPZ => !$cpu.get_flag(Flag::Zero),
                Opcode::BAD => true,
                _ => unreachable!()
            } {
                break;
            }
            $cpu.cycle_counter += 1;
        }
    };
}

#[derive(Copy, Clone, PartialEq)]
pub enum Flag {
    Carry = 0, // For unsigned arithmetics
    Parity = 2, // Popcount
    AuxiliaryCarry = 4, // For BCD arithmetics
    Zero = 6,
    Sign = 7,
    Trap = 8,
    Interrupt = 9,
    Direction = 10, // For string operations
    Overflow = 11 // For signed arithmetics
}

#[derive(Copy, Clone, PartialEq)]
pub enum ExecutionState {
    Running,
    Paused,
    DebuggerHandlesInterrupt(u8),
    WaitForInterrupt
}

pub struct CPU {
    registers: [u16; 12],
    flags: u16,
    pub instruction: crate::machinecode::Instruction,
    pub interrupt_breakpoints: [bool; 0x100],
    pub execution_state: ExecutionState,
    pub cycle_counter: u64,
    pub first_value: u32,
    pub second_value: u32,
    pub result_value: u32,
    pub instruction_profile: std::collections::HashMap::<crate::machinecode::Instruction, usize>,
    pub instruction_profile_recording: bool
}

impl CPU {
    pub fn new() -> Self {
        Self {
            registers: unsafe { std::mem::zeroed() },
            flags: (1<<(Flag::Interrupt as usize)),
            instruction: unsafe { std::mem::zeroed() },
            interrupt_breakpoints: unsafe { std::mem::zeroed() },
            execution_state: ExecutionState::Running,
            cycle_counter: 0,
            first_value: 0,
            second_value: 0,
            result_value: 0,
            instruction_profile: std::collections::HashMap::new(),
            instruction_profile_recording: false
        }
    }

    pub fn get_flag(&self, flag: Flag) -> bool {
        self.flags&(1<<(flag as u32)) != 0
    }

    pub fn reset_flag(&mut self, flag: Flag) {
        self.flags &= !(1<<(flag as u32));
    }

    pub fn set_flag(&mut self, flag: Flag, value: u16) {
        self.flags |= (value as u16)<<(flag as u32);
    }

    fn set_arithmetic_flags(&mut self) {
        self.reset_flag(Flag::Carry);
        self.reset_flag(Flag::AuxiliaryCarry);
        self.reset_flag(Flag::Overflow);
        self.reset_flag(Flag::Parity);
        self.reset_flag(Flag::Zero);
        self.reset_flag(Flag::Sign);
        let truncated_result = self.result_value&lsb_mask(self.instruction.data_width as u32);
        self.set_flag(Flag::Parity, (truncated_result.count_ones()&1 == 0) as u16);
        self.set_flag(Flag::Zero, (truncated_result == 0) as u16);
        self.set_flag(Flag::Sign, ((self.result_value>>(self.instruction.data_width-1))&1) as u16);
    }

    fn set_addition_flags(&mut self) {
        self.set_flag(Flag::AuxiliaryCarry, ((self.first_value&0xF)+(self.second_value&0xF) > 0xF) as u16);
        self.set_flag(Flag::Carry, ((self.result_value>>self.instruction.data_width)&1) as u16);
        self.set_flag(Flag::Overflow, (
            ((self.first_value^self.second_value)>>(self.instruction.data_width-1))&1 == 0 &&
            ((self.first_value^self.result_value)>>(self.instruction.data_width-1))&1 == 1
        ) as u16);
    }

    fn set_substraction_flags(&mut self) {
        self.set_flag(Flag::AuxiliaryCarry, (((self.first_value&0xF) as i32-(self.second_value&0xF) as i32) < 0x0) as u16);
        self.set_flag(Flag::Carry, ((self.result_value>>self.instruction.data_width)&1) as u16);
        self.set_flag(Flag::Overflow, (
            ((self.first_value^self.second_value)>>(self.instruction.data_width-1))&1 == 1 &&
            ((self.first_value^self.result_value)>>(self.instruction.data_width-1))&1 == 1
        ) as u16);
    }

    pub fn get_register(&self, operand: Operand) -> u16 {
        match operand as usize {
            0..=11 => {
                self.registers[operand as usize]
            },
            12..=15 => {
                self.registers[operand as usize-Operand::AL as usize]&0xFF
            },
            16..=19 => {
                self.registers[operand as usize-Operand::AH as usize]>>8
            },
            _ => unreachable!()
        }
    }

    pub fn set_register(&mut self, operand: Operand, value: u16) {
        match operand as usize {
            0..=11 => {
                self.registers[operand as usize] = value;
            },
            12..=15 => {
                crate::bit_utils::write_low_byte_of_u16(&mut self.registers[operand as usize-Operand::AL as usize], value as u8);
            },
            16..=19 => {
                crate::bit_utils::write_high_byte_of_u16(&mut self.registers[operand as usize-Operand::AH as usize], value as u8);
            },
            _ => unreachable!()
        }
    }

    fn calculate_displacement(&self, operand: Operand) -> (Operand, u16) {
        let (segment_register, offset) = match operand {
            Operand::DisplacementBXSI => (Operand::DS, self.get_register(Operand::BX).wrapping_add(self.get_register(Operand::SI))),
            Operand::DisplacementBXDI => (Operand::DS, self.get_register(Operand::BX).wrapping_add(self.get_register(Operand::DI))),
            Operand::DisplacementBPSI => (Operand::SS, self.get_register(Operand::BP).wrapping_add(self.get_register(Operand::SI))),
            Operand::DisplacementBPDI => (Operand::DS, self.get_register(Operand::BP).wrapping_add(self.get_register(Operand::DI))),
            Operand::DisplacementSI => (Operand::DS, self.get_register(Operand::SI)),
            Operand::DisplacementDI => (Operand::DS, self.get_register(Operand::DI)),
            Operand::DisplacementBP => (Operand::SS, self.get_register(Operand::BP)),
            Operand::DisplacementBX => (Operand::DS, self.get_register(Operand::BX)),
            Operand::Displacement => (Operand::DS, 0),
            _ => unreachable!()
        };
        (segment_register, (self.instruction.displacement as u16).wrapping_add(offset))
    }

    pub fn memory_operand(&mut self, bus: &mut crate::bus::BUS, enable_segment_override: bool, mut segment_register: Operand, offset: u16) -> *mut u8 {
        if enable_segment_override && self.instruction.segment_override != Operand::None {
            segment_register = self.instruction.segment_override;
        }
        bus.get_memory(self, crate::bus::BUS::physical_address(self.registers[segment_register as usize], offset))
    }

    fn register_or_memory_operand(&mut self, bus: &mut crate::bus::BUS, operand: Operand) -> *mut u8 {
        match operand as usize {
            0..=11 => {
                &mut self.registers[operand as usize] as *mut u16 as *mut u8
            },
            12..=15 => {
                (&mut self.registers[operand as usize-Operand::AL as usize] as *mut u16 as *mut u8)
            },
            16..=19 => {
                unsafe { (&mut self.registers[operand as usize-Operand::AH as usize] as *mut u16 as *mut u8).offset(1) }
            },
            _ => {
                let (segment_register, offset) = self.calculate_displacement(operand);
                self.memory_operand(bus, true, segment_register, offset)
            }
        }
    }

    fn get_operand(&mut self, bus: &mut crate::bus::BUS, operand: Operand, data_width: u8) -> (*mut u8, u32) {
        if operand == Operand::None {
            (std::ptr::null_mut(), self.instruction.immediate&crate::bit_utils::lsb_mask(data_width as u32))
        } else {
            let src = self.register_or_memory_operand(bus, operand);
            (src, bus.read_from_memory(self, src, data_width))
        }
    }

    fn push_onto_stack(&mut self, bus: &mut crate::bus::BUS, value: u16) {
        self.set_register(Operand::SP, self.get_register(Operand::SP).wrapping_sub(2));
        let dst = self.memory_operand(bus, false, Operand::SS, self.get_register(Operand::SP));
        bus.write_to_memory(self, dst, 16, value as u32);
    }

    fn pop_from_stack(&mut self, bus: &mut crate::bus::BUS) -> u16 {
        let src = self.memory_operand(bus, false, Operand::SS, self.get_register(Operand::SP));
        self.set_register(Operand::SP, self.get_register(Operand::SP).wrapping_add(2));
        bus.read_from_memory(self, src, 16) as u16
    }

    fn long_jump(&mut self, bus: &mut crate::bus::BUS, address: u32) {
        if crate::bus::BUS::physical_address((address>>16) as u16, 0) < 0xF0000 {
            self.push_onto_stack(bus, self.get_register(Operand::CS));
            self.push_onto_stack(bus, self.instruction.position as u16);
            self.set_register(Operand::CS, (address>>16) as u16);
            self.instruction.position = address as u16;
        } else {
            self.flags = self.pop_from_stack(bus);
            // TODO
        }
    }

    fn invoke_interrupt_handler(&mut self, bus: &mut crate::bus::BUS, interrupt: u8) {
        self.push_onto_stack(bus, self.flags);
        self.reset_flag(Flag::Interrupt);
        self.reset_flag(Flag::Trap);
        let src = bus.get_memory(self, interrupt as usize*4);
        let address = bus.read_from_memory(self, src, 32);
        self.long_jump(bus, address);
    }

    fn software_interrupt(&mut self, bus: &mut crate::bus::BUS, interrupt: u8) {
        if self.interrupt_breakpoints[interrupt as usize] {
            self.execution_state = ExecutionState::DebuggerHandlesInterrupt(interrupt);
        } else if !bus.handle_interrupt(self, interrupt) {
            self.invoke_interrupt_handler(bus, interrupt);
        }
    }

    pub fn execute_instruction(&mut self, bus: &mut crate::bus::BUS) {
        bus.tick(self);
        if self.get_flag(Flag::Interrupt) {
            // TODO
        }
        if self.get_flag(Flag::Trap) {
            self.software_interrupt(bus, 1);
        }
        let mut read_buffer = unsafe { std::slice::from_raw_parts(bus.get_memory(self, crate::bus::BUS::physical_address(self.get_register(Operand::CS), self.instruction.position)), 8) };
        if !crate::machinecode::decode_instruction(&mut read_buffer, &mut self.instruction).is_ok() {
            panic!();
        }
        if self.instruction_profile_recording {
            let mut instruction_profile_entry = self.instruction;
            instruction_profile_entry.position = 0;
            instruction_profile_entry.buffer = unsafe { std::mem::zeroed() };
            instruction_profile_entry.immediate = 0xFFFFFFFF;
            instruction_profile_entry.displacement = 0;
            match self.instruction_profile.get_mut(&instruction_profile_entry) {
                Some(counter) => { *counter += 1; },
                None => { self.instruction_profile.insert(instruction_profile_entry, 1); }
            }
        }
        self.instruction.position = self.instruction.position+self.instruction.length as u16;
        self.cycle_counter += 1;
        match self.instruction.opcode {
            Opcode::ADD => {
                binary_arithmetic_operation!(self, bus, true, 0, .wrapping_add);
                self.set_addition_flags();
            },
            Opcode::OR => {
                binary_arithmetic_operation!(self, bus, true, 0, |);
            },
            Opcode::ADC => {
                binary_arithmetic_operation!(self, bus, true, self.get_flag(Flag::Carry) as u32, .wrapping_add);
                self.set_addition_flags();
            },
            Opcode::SBB => {
                binary_arithmetic_operation!(self, bus, true, self.get_flag(Flag::Carry) as u32, .wrapping_sub);
                self.set_substraction_flags();
            },
            Opcode::AND => {
                binary_arithmetic_operation!(self, bus, true, 0, &);
            },
            Opcode::SUB => {
                binary_arithmetic_operation!(self, bus, true, 0, .wrapping_sub);
                self.set_substraction_flags();
            },
            Opcode::XOR => {
                binary_arithmetic_operation!(self, bus, true, 0, ^);
            },
            Opcode::CMP => {
                binary_arithmetic_operation!(self, bus, false, 0, .wrapping_sub);
                self.set_substraction_flags();
            },
            Opcode::TEST => {
                binary_arithmetic_operation!(self, bus, false, 0, &);
            },
            Opcode::ROL => {
                bit_shift_operation!(self, bus, u8, u16, rotate_left);
                let carry = (self.first_value.wrapping_shr(self.instruction.data_width as u32-self.second_value)&1) as u16;
                self.set_flag(Flag::Carry, carry);
                self.set_flag(Flag::Overflow, ((self.result_value>>(self.instruction.data_width-1)) as u16)^carry);
            },
            Opcode::ROR => {
                bit_shift_operation!(self, bus, u8, u16, rotate_right);
                self.set_flag(Flag::Carry, (self.first_value.wrapping_shr(self.second_value-1)&1) as u16);
                self.set_flag(Flag::Overflow, ((self.result_value>>(self.instruction.data_width-1))^(self.result_value>>(self.instruction.data_width-2))) as u16);
            },
            Opcode::RCL => {
                rotate_with_carry_operation!(self, bus, carry, ((self.result_value&lsb_mask(1+self.instruction.data_width as u32-self.second_value))<<self.second_value)|((self.result_value&msb_mask(31-self.instruction.data_width as u32+self.second_value))>>(1+self.instruction.data_width as u32-self.second_value)));
                self.set_flag(Flag::Overflow, ((self.result_value>>(self.instruction.data_width-1)) as u16)^carry);
            },
            Opcode::RCR => {
                rotate_with_carry_operation!(self, bus, carry, ((self.result_value&lsb_mask(self.second_value))<<(1+self.instruction.data_width as u32-self.second_value))|((self.result_value&msb_mask(32-self.second_value))>>self.second_value));
                self.set_flag(Flag::Overflow, ((self.result_value>>(self.instruction.data_width-1))^(self.result_value>>(self.instruction.data_width-2))) as u16);
            },
            Opcode::SHL | Opcode::SAL => {
                bit_shift_operation!(self, bus, u8, u16, wrapping_shl);
                let carry = (self.first_value.wrapping_shr(self.instruction.data_width as u32-self.second_value)&1) as u16;
                self.set_flag(Flag::Carry, carry);
                self.set_flag(Flag::Overflow, ((self.result_value>>(self.instruction.data_width-1)) as u16)^carry);
            },
            Opcode::SHR => {
                bit_shift_operation!(self, bus, u8, u16, wrapping_shr);
                if self.second_value > 0 {
                    self.set_flag(Flag::Carry, (self.first_value.wrapping_shr(self.second_value-1)&1) as u16);
                }
                self.set_flag(Flag::Overflow, (self.first_value>>(self.instruction.data_width-1)) as u16);
            },
            Opcode::SAR => {
                bit_shift_operation!(self, bus, i8, i16, wrapping_shr);
                if self.second_value > 0 {
                    self.set_flag(Flag::Carry, (self.first_value.wrapping_shr(self.second_value-1)&1) as u16);
                }
            },
            Opcode::NOT => {
                let (dst, value) = self.get_operand(bus, self.instruction.first_operand, self.instruction.data_width);
                bus.write_to_memory(self, dst, self.instruction.data_width, !value);
            },
            Opcode::NEG => {
                self.first_value = 0;
                unary_arithmetic_operation!(self, bus, .wrapping_sub);
                self.set_substraction_flags();
                let zero = self.get_flag(Flag::Zero);
                self.reset_flag(Flag::Carry);
                self.set_flag(Flag::Carry, !zero as u16);
            },
            Opcode::INC => {
                self.first_value = 1;
                unary_arithmetic_operation!(self, bus, .wrapping_add);
                self.set_addition_flags();
            },
            Opcode::DEC => {
                self.first_value = u32::max_value();
                unary_arithmetic_operation!(self, bus, .wrapping_add);
                self.set_addition_flags();
            },
            Opcode::MUL => {
                multiplication_operation!(self, bus, u8, u16, u32);
            },
            Opcode::IMUL => {
                multiplication_operation!(self, bus, i8, i16, i32);
            },
            Opcode::DIV => {
                division_operation!(self, bus, u8, u16, u32);
            },
            Opcode::IDIV => {
                division_operation!(self, bus, i8, i16, i32);
            },
            Opcode::DAA => {
                decimal_adjust_operation!(self, wrapping_add);
            },
            Opcode::DAS => {
                decimal_adjust_operation!(self, wrapping_sub);
            },
            Opcode::AAA => {
                ascii_adjust_operation!(self, wrapping_add);
            },
            Opcode::AAS => {
                ascii_adjust_operation!(self, wrapping_sub);
            },
            Opcode::AAM => {
                let divisor = self.instruction.immediate as u16;
                self.set_register(Operand::AX, (((self.get_register(Operand::AX)&0xFF)/divisor)<<8)|((self.get_register(Operand::AX)&0xFF)%divisor));
            },
            Opcode::AAD => {
                let factor = self.instruction.immediate as u16;
                self.set_register(Operand::AX, ((self.get_register(Operand::AX)&0xFF)+(self.get_register(Operand::AX)>>8)*factor)&0xFF);
            },
            Opcode::NOP => {},
            // Opcode::WAIT => {},
            // Opcode::LOCK => {},
            Opcode::HLT => {
                self.execution_state = ExecutionState::WaitForInterrupt;
            },
            Opcode::INT => {
                self.software_interrupt(bus, self.instruction.immediate as u8);
            },
            Opcode::INTO => {
                if self.get_flag(Flag::Overflow) {
                    self.software_interrupt(bus, 4);
                }
            },
            Opcode::POP => {
                let dst = self.register_or_memory_operand(bus, self.instruction.first_operand);
                let value = self.pop_from_stack(bus);
                bus.write_to_memory(self, dst, 16, value as u32);
            },
            Opcode::POPF => {
                self.flags = self.pop_from_stack(bus);
            },
            Opcode::PUSH => {
                let value = self.get_operand(bus, self.instruction.first_operand, 16).1;
                self.push_onto_stack(bus, value as u16);
            },
            Opcode::PUSHF => {
                self.push_onto_stack(bus, self.flags);
            },
            Opcode::RET => {
                self.instruction.position = self.pop_from_stack(bus);
                if self.instruction.immediate != 0xFFFFFFFF {
                    self.set_register(Operand::SP, self.get_register(Operand::SP).wrapping_add(self.instruction.immediate as u16));
                }
            },
            Opcode::RETF => {
                self.instruction.position = self.pop_from_stack(bus);
                let cs = self.pop_from_stack(bus);
                self.set_register(Operand::CS, cs);
                if self.instruction.immediate != 0xFFFFFFFF {
                    self.set_register(Operand::SP, self.get_register(Operand::SP).wrapping_add(self.instruction.immediate as u16));
                }
            },
            Opcode::IRET => {
                self.instruction.position = self.pop_from_stack(bus);
                let cs = self.pop_from_stack(bus);
                self.set_register(Operand::CS, cs);
                self.flags = self.pop_from_stack(bus);
            },
            Opcode::LCALL => {
                let address = self.get_operand(bus, self.instruction.first_operand, 32).1;
                self.long_jump(bus, address);
            },
            Opcode::CALL => {
                self.push_onto_stack(bus, self.instruction.position as u16);
                self.instruction.position = self.get_operand(bus, self.instruction.first_operand, 16).1 as u16;
            },
            Opcode::LJMP => {
                let address = self.get_operand(bus, self.instruction.first_operand, 32).1;
                self.set_register(Operand::CS, (address>>16) as u16);
                self.instruction.position = address as u16;
            },
            Opcode::JMP => {
                self.instruction.position = self.get_operand(bus, self.instruction.first_operand, 16).1 as u16;
            },
            Opcode::JO | Opcode::JNO | Opcode::JB | Opcode::JNB | Opcode::JE | Opcode::JNE | Opcode::JBE | Opcode::JNBE | Opcode::JS | Opcode::JNS | Opcode::JP | Opcode::JNP | Opcode::JL | Opcode::JNL | Opcode::JLE | Opcode::JNLE | Opcode::JCXZ => {
                let condition = match self.instruction.opcode {
                    Opcode::JO => self.get_flag(Flag::Overflow),
                    Opcode::JNO => !self.get_flag(Flag::Overflow),
                    Opcode::JB => self.get_flag(Flag::Carry),
                    Opcode::JNB => !self.get_flag(Flag::Carry),
                    Opcode::JE => self.get_flag(Flag::Zero),
                    Opcode::JNE => !self.get_flag(Flag::Zero),
                    Opcode::JBE => self.get_flag(Flag::Carry) || self.get_flag(Flag::Zero),
                    Opcode::JNBE => !(self.get_flag(Flag::Carry) || self.get_flag(Flag::Zero)),
                    Opcode::JS => self.get_flag(Flag::Sign),
                    Opcode::JNS => !self.get_flag(Flag::Sign),
                    Opcode::JP => self.get_flag(Flag::Parity),
                    Opcode::JNP => !self.get_flag(Flag::Parity),
                    Opcode::JL => self.get_flag(Flag::Sign) != self.get_flag(Flag::Overflow),
                    Opcode::JNL => self.get_flag(Flag::Sign) == self.get_flag(Flag::Overflow),
                    Opcode::JLE => self.get_flag(Flag::Sign) != self.get_flag(Flag::Overflow) || self.get_flag(Flag::Zero),
                    Opcode::JNLE => self.get_flag(Flag::Sign) == self.get_flag(Flag::Overflow) && !self.get_flag(Flag::Zero),
                    Opcode::JCXZ => self.get_register(Operand::CX) == 0,
                    _ => unreachable!()
                };
                if condition {
                    self.instruction.position = self.instruction.immediate as u16;
                }
            },
            Opcode::LOOPNZ | Opcode::LOOPZ | Opcode::LOOP => {
                self.set_register(Operand::CX, self.get_register(Operand::CX).wrapping_sub(1));
                if match self.instruction.opcode {
                    Opcode::LOOPNZ => !self.get_flag(Flag::Zero),
                    Opcode::LOOPZ => self.get_flag(Flag::Zero),
                    Opcode::LOOP => true,
                    _ => unreachable!()
                } && self.get_register(Operand::CX) > 0 {
                    self.instruction.position = self.instruction.immediate as u16;
                }
            },
            Opcode::LES | Opcode::LDS => {
                let dst = self.register_or_memory_operand(bus, self.instruction.first_operand);
                let value = self.get_operand(bus, self.instruction.second_operand, 32).1;
                self.set_register(if self.instruction.opcode == Opcode::LES { Operand::ES } else { Operand::DS }, (value>>16) as u16);
                bus.write_to_memory(self, dst, 16, value&0xFFFF);
            },
            Opcode::LEA => {
                let dst = self.register_or_memory_operand(bus, self.instruction.first_operand);
                let (_segment_register, offset) = self.calculate_displacement(self.instruction.second_operand);
                bus.write_to_memory(self, dst, 16, offset as u32);
            },
            Opcode::CLC => {
                self.reset_flag(Flag::Carry);
            },
            Opcode::STC => {
                self.set_flag(Flag::Carry, 1);
            },
            Opcode::CLI => {
                self.reset_flag(Flag::Interrupt);
            },
            Opcode::STI => {
                self.set_flag(Flag::Interrupt, 1);
            },
            Opcode::CLD => {
                self.reset_flag(Flag::Direction);
            },
            Opcode::STD => {
                self.set_flag(Flag::Direction, 1);
            },
            Opcode::CMC => {
                let carry = !self.get_flag(Flag::Carry);
                self.reset_flag(Flag::Carry);
                self.set_flag(Flag::Carry, carry as u16);
            },
            Opcode::CBW => {
                let value = self.get_register(Operand::AL);
                self.set_register(Operand::AX, value as i8 as i16 as u16);
            },
            Opcode::CWD => {
                let value = self.get_register(Operand::AX);
                self.set_register(Operand::DX, ((value as i16 as i32)>>16) as u16);
            },
            Opcode::SAHF => {
                let value = self.get_register(Operand::AH) as u8;
                crate::bit_utils::write_low_byte_of_u16(&mut self.flags, value);
            },
            Opcode::LAHF => {
                self.set_register(Operand::AH, self.flags&0xFF);
            },
            Opcode::MOV => {
                let dst = self.register_or_memory_operand(bus, self.instruction.first_operand);
                let value = self.get_operand(bus, self.instruction.second_operand, self.instruction.data_width).1;
                bus.write_to_memory(self, dst, self.instruction.data_width, value);
            },
            Opcode::XCHG => {
                let (ptr_a, value_a) = self.get_operand(bus, self.instruction.first_operand, self.instruction.data_width);
                let (ptr_b, value_b) = self.get_operand(bus, self.instruction.second_operand, self.instruction.data_width);
                bus.write_to_memory(self, ptr_a, self.instruction.data_width, value_b);
                bus.write_to_memory(self, ptr_b, self.instruction.data_width, value_a);
            },
            Opcode::XLAT => {
                let src = self.memory_operand(bus, true, Operand::DS, self.get_register(Operand::AL));
                let value = bus.read_from_memory(self, src, 8);
                self.set_register(Operand::AL, value as u16);
            },
            Opcode::IN => {
                let src_offset = self.get_operand(bus, self.instruction.second_operand, 16).1 as u16;
                let dst = self.register_or_memory_operand(bus, self.instruction.first_operand);
                let value = if self.instruction.data_width == 16 {
                    ((bus.read_from_port(self, src_offset+1) as u32)<<8)|(bus.read_from_port(self, src_offset) as u32)
                } else {
                    bus.read_from_port(self, src_offset) as u32
                };
                bus.write_to_memory(self, dst, self.instruction.data_width, value);
            },
            Opcode::OUT => {
                let dst_offset = self.get_operand(bus, self.instruction.first_operand, 16).1 as u16;
                let value = self.get_operand(bus, self.instruction.second_operand, self.instruction.data_width).1;
                if self.instruction.data_width == 16 {
                    bus.write_to_port(self, dst_offset, value as u8);
                    bus.write_to_port(self, dst_offset+1, (value>>8) as u8);
                } else {
                    bus.write_to_port(self, dst_offset, value as u8);
                }
            },
            Opcode::MOVSB => {
                string_operation!(self, bus, dst, src, u8, 1, 1, {
                    self.second_value = bus.read_from_memory(self, src, 8);
                    bus.write_to_memory(self, dst, self.instruction.data_width, self.second_value);
                });
            },
            Opcode::MOVSW => {
                string_operation!(self, bus, dst, src, u16, 2, 2, {
                    self.second_value = bus.read_from_memory(self, src, 16);
                    bus.write_to_memory(self, dst, self.instruction.data_width, self.second_value);
                });
            },
            Opcode::CMPSB => {
                string_operation!(self, bus, dst, src, u8, 1, 1, {
                    self.first_value = bus.read_from_memory(self, dst, 8);
                    self.second_value = bus.read_from_memory(self, src, 8);
                    self.result_value = self.first_value.wrapping_sub(self.second_value);
                    self.set_arithmetic_flags();
                    self.set_substraction_flags();
                });
            },
            Opcode::CMPSW => {
                string_operation!(self, bus, dst, src, u16, 2, 2, {
                    self.first_value = bus.read_from_memory(self, dst, 16);
                    self.second_value = bus.read_from_memory(self, src, 16);
                    self.result_value = self.first_value.wrapping_sub(self.second_value);
                    self.set_arithmetic_flags();
                    self.set_substraction_flags();
                });
            },
            Opcode::STOSB => {
                string_operation!(self, bus, dst, _src, u8, 1, 0,
                    bus.write_to_memory(self, dst, self.instruction.data_width, self.get_register(self.instruction.second_operand) as u32)
                );
            },
            Opcode::STOSW => {
                string_operation!(self, bus, dst, _src, u16, 2, 0,
                    bus.write_to_memory(self, dst, self.instruction.data_width, self.get_register(self.instruction.second_operand) as u32)
                );
            },
            Opcode::LODSB => {
                string_operation!(self, bus, _dst, src, u8, 0, 1, {
                    self.second_value = bus.read_from_memory(self, src, self.instruction.data_width);
                    self.set_register(self.instruction.first_operand, self.second_value as u16);
                });
            },
            Opcode::LODSW => {
                string_operation!(self, bus, _dst, src, u16, 0, 2, {
                    self.second_value = bus.read_from_memory(self, src, self.instruction.data_width);
                    self.set_register(self.instruction.first_operand, self.second_value as u16);
                });
            },
            Opcode::SCASB => {
                string_operation!(self, bus, dst, _src, u8, 1, 0, {
                    self.first_value = self.get_register(self.instruction.second_operand) as u32;
                    self.second_value = bus.read_from_memory(self, dst, self.instruction.data_width);
                    self.result_value = self.first_value.wrapping_sub(self.second_value);
                    self.set_arithmetic_flags();
                    self.set_substraction_flags();
                });
            },
            Opcode::SCASW => {
                string_operation!(self, bus, dst, _src, u16, 2, 0, {
                    self.first_value = self.get_register(self.instruction.second_operand) as u32;
                    self.second_value = bus.read_from_memory(self, dst, self.instruction.data_width);
                    self.result_value = self.first_value.wrapping_sub(self.second_value);
                    self.set_arithmetic_flags();
                    self.set_substraction_flags();
                });
            },
            Opcode::BAD => {
                let read_buffer = unsafe { std::slice::from_raw_parts(bus.get_memory(self, crate::bus::BUS::physical_address(self.get_register(Operand::CS), self.instruction.position-self.instruction.length as u16)), 8) };
                println!("CPU ({}): Encountered BAD Instruction at={:04X}:{:04X} machinecode={:?}", self.cycle_counter, self.get_register(Operand::CS), self.instruction.position-self.instruction.length as u16, read_buffer);
                self.software_interrupt(bus, 6);
            },
            _ => unreachable!()
        }
    }
}
