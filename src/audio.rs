use cpal::traits::{DeviceTrait, HostTrait, EventLoopTrait};

const CLOCK_FREQUENCY_FACTOR: u64 = 3;
const ENVELOPE_MAX: u32 = 0x1FF;
const SICLENCE_THRESHOLD: u32 = 0x180;
const ENVELOPE_CLOCK_FREQUENCY: f32 = 14318180.0/(8*36) as f32; // 49715.9 Hz
const KEY_FREQUENCY_FACTOR: f32 = 20.973;
const PHASE_BITS: usize = 10;
const PHASE_SHIFT: usize = 18;
const PHASE_MASK: i32 = (1<<PHASE_BITS)-1;
const VIBRATO_SHIFT: usize = 10;
const VIBRATO_MAX: usize = 8;
const TREMOLO_SHIFT: usize = 8;
const TREMOLO_MAX: u32 = 52;
const ENVELOPE_PHASE_SHIFT: usize = 24;
const ENVELOPE_PHASE_MASK: u32 = (1<<ENVELOPE_PHASE_SHIFT)-1;
static mut EXP_TABLE: &'static mut [u16] = &mut [0; 0x100];
static mut SIN_TABLE: &'static mut [u16] = &mut [0; 0x200];
pub static mut FREQUENCY_MULTIPLIER_TABLE: &'static mut [u32] = &mut [0; 16];
pub static mut KEY_SCALING_LEVEL_TABLE: &'static mut [u8] = &mut [0; 0x80];
pub static mut VIBRATO_TABLE: &'static [i8] = &[3, 7, 3, 0, -3, -7, -3, 0];
pub static mut KEY_SCALE_LEVEL_SHIFT_TABLE: &'static [u8] = &[7, 1, 2, 0];
pub static mut RATE_INCREMENT_TABLE: &'static mut [u32] = &mut [0; 77];

pub fn fill_lookup_tables(output_sample_rate: u32) {
    unsafe {
        for i in 0..EXP_TABLE.len() {
            EXP_TABLE[0xFF-i] = (((2 as f32).powf(i as f32/0x100 as f32)-1.0)*2048.0) as u16+2048;
        }
        for i in 0..SIN_TABLE.len() {
            SIN_TABLE[i] = (0.5-((i as f32+0.5)/512.0*std::f32::consts::PI).sin().log2()*256.0) as u16;
        }
        let frequency_factor = (1<<(PHASE_BITS+PHASE_SHIFT)) as f32/output_sample_rate as f32/KEY_FREQUENCY_FACTOR;
        let frequency_multipliers = [0.5, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 10.0, 12.0, 12.0, 15.0, 15.0];
        for i in 0..FREQUENCY_MULTIPLIER_TABLE.len() {
            FREQUENCY_MULTIPLIER_TABLE[i] = (frequency_factor*frequency_multipliers[i]+0.5) as u32;
        }
        let key_scaling_level_template = [64, 32, 24, 19, 16, 12, 11, 10, 8, 6, 5, 4, 3, 2, 1, 0];
        for octave in 0..8 {
            for i in 0..16 {
                KEY_SCALING_LEVEL_TABLE[octave*16+i] = ((octave*8) as isize-key_scaling_level_template[i] as isize).max(0) as u8;
            }
        }
        let rate_factor = (1<<ENVELOPE_PHASE_SHIFT) as f32/output_sample_rate as f32*ENVELOPE_CLOCK_FREQUENCY;
        let envelope_template = [4, 5, 6, 7, 8, 10, 12, 14, 16, 20, 24, 28, 32, 64];
        for i in 0..RATE_INCREMENT_TABLE.len() {
            let (shift, select) = match i {
                0..=51 => (12-i/4, i&3),
                52..=59 => (0, i-12*4),
                60..=75 => (0, 12),
                _ => (0, 13)
            };
            RATE_INCREMENT_TABLE[i] = ((rate_factor*envelope_template[select] as f32) as u32)>>(shift+3);
        }
    }
}

