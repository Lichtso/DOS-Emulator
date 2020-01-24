#[derive(Copy, Clone, PartialEq)]
enum AccessMode {
    HighThenLow = 0,
    AlwaysLow = 1,
    AlwaysHigh = 2,
    LowThenHigh = 3
}

#[derive(Copy, Clone)]
pub struct Timer {
    operation_mode: u8,
    access_mode: AccessMode,
    latch_read: u16,
    reload: u16,
    trigger_at_cycle: u64,
    is_latched: bool,
    input_mask: bool
}

pub struct ProgrammableIntervalTimer {
    timers: [Timer; 3]
}

impl ProgrammableIntervalTimer {
    pub fn new() -> Self {
        Self {
            timers: [Timer {
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

    fn calculate_reload_of_timer(&mut self, timer: usize) -> u64 {
        let mut reload = self.timers[timer].reload as u64;
        if reload == 0 { reload = 0x10000; }
        (match self.timers[timer].operation_mode {
            3 | 7 => 4,
            _ => 2
        })*reload
    }

    fn calculate_counter_of_timer(&mut self, timer: usize, cycle_counter: u64) -> u16 {
        if self.timers[timer].trigger_at_cycle == u64::max_value() {
            return 0;
        }
        let reload_value = self.calculate_reload_of_timer(timer);
        let last_start = cycle_counter-self.timers[timer].trigger_at_cycle+reload_value;
        match self.timers[timer].operation_mode {
            2 | 6 | 3 | 7 => (last_start%reload_value) as u16,
            _ => last_start as u16
        }
    }

    fn calculate_output_of_timer(&mut self, cycle_counter: u64, timer: usize) -> bool {
        if self.timers[timer].trigger_at_cycle == u64::max_value() {
            return self.timers[timer].operation_mode > 1;
        }
        let reload_value = self.calculate_reload_of_timer(timer);
        let last_start = cycle_counter-(self.timers[timer].trigger_at_cycle-reload_value);
        match self.timers[timer].operation_mode {
            0 | 1 => last_start >= reload_value,
            3 | 7 => last_start*2 >= reload_value,
            _ => true
        }
    }

    fn push_beeper_event(&mut self, config: &mut crate::config::Config, audio_event_dst: &mut std::sync::mpsc::Sender<crate::audio::AudioEvent>, cycle_counter: u64) {
        if config.audio.beeper_enabled {
            let frequency = config.timing.clock_frequency as f32/(self.calculate_reload_of_timer(2) as f32);
            audio_event_dst.send(crate::audio::AudioEvent {
                cycle_counter: cycle_counter,
                body: crate::audio::AudioEventBody::Beeper(if self.timers[2].input_mask { frequency } else { 0.0 })
            }).unwrap();
        }
    }

    pub fn scheduled_handler(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, handler_schedule: &mut crate::bus::HandlerSchedule, timer: usize) {
        match self.timers[timer].operation_mode {
            2 | 6 | 3 | 7 => {
                self.timers[timer].trigger_at_cycle += self.calculate_reload_of_timer(timer);
                handler_schedule.schedule_handler(crate::bus::HandlerScheduleEntry {
                    trigger_at_cycle: self.timers[timer].trigger_at_cycle,
                    kind: crate::bus::HandlerScheduleEntryKind::from(timer)
                });
            },
            _ => {}
        }
        if timer == 0 {
            pic.request_interrupt(cpu, 0);
        }
    }

    pub fn read_from_port(&mut self, cycle_counter: u64, address: u16) -> u8 {
        match address {
            0x61 => (self.calculate_output_of_timer(cycle_counter, 2) as u8)<<5|(self.timers[2].input_mask as u8),
            0x40..=0x42 => {
                let timer = (address-0x40) as usize;
                let value = if self.timers[timer].is_latched { self.timers[timer].latch_read } else { self.calculate_counter_of_timer(timer, cycle_counter) };
                let value = match self.timers[timer].access_mode {
                    AccessMode::LowThenHigh | AccessMode::AlwaysLow => value as u8,
                    AccessMode::HighThenLow | AccessMode::AlwaysHigh => (value>>8) as u8
                };
                if self.timers[timer].access_mode != AccessMode::LowThenHigh {
                    self.timers[timer].is_latched = false;
                }
                self.timers[timer].access_mode = match self.timers[timer].access_mode {
                    AccessMode::LowThenHigh => AccessMode::HighThenLow,
                    AccessMode::HighThenLow => AccessMode::LowThenHigh,
                    _ => self.timers[timer].access_mode
                };
                value
            },
            _ => 0
        }
    }

    pub fn write_to_port(&mut self, cycle_counter: u64, handler_schedule: &mut crate::bus::HandlerSchedule, config: &mut crate::config::Config, audio_event_dst: &mut std::sync::mpsc::Sender<crate::audio::AudioEvent>, address: u16, value: u8) {
        match address {
            0x61 => {
                let input_mask = value&1 == 1;
                let changed = self.timers[2].input_mask != input_mask;
                self.timers[2].input_mask = input_mask;
                if changed {
                    self.push_beeper_event(config, audio_event_dst, cycle_counter);
                }
            },
            0x40..=0x42 => {
                let timer = (address-0x40) as usize;
                match self.timers[timer].access_mode {
                    AccessMode::LowThenHigh | AccessMode::AlwaysLow => {
                        crate::bit_utils::write_low_byte_of_u16(&mut self.timers[timer].reload, value);
                    },
                    AccessMode::HighThenLow | AccessMode::AlwaysHigh => {
                        crate::bit_utils::write_high_byte_of_u16(&mut self.timers[timer].reload, value);
                    }
                }
                if self.timers[timer].access_mode != AccessMode::LowThenHigh {
                    if timer == 2 {
                        self.push_beeper_event(config, audio_event_dst, cycle_counter);
                    }
                    self.timers[timer].trigger_at_cycle = cycle_counter+self.calculate_reload_of_timer(timer);
                    handler_schedule.schedule_handler(crate::bus::HandlerScheduleEntry {
                        kind: crate::bus::HandlerScheduleEntryKind::from(timer),
                        trigger_at_cycle: self.timers[timer].trigger_at_cycle
                    });
                }
                self.timers[timer].access_mode = match self.timers[timer].access_mode {
                    AccessMode::LowThenHigh => AccessMode::HighThenLow,
                    AccessMode::HighThenLow => AccessMode::LowThenHigh,
                    _ => self.timers[timer].access_mode
                };
            },
            0x43 => {
                let timer = (value>>6) as usize;
                let operation_mode = (value>>1)&7;
                let access_mode = (value>>4)&3;
                if access_mode == 0 {
                    self.timers[timer].is_latched = true;
                    self.timers[timer].latch_read = self.calculate_counter_of_timer(timer, cycle_counter);
                } else {
                    self.timers[timer].operation_mode = operation_mode;
                    self.timers[timer].access_mode = unsafe { std::mem::transmute::<u8, AccessMode>(access_mode) };
                    self.timers[timer].reload = 0;
                    self.timers[timer].trigger_at_cycle = u64::max_value();
                    self.timers[timer].is_latched = false;
                    handler_schedule.cancel_handler(crate::bus::HandlerScheduleEntryKind::from(timer));
                }
            },
            _ => {}
        }
    }
}
