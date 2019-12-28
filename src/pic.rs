pub struct ProgrammableInterruptController {
    mask: u16,
    pending: u16,
    handled_interrupt: u8,
    most_important_interrupt: u8
}

impl ProgrammableInterruptController {
    pub fn new() -> Self {
        Self {
            mask: 0xFFFF,
            pending: 0,
            handled_interrupt: 16,
            most_important_interrupt: 16
        }
    }

    pub fn get_interrupt_to_handle(&mut self) -> u8 {
        if self.most_important_interrupt == 16 {
            0
        } else {
            self.handled_interrupt = self.most_important_interrupt;
            (if self.most_important_interrupt < 8 { 0x08 } else { 0x70 }) + self.handled_interrupt
        }
    }

    pub fn request_interrupt(&mut self, cpu: &mut crate::cpu::CPU, interrupt: u8) {
        if (self.mask>>(interrupt as usize))&1 == 1 && (self.pending>>(interrupt as usize))&1 == 0 {
            self.pending |= 1<<(interrupt as usize);
            self.most_important_interrupt = self.pending.trailing_zeros() as u8;
        }
        if cpu.execution_state == crate::cpu::ExecutionState::WaitForInterrupt {
            cpu.execution_state = crate::cpu::ExecutionState::Running;
        }
    }

    pub fn read_from_port(&mut self, _cycle_counter: u64, address: u16) -> u8 {
        match address {
            0x21 => self.mask as u8,
            0xA1 => (self.mask>>8) as u8,
            _ => 0
        }
    }

    fn end_interrupt(&mut self, _cycle_counter: u64, interrupt: u8) {
        self.pending &= !(1<<(interrupt as usize));
        self.most_important_interrupt = self.pending.trailing_zeros() as u8;
    }

    pub fn write_to_port(&mut self, cycle_counter: u64, address: u16, value: u8) {
        match address {
            0x21 => { crate::bit_utils::write_low_byte_of_u16(&mut self.mask, value) },
            0xA1 => { crate::bit_utils::write_high_byte_of_u16(&mut self.mask, value) },
            _ => {
                match value {
                    0x20 => { // Non Specific End of Interrupt
                        self.end_interrupt(cycle_counter, self.handled_interrupt);
                    },
                    0x60..=0x67 => { // Specific End of Interrupt
                        self.end_interrupt(cycle_counter, value&7);
                    },
                    _ => {
                        println!("PIC ({}): Unsupported command={}", cycle_counter, value);
                    }
                }
            }
        }
    }
}