fn lookup_sin(phase: i32) -> i32 {
    unsafe { SIN_TABLE[phase as usize&0x1FF] as i32 }
}

fn lookup_exp(total: i32) -> i32 {
    unsafe { (EXP_TABLE[total as usize&0xFF] as i32)>>(total>>8) }
}

fn zero_second_half(value: i32, phase: i32) -> i32 {
    value|(((phase < 0x200) as i32-1)&0xFFF)
}

fn negate_second_half(value: i32, phase: i32) -> i32 {
    let sign_mask = -((phase >= 0x200) as i32);
    (value^sign_mask)-sign_mask
}

fn calculate_sample(waveform: u8, volume: i32, phase: i32) -> i32 {
    match waveform {
        0 => negate_second_half(lookup_exp(lookup_sin(phase)+volume), phase),
        1 => lookup_exp(zero_second_half(lookup_sin(phase), phase)+volume),
        2 => lookup_exp(lookup_sin(phase)+volume),
        3 => lookup_exp(zero_second_half(lookup_sin(phase&0xFF), (phase*2)&PHASE_MASK)+volume),
        4 => negate_second_half(lookup_exp(zero_second_half(lookup_sin(phase*2), phase)+volume), phase*2),
        5 => lookup_exp(zero_second_half(lookup_sin(phase*2), phase)+volume),
        6 => negate_second_half(lookup_exp(volume), phase),
        7 => negate_second_half(lookup_exp(negate_second_half((phase+(phase >= 0x200) as i32)<<3, phase)&0xFFF+volume), phase),
        _ => unreachable!()
    }
}

pub enum AudioEventBody {
    Beeper(f32),
    SoundBlasterUpdate(crate::sound_blaster::SoundBlasterSynthesis),
    SoundBlasterUpdateChannel(usize, crate::sound_blaster::ChannelSynthesis),
    SoundBlasterUpdateOscillator(usize, crate::sound_blaster::OscillatorSynthesis),
    SoundBlasterUpdateKeyState(usize, bool)
}

pub struct AudioEvent {
    pub cycle_counter: u64,
    pub body: AudioEventBody
}

#[derive(Copy, Clone, PartialEq)]
enum EnvelopeState {
    Off,
    Attack,
    Decay,
    Sustain,
    Release
}

struct Oscillator {
    settings: crate::sound_blaster::OscillatorSynthesis,
    envelope_state: EnvelopeState,
    envelope_phase: u32,
    envelope_volume: u32,
    phase: u32
}

impl Oscillator {
    fn set_key_state(&mut self, key_state: bool) {
        if key_state {
            self.envelope_volume = ENVELOPE_MAX;
            self.envelope_state = EnvelopeState::Attack;
            self.envelope_phase = 0;
        } else {
            self.envelope_state = EnvelopeState::Release;
        }
    }

    fn calculate_volume_and_phase(&mut self) -> (u32, i32) {
        match self.envelope_state {
            EnvelopeState::Off => {
                self.envelope_volume = ENVELOPE_MAX;
            },
            EnvelopeState::Attack => {
                self.envelope_phase += self.settings.attack_increment;
                self.envelope_volume = (self.envelope_volume as i32+(-1-(self.envelope_volume/8) as i32)*(self.envelope_phase>>ENVELOPE_PHASE_SHIFT) as i32) as u32;
                self.envelope_phase &= ENVELOPE_PHASE_MASK;
                if self.envelope_volume > ENVELOPE_MAX {
                    self.envelope_volume = 0;
                    self.envelope_state = EnvelopeState::Decay;
                    self.envelope_phase = 0;
                }
            },
            EnvelopeState::Decay => {
                self.envelope_phase += self.settings.decay_increment;
                self.envelope_volume += self.envelope_phase>>ENVELOPE_PHASE_SHIFT;
                self.envelope_phase &= ENVELOPE_PHASE_MASK;
                if self.envelope_volume >= self.settings.sustain_volume {
                    self.envelope_phase = 0;
                    if self.envelope_volume >= ENVELOPE_MAX {
                        self.envelope_volume = ENVELOPE_MAX;
                        self.envelope_state = EnvelopeState::Off;
                    } else if self.settings.sustain_enabled {
                        self.envelope_volume = self.settings.sustain_volume;
                        self.envelope_state = EnvelopeState::Sustain;
                    } else {
                        self.envelope_state = EnvelopeState::Release;
                    }
                }
            },
            EnvelopeState::Sustain => {},
            EnvelopeState::Release => {
                self.envelope_phase += self.settings.release_increment;
                self.envelope_volume += self.envelope_phase>>ENVELOPE_PHASE_SHIFT;
                self.envelope_phase &= ENVELOPE_PHASE_MASK;
                if self.envelope_volume >= ENVELOPE_MAX {
                    self.envelope_volume = ENVELOPE_MAX;
                    self.envelope_state = EnvelopeState::Off;
                    self.envelope_phase = 0;
                }
            }
        }
        (self.settings.volume as u32+self.envelope_volume, (self.phase>>PHASE_SHIFT) as i32)
    }

