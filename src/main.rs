extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate chrono;
extern crate clap;

mod bit_utils;
mod machinecode;
mod disassembler;
mod bus;
mod cpu;
mod dos;

fn main() {
    let matches = clap::App::new("DOS Emulator")
        .version("0.1.0")
        .author("Alexander Meißner <AlexanderMeissner@gmx.net>")
        .about("DOSBox clone written in Rust")
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
    bus.dos.mount_point_c = matches.value_of("path C").map_or(std::env::current_dir().unwrap(), |v| std::path::Path::new(v).to_path_buf());
    bus.dos.load_executable(&mut cpu, &mut bus.ram, std::path::Path::new(matches.value_of("executable").unwrap())).unwrap();
    std::thread::Builder::new().name("worker".to_string()).spawn(move || {
        loop {
            if cpu.execution_state == crate::cpu::ExecutionState::Running {
                cpu.execute_instruction(&mut bus);
            } else {
                return;
            }
        }
    }).unwrap();
}
