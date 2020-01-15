#[derive(Copy, Clone, PartialEq)]
enum AccessMode {
    HighThenLow = 0,
    AlwaysLow = 1,
    AlwaysHigh = 2,
    LowThenHigh = 3
}

#[derive(Copy, Clone)]
pub struct ProgrammableIntervalTimerChannel {
    operation_mode: u8,
    access_mode: AccessMode,
    latch_read: u16,
    reload: u16,
    trigger_at_cycle: u64,
    is_latched: bool,
    input_mask: bool
}

pub struct ProgrammableIntervalTimer {
    pub clock_frequency: f64,
    pub beeper_event_buffer: [(u64, f32); 16],
    pub beeper_event_write_pos: usize,
    pub beeper_event_read_pos: usize,
    channels: [ProgrammableIntervalTimerChannel; 3]
}

impl ProgrammableIntervalTimer {
    pub fn new() -> Self {
        Self {
            clock_frequency: 0.0,
            beeper_event_buffer: unsafe { std::mem::zeroed() },
            beeper_event_write_pos: 0,
            beeper_event_read_pos: 0,
            channels: [ProgrammableIntervalTimerChannel {
                operation_mode: 0,
                access_mode: AccessMode::LowThenHigh,
                latch_read: 0,
                reload: 0,
                trigger_at_cycle: u64::max_value(),
                is_latched: false,
                input_mask: true
            }; 3]
        }
    }

    fn push_beeper_event(&mut self, cycle_counter: u64) {
        let frequency = self.clock_frequency as f32/(self.calculate_reload_of_channel(2) as f32);
        self.beeper_event_buffer[self.beeper_event_write_pos] = (cycle_counter, if self.channels[2].input_mask { frequency } else { 0.0 });
        self.beeper_event_write_pos = (self.beeper_event_write_pos+1)%self.beeper_event_buffer.len();
    }

    fn calculate_reload_of_channel(&mut self, channel: usize) -> u64 {
        let mut reload = self.channels[channel].reload as u64;
        if reload == 0 { reload = 0x10000; }
        (match self.channels[channel].operation_mode {
            3 | 7 => 4,
            _ => 2
        })*reload
    }

    fn calculate_counter_of_channel(&mut self, channel: usize, cycle_counter: u64) -> u16 {
        if self.channels[channel].trigger_at_cycle == u64::max_value() {
            return 0;
        }
        let reload_value = self.calculate_reload_of_channel(channel);
        let last_start = cycle_counter-self.channels[channel].trigger_at_cycle+reload_value;
        match self.channels[channel].operation_mode {
            2 | 6 | 3 | 7 => (last_start%reload_value) as u16,
            _ => last_start as u16
        }
    }

    fn calculate_output_of_channel(&mut self, channel: usize, cycle_counter: u64) -> bool {
        if self.channels[channel].trigger_at_cycle == u64::max_value() {
            return self.channels[channel].operation_mode > 1;
        }
        let reload_value = self.calculate_reload_of_channel(channel);
        let last_start = cycle_counter-(self.channels[channel].trigger_at_cycle-reload_value);
        match self.channels[channel].operation_mode {
            0 | 1 => last_start >= reload_value,
            3 | 7 => last_start*2 >= reload_value,
            _ => true
        }
    }

    pub fn scheduled_handler(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, handler_schedule: &mut crate::bus::HandlerSchedule, channel: usize) {
        match self.channels[channel].operation_mode {
            2 | 6 | 3 | 7 => {
                self.channels[channel].trigger_at_cycle += self.calculate_reload_of_channel(channel);
                handler_schedule.schedule_handler(crate::bus::HandlerScheduleEntry {
                    trigger_at_cycle: self.channels[channel].trigger_at_cycle,
                    kind: crate::bus::HandlerScheduleEntryKind::from(channel)
                });
            },
            _ => {}
        }
        if channel == 0 {
            pic.request_interrupt(cpu, 0);
        }
    }

    pub fn read_from_port(&mut self, cycle_counter: u64, address: u16) -> u8 {
        match address {
            0x61 => (self.calculate_output_of_channel(2, cycle_counter) as u8)<<5|(self.channels[2].input_mask as u8),
            0x40..=0x42 => {
                let channel = (address-0x40) as usize;
                let value = if self.channels[channel].is_latched { self.channels[channel].latch_read } else { self.calculate_counter_of_channel(channel, cycle_counter) };
                let value = match self.channels[channel].access_mode {
                    AccessMode::LowThenHigh | AccessMode::AlwaysLow => value as u8,
                    AccessMode::HighThenLow | AccessMode::AlwaysHigh => (value>>8) as u8
                };
                if self.channels[channel].access_mode != AccessMode::LowThenHigh {
                    self.channels[channel].is_latched = false;
                }
                self.channels[channel].access_mode = match self.channels[channel].access_mode {
                    AccessMode::LowThenHigh => AccessMode::HighThenLow,
                    AccessMode::HighThenLow => AccessMode::LowThenHigh,
                    _ => self.channels[channel].access_mode
                };
                value
            },
            _ => 0
        }
    }

    pub fn write_to_port(&mut self, cycle_counter: u64, handler_schedule: &mut crate::bus::HandlerSchedule, address: u16, value: u8) {
        match address {
            0x61 => {
                let input_mask = value&1 == 1;
                let changed = self.channels[2].input_mask != input_mask;
                self.channels[2].input_mask = input_mask;
                if changed {
                    self.push_beeper_event(cycle_counter);
                }
            },
            0x40..=0x42 => {
                let channel = (address-0x40) as usize;
                match self.channels[channel].access_mode {
                    AccessMode::LowThenHigh | AccessMode::AlwaysLow => {
                        crate::bit_utils::write_low_byte_of_u16(&mut self.channels[channel].reload, value);
                    },
                    AccessMode::HighThenLow | AccessMode::AlwaysHigh => {
                        crate::bit_utils::write_high_byte_of_u16(&mut self.channels[channel].reload, value);
                    }
                }
                if self.channels[channel].access_mode != AccessMode::LowThenHigh {
                    if channel == 2 {
                        self.push_beeper_event(cycle_counter);
                    }
                    self.channels[channel].trigger_at_cycle = cycle_counter+self.calculate_reload_of_channel(channel);
                    handler_schedule.schedule_handler(crate::bus::HandlerScheduleEntry {
                        kind: crate::bus::HandlerScheduleEntryKind::from(channel),
                        trigger_at_cycle: self.channels[channel].trigger_at_cycle
                    });
                }
                self.channels[channel].access_mode = match self.channels[channel].access_mode {
                    AccessMode::LowThenHigh => AccessMode::HighThenLow,
                    AccessMode::HighThenLow => AccessMode::LowThenHigh,
                    _ => self.channels[channel].access_mode
                };
            },
            0x43 => {
                let channel = (value>>6) as usize;
                let operation_mode = (value>>1)&7;
                let access_mode = (value>>4)&3;
                if access_mode == 0 {
                    self.channels[channel].is_latched = true;
                    self.channels[channel].latch_read = self.calculate_counter_of_channel(channel, cycle_counter);
                } else {
                    self.channels[channel].operation_mode = operation_mode;
                    self.channels[channel].access_mode = unsafe { std::mem::transmute::<u8, AccessMode>(access_mode) };
                    self.channels[channel].reload = 0;
                    self.channels[channel].trigger_at_cycle = u64::max_value();
                    self.channels[channel].is_latched = false;
                    handler_schedule.cancel_handler(crate::bus::HandlerScheduleEntryKind::from(channel));
                }
            },
            _ => {}
        }
    }
}
