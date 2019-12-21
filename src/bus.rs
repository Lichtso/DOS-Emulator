pub struct BUS {

}

impl BUS {
    pub fn new() -> Self {
        Self {

        }
    }

    pub fn physical_address(segment: u16, offset: u16) -> usize {
        ((segment as usize)<<4)+(offset as usize)
    }

    pub fn get_memory(&mut self, _cpu: &mut crate::cpu::CPU, address: usize) -> *mut u8 {
        std::ptr::null_mut()
    }

    pub fn read_from_memory(&mut self, cpu: &mut crate::cpu::CPU, src: *const u8, data_width: u8) -> u32 {
        0
    }

    pub fn write_to_memory(&mut self, cpu: &mut crate::cpu::CPU, dst: *mut u8, data_width: u8, value: u32) {

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
