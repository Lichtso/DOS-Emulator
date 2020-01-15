use cpal::traits::{DeviceTrait, HostTrait, EventLoopTrait};

macro_rules! fill_buffer {
    ($cpu:ident, $bus:ident, $channels:ident, $sample_rate:ident, $beeper_phase:ident, $buffer:ident, $signal:ident, $T:ty, $($convert:tt)*) => {
        let sample_count = $buffer.len()/$channels;
        let prev_beeper_event = $bus.pit.beeper_event_buffer[($bus.pit.beeper_event_read_pos-1)%$bus.pit.beeper_event_buffer.len()];
        let clock_cycles_per_sample = ($bus.pit.clock_frequency/$sample_rate as f64) as u64;
        let mut turns_per_sample = prev_beeper_event.1 as f32/$sample_rate as f32;
        let mut cycle_counter = $cpu.cycle_counter-clock_cycles_per_sample*sample_count as u64;
        let mut beeper_active = prev_beeper_event.1 != 0.0;
        if $cpu.execution_state != crate::cpu::ExecutionState::Running {
            beeper_active = false;
        }
        for i in 0..sample_count {
            let next_beeper_event = $bus.pit.beeper_event_buffer[$bus.pit.beeper_event_read_pos];
            if $bus.pit.beeper_event_read_pos != $bus.pit.beeper_event_write_pos && cycle_counter >= next_beeper_event.0 {
                $bus.pit.beeper_event_read_pos = ($bus.pit.beeper_event_read_pos+1)%$bus.pit.beeper_event_buffer.len();
                turns_per_sample = next_beeper_event.1 as f32/$sample_rate as f32;
                beeper_active = next_beeper_event.1 != 0.0;
            }
            cycle_counter += clock_cycles_per_sample;
            let beeper = if beeper_active {
                $beeper_phase += turns_per_sample;
                (if $beeper_phase.fract() < 0.5 { -1.0 } else { 1.0 })*$bus.config.audio.beeper_volume
            } else { 0.0 };
            for c in 0..$channels {
                let $signal = beeper;
                $buffer[i*$channels+c] = $($convert)*;
            }
        }
        $beeper_phase = $beeper_phase.fract();
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
    let channels = format.channels as usize;
    let sample_rate = match format.sample_rate { cpal::SampleRate(sample_rate) => sample_rate };
    println!("AUDIO: channels={} sample_rate={} data_type={:?}", channels, sample_rate, format.data_type);
    let stream_id = event_loop.build_output_stream(&device, &format).unwrap();
    let mut beeper_phase: f32 = 0.0;
    event_loop.play_stream(stream_id).unwrap();
    event_loop.run(move |_stream_id, stream_data| {
        match stream_data.unwrap() {
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::U16(mut buffer) } => {
                fill_buffer!(cpu, bus, channels, sample_rate, beeper_phase, buffer, signal, u16, (signal*(i16::max_value() as f32)) as u16+u16::max_value()/2);
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::I16(mut buffer) } => {
                fill_buffer!(cpu, bus, channels, sample_rate, beeper_phase, buffer, signal, i16, (signal*(i16::max_value() as f32)) as i16);
            },
            cpal::StreamData::Output { buffer: cpal::UnknownTypeOutputBuffer::F32(mut buffer) } => {
                fill_buffer!(cpu, bus, channels, sample_rate, beeper_phase, buffer, signal, f32, signal);
            },
            _ => ()
        }
    });
}
