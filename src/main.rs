#![no_std]
#![no_main]


use core::panic::PanicInfo;
use core::fmt::Write;
use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use limine::request::FramebufferRequest;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Size;
use embedded_graphics::prelude::OriginDimensions;

use limine::framebuffer::Framebuffer as LimineFramebuffer;


mod serial;
mod vga_buffer;

#[panic_handler]
fn panic(_info: &PanicInfo) -> ! {
    let mut serial_writer = serial::Writer::new(0x3F8);
    writeln!(serial_writer, "Panic occurred: {}", _info).unwrap();
    loop {}
}

static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();



pub struct Framebuffer<'a> {
    pub fb: &'a mut LimineFramebuffer<'a>,
}

impl<'a> Framebuffer<'a> {
    pub fn new(fb: &'a mut LimineFramebuffer<'a>) -> Self {
        Self { fb }
    }

    fn put_pixel(&mut self, x: usize, y: usize, color: Rgb888) {
        let bytes_per_pixel = (self.fb.bpp() / 8) as usize;
        let pitch = self.fb.pitch() as usize;
        let base = self.fb.addr() as *mut u8;

        unsafe {
            let pixel_ptr = base.add(y * pitch + x * bytes_per_pixel) as *mut u32;
            // Rgb888 -> u32 (0x00RRGGBB)
            let color_u32 = ((color.r() as u32) << 16) |
                            ((color.g() as u32) << 8) |
                             (color.b() as u32);
            *pixel_ptr = color_u32;
        }
    }
}

impl<'a> DrawTarget for Framebuffer<'a> {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Rgb888>>,
    {
        for Pixel(coord, color) in pixels {
            if coord.x >= 0 && coord.y >= 0 &&
               (coord.x as usize) < self.fb.width() as usize &&
               (coord.y as usize) < self.fb.height() as usize
            {
                self.put_pixel(coord.x as usize, coord.y as usize, color);
            }
        }
        Ok(())
    }
}

impl<'a> OriginDimensions for Framebuffer<'a> {
    fn size(&self) -> Size {
        Size::new(self.fb.width() as u32, self.fb.height() as u32)
    }
}

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

                let style = MonoTextStyle::new(&FONT_8X13, Rgb888::WHITE);
                Text::new("Hello World!", Point::new(10, 10), style).draw(&mut Framebuffer::new(&mut fb)).unwrap();
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