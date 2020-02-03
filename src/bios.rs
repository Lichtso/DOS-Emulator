use chrono::prelude::*;

static KEYCODE_TO_ASCII: &'static [u16] = &[
      0x0000, 0x0000, 0x0000, 0x0000,
      0x011b, 0x011b, 0x011b, 0x01f0, // Escape
      0x0231, 0x0221, 0x0000, 0x7800, // 1 !
      0x0332, 0x0340, 0x0300, 0x7900, // 2 @
      0x0433, 0x0423, 0x0000, 0x7a00, // 3 #
      0x0534, 0x0524, 0x0000, 0x7b00, // 4 $
      0x0635, 0x0625, 0x0000, 0x7c00, // 5 %
      0x0736, 0x075e, 0x071e, 0x7d00, // 6 ^
      0x0837, 0x0826, 0x0000, 0x7e00, // 7 &
      0x0938, 0x092a, 0x0000, 0x7f00, // 8 *
      0x0a39, 0x0a28, 0x0000, 0x8000, // 9 (
      0x0b30, 0x0b29, 0x0000, 0x8100, // 0 )
      0x0c2d, 0x0c5f, 0x0c1f, 0x8200, // - _
      0x0d3d, 0x0d2b, 0x0000, 0x8300, // = +
      0x0e08, 0x0e08, 0x0e7f, 0x0ef0, // Backspace
      0x0f09, 0x0f00, 0x9400, 0x0000, // Tab
      0x1071, 0x1051, 0x1011, 0x1000, // Q
      0x1177, 0x1157, 0x1117, 0x1100, // W
      0x1265, 0x1245, 0x1205, 0x1200, // E
      0x1372, 0x1352, 0x1312, 0x1300, // R
      0x1474, 0x1454, 0x1414, 0x1400, // T
      0x1579, 0x1559, 0x1519, 0x1500, // Y
      0x1675, 0x1655, 0x1615, 0x1600, // U
      0x1769, 0x1749, 0x1709, 0x1700, // I
      0x186f, 0x184f, 0x180f, 0x1800, // O
      0x1970, 0x1950, 0x1910, 0x1900, // P
      0x1a5b, 0x1a7b, 0x1a1b, 0x1af0, // [ {
      0x1b5d, 0x1b7d, 0x1b1d, 0x1bf0, // ] }
      0x1c0d, 0x1c0d, 0x1c0a, 0x0000, // Enter
      0x0000, 0x0000, 0x0000, 0x0000, // LCtrl
      0x1e61, 0x1e41, 0x1e01, 0x1e00, // A
      0x1f73, 0x1f53, 0x1f13, 0x1f00, // S
      0x2064, 0x2044, 0x2004, 0x2000, // D
      0x2166, 0x2146, 0x2106, 0x2100, // F
      0x2267, 0x2247, 0x2207, 0x2200, // G
      0x2368, 0x2348, 0x2308, 0x2300, // H
      0x246a, 0x244a, 0x240a, 0x2400, // J
      0x256b, 0x254b, 0x250b, 0x2500, // K
      0x266c, 0x264c, 0x260c, 0x2600, // L
      0x273b, 0x273a, 0x0000, 0x27f0, // ; :
      0x2827, 0x2822, 0x0000, 0x28f0, // ' "
      0x2960, 0x297e, 0x0000, 0x29f0, // ` ~
      0x0000, 0x0000, 0x0000, 0x0000, // LShift
      0x2b5c, 0x2b7c, 0x2b1c, 0x2bf0, // | \
      0x2c7a, 0x2c5a, 0x2c1a, 0x2c00, // Z
      0x2d78, 0x2d58, 0x2d18, 0x2d00, // X
      0x2e63, 0x2e43, 0x2e03, 0x2e00, // C
      0x2f76, 0x2f56, 0x2f16, 0x2f00, // V
      0x3062, 0x3042, 0x3002, 0x3000, // B
      0x316e, 0x314e, 0x310e, 0x3100, // N
      0x326d, 0x324d, 0x320d, 0x3200, // M
      0x332c, 0x333c, 0x0000, 0x33f0, // , <
      0x342e, 0x343e, 0x0000, 0x34f0, // . >
      0x352f, 0x353f, 0x0000, 0x35f0, // / ?
      0x0000, 0x0000, 0x0000, 0x0000, // RShift
      0x372a, 0x372a, 0x9600, 0x37f0, // *
      0x0000, 0x0000, 0x0000, 0x0000, // LAlt
      0x3920, 0x3920, 0x3920, 0x3920, // Space
      0x0000, 0x0000, 0x0000, 0x0000, // CapsLock
      0x3b00, 0x5400, 0x5e00, 0x6800, // F1
      0x3c00, 0x5500, 0x5f00, 0x6900, // F2
      0x3d00, 0x5600, 0x6000, 0x6a00, // F3
      0x3e00, 0x5700, 0x6100, 0x6b00, // F4
      0x3f00, 0x5800, 0x6200, 0x6c00, // F5
      0x4000, 0x5900, 0x6300, 0x6d00, // F6
      0x4100, 0x5a00, 0x6400, 0x6e00, // F7
      0x4200, 0x5b00, 0x6500, 0x6f00, // F8
      0x4300, 0x5c00, 0x6600, 0x7000, // F9
      0x4400, 0x5d00, 0x6700, 0x7100, // F10
      0x0000, 0x0000, 0x0000, 0x0000, // NumLock
      0x0000, 0x0000, 0x0000, 0x0000, // ScrollLock
      0x4700, 0x4737, 0x7700, 0x0007, // 7 Home
      0x4800, 0x4838, 0x8d00, 0x0008, // 8 Up
      0x4900, 0x4939, 0x8400, 0x0009, // 9 PgUp
      0x4a2d, 0x4a2d, 0x8e00, 0x4af0, // -
      0x4b00, 0x4b34, 0x7300, 0x0004, // 4 Left
      0x4cf0, 0x4c35, 0x8f00, 0x0005, // 5
      0x4d00, 0x4d36, 0x7400, 0x0006, // 6 Right
      0x4e2b, 0x4e2b, 0x9000, 0x4ef0, // +
      0x4f00, 0x4f31, 0x7500, 0x0001, // 1 End
      0x5000, 0x5032, 0x9100, 0x0002, // 2 Down
      0x5100, 0x5133, 0x7600, 0x0003, // 3 PgDn
      0x5200, 0x5230, 0x9200, 0x0000, // 0 Ins
      0x5300, 0x532e, 0x9300, 0x0000 // Del
];

