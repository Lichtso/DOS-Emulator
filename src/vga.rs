pub struct VideoGraphicsArray {
    pub width: u16,
    pub height: u16,
    pub video_mode_dirty: bool,
    pub vram_dirty: bool,
    pub palette_dirty: bool,
    pub vram_mapping: (usize, usize),
    pub vram: Vec<u8>,
    pub palette_rgba: [u32; 16],
    latch: u32,
    pub is_next_atc_data: bool,
    atc_index: u8,
    palette: [u8; 16],
    mode_control: u8,
    overscan_color: u8,
    color_plane_enable: u8,
    horizontal_pel_panning: u8,
    color_select: u8,
    sequencer_index: u8,
    sequencer_reset: u8,
    clocking_mode: u8,
    map_mask: u8,
    character_map_select: u8,
    memory_mode: u8,
    gdc_index: u8,
    set_reset: u8,
    enable_set_reset: u8,
    color_compare: u8,
    data_rotate_and_operation: u8,
    read_map_select: u8,
    read_write_mode: u8,
    miscellaneous: u8,
    color_dont_care: u8,
    bit_mask: u8,
    crt_index: u8,
    horizontal_total: u8,
    horizontal_display_end: u8,
    horizontal_blanking_start: u8,
    horizontal_blanking_end: u8,
    horizontal_retrace_start: u8,
    horizontal_retrace_end: u8,
    vertical_total: u8,
    overflow: u8,
    maximum_scan_line: u8,
    vertical_retrace_start: u8,
    vertical_retrace_end: u8,
    vertical_display_end: u8,
    vertical_blanking_start: u8,
    vertical_blanking_end: u8,
    full_data_rotate: u32,
    full_data_operation: u32,
    full_map_mask: u32,
    full_not_map_mask: u32,
    full_bit_mask: u32,
    full_set_reset: u32,
    full_enable_set_reset: u32,
    full_not_enable_set_reset: u32,
    full_enable_and_set_reset: u32
}

impl VideoGraphicsArray {
    pub fn new() -> Self {
        let mut vga = Self {
            width: 0,
            height: 0,
            video_mode_dirty: false,
            vram_dirty: false,
            palette_dirty: false,
            vram_mapping: (0xA0000, 0x20000),
            vram: Vec::with_capacity(0x40000),
            palette_rgba: unsafe { std::mem::zeroed() },
            latch: 0,
            is_next_atc_data: false,
            atc_index: 0,
            palette: unsafe { std::mem::zeroed() },
            mode_control: 0,
            overscan_color: 0,
            color_plane_enable: 0,
            horizontal_pel_panning: 0,
            color_select: 0,
            sequencer_index: 0,
            sequencer_reset: 0,
            clocking_mode: 0,
            map_mask: 0xFF,
            character_map_select: 0,
            memory_mode: 0,
            gdc_index: 0,
            set_reset: 0,
            enable_set_reset: 0,
            color_compare: 0,
            data_rotate_and_operation: 0,
            read_map_select: 0,
            read_write_mode: 0,
            miscellaneous: 0,
            color_dont_care: 0,
            bit_mask: 0,
            crt_index: 0,
            horizontal_total: 0,
            horizontal_display_end: 0,
            horizontal_blanking_start: 0,
            horizontal_blanking_end: 0,
            horizontal_retrace_start: 0,
            horizontal_retrace_end: 0,
            vertical_total: 0,
            overflow: 0,
            maximum_scan_line: 0,
            vertical_retrace_start: 0,
            vertical_retrace_end: 0,
            vertical_display_end: 0,
            vertical_blanking_start: 0,
            vertical_blanking_end: 0,
            full_data_rotate: 0,
            full_data_operation: 0,
            full_map_mask: 0,
            full_not_map_mask: 0,
            full_bit_mask: 0,
            full_set_reset: 0,
            full_enable_set_reset: 0,
            full_not_enable_set_reset: 0,
            full_enable_and_set_reset: 0
        };
        vga.vram.resize(vga.vram.capacity(), 0);
        vga
    }

