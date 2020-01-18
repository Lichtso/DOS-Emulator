use cpal::traits::{DeviceTrait, HostTrait, EventLoopTrait};

pub enum AudioEventBody {
    Beeper(f32)
}

pub struct AudioEvent {
    pub cycle_counter: u64,
    pub body: AudioEventBody
}

struct AudioRenderer {
    output_channels: usize,
    output_sample_rate: u32,
    clock_cycles_per_sample: u64,
    cycle_counter: u64,
    audio_event: AudioEvent,
    beeper_turns_per_sample: f32,
    beeper_phase: f32,
    beeper_active: bool,
}

macro_rules! fill_buffer {
    ($cpu:ident, $bus:ident, $audio_renderer:ident, $buffer:ident, $signal:ident, $T:ty, $($convert:tt)*) => {
        let sample_count = $buffer.len()/$audio_renderer.output_channels;
        $audio_renderer.clock_cycles_per_sample = ($bus.pit.clock_frequency/$audio_renderer.output_sample_rate as f64) as u64;
        $audio_renderer.cycle_counter = $cpu.cycle_counter-$audio_renderer.clock_cycles_per_sample*sample_count as u64;
        if $cpu.execution_state != crate::cpu::ExecutionState::Running {
            for element in $buffer.iter_mut() {
                let $signal: f32 = 0.0;
                *element = $($convert)*;
            }
        } else {
            for i in 0..sample_count {
                if $audio_renderer.audio_event.cycle_counter <= $audio_renderer.cycle_counter {
                    match $audio_renderer.audio_event.body {
                        AudioEventBody::Beeper(beeper_frequency) => {
                            $audio_renderer.beeper_turns_per_sample = beeper_frequency as f32/$audio_renderer.output_sample_rate as f32;
                            $audio_renderer.beeper_active = beeper_frequency != 0.0;
                        }
                    }
                    while let Ok(audio_event) = $bus.audio_event_src.try_recv() {
                        $audio_renderer.audio_event = audio_event;
                        if $audio_renderer.audio_event.cycle_counter > $audio_renderer.cycle_counter {
                            break;
                        }
                    }
                }
                $audio_renderer.cycle_counter += $audio_renderer.clock_cycles_per_sample;
                let beeper = if $audio_renderer.beeper_active {
                    $audio_renderer.beeper_phase += $audio_renderer.beeper_turns_per_sample;
                    (if $audio_renderer.beeper_phase.fract() < 0.5 { -1.0 } else { 1.0 })*$bus.config.audio.beeper_volume
                } else { 0.0 };
                for c in 0..$audio_renderer.output_channels {
                    let $signal = beeper;
                    $buffer[i*$audio_renderer.output_channels+c] = $($convert)*;
                }
            }
            $audio_renderer.beeper_phase = $audio_renderer.beeper_phase.fract();
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
    let mut audio_renderer = AudioRenderer {
        output_channels: format.channels as usize,
        output_sample_rate: match format.sample_rate { cpal::SampleRate(sample_rate) => sample_rate },
        clock_cycles_per_sample: 0,
        cycle_counter: 0,
        audio_event: AudioEvent {
            cycle_counter: 0,
            body: AudioEventBody::Beeper(0.0)
        },
        beeper_turns_per_sample: 0.0,
        beeper_phase: 0.0,
        beeper_active: false
    };
    println!("AUDIO: channels={} sample_rate={} data_type={:?}", audio_renderer.output_channels, audio_renderer.output_sample_rate, format.data_type);
    event_loop.play_stream(stream_id).unwrap();
    event_loop.run(move |_stream_id, stream_data| {
        match stream_data.unwrap() {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                fill_buffer!(cpu, bus, audio_renderer, buffer, signal, u16, (signal*(i16::max_value() as f32)) as u16+u16::max_value()/2);
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                fill_buffer!(cpu, bus, audio_renderer, buffer, signal, i16, (signal*(i16::max_value() as f32)) as i16);
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                fill_buffer!(cpu, bus, audio_renderer, buffer, signal, f32, signal);
            },
            _ => ()
        }
    });
}