enum VideoCategory {
	CGA2,
    CGA4,
	EGA,
    VGA,
	TEXT
}

#[allow(dead_code)]
struct VideoMode {
    index: u16,
    category: VideoCategory,
    sw: u16,
    sh: u16,
    tw: u8,
    th: u8,
    cw: u8,
    ch: u8,
    pt: u8,
    htot: u16,
    vtot: u16,
    hde: u16,
    vde: u16
}

static VIDEO_MODES: &'static [VideoMode] = &[
    VideoMode { index: 0x000, category: VideoCategory::TEXT, sw: 360, sh: 400, tw: 40, th: 25, cw: 9, ch: 16, pt: 8, htot: 50 , vtot: 449, hde: 40, vde: 400 },
    VideoMode { index: 0x001, category: VideoCategory::TEXT, sw: 360, sh: 400, tw: 40, th: 25, cw: 9, ch: 16, pt: 8, htot: 50 , vtot: 449, hde: 40, vde: 400 },
    VideoMode { index: 0x002, category: VideoCategory::TEXT, sw: 720, sh: 400, tw: 80, th: 25, cw: 9, ch: 16, pt: 8, htot: 100, vtot: 449, hde: 80, vde: 400 },
    VideoMode { index: 0x003, category: VideoCategory::TEXT, sw: 720, sh: 400, tw: 80, th: 25, cw: 9, ch: 16, pt: 8, htot: 100, vtot: 449, hde: 80, vde: 400 },
    VideoMode { index: 0x004, category: VideoCategory::CGA4, sw: 320, sh: 200, tw: 40, th: 25, cw: 8, ch: 8 , pt: 1, htot: 50 , vtot: 449, hde: 40, vde: 400 },
    VideoMode { index: 0x005, category: VideoCategory::CGA4, sw: 320, sh: 200, tw: 40, th: 25, cw: 8, ch: 8 , pt: 1, htot: 50 , vtot: 449, hde: 40, vde: 400 },
    VideoMode { index: 0x006, category: VideoCategory::CGA2, sw: 640, sh: 200, tw: 80, th: 25, cw: 8, ch: 8 , pt: 1, htot: 100, vtot: 449, hde: 80, vde: 400 },
    VideoMode { index: 0x007, category: VideoCategory::TEXT, sw: 720, sh: 400, tw: 80, th: 25, cw: 9, ch: 16, pt: 8, htot: 100, vtot: 449, hde: 80, vde: 400 },
    VideoMode { index: 0x00D, category: VideoCategory::EGA, sw: 320, sh: 200, tw: 40, th: 25, cw: 8, ch: 8 , pt: 8, htot: 50 , vtot: 449, hde: 40, vde: 400 },
    VideoMode { index: 0x00E, category: VideoCategory::EGA, sw: 640, sh: 200, tw: 80, th: 25, cw: 8, ch: 8 , pt: 4, htot: 100, vtot: 449, hde: 80, vde: 400 },
    VideoMode { index: 0x00F, category: VideoCategory::EGA, sw: 640, sh: 350, tw: 80, th: 25, cw: 8, ch: 14, pt: 2, htot: 100, vtot: 449, hde: 80, vde: 350 },
    VideoMode { index: 0x010, category: VideoCategory::EGA, sw: 640, sh: 350, tw: 80, th: 25, cw: 8, ch: 14, pt: 2, htot: 100, vtot: 449, hde: 80, vde: 350 },
    VideoMode { index: 0x011, category: VideoCategory::EGA, sw: 640, sh: 480, tw: 80, th: 30, cw: 8, ch: 16, pt: 1, htot: 100, vtot: 525, hde: 80, vde: 480 },
    VideoMode { index: 0x012, category: VideoCategory::EGA, sw: 640, sh: 480, tw: 80, th: 30, cw: 8, ch: 16, pt: 1, htot: 100, vtot: 525, hde: 80, vde: 480 },
    VideoMode { index: 0x013, category: VideoCategory::VGA, sw: 320, sh: 200, tw: 40, th: 25, cw: 8, ch: 8 , pt: 1, htot: 100, vtot: 449, hde: 80, vde: 400 }
];

