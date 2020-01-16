use chrono::prelude::*;
use serde::Deserialize;
use serde::Serialize;



#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq, Display)]
pub enum HandlerScheduleEntryKind {
    ProgrammableIntervalTimerChannel0 = 0,
    ProgrammableIntervalTimerChannel1 = 1,
    ProgrammableIntervalTimerChannel2 = 2,
    PS2Controller = 3,
    None
}

impl From<usize> for HandlerScheduleEntryKind {
    fn from(value: usize) -> Self {
        unsafe { std::mem::transmute::<u8, Self>(value as u8) }
    }
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub struct HandlerScheduleEntry {
    pub kind: HandlerScheduleEntryKind,
    pub trigger_at_cycle: u64
}

impl PartialOrd for HandlerScheduleEntry {
    fn partial_cmp(&self, other: &Self) -> Option<core::cmp::Ordering> {
        Some(other.trigger_at_cycle.cmp(&self.trigger_at_cycle))
    }
}

impl Ord for HandlerScheduleEntry {
    fn cmp(&self, other: &Self) -> core::cmp::Ordering {
        other.trigger_at_cycle.cmp(&self.trigger_at_cycle)
    }
}

pub struct HandlerSchedule {
    slots: [HandlerScheduleEntry; 4],
    next_index: usize,
    pub next_trigger_cycle: u64
}

impl HandlerSchedule {
    pub fn new() -> Self {
        Self {
            slots: [HandlerScheduleEntry{kind: HandlerScheduleEntryKind::None, trigger_at_cycle: 0}; 4],
            next_index: 0,
            next_trigger_cycle: u64::max_value(),
        }
    }

    fn update_next_trigger_cycle(&mut self) {
        self.next_trigger_cycle = u64::max_value();
        for index in 0..self.slots.len() {
            if self.slots[index].kind != HandlerScheduleEntryKind::None && self.slots[index].trigger_at_cycle < self.next_trigger_cycle {
                self.next_index = index;
                self.next_trigger_cycle = self.slots[index].trigger_at_cycle;
            }
        }
    }

    pub fn schedule_handler(&mut self, scheduled_handler: HandlerScheduleEntry) {
        self.slots[scheduled_handler.kind as usize] = scheduled_handler;
        self.update_next_trigger_cycle();
    }

    pub fn cancel_handler(&mut self, scheduled_handler_kind: HandlerScheduleEntryKind) {
        self.slots[scheduled_handler_kind as usize].kind = HandlerScheduleEntryKind::None;
        self.update_next_trigger_cycle();
    }