    fn calculate_sample(&mut self, lfo: &LowFrequencyOscillator, mut volume: u32, phase: i32) -> i32 {
        let silent = volume >= SICLENCE_THRESHOLD;
        let vibrato_value = ((self.settings.vibrato>>lfo.vibrato_shift)^lfo.vibrato_sign_mask) as i32-lfo.vibrato_sign_mask as i32;
        self.phase = (self.phase as i32).wrapping_add(self.settings.phase_increment as i32+vibrato_value) as u32;
        volume += lfo.tremolo_value*(self.settings.tremolo_enabled as u32);
        let signal = calculate_sample(self.settings.waveform, (volume as i32)<<3, phase&PHASE_MASK);
        if silent { 0 } else { signal }
    }

    fn calculate_signal(&mut self, lfo: &LowFrequencyOscillator, phase_modulation: i32) -> i32 {
        if self.envelope_state != EnvelopeState::Off {
            let (volume, phase) = self.calculate_volume_and_phase();
            self.calculate_sample(lfo, volume, phase+phase_modulation)
        } else { 0 }
    }
}

struct Channel {
    settings: crate::sound_blaster::ChannelSynthesis,
    prev: [i32; 2]
}

struct LowFrequencyOscillator {
    vibrato_phase: u32,
    tremolo_phase: u32,
    tremolo_value: u32,
    vibrato_sign_mask: u32,
    vibrato_shift: u8
}

struct AudioRenderer {
    settings: crate::sound_blaster::SoundBlasterSynthesis,
    output_channels: usize,
    output_sample_rate: u32,
    clock_cycles_per_sample: u64,
    cycle_counter: u64,
    noise_value: u32,
    lfo: LowFrequencyOscillator,
    audio_event: AudioEvent,
    channels: [Channel; 9],
    oscillators: [Oscillator; 19]
}

impl AudioRenderer {
    fn new(format: &cpal::Format) -> Self {
        let mut audio_renderer = Self {
            settings: unsafe { std::mem::zeroed() },
            output_channels: format.channels as usize,
            output_sample_rate: match format.sample_rate { cpal::SampleRate(sample_rate) => sample_rate },
            clock_cycles_per_sample: 0,
            cycle_counter: 0,
            noise_value: 1,
            lfo: unsafe { std::mem::zeroed() },
            audio_event: AudioEvent {
                cycle_counter: 0,
                body: AudioEventBody::Beeper(0.0)
            },
            channels: unsafe { std::mem::zeroed() },
            oscillators: unsafe { std::mem::zeroed() }
        };
        fill_lookup_tables(audio_renderer.output_sample_rate);
        let beeper = &mut audio_renderer.oscillators[audio_renderer.channels.len()*2];
        beeper.settings.waveform = 6;
        beeper.settings.sustain_enabled = true;
        beeper.settings.attack_increment = unsafe { RATE_INCREMENT_TABLE[70] };
        beeper.settings.decay_increment = unsafe { RATE_INCREMENT_TABLE[70] };
        beeper.settings.release_increment = unsafe { RATE_INCREMENT_TABLE[70] };
        audio_renderer
    }

