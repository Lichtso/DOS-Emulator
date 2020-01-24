#[derive(Copy, Clone)]
struct Timer {
    enabled: bool,
    expired: bool,
    latch: u8,
    trigger_at_cycle: u64
}

#[derive(Copy, Clone, Debug)]
pub enum ConnectionMode {
    FM,
    AM
}

#[derive(Copy, Clone, Debug)]
pub struct OscillatorSynthesis {
    pub tremolo_enabled: bool,
    pub vibrato_enabled: bool,
    pub sustain_enabled: bool,
    pub waveform: u8,
    pub attack_increment: u32,
    pub decay_increment: u32,
    pub sustain_volume: u32,
    pub release_increment: u32,
    pub phase_increment: u32,
    pub vibrato: u32,
    pub volume: u16
}

#[derive(Copy, Clone, Debug)]
pub struct ChannelSynthesis {
    pub feedback_strength: u8,
    pub connection_mode: ConnectionMode
}

#[derive(Copy, Clone, Debug)]
pub struct SoundBlasterSynthesis {
    pub tremolo_strength: u8,
    pub vibrato_strength: u8,
    pub rhythm_enabled: bool
}

struct Oscillator {
    key_state: bool,
    key_scaling_rate_enabled: bool,
    frequency_multiplier: u8,
    total_level: u8,
    key_scaling_level: u8,
    attack_rate: u8,
    decay_rate: u8,
    sustain_level: u8,
    release_rate: u8,
    synthesis: OscillatorSynthesis
}

struct Channel {
    key_index: u16,
    octave: u8,
    oscillators: [Oscillator; 2],
    synthesis: ChannelSynthesis
}

pub struct SoundBlaster {
    register_index: u8,
    waveform_control: bool,
    keyboard_split_note_select: bool,
    timers: [Timer; 2],
    channels: [Channel; 9],
    synthesis: SoundBlasterSynthesis
}

impl SoundBlaster {
    pub fn new() -> Self {
        Self {
            register_index: 0,
            waveform_control: false,
            keyboard_split_note_select: false,
            timers: [Timer {
                enabled: false,
                expired: false,
                latch: 0,
                trigger_at_cycle: u64::max_value()
            }; 2],
            channels: unsafe { std::mem::zeroed() },
            synthesis: unsafe { std::mem::zeroed() }
        }
    }

    fn set_timer(&mut self, cycle_counter: u64, handler_schedule: &mut crate::bus::HandlerSchedule, timer: usize, enabled: bool) {
        self.timers[timer].enabled = enabled;
        if self.timers[timer].enabled {
            self.timers[timer].trigger_at_cycle = cycle_counter+if timer == 0 { 382 } else { 1527 }*(0x100-self.timers[timer].latch as u64);
            handler_schedule.schedule_handler(crate::bus::HandlerScheduleEntry {
                trigger_at_cycle: self.timers[timer].trigger_at_cycle,
                kind: crate::bus::HandlerScheduleEntryKind::from(crate::bus::HandlerScheduleEntryKind::SoundBlasterTimerChannel0 as usize+timer)
            });
        } else {
            self.timers[timer].trigger_at_cycle = u64::max_value();
            handler_schedule.cancel_handler(crate::bus::HandlerScheduleEntryKind::from(crate::bus::HandlerScheduleEntryKind::SoundBlasterTimerChannel0 as usize+timer));
        }
    }

    pub fn scheduled_handler(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, timer: usize) {
        self.timers[timer].expired = true;
        self.timers[timer].trigger_at_cycle = u64::max_value();
        pic.request_interrupt(cpu, 0);
    }

    fn oscillator_at_address(&mut self, address: u8) -> &mut Oscillator {
        let channel_index = (address&7)%3+(address&0x1F)/8*3;
        let oscillator_index = (address&7)/3;
        &mut self.channels[channel_index as usize].oscillators[oscillator_index as usize]
    }