#[repr(C, packed)]
pub struct BIOS {
    pad0: [u8; 16],
    inital_video_mode: u8,
    pad1: [u8; 6],
    keyboard_flags1: u8,
    keyboard_flags2: u8,
    keyboard_token: u8,
    keyboard_buffer_head: u16,
    keyboard_buffer_tail: u16,
    keyboard_buffer: [u16; 0x10],
    pad2: [u8; 11],
    video_mode: u8,
    video_colums: u16,
    video_memory_length: u16,
    video_memory_address: u16,
    cursor_pos: u16,
    pad3: [u8; 14],
    cursor_type: u16,
    video_current_page: u8,
    crtc_address: u16,
    current_msr: u8,
    current_pal: u8,
    pad4: [u8; 25],
    keyboard_buffer_start: u16,
    keyboard_buffer_end: u16,
    video_rows_minus_one: u8,
    char_height: u16,
    video_control: u8,
    video_switches: u8,
    video_modeset_control: u8,
    pad5: [u8; 12],
    keyboard_flags3: u8,
    keyboard_leds: u8
}

impl BIOS {
    pub fn from_ram(ram: &mut [u8]) -> &mut Self {
        unsafe { &mut *std::mem::transmute::<*mut u8, *mut Self>(&mut ram[0x400] as *mut u8) }
    }

