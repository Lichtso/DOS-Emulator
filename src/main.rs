extern crate strum;
#[macro_use]
extern crate strum_macros;
extern crate chrono;
extern crate clap;
extern crate serde;
extern crate toml;
extern crate libc;
extern crate termion;
extern crate glutin;
extern crate cpal;

mod bit_utils;
mod machinecode;
mod disassembler;
mod config;
mod bus;
mod cpu;
mod dos;
mod bios;
mod pic;
mod pit;
mod ps2_controller;
mod sound_blaster;
mod vga;
mod debugger;
mod gui;
mod keyboard_mapping;
mod audio;

use termion::input::TermRead;
use std::os::unix::io::AsRawFd;
use std::io::Write;



fn main() {
    let matches = clap::App::new("dos-emulator")
        .version("0.1.0")
        .author("Alexander Meißner <AlexanderMeissner@gmx.net>")
        .about("Emulator of the IBM PC running DOS written in Rust")
        .arg(clap::Arg::with_name("config")
            .long("config")
            .help("configuration toml")
            .takes_value(true))
        .arg(clap::Arg::with_name("environment")
            .long("env")
            .help("environment variables for the guest executable")
            .multiple(true)
            .number_of_values(1))
        .arg(clap::Arg::with_name("arguments")
            .long("args")
            .help("command line arguments for the guest executable")
            .takes_value(true))
        .arg(clap::Arg::with_name("path C")
            .short("C")
            .help("Where to mount C: at")
            .takes_value(true))
        .arg(clap::Arg::with_name("executable")
            .help("Executable file to run, must be inside the mounting of C:")
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
    bus.dos.load_executable(&mut cpu, &mut bus.ram,
        std::path::Path::new(matches.value_of("executable").unwrap()),
        match matches.values_of("environment") {
            Some(environments) => environments.collect(),
            None => vec![""]
        },
        matches.value_of("arguments").unwrap_or("")
    ).unwrap();
    let clock_frequency = bus.config.timing.clock_frequency;
    let cpu_cycles_per_compensation_interval = (clock_frequency/bus.config.timing.compensation_frequency) as u64;
    let cpu_ptr = { &mut *cpu as *mut crate::cpu::CPU as usize };
    let bus_ptr = { &mut *bus as *mut crate::bus::BUS as usize };
    if bus.config.audio.beeper_enabled || bus.config.audio.sound_blaster_enabled {
        std::thread::Builder::new().name("audio".to_string()).spawn(move || {
            crate::audio::run_loop(cpu_ptr, bus_ptr);
        }).unwrap();
    }
    std::thread::Builder::new().name("cli".to_string()).spawn(move || {
        let mut stdin = termion::async_stdin();
        let stdout_fd = std::io::stdout().lock().as_raw_fd();
        let mut termios_restore: libc::termios = unsafe { std::mem::zeroed() };
        unsafe {
            libc::tcgetattr(stdout_fd, &mut termios_restore);
            let mut termios = termios_restore;
            termios.c_lflag &= !(libc::ICANON|libc::ECHO|libc::ISIG);
            libc::tcsetattr(stdout_fd, libc::TCSANOW, &mut termios);
        }
        print!("{}", termion::cursor::Hide);
        let mut last_cycle_count: u64 = 0;
        let mut last_time = std::time::SystemTime::now();
        let mut debugger = crate::debugger::Debugger::new();
        let mut keyboard_mapping = crate::keyboard_mapping::KeyboardMapping::new();
        keyboard_mapping.load_config(&bus.config);
        while !bus.terminate {
            while let Ok(event) = bus.input_event_src.try_recv() {
                match event {
                    crate::gui::InputEvent::Termination => { bus.terminate = true; },
                    crate::gui::InputEvent::Key(scancode, pressed) => {
                        keyboard_mapping.handle_gui_key(&mut cpu, &mut bus, scancode, pressed);
                    },
                    _ => {}
                }
            }
            let stdin_borrow = &mut stdin;
            for key_option in stdin_borrow.keys() {
                match key_option.unwrap() {
                    termion::event::Key::Ctrl('c') => {
                        bus.terminate = true;
                    },
                    key => {
                        if cpu.execution_state == crate::cpu::ExecutionState::Running || cpu.execution_state == crate::cpu::ExecutionState::WaitForInterrupt {
                            match key {
                                termion::event::Key::Char('p') => {
                                    debugger.pause(&mut cpu, &mut bus);
                                },
                                termion::event::Key::Char('k') => {
                                    keyboard_mapping.activate(&mut cpu);
                                },
                                _ => {}
                            }
                        } else {
                            if keyboard_mapping.mapping_tool_is_active {
                                keyboard_mapping.handle_cli_key(&mut cpu, key);
                            } else {
                                debugger.handle_input(&mut cpu, &mut bus, key);
                            }
                        }
                    }
                }
            }
            if cpu.execution_state == crate::cpu::ExecutionState::Running {
                for _ in 0..cpu_cycles_per_compensation_interval {
                    cpu.execute_instruction(&mut bus);
                    if cpu.execution_state != crate::cpu::ExecutionState::Running {
                        debugger.pause(&mut cpu, &mut bus);
                        break;
                    }
                }
                let should = (cpu.cycle_counter-last_cycle_count) as f64/clock_frequency;
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
        print!("{}{}", termion::cursor::Show, termion::style::Reset);
        std::io::stdout().flush().unwrap();
        unsafe {
            libc::tcsetattr(stdout_fd, libc::TCSANOW, &mut termios_restore);
        }
        keyboard_mapping.save_config(&mut bus.config);
        std::fs::write(&config_path, toml::to_string(&bus.config).unwrap()).unwrap();
        std::process::exit(0);
    }).unwrap();
    crate::gui::run_loop(bus_ptr);
}