    fn send_channel_update(&mut self, cycle_counter: u64, audio_event_dst: &mut std::sync::mpsc::Sender<crate::audio::AudioEvent>, channel_index: usize) {
        let channel = &mut self.channels[channel_index];
        let key_scaling_level_base = unsafe { crate::audio::KEY_SCALING_LEVEL_TABLE[((channel.octave as usize)<<4)|((channel.key_index as usize)>>6)] as u16 };
        let key_scaling_rate = (channel.octave<<1)|((channel.key_index>>if self.keyboard_split_note_select { 8 } else { 9 }) as u8&1);
        let frequency_index = (channel.key_index as u32)<<(channel.octave as u32);
        let vibrato = (channel.key_index as u32)>>7<<(channel.octave as u32);
        for oscillator_index in 0..2 {
            let oscillator = &mut channel.oscillators[oscillator_index];
            oscillator.synthesis.volume = (oscillator.total_level<<2) as u16+(key_scaling_level_base>>unsafe { crate::audio::KEY_SCALE_LEVEL_SHIFT_TABLE[oscillator.key_scaling_level as usize] });
            let frequency_multiplier = unsafe { crate::audio::FREQUENCY_MULTIPLIER_TABLE[oscillator.frequency_multiplier as usize] };
            oscillator.synthesis.phase_increment = frequency_index*frequency_multiplier;
            oscillator.synthesis.vibrato = vibrato*frequency_multiplier;
            let key_scaling_rate = if oscillator.key_scaling_rate_enabled { key_scaling_rate } else { key_scaling_rate>>2 };
            oscillator.synthesis.attack_increment = (oscillator.attack_rate > 0) as u32*unsafe { crate::audio::RATE_INCREMENT_TABLE[(oscillator.attack_rate*4+key_scaling_rate) as usize] };
            oscillator.synthesis.decay_increment = (oscillator.decay_rate > 0) as u32*unsafe { crate::audio::RATE_INCREMENT_TABLE[(oscillator.decay_rate*4+key_scaling_rate) as usize] };
            oscillator.synthesis.release_increment = (oscillator.release_rate > 0) as u32*unsafe { crate::audio::RATE_INCREMENT_TABLE[(oscillator.release_rate*4+key_scaling_rate) as usize] };
            if (oscillator.attack_rate*4+key_scaling_rate) >= 60 {
                oscillator.synthesis.attack_increment = unsafe { crate::audio::RATE_INCREMENT_TABLE[76] };
            }
            audio_event_dst.send(crate::audio::AudioEvent {
                cycle_counter: cycle_counter,
                body: crate::audio::AudioEventBody::SoundBlasterUpdateOscillator(channel_index*2+oscillator_index, oscillator.synthesis)
            }).unwrap();
        }
        audio_event_dst.send(crate::audio::AudioEvent {
            cycle_counter: cycle_counter,
            body: crate::audio::AudioEventBody::SoundBlasterUpdateChannel(channel_index, channel.synthesis)
        }).unwrap();
    }

    fn send_key_state(&mut self, cycle_counter: u64, audio_event_dst: &mut std::sync::mpsc::Sender<crate::audio::AudioEvent>, channel_index: usize, oscillator_index: usize, next_key_state: bool) {
        let channel = &mut self.channels[channel_index];
        let oscillator = &mut channel.oscillators[oscillator_index];
        if oscillator.key_state == next_key_state {
            return;
        }
        audio_event_dst.send(crate::audio::AudioEvent {
            cycle_counter: cycle_counter,
            body: crate::audio::AudioEventBody::SoundBlasterUpdateKeyState(channel_index*2+oscillator_index, next_key_state)
        }).unwrap();
        oscillator.key_state = next_key_state;
    }

    pub fn read_from_port(&mut self, cycle_counter: u64, address: u16) -> u8 {
        match address {
            0x220 | 0x222 | 0x388 => (((self.timers[0].expired || self.timers[1].expired) as u8)<<7)|
                                     ((self.timers[0].expired as u8)<<6)|((self.timers[1].expired as u8)<<5),
            _ => {
                println!("SB ({}): Unsupported port read address={:04X}", cycle_counter, address);
                0
            }
        }
    }

