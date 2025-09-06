
use crate::alloc::string::ToString;

use core::fmt;

use embedded_graphics::mono_font::ascii::FONT_8X13;
use embedded_graphics::mono_font::MonoTextStyle;
use embedded_graphics::text::Text;
use embedded_graphics::{
    pixelcolor::Rgb888,
    prelude::*,
};

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::Size;
use embedded_graphics::prelude::OriginDimensions;

use limine::request::FramebufferRequest;
use limine::framebuffer::Framebuffer as LimineFramebuffer;

use lazy_static::lazy_static;

use spin::Mutex;


static FRAMEBUFFER_REQUEST: FramebufferRequest = FramebufferRequest::new();


pub struct Framebuffer {
    pub fb: LimineFramebuffer<'static>,
}

impl Framebuffer {
    pub fn new(fb: LimineFramebuffer<'static>) -> Self {
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

impl DrawTarget for Framebuffer {
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

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        Size::new(self.fb.width() as u32, self.fb.height() as u32)
    }
}


const BUFFER_HEIGHT: usize = 20;

pub struct Writer {
    current_line: usize,
    color_code: Rgb888,
    framebuffer: Framebuffer,
    text_buffer: alloc::vec::Vec<alloc::string::String>,
}


impl Writer {
    pub fn new() -> Self {
        let framebuffer_response = FRAMEBUFFER_REQUEST.get_response();
        match framebuffer_response {
            None => {
                panic!("Null framebuffer request!");
            }
            Some(framebuffer_response) => {
                let first_framebuffer = framebuffer_response.framebuffers().next().unwrap();
                return Self { 
                    current_line: 0,
                    color_code: Rgb888::WHITE,
                    framebuffer: Framebuffer::new(first_framebuffer),
                    text_buffer: alloc::vec::Vec::with_capacity(BUFFER_HEIGHT)
                };
            }
        }
    }

    fn refresh_display(&mut self) {
        for (i, line) in self.text_buffer.iter().enumerate() {
            let font = FONT_8X13;
            let y = (i + 1) * font.character_size.height as usize;

            let style = MonoTextStyle::new(&font, self.color_code);
            Text::new(line, Point::new(0, y as i32), style).draw(&mut self.framebuffer).unwrap();
        }
    }

    pub fn write_string(&mut self, text: &str) {
        self.text_buffer.push(text.to_string());
        self.current_line += 1;
        self.refresh_display();
    }

    pub fn write_fmt(&mut self, args: fmt::Arguments) {
        self.write_string(&args.to_string());
    }
}


impl fmt::Write for Writer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_string(s);
        Ok(())
    }
}


lazy_static! {
    pub static ref WRITER: Mutex<Writer> = Mutex::new(Writer::new());
}