    fn handle_audio_event(&mut self) -> bool {
        if CLOCK_FREQUENCY_FACTOR*self.audio_event.cycle_counter > self.cycle_counter {
            return false;
        }
        self.audio_event.cycle_counter = u64::max_value();
        match &self.audio_event.body {
            AudioEventBody::Beeper(frequency) => {
                let beeper = &mut self.oscillators[self.channels.len()*2];
                beeper.settings.phase_increment = (*frequency*KEY_FREQUENCY_FACTOR) as u32*unsafe { FREQUENCY_MULTIPLIER_TABLE[1] };
                beeper.set_key_state(*frequency > 0.0);
            },
            AudioEventBody::SoundBlasterUpdate(settings) => {
                self.settings = *settings;
            },
            AudioEventBody::SoundBlasterUpdateChannel(channel_index, settings) => {
                self.channels[*channel_index].settings = *settings;
            },
            AudioEventBody::SoundBlasterUpdateOscillator(oscillator_index, settings) => {
                self.oscillators[*oscillator_index].settings = *settings;
            },
            AudioEventBody::SoundBlasterUpdateKeyState(oscillator_index, key_state) => {
                self.oscillators[*oscillator_index].set_key_state(*key_state);
            }
        }
        true
    }

    fn handle_audio_events(&mut self, bus: &mut crate::bus::BUS) {
        if self.audio_event.cycle_counter == u64::max_value() {
            while let Ok(audio_event) = bus.audio_event_src.try_recv() {
                self.audio_event = audio_event;
                if !self.handle_audio_event() {
                    break;
                }
            }
        } else {
            self.handle_audio_event();
        }
    }

    fn calculate_signal(&mut self) -> i16 {
        self.cycle_counter += self.clock_cycles_per_sample;
        if self.noise_value&1 != 0 {
            self.noise_value ^= 0x800302;
        }
        self.noise_value >>= 1;
        self.lfo.vibrato_phase = self.lfo.vibrato_phase.wrapping_add(1);
        let vibrato_index = (self.lfo.vibrato_phase as usize>>VIBRATO_SHIFT)%VIBRATO_MAX;
        unsafe {
            self.lfo.vibrato_sign_mask = ((VIBRATO_TABLE[vibrato_index] as i32)>>3) as u32;
            self.lfo.vibrato_shift = VIBRATO_TABLE[vibrato_index] as u8%8+self.settings.vibrato_strength;
        }
        self.lfo.tremolo_phase = self.lfo.tremolo_phase.wrapping_add(1);
        let tremolo_index = (self.lfo.tremolo_phase as u32>>TREMOLO_SHIFT)%TREMOLO_MAX;
        self.lfo.tremolo_value = (if tremolo_index < TREMOLO_MAX/2 { tremolo_index } else { TREMOLO_MAX-1-tremolo_index })>>self.settings.tremolo_strength;
        let mut signal: i32 = 0;
        let mut channel_index: usize = 0;
        while channel_index < self.channels.len() {
            let channel = &mut self.channels[channel_index];
            let phase_modulation = (((channel.prev[0]+channel.prev[1]) as u32)>>channel.settings.feedback_strength) as i32;
            channel.prev[0] = channel.prev[1];
            let signal_0 = self.oscillators[channel_index*2].calculate_signal(&self.lfo, phase_modulation);
            channel.prev[1] = signal_0;
            let signal_1 = self.oscillators[channel_index*2+1].calculate_signal(&self.lfo, match channel.settings.connection_mode {
                crate::sound_blaster::ConnectionMode::FM => channel.prev[0],
                crate::sound_blaster::ConnectionMode::AM => 0
            });
            if self.settings.rhythm_enabled && channel_index == 6 {
                let noise_bit = self.noise_value as i32&1;
                let (volume_2, phase_2) = self.oscillators[channel_index*2+2].calculate_volume_and_phase();
                let volume_3 = self.oscillators[channel_index*2+3].calculate_volume_and_phase().0;
                let mut percussion_signal = self.oscillators[channel_index*2+4].calculate_signal(&self.lfo, 0);
                let (volume_5, phase_5) = self.oscillators[channel_index*2+5].calculate_volume_and_phase();
                let phase_bit = if ((phase_2&0x88)^((phase_2<<5)&0x80))|((phase_5^(phase_5<<2))&0x20) != 0 { 2 } else { 0 };
                let phase_2 = (phase_bit<<8)|(0x34<<(phase_bit^(noise_bit<<1)));
                let phase_3 = (0x100+(phase_2&0x100))^(noise_bit<<8);
                let phase_5 = (1+phase_bit)<<8;
                percussion_signal += self.oscillators[channel_index*2+2].calculate_sample(&self.lfo, volume_2, phase_2);
                percussion_signal += self.oscillators[channel_index*2+3].calculate_sample(&self.lfo, volume_3, phase_3);
                percussion_signal += self.oscillators[channel_index*2+5].calculate_sample(&self.lfo, volume_5, phase_5);
                percussion_signal += signal_1;
                signal += percussion_signal;
                channel_index += 3;
            } else {
                signal += match channel.settings.connection_mode {
                    crate::sound_blaster::ConnectionMode::FM => signal_1,
                    crate::sound_blaster::ConnectionMode::AM => channel.prev[0]+signal_1
                };
                channel_index += 1;
            }
        }
        signal += self.oscillators[self.channels.len()*2].calculate_signal(&self.lfo, 0);
        signal as i16
    }
}