    pub fn write_to_port(&mut self, cycle_counter: u64, handler_schedule: &mut crate::bus::HandlerSchedule, config: &mut crate::config::Config, audio_event_dst: &mut std::sync::mpsc::Sender<crate::audio::AudioEvent>, address: u16, value: u8) {
        if !config.audio.sound_blaster_enabled {
            return;
        }
        match address {
            0x220 | 0x222 | 0x388 => {
                self.register_index = value;
            },
            0x221 | 0x223 | 0x389 => {
                match self.register_index {
                    0x01 => { self.waveform_control = value&(1<<5) != 0; }
                    0x02 => { self.timers[0].latch = value; },
                    0x03 => { self.timers[1].latch = value; },
                    0x04 => {
                        if value&(1<<7) != 0 {
                             self.timers[0].expired = false;
                             self.timers[1].expired = false;
                        } else {
                            if (value>>6) != 0 {
                                self.set_timer(cycle_counter, handler_schedule, 0, (value>>0) != 0);
                            }
                            if (value>>5) != 0 {
                                self.set_timer(cycle_counter, handler_schedule, 0, (value>>1) != 0);
                            }
                        }
                    },
                    0x08 => {
                        self.keyboard_split_note_select = (value>>6)&1 != 0;
                    },
                    0x20..=0x25 | 0x28..=0x2D | 0x30..=0x35 => {
                        let oscillator = self.oscillator_at_address(self.register_index);
                        oscillator.synthesis.tremolo_enabled = (value>>7)&1 != 0;
                        oscillator.synthesis.vibrato_enabled = (value>>6)&1 != 0;
                        oscillator.synthesis.sustain_enabled = (value>>5)&1 != 0;
                        oscillator.key_scaling_rate_enabled = (value>>4)&1 != 0;
                        oscillator.frequency_multiplier = value&0x0F;
                    },
                    0x40..=0x45 | 0x48..=0x4D | 0x50..=0x55 => {
                        let oscillator = self.oscillator_at_address(self.register_index);
                        oscillator.total_level = value&0x3F;
                        oscillator.key_scaling_level = (value>>6)&0x03;
                    },
                    0x60..=0x65 | 0x68..=0x6D | 0x70..=0x75 => {
                        let oscillator = self.oscillator_at_address(self.register_index);
                        oscillator.attack_rate = (value>>4)&0x0F;
                        oscillator.decay_rate = value&0x0F;
                    },
                    0x80..=0x85 | 0x88..=0x8D | 0x90..=0x95 => {
                        let oscillator = self.oscillator_at_address(self.register_index);
                        oscillator.sustain_level = (value>>4)&0x0F;
                        oscillator.release_rate = value&0x0F;
                        oscillator.synthesis.sustain_volume = (if oscillator.sustain_level == 0xF { 31 } else { oscillator.sustain_level as u32 })<<4;
                    },
                    0xA0..=0xA8 => {
                        let mut channel = &mut self.channels[self.register_index as usize-0xA0];
                        channel.key_index &= 0xFF00;
                        channel.key_index |= value as u16;
                    },
                    0xB0..=0xB8 => {
                        let channel_index = self.register_index as usize-0xB0;
                        let mut channel = &mut self.channels[channel_index];
                        let next_key_state = (value>>5)&1 != 0;
                        channel.octave = (value>>2)&0x07;
                        channel.key_index &= 0x00FF;
                        channel.key_index |= (value as u16&0x03)<<8;
                        self.send_channel_update(cycle_counter, audio_event_dst, channel_index);
                        for i in 0..2 {
                            self.send_key_state(cycle_counter, audio_event_dst, channel_index, i, next_key_state);
                        }
                    },
                    0xBD => {
                        self.synthesis.tremolo_strength = if (value>>7)&1 != 0 { 0 } else { 2 };
                        self.synthesis.vibrato_strength = if (value>>6)&1 != 0 { 0 } else { 1 };
                        self.synthesis.rhythm_enabled = (value>>5)&1 != 0;
                        audio_event_dst.send(crate::audio::AudioEvent {
                            cycle_counter: cycle_counter,
                            body: crate::audio::AudioEventBody::SoundBlasterUpdate(self.synthesis)
                        }).unwrap();
                        if self.synthesis.rhythm_enabled {
                            let bass_drum = (value>>4)&1 != 0;
                            self.send_key_state(cycle_counter, audio_event_dst, 6, 0, bass_drum);
                            self.send_key_state(cycle_counter, audio_event_dst, 6, 1, bass_drum);
                            let hi_hat = (value>>0)&1 != 0;
                            self.send_key_state(cycle_counter, audio_event_dst, 7, 0, hi_hat);
                            let snare = (value>>3)&1 != 0;
                            self.send_key_state(cycle_counter, audio_event_dst, 7, 1, snare);
                            let tom_tom = (value>>2)&1 != 0;
                            self.send_key_state(cycle_counter, audio_event_dst, 8, 0, tom_tom);
                            let cymbal = (value>>1)&1 != 0;
                            self.send_key_state(cycle_counter, audio_event_dst, 8, 1, cymbal);
                        } else {
                            for channel_index in 6..9 {
                                for oscillator_index in 0..2 {
                                    self.send_key_state(cycle_counter, audio_event_dst, channel_index, oscillator_index, false);
                                }
                            }
                        }
                    },
                    0xC0..=0xC8 => {
                        let mut channel = &mut self.channels[self.register_index as usize-0xC0];
                        channel.synthesis.feedback_strength = (value>>1)&0x07;
                        channel.synthesis.feedback_strength = if channel.synthesis.feedback_strength == 0 { 31 } else { 9-channel.synthesis.feedback_strength };
                        channel.synthesis.connection_mode = if value&1 == 0 { ConnectionMode::FM } else { ConnectionMode::AM };
                    },
                    0xE0..=0xE5 | 0xE8..=0xED | 0xF0..=0xF5 => {
                        let waveform_control = self.waveform_control;
                        let oscillator = self.oscillator_at_address(self.register_index);
                        oscillator.synthesis.waveform = if waveform_control { value&0x07 } else { 0 };
                    },
                    _ => {}
                }
            },
            _ => {
                println!("SB ({}): Unsupported port write address={:04X} value={:02X}", cycle_counter, address, value);
            }
        }
    }
}
