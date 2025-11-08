/* The console of the kernel writing to serial port and framebuffer. */

use core::fmt::{self, Write};
use alloc::string::ToString;
use embedded_graphics::text::renderer::TextRenderer;
use x86_64::instructions::interrupts;
use lazy_static::lazy_static;
use spin::Mutex;
use crate::serial;

use bootloader_api::info::FrameBuffer;
use bootloader_api::info::PixelFormat;

use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::text::Text;

use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::Pixel;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::Drawable;

use nostd::string::String;
use nostd::vec::Vec;

pub struct ScrollbackBuffer {
    lines: Vec<String>,
    max_lines: usize,
}

impl ScrollbackBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: Vec::new(),
            max_lines,
        }
    }

    pub fn push_string(&mut self, text: String) {
        for line in text.lines() {
            self.lines.push(line.to_string());
        }
        while self.lines.len() >= self.max_lines {
            self.lines.remove(0); // Remove the oldest line
        }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }
}


struct ConsoleFramebuffer {
    fbptr: *mut u8,
    fbwidth: usize,
    fbheight: usize,
    fbstride: usize,
    bytes_per_pixel: usize,
    pixel_format: PixelFormat,
}


/// A Console type which keeps track of dimensional and address data for the
/// FrameBuffer provided by UEFI
pub struct Console {
    console_framebuffer: ConsoleFramebuffer,
    /// The text buffer to be rendered
    buffer: ScrollbackBuffer,
}

impl OriginDimensions for ConsoleFramebuffer {
    fn size(&self) -> Size {
        Size::new(self.fbwidth as u32, self.fbheight as u32)
    }
}

pub enum ConsoleError {
    BoundsError,
}

impl Console {
    pub fn new_from_bootinfo(frame_buffer: &mut FrameBuffer) -> Self {
        let fb_info = frame_buffer.info();
        
        Console {
            console_framebuffer: ConsoleFramebuffer {
                fbptr: frame_buffer.buffer_mut().as_mut_ptr(),
                fbwidth: fb_info.width,
                fbheight: fb_info.height,
                fbstride: fb_info.stride,
                bytes_per_pixel: fb_info.bytes_per_pixel,
                pixel_format: fb_info.pixel_format,
            },
            buffer: ScrollbackBuffer::new(100), // 100 lines of scrollback
        }
    }

    fn clear_screen(&mut self) {
        let fbsize = self.console_framebuffer.fbstride * self.console_framebuffer.fbheight * self.console_framebuffer.bytes_per_pixel;
        let fb = unsafe { core::slice::from_raw_parts_mut(self.console_framebuffer.fbptr, fbsize) };
        for byte in fb.iter_mut() {
            *byte = 0;
        }
    }

    fn redraw(&mut self) {
        self.clear_screen();
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let mut y = text_style.line_height() as i32; // Start a bit down from the top
        for line in self.buffer.lines() {
            Text::new(line, Point::new(0, y), text_style).draw(&mut self.console_framebuffer).ok();
            y += text_style.line_height() as i32;
        }
    }


    pub fn write_str<'a>(&mut self, s: &'a str) -> Result<(), ConsoleError> {
        self.buffer.push_string(s.to_string());
        self.redraw();
        Ok(())
    }
}

impl fmt::Write for Console {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        self.write_str(s).map_err(|_| fmt::Error)
    }
}

impl DrawTarget for ConsoleFramebuffer {
    /// Code is simplified (for now) by statically setting the Color to Rgb888
    type Color = Rgb888;
    type Error = ConsoleError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x: px, y: py }, color) in pixels.into_iter() {
            // Convert point positions to usize
            let x = px as usize;
            let y = py as usize;

            if (x < self.fbwidth) && (y < self.fbheight) {
                /* Calculate offset into framebuffer */
                let offset = (y * (self.fbstride * self.bytes_per_pixel)) + (x * self.bytes_per_pixel);
                let fbsize = self.fbstride * self.fbheight * self.bytes_per_pixel;
                let fb = unsafe { core::slice::from_raw_parts_mut(self.fbptr, fbsize) };
                fb[offset + 1] = color.g();

                // Support swapped-ordering when we are a BGR versus RGB Console. This handles
                // the conversion required because we set the DrawTarget's Color type to Rgb888
                // for code simplicity.
                if self.pixel_format == PixelFormat::Bgr {
                    fb[offset] = color.b();
                    fb[offset + 2] = color.r();
                } else {
                    fb[offset] = color.r();
                    fb[offset + 2] = color.b();
                }
            } else {
                // If given an invalid bound, then return an error
                return Err(ConsoleError::BoundsError)
            }
        }
        Ok(())
    }
}

lazy_static! {
    pub static ref WRITER: Mutex<serial::Writer> = Mutex::new(serial::Writer::new(0x3F8)); // COM1
}

#[macro_export]
macro_rules! print {
    ($($arg:tt)*) => ($crate::console::_print(format_args!($($arg)*)));
}

#[macro_export]
macro_rules! println {
    () => ($crate::print!("\n"));
    ($($arg:tt)*) => ($crate::print!("{}\n", format_args!($($arg)*)));
}

#[doc(hidden)]
pub fn _print(args: fmt::Arguments) {
    interrupts::without_interrupts(|| {
        WRITER.lock().write_fmt(args).unwrap();
    });
}