macro_rules! fill_buffer {
    ($cpu:ident, $bus:ident, $audio_renderer:ident, $buffer:ident, $signal:ident, $T:ty, $($convert:tt)*) => {
        let sample_count = $buffer.len()/$audio_renderer.output_channels;
        $audio_renderer.clock_cycles_per_sample = (CLOCK_FREQUENCY_FACTOR as f64*$bus.config.timing.clock_frequency/$audio_renderer.output_sample_rate as f64) as u64;
        $audio_renderer.cycle_counter = (CLOCK_FREQUENCY_FACTOR as i64*$cpu.cycle_counter as i64-$audio_renderer.clock_cycles_per_sample as i64*sample_count as i64).max(0) as u64;
        if $cpu.execution_state != crate::cpu::ExecutionState::Running {
            for element in $buffer.iter_mut() {
                let $signal: i16 = 0;
                *element = $($convert)*;
            }
        } else {
            for i in 0..sample_count {
                $audio_renderer.handle_audio_events($bus);
                let $signal = $audio_renderer.calculate_signal();
                for c in 0..$audio_renderer.output_channels {
                    $buffer[i*$audio_renderer.output_channels+c] = $($convert)*;
                }
            }
        }
    };
}

pub fn run_loop(cpu_ptr: usize, bus_ptr: usize) {
    let cpu = unsafe { &mut *(cpu_ptr as *mut crate::cpu::CPU) };
    let bus = unsafe { &mut *(bus_ptr as *mut crate::bus::BUS) };
    let host = cpal::default_host();
    let event_loop = host.event_loop();
    let device = host.default_output_device().unwrap();
    let mut supported_formats_range = device.supported_output_formats().unwrap();
    let format = supported_formats_range.next().unwrap().with_max_sample_rate();
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    let mut audio_renderer = AudioRenderer::new(&format);
    println!("AUDIO: channels={} sample_rate={} data_type={:?}", audio_renderer.output_channels, audio_renderer.output_sample_rate, format.data_type);
    event_loop.play_stream(stream_id).unwrap();
    event_loop.run(move |_stream_id, stream_data| {
        match stream_data.unwrap() {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                fill_buffer!(cpu, bus, audio_renderer, buffer, signal, u16, (signal-i16::min_value()) as u16);
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                fill_buffer!(cpu, bus, audio_renderer, buffer, signal, i16, signal);
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                fill_buffer!(cpu, bus, audio_renderer, buffer, signal, f32, signal as f32/i16::max_value() as f32);
            },
            _ => ()
        }
    });
}
