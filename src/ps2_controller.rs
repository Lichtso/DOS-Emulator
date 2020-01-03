pub struct PS2Controller {
    data_buffer: [u8; 16],
    write_pos: usize,
    read_pos: usize,
    // status: u8
}

impl PS2Controller {
    pub fn new() -> Self {
        Self {
            data_buffer: unsafe { std::mem::zeroed() },
            write_pos: 0,
            read_pos: 0,
            // status: 0
        }
    }

    pub fn is_data_available(&mut self) -> bool {
        self.read_pos != self.write_pos
    }

    fn schedule_pop_data(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, handler_schedule: &mut crate::bus::HandlerSchedule) {
        if self.is_data_available() {
            pic.request_interrupt(cpu, 1);
            handler_schedule.schedule_handler(crate::bus::HandlerScheduleEntry {
                kind: crate::bus::HandlerScheduleEntryKind::PS2Controller,
                trigger_at_cycle: cpu.cycle_counter+160
            });
        }
    }

    pub fn push_data(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, handler_schedule: &mut crate::bus::HandlerSchedule, value: u8) {
        let new_write_pos = (self.write_pos+1)%self.data_buffer.len();
        if new_write_pos != self.read_pos {
            self.data_buffer[self.write_pos] = value;
            self.write_pos = new_write_pos;
            self.schedule_pop_data(cpu, pic, handler_schedule);
        } else {
            println!("PS/2 ({}): Data buffer overflow, dropped={:02X}", cpu.cycle_counter, value);
        }
    }

    pub fn pop_data(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, handler_schedule: &mut crate::bus::HandlerSchedule) -> Option<u8> {
        if self.is_data_available() {
            let value = self.data_buffer[self.read_pos];
            self.read_pos = (self.read_pos+1)%self.data_buffer.len();
            self.schedule_pop_data(cpu, pic, handler_schedule);
            Some(value)
        } else {
            println!("PS/2 ({}): Data buffer underflow", cpu.cycle_counter);
            None
        }
    }

    pub fn read_from_port(&mut self, cycle_counter: u64, address: u16) -> u8 {
        match address {
            0x60 => {
                if self.read_pos != self.write_pos {
                    self.data_buffer[self.read_pos]
                } else {
                    println!("PS/2 ({}): Data buffer underflow", cycle_counter);
                    0
                }
            },
            // 0x64 => self.status,
            _ => {
                println!("PS/2 ({}): Unsupported port read address={:04X}", cycle_counter, address);
                0
            }
        }
    }

    pub fn write_to_port(&mut self, cycle_counter: u64, address: u16, value: u8) {
        match address {
            // 0x60 => { self.data = value; },
            // 0x64 => { },
            _ => {
                println!("PS/2 ({}): Unsupported port write address={:04X} value={:02X}", cycle_counter, address, value);
            }
        }
    }
}