    pub fn read_from_port(&mut self, cycle_counter: u64, address: u16) -> u8 {
        match address {
            0x3C1 => {
                let value = if self.is_next_atc_data {
                    match self.atc_index {
                        0x00..=0x0F => self.palette[self.atc_index as usize],
                        0x10 => self.mode_control,
                        0x11 => self.overscan_color,
                        0x12 => self.color_plane_enable,
                        0x13 => self.horizontal_pel_panning,
                        0x14 => self.color_select,
                        _ => {
                            println!("VGA ({}): Unsupported port read address={:04X} index={:04X}", cycle_counter, address, self.atc_index);
                            0
                        }
                    }
                } else {
                    self.atc_index
                };
                self.is_next_atc_data = !self.is_next_atc_data;
                value
            },
            0x3C4 => self.sequencer_index,
            0x3C5 => {
                match self.sequencer_index {
                    0x00 => self.sequencer_reset,
                    0x01 => self.clocking_mode,
                    0x02 => self.map_mask,
                    0x03 => self.character_map_select,
                    0x04 => self.memory_mode,
                    _ => {
                        println!("VGA ({}): Unsupported port read address={:04X} index={:04X}", cycle_counter, address, self.sequencer_index);
                        0
                    }
                }
            },
            0x3CE => self.gdc_index,
            0x3CF => {
                match self.gdc_index {
                    0x00 => self.set_reset,
                    0x01 => self.enable_set_reset,
                    0x02 => self.color_compare,
                    0x03 => self.data_rotate_and_operation,
                    0x04 => self.read_map_select,
                    0x05 => self.read_write_mode,
                    0x06 => self.miscellaneous,
                    0x07 => self.color_dont_care,
                    0x08 => self.bit_mask,
                    _ => {
                        println!("VGA ({}): Unsupported port read address={:04X} index={:04X}", cycle_counter, address, self.gdc_index);
                        0
                    }
                }
            },
            0x3D4 => self.crt_index,
            0x3D5 => {
                match self.crt_index {
                    0x00 => self.horizontal_total,
                    0x01 => self.horizontal_display_end,
                    0x02 => self.horizontal_blanking_start,
                    0x03 => self.horizontal_blanking_end,
                    0x04 => self.horizontal_retrace_start,
                    0x05 => self.horizontal_retrace_end,
                    0x06 => self.vertical_total,
                    0x07 => self.overflow,
                    0x09 => self.maximum_scan_line,
                    0x10 => self.vertical_retrace_start,
                    0x11 => self.vertical_retrace_end,
                    0x12 => self.vertical_display_end,
                    0x15 => self.vertical_blanking_start,
                    0x16 => self.vertical_blanking_end,
                    _ => {
                        println!("VGA ({}): Unsupported port read address={:04X} index={:04X}", cycle_counter, address, self.gdc_index);
                        0
                    }
                }
            },
            0x3DA => {
                self.is_next_atc_data = false;
                println!("VGA ({}): Unsupported port read address={:04X}", cycle_counter, address);
                0
            },
            _ => {
                println!("VGA ({}): Unsupported port read address={:04X}", cycle_counter, address);
                0
            }
        }
    }

    pub fn write_to_port(&mut self, cycle_counter: u64, address: u16, value: u8) {
        match address {
            0x3C0 => {
                if self.is_next_atc_data {
                    match self.atc_index {
                        0x00..=0x0F => {
                            self.palette[self.atc_index as usize] = value;
                            let r = ((((value>>5)&1)*0x55)+(((value>>2)&1)*0xAA)) as u32;
                            let g = ((((value>>4)&1)*0x55)+(((value>>1)&1)*0xAA)) as u32;
                            let b = ((((value>>3)&1)*0x55)+(((value>>0)&1)*0xAA)) as u32;
                            self.palette_rgba[self.atc_index as usize] = 0xFF000000|(b<<16)|(g<<8)|r;
                            self.palette_dirty = true;
                        }
                        0x10 => { self.mode_control = value; },
                        0x11 => { self.overscan_color = value; },
                        0x12 => { self.color_plane_enable = value; },
                        0x13 => { self.horizontal_pel_panning = value; },
                        0x14 => { self.color_select = value; },
                        _ => {
                            println!("VGA ({}): Unsupported port write address={:04X} index={:04X} value={:02X}", cycle_counter, address, self.atc_index, value);
                        }
                    }
                } else {
                    self.atc_index = value;
                }
                self.is_next_atc_data = !self.is_next_atc_data;
            },
            0x3C4 => { self.sequencer_index = value; },
            0x3C5 => {
                match self.sequencer_index {
                    0x00 => { self.sequencer_reset = value; },
                    0x01 => { self.clocking_mode = value; },
                    0x02 => {
                        self.map_mask = value;
                        self.full_map_mask = VideoGraphicsArray::spread_4(self.map_mask);
                        self.full_not_map_mask = !self.full_map_mask;
                    },
                    0x03 => { self.character_map_select = value; },
                    0x04 => { self.memory_mode = value; },
                    _ => {
                        println!("VGA ({}): Unsupported port write address={:04X} index={:04X} value={:02X}", cycle_counter, address, self.sequencer_index, value);
                    }
                }
            },
            0x3CE => { self.gdc_index = value; },
            0x3CF => {
                match self.gdc_index {
                    0x00 => {
                        self.set_reset = value&0x0F;
                        self.full_set_reset = VideoGraphicsArray::spread_4(self.set_reset);
                		self.full_enable_and_set_reset = self.full_set_reset&self.full_enable_set_reset;
                    },
                    0x01 => {
                        self.enable_set_reset = value&0x0F;
                        self.full_enable_set_reset = VideoGraphicsArray::spread_4(self.enable_set_reset);
                		self.full_not_enable_set_reset = !self.full_enable_set_reset;
                		self.full_enable_and_set_reset = self.full_set_reset&self.full_enable_set_reset;
                    },
                    0x02 => { self.color_compare = value&0x0F; },
                    0x03 => {
                        self.data_rotate_and_operation = value;
                        self.full_data_rotate = (self.data_rotate_and_operation as u32)&7;
                        self.full_data_operation = ((self.data_rotate_and_operation>>3)&3) as u32;
                    },
                    0x04 => { self.read_map_select = value&0x03; },
                    0x05 => { self.read_write_mode = value&0x0B; },
                    0x06 => {
                        self.miscellaneous = value&0x0F;
                        self.vram_mapping = match self.miscellaneous>>2 {
                            0 => (0xA0000, 0x20000),
                            1 => (0xA0000, 0x10000),
                            2 => (0xB0000, 0x8000),
                            3 => (0xB8000, 0x8000),
                            _ => unreachable!()
                        };
                    },
                    0x07 => { self.color_dont_care = value&0x0F; },
                    0x08 => {
                        self.bit_mask = value;
                        self.full_bit_mask = VideoGraphicsArray::replicate_8(self.bit_mask);
                    },
                    _ => {
                        println!("VGA ({}): Unsupported port write address={:04X} index={:04X} value={:02X}", cycle_counter, address, self.gdc_index, value);
                    }
                }
            },
            0x3D4 => { self.crt_index = value; },
            0x3D5 => {
                match self.crt_index {
                    0x00 => { self.horizontal_total = value; },
                    0x01 => { self.horizontal_display_end = value; },
                    0x02 => { self.horizontal_blanking_start = value; },
                    0x03 => { self.horizontal_blanking_end = value; },
                    0x04 => { self.horizontal_retrace_start = value; },
                    0x05 => { self.horizontal_retrace_end = value; },
                    0x06 => { self.vertical_total = value; },
                    0x07 => { self.overflow = value; },
                    0x09 => { self.maximum_scan_line = value; },
                    0x10 => { self.vertical_retrace_start = value; },
                    0x11 => { self.vertical_retrace_end = value; },
                    0x12 => { self.vertical_display_end = value; },
                    0x15 => { self.vertical_blanking_start = value; },
                    0x16 => { self.vertical_blanking_end = value; },
                    _ => {
                        println!("VGA ({}): Unsupported port write address={:04X} index={:04X} value={:02X}", cycle_counter, address, self.gdc_index, value);
                    }
                }
            },
            _ => {
                println!("VGA ({}): Unsupported port write address={:04X} value={:02X}", cycle_counter, address, value);
            }
        }
    }

