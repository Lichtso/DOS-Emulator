extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate chrono;
extern crate clap;
extern crate serde;
extern crate toml;

mod bit_utils;
mod machinecode;
mod disassembler;
mod bus;
mod cpu;
mod dos;
mod bios;
mod pic;
mod pit;
mod ps2_controller;

fn main() {
    let matches = clap::App::new("DOS Emulator")
        .version("0.1.0")
        .author("Alexander Meißner <AlexanderMeissner@gmx.net>")
        .about("DOSBox clone written in Rust")
        .arg(clap::Arg::with_name("config")
            .long("config")
            .help("configuration toml")
            .takes_value(true))
        .arg(clap::Arg::with_name("path C")
            .short("C")
            .help("Where to mount C: at")
            .takes_value(true))
        .arg(clap::Arg::with_name("executable")
            .help("Executable file to run, must be inside root")
            .required(true)
            .index(1))
        .get_matches();
    let mut cpu = Box::new(crate::cpu::CPU::new());
    let mut bus = Box::new(crate::bus::BUS::new());
    let config_path = matches.value_of("config").map_or(std::path::Path::new("config.toml").to_path_buf(), |v| std::path::Path::new(v).to_path_buf());
    bus.config = toml::from_str(std::fs::read_to_string(config_path).unwrap().as_str()).unwrap();
    bus.dos.mount_point_c = matches.value_of("path C").map_or(std::env::current_dir().unwrap(), |v| std::path::Path::new(v).to_path_buf());
    bus.dos.load_executable(&mut cpu, &mut bus.ram, std::path::Path::new(matches.value_of("executable").unwrap())).unwrap();
    let cpu_frequency = bus.config.timing.cpu_frequency;
    let cpu_cycles_per_compensation_interval = (bus.config.timing.cpu_frequency/bus.config.timing.compensation_frequency) as u64;
    std::thread::Builder::new().name("worker".to_string()).spawn(move || {
        let mut last_cycle_count: u64 = 0;
        let mut last_time = std::time::SystemTime::now();
        loop {
            if cpu.execution_state == crate::cpu::ExecutionState::Running {
                for _ in 0..cpu_cycles_per_compensation_interval {
                    cpu.execute_instruction(&mut bus);
                    if cpu.execution_state != crate::cpu::ExecutionState::Running {
                        break;
                    }
                }
                let should = (cpu.cycle_counter-last_cycle_count) as f64/cpu_frequency;
                let elapsed = last_time.elapsed().unwrap().as_secs_f64();
                let compensation_delay = (should-elapsed).max(0.0);
                if compensation_delay > 0.0 {
                    std::thread::sleep(std::time::Duration::from_secs_f64(compensation_delay));
                }
                last_cycle_count = cpu.cycle_counter;
                last_time = std::time::SystemTime::now();
            } else {
                std::thread::sleep(std::time::Duration::from_millis(50));
            }
        }
    }).unwrap();
}
