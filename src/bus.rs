pub struct BUS {
    rom: [u8; 4],
    pub ram: Vec<u8>
}

impl BUS {
    pub fn new() -> Self {
        let mut bus = Self {
            rom: [0, 0, 0, 0],
            ram: Vec::with_capacity(0xA0000)
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
        false
    }

    pub fn tick(&mut self, cpu: &mut crate::cpu::CPU) {

    }
}