    pub fn setup(&mut self) {
        self.keyboard_buffer_start = 0x1E;
        self.keyboard_buffer_end = self.keyboard_buffer_start+0x20;
        self.keyboard_buffer_head = self.keyboard_buffer_start;
        self.keyboard_buffer_tail = self.keyboard_buffer_start;
        self.keyboard_token = 0;
        self.keyboard_flags1 = 0;
        self.keyboard_flags2 = 0;
        self.keyboard_flags3 = 0;
        self.keyboard_leds = 0;
        self.crtc_address = 0x3D4;
    }

    pub fn keyboard_buffer_push(&mut self, cycle_counter: u64, keycode: u16) -> bool {
        if keycode == 0 {
            return false;
        }
        let mut new_tail = self.keyboard_buffer_tail+2;
        if new_tail >= self.keyboard_buffer_end {
            new_tail = self.keyboard_buffer_start;
        }
        if new_tail == self.keyboard_buffer_head {
            println!("BIOS ({}): Keyboard Buffer Overflow, dropped={:04X}", cycle_counter, keycode);
            return false;
        }
        self.keyboard_buffer[((self.keyboard_buffer_head-self.keyboard_buffer_start)/2) as usize] = keycode;
        self.keyboard_buffer_tail = new_tail;
        return true;
    }

    pub fn keyboard_buffer_pop(&mut self) -> Option<u16> {
        if self.keyboard_buffer_head == self.keyboard_buffer_tail {
            return None;
        }
        let keycode = self.keyboard_buffer[((self.keyboard_buffer_head-self.keyboard_buffer_start)/2) as usize];
        self.keyboard_buffer_head += 2;
        if self.keyboard_buffer_head >= self.keyboard_buffer_end {
            self.keyboard_buffer_head = self.keyboard_buffer_start;
        }
        Some(keycode)
    }