    fn replicate_8(value: u8) -> u32 {
        let result = value as u32;
        (result<<24)|(result<<16)|(result<<8)|(result<<0)
    }

    fn spread_4(value: u8) -> u32 {
        let result = value as u32;
        (0x000000FF*((result>>0)&1))|
        (0x0000FF00*((result>>1)&1))|
        (0x00FF0000*((result>>2)&1))|
        (0xFF000000*((result>>3)&1))
    }

    fn rastering_operation(&mut self, input: u32, mask: u32) -> u32 {
        match self.full_data_operation {
            0 => (input&mask)|(self.latch&!mask),
	        1 => (input|!mask)&self.latch,
	        2 => (input&mask)|self.latch,
	        3 => (input&mask)^self.latch,
            _ => unreachable!()
        }
    }

    pub fn read_from_memory(&mut self, _cycle_counter: u64, address: usize) -> u8 {
        self.latch = crate::bit_utils::read_from_buffer_u32(&self.vram[address*4..]);
        let value = match (self.read_write_mode>>3)&1 {
            0 => (self.latch>>(8*self.read_map_select)) as u8,
            1 => {
                let mut result: u32 = self.latch&VideoGraphicsArray::spread_4(self.color_dont_care);
                result ^= VideoGraphicsArray::spread_4(self.color_compare&self.color_dont_care);
			    !(((result>>24) as u8)|((result>>16) as u8)|((result>>8) as u8)|((result>>0) as u8))
            }
            _ => unreachable!()
        };
        value
    }

    pub fn write_to_memory(&mut self, _cycle_counter: u64, address: usize, value: u8) {
        let mut result: u32;
        match self.read_write_mode&0x03 {
            0 => {
                result = value.rotate_right(self.full_data_rotate) as u32;
                result = VideoGraphicsArray::replicate_8(result as u8);
                result = (result&self.full_not_enable_set_reset)|self.full_enable_and_set_reset;
                result = self.rastering_operation(result, self.full_bit_mask);
            },
            1 => {
                result = self.latch;
            },
            2 => {
                result = self.rastering_operation(VideoGraphicsArray::spread_4(value), self.full_bit_mask);
            },
            3 => {
                result = value.rotate_right(self.full_data_rotate) as u32;
                result = VideoGraphicsArray::replicate_8(result as u8);
                result &= self.full_bit_mask;
                result = self.rastering_operation(self.full_set_reset, result);
            },
            _ => unreachable!()
        }
        let original = crate::bit_utils::read_from_buffer_u32(&self.vram[address*4..]);
        result = (original&self.full_not_map_mask)|(result&self.full_map_mask);
        crate::bit_utils::write_to_buffer_u32(&mut self.vram[address*4..], result);
        self.vram_dirty = true;
    }
}
