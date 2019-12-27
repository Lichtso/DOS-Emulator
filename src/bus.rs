use chrono::prelude::*;
use serde::Deserialize;



#[derive(Deserialize)]
pub struct Config {
    pub timing: Timing
}

#[derive(Deserialize)]
pub struct Timing {
    pub cpu_frequency: f64,
    pub compensation_frequency: f64,
    pub window_update_frequency: f64
}

pub struct BUS {
    rom: [u8; 4],
    pub ram: Vec<u8>,
    pub dos: crate::dos::DiskOperatingSystem,
    pub config: Config
}

impl BUS {
    pub fn new() -> Self {
        let mut bus = Self {
            rom: [0, 0, 0, 0],
            ram: Vec::with_capacity(0xA0000),
            dos: crate::dos::DiskOperatingSystem::new(),
            config: Config {
                timing: unsafe { std::mem::zeroed() }
            }
        };
        bus.ram.resize(bus.ram.capacity(), 0);
        bus
    }

    pub fn physical_address(segment: u16, offset: u16) -> usize {
        ((segment as usize)<<4)+(offset as usize)
    }

    pub fn get_memory(&mut self, _cpu: &mut crate::cpu::CPU, address: usize) -> *mut u8 {
        if address < self.ram.capacity() {
            &mut self.ram[address]
        } else {
            &mut self.rom[0]
        }
    }

    pub fn read_from_memory(&mut self, cpu: &mut crate::cpu::CPU, src: *const u8, data_width: u8) -> u32 {
        match data_width {
            8 => unsafe { *src as u32 },
            16 => unsafe { *(src as *const u16) as u32 },
            32 => unsafe { *(src as *const u32) },
            _ => 0
        }
    }

    pub fn write_to_memory(&mut self, cpu: &mut crate::cpu::CPU, dst: *mut u8, data_width: u8, value: u32) {
        match data_width {
            8 => unsafe { *dst = value as u8; },
            16 => unsafe { *(dst as *mut u16) = value as u16; },
            32 => unsafe { *(dst as *mut u32) = value; },
            _ => { }
        }
    }

    pub fn read_from_port(&mut self, cpu: &mut crate::cpu::CPU, address: u16) -> u8 {
        0
    }

    pub fn write_to_port(&mut self, cpu: &mut crate::cpu::CPU, address: u16, value: u8) {

    }

    pub fn handle_interrupt(&mut self, cpu: &mut crate::cpu::CPU, interrupt: u8) -> bool {
        match interrupt {
            0x1A => match cpu.get_register(crate::machinecode::Operand::AX)>>8 {
                0x00 => {
                    let now = chrono::Local::now();
                    let time = ((now.hour()*3600+now.minute()*60+now.second()) as u64)*1573040/86400;
                    cpu.set_register(crate::machinecode::Operand::CX, (time>>16) as u16);
                    cpu.set_register(crate::machinecode::Operand::BX, time as u16);
                    println!("Clock ({}): Get System Time", cpu.cycle_counter);
                    true
                },
                _ => false
            },
            0x20 => {
                println!("DOS ({}): Exit", cpu.cycle_counter);
                std::process::exit(0);
            },
            0x21 => {
                self.dos.handle_interrupt(cpu, &mut self.ram);
                true
            },
            _ => {
                panic!("BUS ({}): Unsupported interrupt={:#02X} AX={:04X} ip/pc={:04X}:{:04X}", cpu.cycle_counter, interrupt, cpu.get_register(crate::machinecode::Operand::AX), cpu.get_register(crate::machinecode::Operand::CS), cpu.instruction.position-cpu.instruction.length as u16);
            }
        }
    }

    pub fn tick(&mut self, cpu: &mut crate::cpu::CPU) {

    }
}
