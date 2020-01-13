#![feature(ptr_offset_from)]

extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate chrono;
extern crate clap;
extern crate serde;
extern crate toml;
extern crate termion;
extern crate glutin;

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
mod vga;
mod debugger;
mod gui;
mod keyboard_mapping;

use termion::input::TermRead;
use termion::raw::IntoRawMode;



fn main() {
    let matches = clap::App::new("dos-emulator")
        .version("0.1.0")
        .author("Alexander Meißner <AlexanderMeissner@gmx.net>")
        .about("Emulator of the IBM PC running DOS written in Rust")
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
    cpu.interrupt_breakpoints[1] = true;
    cpu.interrupt_breakpoints[3] = true;
    let config_path = matches.value_of("config").map_or(std::path::Path::new("config.toml").to_path_buf(), |v| std::path::Path::new(v).to_path_buf());
    bus.config = toml::from_str(std::fs::read_to_string(&config_path).unwrap().as_str()).unwrap();
    bus.dos.mount_point_c = matches.value_of("path C").map_or(std::env::current_dir().unwrap(), |v| std::path::Path::new(v).to_path_buf());
    bus.dos.load_executable(&mut cpu, &mut bus.ram, std::path::Path::new(matches.value_of("executable").unwrap())).unwrap();
    bus.pit.clock_frequency = bus.config.timing.cpu_frequency/4.0;
    let cpu_frequency = bus.config.timing.cpu_frequency;
    let cpu_cycles_per_compensation_interval = (bus.config.timing.cpu_frequency/bus.config.timing.compensation_frequency) as u64;
    let bus_ptr = { &mut *bus as *mut crate::bus::BUS };
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::Builder::new().name("worker".to_string()).spawn(move || {
        let mut last_cycle_count: u64 = 0;
        let mut last_time = std::time::SystemTime::now();
        let mut terminate = false;
        let mut debugger = crate::debugger::Debugger::new();
        let mut keyboard_mapping = crate::keyboard_mapping::KeyboardMapping::new();
        keyboard_mapping.load_config(&bus.config);
        let mut stdin = termion::async_stdin();
        let mut stdout = std::io::stdout().into_raw_mode().unwrap();
        stdout.suspend_raw_mode().unwrap();
        while !terminate {
            while let Ok(event) = receiver.try_recv() {
                match event {
                    crate::gui::InputEvent::Termination => { terminate = true; },
                    crate::gui::InputEvent::Continue => {
                        if !keyboard_mapping.mapping_tool_is_active {
                            debugger.unpause(&mut cpu, &mut bus, &mut stdout);
                        }
                    },
                    crate::gui::InputEvent::Pause => {
                        if !keyboard_mapping.mapping_tool_is_active {
                            debugger.pause(&mut cpu, &mut bus, &mut stdout);
                        }
                    },
                    crate::gui::InputEvent::Key(scancode, pressed) => {
                        keyboard_mapping.handle_gui_key(&mut cpu, &mut bus, &mut stdout, scancode, pressed);
                    }
                }
            }
            let stdin_borrow = &mut stdin;
            for key in stdin_borrow.keys() {
                match key.unwrap() {
                    termion::event::Key::Ctrl('c') => {
                        terminate = true;
                    },
                    termion::event::Key::Char('k') => {
                        keyboard_mapping.activate(&mut cpu, &mut stdout);
                    },
                    termion::event::Key::Char('p') => {
                        debugger.pause(&mut cpu, &mut bus, &mut stdout);
                    },
                    key => {
                        if keyboard_mapping.mapping_tool_is_active {
                            keyboard_mapping.handle_cli_key(&mut stdout, key);
                        } else {
                            debugger.handle_input(&mut cpu, &mut bus, &mut stdout, key);
                        }
                    }
                }
            }
            if cpu.execution_state == crate::cpu::ExecutionState::Running {
                for _ in 0..cpu_cycles_per_compensation_interval {
                    cpu.execute_instruction(&mut bus);
                    if cpu.execution_state != crate::cpu::ExecutionState::Running {
                        debugger.pause(&mut cpu, &mut bus, &mut stdout);
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
        debugger.unpause(&mut cpu, &mut bus, &mut stdout);
        keyboard_mapping.save_config(&mut bus.config);
        std::fs::write(&config_path, toml::to_string(&bus.config).unwrap()).unwrap();
        std::process::exit(0);
    }).unwrap();
    crate::gui::run_loop(sender, bus_ptr);
}
