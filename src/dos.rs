use std::mem;
use std::slice;
use std::io::prelude::*;
use chrono::prelude::*;

use crate::machinecode::Operand;
use crate::cpu::Flag;

#[repr(C, packed)]
struct MZDOS {
    magic: u16,
    bytes_in_last_page: u16,
    page_count: u16, // *512
    relocation_count: u16,
    code_offset: u16, // *16
    minimum_allocation: u16, // *16
    maximum_allocation: u16, // *16
    initial_ss: u16,
    initial_sp: u16,
    checksum: u16,
    initial_ip: u16,
    initial_cs: u16,
    relocation_table_offset: u16,
    overlay: u16
}

#[repr(C, packed)]
pub struct ProgramSegmentPrefix {
    int20: [u8; 2],
    allocation_end: u16,
    reserved0: [u8; 1],
    int21: [u8; 5],
    previous_program_terminate_address: u32,
    previous_program_break_address: u32,
    previous_program_critical_error_address: u32,
    parent_psp_segment: u16,
    job_file_table: [u8; 20],
    environment_segment: u16,
    stack_restore_offset: u16,
    stack_restore_segment: u16,
    job_file_table_size: u16,
    job_file_table_ptr: u32,
    previous_psp: u32,
    reserved1: [u8; 4],
    dos_version_to_return: u16,
    reserved2: [u8; 14],
    int21retf: [u8; 3],
    reserved3: [u8; 2],
    fcb1_extension: [u8; 7],
    fcb1: [u8; 16],
    fcb2: [u8; 20],
    parameter_length: u8,
    parameter: [u8; 127]
}

#[repr(C, packed)]
struct FindFirstDataBlock {
    drive_letter: u8,
    search_template: [u8; 11],
    search_attributes: u8,
    entry_index_in_directory: u16,
    parent_directory_cluster_number: u16,
    reserved: [u8; 4],
    attribute: u8,
    file_time: u16,
    file_date: u16,
    file_size: u32,
    filename: [u8; 13]
}

fn path_from_ptr(mount_point_c: &std::path::Path, ptr: *const u8) -> std::result::Result<std::path::PathBuf, &'static str> {
    let path = match unsafe { std::ffi::CStr::from_ptr(ptr as *const i8) }.to_str() {
        Ok(path) => path,
        Err(_) => { return Err("Could not read file path"); }
    };
    if &path[0..3] != "C:\\" {
        return Err("File path does not start with C:\\");
    }
    Ok(mount_point_c.join(&path[3..].replace("\\", "/")))
}

macro_rules! get_path {
    ($dos:ident, $cpu:ident, $ram:ident) => {
        {
            let address = crate::bus::BUS::physical_address($cpu.get_register(Operand::DS), $cpu.get_register(Operand::DX));
            match path_from_ptr(&$dos.mount_point_c, &mut $ram[address as usize]) {
                Ok(path) => path,
                Err(_) => {
                    println!("FS ({}): err=(Path not found)", $cpu.cycle_counter);
                    $cpu.set_register(Operand::AX, 3); // Path not found
                    return;
                }
            }
        }
    };
}