    pub fn get_next_to_handle(&mut self) -> HandlerScheduleEntryKind {
        let kind = self.slots[self.next_index].kind;
        self.cancel_handler(HandlerScheduleEntryKind::from(self.next_index));
        kind
    }
}

#[derive(Deserialize, Serialize)]
pub struct Config {
    pub timing: Timing,
    pub audio: Audio,
    pub keymap: toml::value::Table
}

#[derive(Deserialize, Serialize)]
pub struct Timing {
    pub clock_frequency: f64,
    pub compensation_frequency: f64,
    pub window_update_frequency: f64
}

#[derive(Deserialize, Serialize)]
pub struct Audio {
    pub enabled: bool,
    pub beeper_volume: f32
}

pub struct BUS {
    rom: [u8; 4],
    pub ram: Vec<u8>,
    pub pit: crate::pit::ProgrammableIntervalTimer,
    pub pic: crate::pic::ProgrammableInterruptController,
    pub ps2_controller: crate::ps2_controller::PS2Controller,
    pub vga: crate::vga::VideoGraphicsArray,
    pub dos: crate::dos::DiskOperatingSystem,
    pub handler_schedule: HandlerSchedule,
    pub config: Config
}

impl BUS {
    pub fn new() -> Self {
        let mut bus = Self {
            rom: [0, 0, 0, 0],
            ram: Vec::with_capacity(0xA0000),
            pit: crate::pit::ProgrammableIntervalTimer::new(),
            pic: crate::pic::ProgrammableInterruptController::new(),
            ps2_controller: crate::ps2_controller::PS2Controller::new(),
            vga: crate::vga::VideoGraphicsArray::new(),
            dos: crate::dos::DiskOperatingSystem::new(),
            handler_schedule: HandlerSchedule::new(),
            config: Config {
                timing: unsafe { std::mem::zeroed() },
                audio: unsafe { std::mem::zeroed() },
                keymap: toml::value::Table::new()
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
        } else if address-self.vga.vram_mapping.0 < self.vga.vram_mapping.1 {
            &mut self.vga.vram[address-self.vga.vram_mapping.0]
        } else {
            &mut self.rom[0]
        }
    }

    pub fn read_from_memory(&mut self, cpu: &mut crate::cpu::CPU, src: *const u8, data_width: u8) -> u32 {
        let vram_offset = src as isize-&self.vga.vram[0] as *const u8 as isize;
        if vram_offset >= 0 && vram_offset < self.vga.vram.capacity() as isize {
            let mut value = self.vga.read_from_memory(cpu.cycle_counter, vram_offset as usize) as u32;
            if data_width == 16 {
                value |= (self.vga.read_from_memory(cpu.cycle_counter, (vram_offset as usize)+1) as u32)<<8;
            }
            value
        } else {
            match data_width {
                8 => unsafe { *src as u32 },
                16 => unsafe { *(src as *const u16) as u32 },
                32 => unsafe { *(src as *const u32) },
                _ => 0
            }
        }
    }

    pub fn write_to_memory(&mut self, cpu: &mut crate::cpu::CPU, dst: *mut u8, data_width: u8, value: u32) {
        let vram_offset = dst as isize-&self.vga.vram[0] as *const u8 as isize;
        if vram_offset >= 0 && vram_offset < self.vga.vram.capacity() as isize {
            self.vga.write_to_memory(cpu.cycle_counter, vram_offset as usize, value as u8);
            if data_width == 16 {
                self.vga.write_to_memory(cpu.cycle_counter, (vram_offset as usize)+1, (value>>8) as u8);
            }
        } else {
            match data_width {
                8 => unsafe { *dst = value as u8; },
                16 => unsafe { *(dst as *mut u16) = value as u16; },
                32 => unsafe { *(dst as *mut u32) = value; },
                _ => { }
            }
        }
    }

    pub fn read_from_port(&mut self, cpu: &mut crate::cpu::CPU, address: u16) -> u8 {
        match address {
            0x0040..=0x0047 | 0x0061 => self.pit.read_from_port(cpu.cycle_counter, address),
            0x0020..=0x0021 | 0x00A0..=0x00A1 => self.pic.read_from_port(cpu.cycle_counter, address),
            0x0060 | 0x0064 => self.ps2_controller.read_from_port(cpu.cycle_counter, address),
            0x03B0..=0x03DF => self.vga.read_from_port(cpu.cycle_counter, address),
            _ => {
                println!("BUS ({}): Unsupported port read address={:04X}", cpu.cycle_counter, address);
                0
            }
        }
    }

    pub fn write_to_port(&mut self, cpu: &mut crate::cpu::CPU, address: u16, value: u8) {
        match address {
            0x0040..=0x0047 | 0x0061 => self.pit.write_to_port(cpu.cycle_counter, &mut self.handler_schedule, address, value),
            0x0020..=0x0021 | 0x00A0..=0x00A1 => self.pic.write_to_port(cpu.cycle_counter, address, value),
            0x0060 | 0x0064 => self.ps2_controller.write_to_port(cpu.cycle_counter, address, value),
            0x03B0..=0x03DF => self.vga.write_to_port(cpu.cycle_counter, address, value),
            _ => {
                println!("BUS ({}): Unsupported port write address={:04X} value={:02X}", cpu.cycle_counter, address, value);
            }
        }
    }

    pub fn handle_interrupt(&mut self, cpu: &mut crate::cpu::CPU, interrupt: u8) -> bool {
        match interrupt {
            0x10 | 0x11 | 0x16 | 0x33 => {
                crate::bios::BIOS::from_ram(&mut self.ram).handle_interrupt(cpu, &mut self.vga, interrupt);
                true
            },
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
        if self.handler_schedule.next_trigger_cycle > cpu.cycle_counter {
            return;
        }
        let kind = self.handler_schedule.get_next_to_handle();
        match kind {
            HandlerScheduleEntryKind::ProgrammableIntervalTimerChannel0 | HandlerScheduleEntryKind::ProgrammableIntervalTimerChannel1 | HandlerScheduleEntryKind::ProgrammableIntervalTimerChannel2 => {
                self.pit.scheduled_handler(cpu, &mut self.pic, &mut self.handler_schedule, kind as usize);
            },
            HandlerScheduleEntryKind::PS2Controller => {
                self.ps2_controller.pop_data(cpu, &mut self.pic, &mut self.handler_schedule).unwrap();
            },
            _ => unreachable!()
        };
    }
}
