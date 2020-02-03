use std::io::Write;

use crate::machinecode::Operand;
use crate::cpu::Flag;

pub struct Debugger {
    instruction: crate::machinecode::Instruction,
    break_points: std::collections::HashMap<u32, u16>,
    data_view_segment: u16,
    data_view_offset: u16
}

impl Debugger {
    pub fn new() -> Self {
        Self {
            instruction: unsafe { std::mem::zeroed() },
            break_points: std::collections::HashMap::new(),
            data_view_segment: 0x1D2B,
            data_view_offset: 0x0000
        }
    }

    pub fn set_break_point_at(&mut self, cpu: &mut crate::cpu::CPU, bus: &mut crate::bus::BUS, segment: u16, offset: u16) -> bool {
        let address = ((segment as u32)<<16)|(offset as u32);
        if self.break_points.contains_key(&address) {
            return false;
        }
        let ptr = bus.get_memory(cpu, crate::bus::BUS::physical_address(segment, offset));
        self.break_points.insert(address, bus.read_from_memory(cpu, ptr, 16) as u16);
        bus.write_to_memory(cpu, ptr, 16, 0x03CD); // INT 3
        true
    }

    pub fn remove_break_point_at(&mut self, cpu: &mut crate::cpu::CPU, bus: &mut crate::bus::BUS, segment: u16, offset: u16) -> bool {
        let address = ((segment as u32)<<16)|(offset as u32);
        match self.break_points.remove(&address) {
            Some(original_machinecode) => {
                let ptr = bus.get_memory(cpu, crate::bus::BUS::physical_address(segment, offset));
                bus.write_to_memory(cpu, ptr, 16, original_machinecode as u32);
                true
            },
            None => false
        }
    }

    pub fn pause(&mut self, cpu: &mut crate::cpu::CPU, bus: &mut crate::bus::BUS) {
        match cpu.execution_state {
            crate::cpu::ExecutionState::Paused => { return; },
            crate::cpu::ExecutionState::Running => { cpu.execution_state = crate::cpu::ExecutionState::Paused; },
            crate::cpu::ExecutionState::DebuggerHandlesInterrupt(interrupt) => {
                if interrupt == 3 {
                    cpu.instruction.position = cpu.instruction.position.wrapping_sub(cpu.instruction.length as u16);
                    self.remove_break_point_at(cpu, bus, cpu.get_register(Operand::CS), cpu.instruction.position);
                }
                cpu.execution_state = crate::cpu::ExecutionState::Paused;
            },
            _ => {}
        }
        print!("{}{}{}", termion::cursor::Goto(1, 1), termion::clear::All, termion::style::Invert);
        print!("{}---(Register Overview                   )---", termion::cursor::Goto(1, 1));
        print!("{}---(Data Overview   Scroll: page up/down)---", termion::cursor::Goto(1, 6));
        print!("{}---(Code Overview                       )---", termion::cursor::Goto(1, 17));
        print!("{}---(Call Stack                          )---", termion::cursor::Goto(1, 29));
        print!("{}", termion::style::Reset);
        self.render(cpu, bus);
    }

    pub fn unpause(&mut self, cpu: &mut crate::cpu::CPU, _bus: &mut crate::bus::BUS) {
        if cpu.execution_state == crate::cpu::ExecutionState::Running {
            return;
        }
        cpu.execution_state = crate::cpu::ExecutionState::Running;
        print!("{}{}", termion::cursor::Goto(1, 1), termion::clear::All);
        std::io::stdout().flush().unwrap();
    }