macro_rules! access_path {
    ($path:expr, $cpu:ident,  $value:ident, $operation_name:expr, $operation:expr, $($on_success:tt)*) => {
        match $operation(&$path) {
            Ok($value) => {
                println!("FS ({}): {} path={:?}", $cpu.cycle_counter, $operation_name, $path);
                $cpu.reset_flag(Flag::Carry);
                $($on_success)*
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => {
                println!("FS ({}): {} path={:?} err=(File not found)", $cpu.cycle_counter, $operation_name, $path);
                $cpu.set_register(Operand::AX, 2); // File not found
                return;
            },
            Err(ref e) if e.kind() == std::io::ErrorKind::PermissionDenied => {
                println!("FS ({}): {} path={:?} err=(Access denied)", $cpu.cycle_counter, $operation_name, $path);
                $cpu.set_register(Operand::AX, 5); // Access denied
                return;
            },
            Err(_) => { panic!(); }
        };
    };
}

pub struct DiskOperatingSystem {
    pub load_segment: u16,
    pub psp_segment: u16,
    pub dta_address: u32,
    pub open_handles: std::collections::HashMap<u16, std::fs::File>,
    pub mount_point_c: std::path::PathBuf,
    read_directory: Option<std::fs::ReadDir>,
    keyboard_spill: u8
}

impl DiskOperatingSystem {
    pub fn new() -> Self {
        Self {
            load_segment: 0,
            psp_segment: 0,
            dta_address: 0,
            open_handles: std::collections::HashMap::new(),
            mount_point_c: std::path::PathBuf::new(),
            read_directory: None,
            keyboard_spill: 0
        }
    }

    fn get_psp(psp_segment: usize, ram: &mut [u8]) -> &mut ProgramSegmentPrefix {
        unsafe { &mut *std::mem::transmute::<*mut u8, *mut ProgramSegmentPrefix>(&mut ram[psp_segment << 4] as *mut u8) }
    }

    pub fn load_executable(&mut self, cpu: &mut crate::cpu::CPU, ram: &mut [u8], executable_path: &std::path::Path) -> std::io::Result<()> {
        let mut file = std::fs::File::open(executable_path)?;
        let mut mz_dos: MZDOS = unsafe { mem::zeroed() };
        unsafe {
            let slice = slice::from_raw_parts_mut(&mut mz_dos as *mut _ as *mut u8, mem::size_of::<MZDOS>());
            file.read_exact(slice)?;
        };
        {
            self.load_segment = 0x01A2;
            self.psp_segment = 0x0192;
            self.dta_address = ((self.psp_segment as u32)<<16)+0x80;
            cpu.set_register(Operand::CX, 0x00FF);
            cpu.set_register(Operand::DX, self.psp_segment);
            cpu.set_register(Operand::SP, mz_dos.initial_sp);
            cpu.set_register(Operand::BP, 0x091C);
            cpu.set_register(Operand::DI, 0x0080);
            cpu.set_register(Operand::ES, self.psp_segment);
            cpu.set_register(Operand::CS, mz_dos.initial_cs+(self.load_segment as u16));
            cpu.set_register(Operand::SS, mz_dos.initial_ss+self.load_segment);
            cpu.set_register(Operand::DS, self.psp_segment);
            cpu.instruction.position = mz_dos.initial_ip;
        }
        // Load Code from File
        {
            let code_begin = mz_dos.code_offset as usize*16;
            let mut code_end = mz_dos.page_count as usize*512;
            if mz_dos.bytes_in_last_page > 0 {
                code_end -= 512-mz_dos.bytes_in_last_page as usize;
            }
            file.seek(std::io::SeekFrom::Start(code_begin as u64))?;
            let load_address = crate::bus::BUS::physical_address(self.load_segment, 0);
            file.read_exact(&mut ram[load_address..load_address+(code_end-code_begin) as usize])?;
        }
        // Relocation
        {
            file.seek(std::io::SeekFrom::Start(mz_dos.relocation_table_offset as u64))?;
            let mut buffer: [u8; 2] = [0, 0];
            for _ in 0..mz_dos.relocation_count {
                file.read_exact(&mut buffer)?;
                let offset = crate::bit_utils::read_from_buffer_u16(&buffer);
                file.read_exact(&mut buffer)?;
                let segment = crate::bit_utils::read_from_buffer_u16(&buffer);
                let address = crate::bus::BUS::physical_address(self.load_segment+segment, offset);
                let value = crate::bit_utils::read_from_buffer_u16(&ram[address..]);
                crate::bit_utils::write_to_buffer_u16(&mut ram[address..], value+self.load_segment as u16);
            }
        }
        // Setup Environment
        {
            let psp = Self::get_psp(self.psp_segment as usize, ram);
            psp.int20 = [0xCD, 0x20];
            psp.allocation_end = 0x9FFF; // mz_dos.minimum_allocation;
            psp.reserved0 = [0x00];
            psp.int21 = [0xEA, 0xFF, 0xFF, 0xAD, 0xDE];
            psp.parent_psp_segment = 0x0118;
            psp.previous_program_terminate_address = 0xF00020C8;
            psp.previous_program_break_address = ((*psp).parent_psp_segment as u32)<<16;
            psp.previous_program_critical_error_address = (((*psp).parent_psp_segment as u32)<<16)|0x0110;
            psp.job_file_table = [0x01, 0x01, 0x01, 0x00, 0x02, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF, 0xFF];
            psp.environment_segment = 0x0188;
            psp.stack_restore_offset = 0x0000;
            psp.stack_restore_segment = 0x0000;
            psp.job_file_table_size = 0x0014;
            psp.job_file_table_ptr = ((self.psp_segment as u32)<<16)|0x0018;
            psp.previous_psp = 0xFFFFFFFF;
            psp.reserved1 = [0x00, 0x00, 0x00, 0x00];
            psp.dos_version_to_return = 0x0005;
            psp.reserved2 = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            psp.int21retf = [0xCD, 0x21, 0xCB];
            psp.reserved3 = [0x00, 0x00];
            psp.fcb1_extension = [0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            psp.fcb1 = [0x00, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0x00];
            psp.fcb2 = [0x00, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x20, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00];
            psp.parameter_length = 0x00;
            psp.parameter[psp.parameter_length as usize] = 0x0D;
            let path = std::path::Path::new("C:").join(executable_path.strip_prefix(&self.mount_point_c).unwrap()).to_str().unwrap().replace("/", "\\");
            let mut environment = "PATH=Z:\\\0COMSPEC=Z:\\COMMAND.COM\0BLASTER=A220 I7 D1 H5 T6\0\0\x01\0".to_string();
            environment.push_str(path.as_str());
            let environment_data = environment.as_bytes();
            let environment_address = crate::bus::BUS::physical_address(psp.environment_segment, 0);
            ram[environment_address..environment_address+environment_data.len()].copy_from_slice(environment_data);
        }
        // Setup BIOS
        crate::bios::BIOS::from_ram(ram).setup();
        // Setup Interrupt Vector
        let interrupt_vector = unsafe { crate::bit_utils::transmute_slice_mut::<u8, u32>(&mut ram[0..]) };
        interrupt_vector[8] = 0xF000FEA5;
        interrupt_vector[9] = 0xF000E987;
        Ok(())
    }

    fn find_next_directory_entry<'a>(&mut self, pattern: &'a str) -> Option<std::path::PathBuf> {
        let read_directory = self.read_directory.as_mut().unwrap();
        match pattern.find("*") {
            Some(index) => {
                let prefix = &pattern[0..index];
                let suffix = &pattern[index+1..];
                loop {
                    match read_directory.next() {
                        Some(entry_result) => {
                            let entry = entry_result.unwrap();
                            let file_name = entry.file_name();
                            let file_name_str = file_name.to_str().unwrap();
                            if file_name_str.starts_with(prefix) && file_name_str.ends_with(suffix) {
                                return Some(entry.path());
                            }
                        },
                        None => { break; }
                    }
                }
            },
            None => {}
        }
        return None;
    }

    pub fn handle_interrupt(&mut self, cpu: &mut crate::cpu::CPU, ram: &mut [u8]) {
        let argument = cpu.get_register(Operand::AL);
        match cpu.get_register(Operand::AH) {
            0x00 => { // Exit
                println!("DOS ({}): Exit", cpu.cycle_counter);
                std::process::exit(0);
            },
            0x07 => { // Direct Character Input (No Echo)
                let bios = crate::bios::BIOS::from_ram(ram);
                if self.keyboard_spill != 0 {
                    cpu.set_register(Operand::AL, self.keyboard_spill as u16);
                    self.keyboard_spill = 0;
                } else {
                    let key_code = bios.keyboard_buffer_pop().unwrap_or(0);
                    cpu.set_register(Operand::AL, key_code&0xFF);
                    self.keyboard_spill = if key_code == 0 || key_code&0xFF != 0 { 0 } else { (key_code>>8) as u8 };
                }
            },
            0x1A => { // Set DTA address
                self.dta_address = ((cpu.get_register(Operand::DS) as u32)<<16)+(cpu.get_register(Operand::DX) as u32);
            },
            0x25 => { // Set Interrupt Handler
                crate::bit_utils::write_to_buffer_u16(&mut ram[argument as usize*4..], cpu.get_register(Operand::DX));
                crate::bit_utils::write_to_buffer_u16(&mut ram[argument as usize*4+2..], cpu.get_register(Operand::DS));
                println!("DOS ({}): Set interrupt={:#04X} handler={:04X}:{:04X}", cpu.cycle_counter, argument, cpu.get_register(Operand::DS), cpu.get_register(Operand::DX));
            },
            0x2F => { // Get DTA address
                cpu.set_register(Operand::BX, self.dta_address as u16);
                cpu.set_register(Operand::ES, (self.dta_address>>16) as u16);
            },
            0x30 => { // Get DOS Version
                cpu.set_register(Operand::AX, 0x0005);
                cpu.set_register(Operand::CX, 0x0000);
                cpu.set_register(Operand::BX, 0xFF00);
            },
            0x35 => { // Get Interrupt Handler
                let address = crate::bit_utils::read_from_buffer_u32(&ram[argument as usize*4..]);
                cpu.set_register(Operand::BX, address as u16);
                cpu.set_register(Operand::ES, (address>>16) as u16);
                println!("DOS ({}): Get interrupt={:#04X} handler={:04X}:{:04X}", cpu.cycle_counter, argument, cpu.get_register(Operand::BX), cpu.get_register(Operand::ES));
            },
            0x3C => { // Create or Truncate File
                cpu.set_flag(Flag::Carry, 1);
                if self.open_handles.len() >= 15 {
                    println!("FS ({}): Create or Truncate File err=(No handle available)", cpu.cycle_counter);
                    cpu.set_register(Operand::AX, 4);
                    return;
                }
                access_path!(get_path!(self, cpu, ram), cpu, file, "Create or Truncate File", std::fs::File::create, {
                    for i in 5..20 {
                        if !self.open_handles.contains_key(&i) {
                            cpu.set_register(Operand::AX, i);
                            break;
                        }
                    }
                    self.open_handles.insert(cpu.get_register(Operand::AX), file);
                });
            },
            0x3D => { // Open Existing File
                cpu.set_flag(Flag::Carry, 1);
                if self.open_handles.len() >= 15 {
                    println!("FS ({}): Open Existing File err=(No handle available)", cpu.cycle_counter);
                    cpu.set_register(Operand::AX, 4);
                    return;
                }
                access_path!(get_path!(self, cpu, ram), cpu, file, "Open Existing File", std::fs::File::open, {
                    for i in 5..20 {
                        if !self.open_handles.contains_key(&i) {
                            cpu.set_register(Operand::AX, i);
                            break;
                        }
                    }
                    self.open_handles.insert(cpu.get_register(Operand::AX), file);
                });
            },
            0x3E => { // Close File
                if self.open_handles.contains_key(&cpu.get_register(Operand::BX)) {
                    println!("FS ({}): Close File fd={}", cpu.cycle_counter, cpu.get_register(Operand::BX));
                    self.open_handles.remove(&cpu.get_register(Operand::BX));
                    cpu.reset_flag(Flag::Carry);
                } else {
                    println!("FS ({}): Close File fd={} err=(Invalid handle)", cpu.cycle_counter, cpu.get_register(Operand::BX));
                    cpu.set_flag(Flag::Carry, 1);
                    cpu.set_register(Operand::AX, 6);
                }
            },
            0x3F => { // Read From File
                match self.open_handles.get(&cpu.get_register(Operand::BX)) {
                    Some(mut file) => {
                        let address = crate::bus::BUS::physical_address(cpu.get_register(Operand::DS), cpu.get_register(Operand::DX));
                        let length = cpu.get_register(Operand::CX) as usize;
                        cpu.set_register(Operand::AX, file.read(&mut ram[address..address+length]).unwrap() as u16);
                        cpu.reset_flag(Flag::Carry);
                    },
                    None => {
                        cpu.set_flag(Flag::Carry, 1);
                        cpu.set_register(Operand::AX, 6); // Invalid handle
                    }
                }
            },
            0x40 => { // Write To File
                match self.open_handles.get(&cpu.get_register(Operand::BX)) {
                    Some(mut file) => {
                        let address = crate::bus::BUS::physical_address(cpu.get_register(Operand::DS), cpu.get_register(Operand::DX));
                        let length = cpu.get_register(Operand::CX) as usize;
                        if length == 0 {
                            file.set_len(file.seek(std::io::SeekFrom::Current(0)).unwrap()).unwrap();
                            cpu.set_register(Operand::AX, 0);
                        } else {
                            cpu.set_register(Operand::AX, file.write(&mut ram[address..address+length]).unwrap() as u16);
                        }
                        cpu.reset_flag(Flag::Carry);
                    },
                    None => {
                        cpu.set_flag(Flag::Carry, 1);
                        cpu.set_register(Operand::AX, 6); // Invalid handle
                    }
                }
            },
            0x41 => { // Delete File
                cpu.set_flag(Flag::Carry, 1);
                if self.open_handles.len() >= 15 {
                    cpu.set_register(Operand::AX, 4); // Too many open files (no handles available)
                    return;
                }
                access_path!(get_path!(self, cpu, ram), cpu, _result, "Delete File", std::fs::remove_file, {

                });
            },
            0x42 => { // Seek In File
                cpu.set_flag(Flag::Carry, 1);
                match self.open_handles.get(&cpu.get_register(Operand::BX)) {
                    Some(mut file) => {
                        let in_position = (((cpu.get_register(Operand::CX) as u32)<<16)+cpu.get_register(Operand::DX) as u32) as i32;
                        let seek = match cpu.get_register(Operand::AX) as u8 {
                            0 => std::io::SeekFrom::Start(in_position as u64),
                            1 => std::io::SeekFrom::Current(in_position as i64),
                            2 => std::io::SeekFrom::End(in_position as i64),
                            _ => {
                                cpu.set_register(Operand::AX, 1); // Function number invalid
                                return;
                            }
                        };
                        let out_position = file.seek(seek).unwrap() as u32;
                        cpu.set_register(Operand::DX, (out_position>>16) as u16);
                        cpu.set_register(Operand::AX, out_position as u16);
                        cpu.reset_flag(Flag::Carry);
                        println!("FS ({}): Seek In File fd={} pos={} result={}", cpu.cycle_counter, cpu.get_register(Operand::BX), in_position, out_position);
                    },
                    None => {
                        cpu.set_register(Operand::AX, 6); // Invalid handle
                    }
                }
            },
            0x44 => { // Get Device Information
                cpu.reset_flag(Flag::Carry);
                cpu.set_register(Operand::DX, 0x80D3);
                cpu.set_register(Operand::AX, cpu.get_register(Operand::DX));
                println!("FS ({}): Get Device Information fd={}", cpu.cycle_counter, cpu.get_register(Operand::BX));
                if cpu.get_register(Operand::BX) >= 2 {
                    std::process::exit(0);
                }
            },
            0x48 => { // Allocate Memory
                let psp = Self::get_psp(self.psp_segment as usize, ram);
                let paragraphs = cpu.get_register(Operand::BX);
                let segment = psp.allocation_end;
                let allocation_end = paragraphs as u32+self.psp_segment as u32;
                if allocation_end < 0x9FFF {
                    cpu.reset_flag(Flag::Carry);
                    cpu.set_register(Operand::AX, segment);
                    psp.allocation_end = allocation_end as u16;
                    println!("DOS ({}): Allocate Memory segment={:04X} paragraphs={:04X}", cpu.cycle_counter, segment, paragraphs);
                } else {
                    let available = 0x9FFF-segment as u32;
                    cpu.set_flag(Flag::Carry, 1);
                    cpu.set_register(Operand::AX, 8); // Insufficient memory
                    cpu.set_register(Operand::BX, available as u16); // Size of largest available block
                    println!("DOS ({}): Get Memory segment={:04X} available={:04X}", cpu.cycle_counter, segment, available);
                }
            },
            0x49 => { // Free Memory
                let segment = cpu.get_register(Operand::ES);
                if segment >= self.psp_segment && segment < 0x9FFF {
                    cpu.reset_flag(Flag::Carry);
                    Self::get_psp(self.psp_segment as usize, ram).allocation_end = segment as u16;
                    println!("DOS ({}): Free Memory allocation_end={:04X}", cpu.cycle_counter, segment);
                } else {
                    cpu.set_flag(Flag::Carry, 1);
                    cpu.set_register(Operand::AX, 9); // Memory block address invalid
                }
            },
            0x4A => { // Resize Memory Block
                let segment = cpu.get_register(Operand::ES);
                let paragraphs = cpu.get_register(Operand::BX);
                if segment == self.psp_segment {
                    let allocation_end = paragraphs as u32+segment as u32+1;
                    if allocation_end < 0x9FFF {
                        cpu.reset_flag(Flag::Carry);
                        Self::get_psp(self.psp_segment as usize, ram).allocation_end = allocation_end as u16;
                        println!("DOS ({}): Resize Memory Block segment={:04X} paragraphs={:04X}", cpu.cycle_counter, segment, paragraphs);
                    } else {
                        cpu.set_flag(Flag::Carry, 1);
                        cpu.set_register(Operand::AX, 8); // Insufficient memory
                    }
                } else {
                    cpu.set_flag(Flag::Carry, 1);
                    cpu.set_register(Operand::AX, 9); // Memory block address invalid
                }
            },
            0x4E | 0x4F => { // Find Matching File
                cpu.set_flag(Flag::Carry, 1);
                let dta_address = crate::bus::BUS::physical_address((self.dta_address>>16) as u16, self.dta_address as u16);
                let dta = unsafe { &mut *std::mem::transmute::<*mut u8, *mut FindFirstDataBlock>(&mut ram[dta_address] as *mut u8) };
                let find_fist = cpu.get_register(Operand::AH) == 0x4E;
                let path = if find_fist {
                    let pattern_path = get_path!(self, cpu, ram);
                    let mut path = pattern_path.clone();
                    for ancestor in pattern_path.ancestors() {
                        if ancestor.starts_with(&self.mount_point_c) && ancestor.ends_with(&self.mount_point_c) {
                            break;
                        }
                        let parent = ancestor.parent().unwrap();
                        let file_name = ancestor.file_name().unwrap();
                        let pattern = file_name.to_str().unwrap();
                        let pattern_bytes = pattern.as_bytes();
                        dta.search_template = unsafe { mem::zeroed() };
                        dta.search_template[0..pattern_bytes.len()].copy_from_slice(&pattern_bytes[0..pattern_bytes.len()]);
                        dta.search_attributes = 0;
                        self.read_directory = Some(std::fs::read_dir(parent).unwrap());
                        match self.find_next_directory_entry(pattern) {
                            Some(match_path) => {
                                path = match_path;
                                break;
                            },
                            None => {}
                        }
                    }
                    path
                } else {
                    let pattern = match unsafe { std::ffi::CStr::from_ptr(&dta.search_template[0] as *const u8 as *const i8) }.to_str() {
                        Ok(path) => path,
                        Err(_) => { panic!(); }
                    };
                    match self.find_next_directory_entry(pattern) {
                        Some(match_path) => match_path,
                        None => {
                            cpu.set_register(Operand::AX, 2); // File not found
                            return;
                        }
                    }
                };
                access_path!(path, cpu, metadata, if find_fist { "Find First Matching File" } else { "Find Next Matching File" }, std::fs::metadata, {
                    let filename = path.file_name().unwrap().to_str().unwrap();
                    let modified = chrono::DateTime::<Local>::from(metadata.modified().unwrap());
                    dta.attribute = 0;
                    dta.file_time = (modified.hour() as u16)<<11|(modified.minute() as u16)<<5|(modified.second() as u16/2);
                    dta.file_date = (modified.year() as u16-1980)<<9|(modified.month() as u16)<<5|(modified.day() as u16);
                    dta.file_size = metadata.len() as u32;
                    dta.filename[0..filename.len()].copy_from_slice(unsafe { std::slice::from_raw_parts(filename.as_ptr(), filename.len()) });
                    dta.filename[filename.len()] = 0x00;
                    cpu.set_register(Operand::AX, 0);
                });
            },
            _ => {
                panic!("DOS ({}): Unsupported syscall {:04X}", cpu.cycle_counter, cpu.get_register(Operand::AX));
            }
        }
    }
}