    pub fn handle_call(&mut self, cpu: &mut crate::cpu::CPU, pic: &mut crate::pic::ProgrammableInterruptController, ps2_controller: &mut crate::ps2_controller::PS2Controller, vga: &mut crate::vga::VideoGraphicsArray, address: u16) {
        match address {
            0xFEA5 => {}, // 0x08 (IRQ0)
            0xE987 => { // 0x09 (IRQ1)
                let keycode = ps2_controller.read_from_port(cpu.cycle_counter, 0x60);
                let lookup_index = keycode as usize*4+
                    if self.keyboard_flags1&0x08 != 0 { 3 } else
                    if self.keyboard_flags1&0x04 != 0 { 2 } else
                    if self.keyboard_flags1&0x03 != 0 { 1 } else
                    { 0 };
                match keycode {
                    0x1D => { // Ctrl Pressed
                        self.keyboard_flags1 |= 0x04;
                    },
                    0x9D => { // Ctrl Released
                        self.keyboard_flags1 &= !0x04;
                    },
                    0x2A => { // LShift Pressed
                        self.keyboard_flags1 |= 0x02;
                    },
                    0xAA => { // LShift Released
                        self.keyboard_flags1 &= !0x02;
                    },
                    0x36 => { // RShift Pressed
                        self.keyboard_flags1 |= 0x01;
                    },
                    0xB6 => { // RShift Released
                        self.keyboard_flags1 &= !0x01;
                    },
                    0x38 => { // Alt Pressed
                        self.keyboard_flags1 |= 0x08;
                    },
                    0xB8 => { // Alt Released
                        self.keyboard_flags1 &= !0x08;
                    },
                    0xE0 => { // Extended Key
                        self.keyboard_flags3 |= 0x02;
                    },
                    0x01..=0x58 => {
                        self.keyboard_buffer_push(cpu.cycle_counter, KEYCODE_TO_ASCII[lookup_index]);
                    },
                    _ => {}
                }
                if keycode != 0xE0 {
                    self.keyboard_flags3 &= !0x02;
                }
                pic.write_to_port(cpu.cycle_counter, 0x20, 0x61);
            },
            0xF065 => { // 0x10
                let command = cpu.get_register(crate::machinecode::Operand::AH) as u8;
                let argument = cpu.get_register(crate::machinecode::Operand::AL) as u8;
                match command {
                    0x00 => {
                        match VIDEO_MODES.iter().position(|video_mode| video_mode.index == argument as u16) {
                            Some(index) => {
                                let video_mode = &VIDEO_MODES[index];
                                vga.width = video_mode.sw;
                                vga.height = video_mode.sh;
                                let (read_write_mode, miscellaneous) = match video_mode.category {
                                    VideoCategory::CGA2 => (0x00, 0x0D),
                                    VideoCategory::CGA4 => (0x20, 0x0F),
                                    VideoCategory::EGA => (0x00, 0x05),
                                    VideoCategory::VGA => (0x40, 0x05),
                                    VideoCategory::TEXT => (0x10, 0x0A)
                                };
                                vga.write_to_port(cpu.cycle_counter, 0x3CE, 0x05);
                                vga.write_to_port(cpu.cycle_counter, 0x3CF, read_write_mode);
                                vga.write_to_port(cpu.cycle_counter, 0x3CE, 0x06);
                                vga.write_to_port(cpu.cycle_counter, 0x3CF, miscellaneous);
                                vga.video_mode_dirty = true;
                                self.video_mode = argument;
                                self.video_memory_address = (vga.vram_mapping.0>>4) as u16;
                                self.video_memory_length = (vga.vram_mapping.1>>4) as u16;
                                self.video_colums = video_mode.tw as u16;
                                self.video_rows_minus_one = video_mode.th-1;
                                self.char_height = video_mode.ch as u16;
                                println!("BIOS ({}): Set video mode={:02X} width={} height={} vram_begin={:05X} vram_len={:05X}", cpu.cycle_counter, argument, vga.width, vga.height, vga.vram_mapping.0, vga.vram_mapping.1);
                            },
                            None => {
                                println!("BIOS ({}): Set unsupported video mode={:02X}", cpu.cycle_counter, argument);
                            }
                        }
                    },
                    0x0F => {
                        cpu.set_register(crate::machinecode::Operand::BH, self.video_current_page as u16);
                        cpu.set_register(crate::machinecode::Operand::AL, (self.video_mode|(self.video_control&0x80)) as u16);
                        cpu.set_register(crate::machinecode::Operand::AH, self.video_colums);
                        println!("BIOS ({}): Get video mode", cpu.cycle_counter);
                    },
                    0x10 => {
                        match argument {
                            0x00 => {
                                vga.is_next_atc_data = false;
                                vga.write_to_port(cpu.cycle_counter, 0x3C0, cpu.get_register(crate::machinecode::Operand::BL) as u8);
                                vga.write_to_port(cpu.cycle_counter, 0x3C0, cpu.get_register(crate::machinecode::Operand::BH) as u8);
                            },
                            _ => {
                                println!("BIOS ({}): Unsupported palette function argument={:02X}", cpu.cycle_counter, argument);
                            }
                        }
                    },
                    _ => {
                        println!("BIOS ({}): Unsupported video command={:02X} argument={:02X}", cpu.cycle_counter, command, argument);
                    }
                }
            },
            0xF84D => { // 0x11
                cpu.set_register(crate::machinecode::Operand::AX, 0xD426); // 1101 0100 0010 0110
                //   80x87 coprocessor installed
                //   pointing device installed
                // > initial video mode: 80x25 color
                //   number of serial ports installed: 2
                //   game port installed
                //   number of parallel ports installed: 3
                println!("BIOS ({}): Get Equipment List", cpu.cycle_counter);
            },
            0xFE6E => { // 0x1A
                let command = cpu.get_register(crate::machinecode::Operand::AH);
                match command {
                    0x00 => {
                        let now = chrono::Local::now();
                        let time = ((now.hour()*3600+now.minute()*60+now.second()) as u64)*1573040/86400;
                        cpu.set_register(crate::machinecode::Operand::CX, (time>>16) as u16);
                        cpu.set_register(crate::machinecode::Operand::BX, time as u16);
                        println!("BIOS ({}): Get System Time", cpu.cycle_counter);
                    },
                    _ => {
                        println!("BIOS ({}): Unsupported System Time command={}", cpu.cycle_counter, command);
                    }
                }
            },
            _ => {
                println!("BIOS ({}): Unsupported call to address=0xF{:04X}", cpu.cycle_counter, address);
            }
        }
    }
}