    pub fn render(&mut self, cpu: &mut crate::cpu::CPU, bus: &mut crate::bus::BUS) {
        println!("{}EAX={:08X}  ESI={:08X}  DS={:04X}   ES={:04X}                       SS={:04X}",
            termion::cursor::Goto(1, 2),
            cpu.get_register(Operand::AX),
            cpu.get_register(Operand::SI),
            cpu.get_register(Operand::DS),
            cpu.get_register(Operand::ES),
            cpu.get_register(Operand::SS)
        );
        println!("EBX={:08X}  EDI={:08X}  CS={:04X}   EIP={:08X}  C{} Z{} S{} O{} A{} P{} D{} I{} T{}",
            cpu.get_register(Operand::BX),
            cpu.get_register(Operand::DI),
            cpu.get_register(Operand::CS),
            cpu.instruction.position,
            cpu.get_flag(Flag::Carry) as u8,
            cpu.get_flag(Flag::Zero) as u8,
            cpu.get_flag(Flag::Sign) as u8,
            cpu.get_flag(Flag::Overflow) as u8,
            cpu.get_flag(Flag::AuxiliaryCarry) as u8,
            cpu.get_flag(Flag::Parity) as u8,
            cpu.get_flag(Flag::Direction) as u8,
            cpu.get_flag(Flag::Interrupt) as u8,
            cpu.get_flag(Flag::Trap) as u8
        );
        println!("ECX={:08X}  EBP={:08X}  res={:04X}  v1={:04X} v2={:04X}",
            cpu.get_register(Operand::CX),
            cpu.get_register(Operand::BP),
            cpu.result_value,
            cpu.first_value,
            cpu.second_value
        );
        print!("EDX={:08X}  ESP={:08X}                                  {}",
            cpu.get_register(Operand::DX),
            cpu.get_register(Operand::SP),
            cpu.cycle_counter
        );
        {
            print!("{}", termion::cursor::Goto(1, 8));
            let mut buffer: [u8; 16] = unsafe { std::mem::zeroed() };
            for line in 0..8 {
                let offset = self.data_view_offset+line*16;
                print!("{:04X}:{:08X}", self.data_view_segment, offset);
                for i in 0..16 {
                    let address = crate::bus::BUS::physical_address(self.data_view_segment, offset.wrapping_add(i));
                    buffer[i as usize] = if address >= bus.vga.vram_mapping.0 {
                        bus.vga.vram[address-bus.vga.vram_mapping.0]
                    } else {
                        bus.ram[address]
                    };
                    print!(" {:02X}", buffer[i as usize]);
                }
                print!("  ");
                for i in 0..16 {
                    print!("{}", if buffer[i] >= 32 && buffer[i] < 127 { buffer[i] as char } else { '⋅' });
                }
                println!("");
            }
        }
        {
            print!("{}", termion::cursor::Goto(1, 18));
            self.instruction.position = cpu.instruction.position;
            let position = crate::bus::BUS::physical_address(cpu.get_register(Operand::CS), cpu.instruction.position) as usize;
            let mut buffer = &bus.ram[position..];
            for _line in 0..10 {
                if !crate::machinecode::decode_instruction(&mut buffer, &mut self.instruction).is_ok() {
                    break;
                }
                println!("{}{:04X}:{:04X}  {}", termion::clear::CurrentLine, cpu.get_register(Operand::CS), self.instruction.position, self.instruction);
                self.instruction.position += self.instruction.length as u16;
            }
        }
        {
            print!("{}", termion::cursor::Goto(1, 30));
            let mut frame_offset = cpu.get_register(Operand::BP);
            for _line in 0..4 {
                let address = crate::bus::BUS::physical_address(cpu.get_register(Operand::SS), frame_offset);
                let bp = crate::bit_utils::read_from_buffer_u16(&bus.ram[address..]);
                let ippc = crate::bit_utils::read_from_buffer_u16(&bus.ram[address+2..]);
                let cs = crate::bit_utils::read_from_buffer_u16(&bus.ram[address+4..]);
                let flags = crate::bit_utils::read_from_buffer_u16(&bus.ram[address+6..]);
                println!("{}{:04X}:{:04X}  {:04X} {:04X} {:04X}", termion::clear::CurrentLine, cpu.get_register(Operand::SS), frame_offset, ippc, cs, flags);
                frame_offset = bp;
            }
        }
        print!("{}{}", termion::cursor::Goto(1, 35), termion::clear::AfterCursor);
        std::io::stdout().flush().unwrap();
    }

    pub fn handle_input(&mut self, cpu: &mut crate::cpu::CPU, bus: &mut crate::bus::BUS, key: termion::event::Key) {
        match key {
            termion::event::Key::Char('p') => {
                let mut file = std::fs::File::create(&std::path::Path::new("profile.txt")).unwrap();
                for (entry, counter) in &cpu.instruction_profile {
                    write!(file, "{}: {}\n", entry, counter).unwrap();
                }
                cpu.instruction_profile_recording = true;
                return;
            },
            termion::event::Key::Char('a') => {
                self.data_view_segment = cpu.get_register(Operand::DS);
                self.data_view_offset = cpu.get_register(Operand::SI)&0xFFF0;
            },
            termion::event::Key::Char('s') => {
                self.data_view_segment = cpu.get_register(Operand::SS);
                self.data_view_offset = cpu.get_register(Operand::SP)&0xFFF0;
            },
            termion::event::Key::Char('d') => {
                self.data_view_segment = cpu.get_register(Operand::ES);
                self.data_view_offset = cpu.get_register(Operand::DI)&0xFFF0;
            },
            termion::event::Key::PageUp => {
                self.data_view_offset = self.data_view_offset.wrapping_sub(0x10);
            },
            termion::event::Key::PageDown => {
                self.data_view_offset = self.data_view_offset.wrapping_add(0x10);
            },
            termion::event::Key::F(5) => {
                self.unpause(cpu, bus);
                return;
            },
            termion::event::Key::F(10) => {
                self.instruction.position = cpu.instruction.position;
                let position = crate::bus::BUS::physical_address(cpu.get_register(Operand::CS), cpu.instruction.position) as usize;
                let mut buffer = &bus.ram[position..];
                crate::machinecode::decode_instruction(&mut buffer, &mut self.instruction).unwrap();
                self.instruction.position = self.instruction.position.wrapping_add(self.instruction.length as u16);
                self.set_break_point_at(cpu, bus, cpu.get_register(Operand::CS), self.instruction.position);
                self.unpause(cpu, bus);
                return;
            },
            termion::event::Key::F(11) => {
                cpu.execute_instruction(bus);
            },
            _ => {}
        }
        self.render(cpu, bus);
    }
}
