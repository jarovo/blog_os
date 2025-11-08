use vga::colors::{Color16, TextModeColor};
use vga::writers::{PrimitiveDrawing, ScreenCharacter, Text80x25, TextWriter};

use crate::{println};

pub fn _init(video_memory: &mut [u8]) {
    let video_memory_start_addr = video_memory.as_mut_ptr().addr() as usize;
    println!("Initializing VGA text mode at address: {:#X}", video_memory_start_addr);
    vga::vga::VGA.lock().set_memory_start(video_memory_start_addr);

    use vga::writers::{Graphics1280x800x256, GraphicsWriter};

    let mode = Graphics1280x800x256::new();
    mode.set_mode();
    mode.clear_screen(0xff);
    mode.draw_line((80, 90), (540, 90), 0x37);
    for (offset, character) in "Hello World!".chars().enumerate() {
        mode.draw_character(270 + offset * 8, 72, character, 0x37)
    }
}

pub fn init(video_memory: &mut [u8]) {
    let video_memory_start_addr = video_memory.as_mut_ptr().addr() as usize;
    println!("Initializing VGA text mode at address: {:#X}", video_memory_start_addr);
    
       
}