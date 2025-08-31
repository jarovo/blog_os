#![no_std]
#![no_main]


use core::panic::PanicInfo;
use core::fmt::Write;
use limine::request::FramebufferRequest;


mod serial;
mod vga_buffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Panic occurred: {}", _info).unwrap();
    loop {}
}

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();

fn main() -> ! {
   let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Hello World from serial!").unwrap();

    let fbs_ptr = FRAMEBUFFER_REQUEST.get_response();

    match fbs_ptr {
        None => writeln!(serial_writer, "No framebuffer found!").unwrap(),
        Some(fbs) => {
            for fb in fbs.framebuffers() {
                writeln!(serial_writer, "Framebuffer found!").unwrap();
                writeln!(serial_writer, "address: {:#?}", fb.addr()).unwrap();
                writeln!(serial_writer, "width: {}", fb.width()).unwrap();
                writeln!(serial_writer, "height: {}", fb.height()).unwrap();
                writeln!(serial_writer, "pitch: {}", fb.pitch()).unwrap();
                writeln!(serial_writer, "bpp: {}", fb.bpp()).unwrap();
            }
        }
    }
    vga_buffer::print_something();
    loop {}
}

#[unsafe(no_mangle)]
pub extern "C" fn _start() -> ! {
    main()
}