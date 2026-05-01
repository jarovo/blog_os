/* The console of the kernel writing to serial port and framebuffer. */


use core::fmt::{Write, Arguments};
use x86_64::instructions::interrupts;
use alloc::vec;
use alloc::vec::Vec;

use bootloader_api::info::FrameBuffer;
use bootloader_api::info::PixelFormat;

use embedded_graphics::geometry::Dimensions;
use embedded_graphics::primitives::Rectangle;
use embedded_graphics::text::renderer::TextRenderer;
use embedded_graphics::mono_font::{ascii::FONT_6X10, MonoTextStyle};
use embedded_graphics::text::Text;
use embedded_graphics::draw_target::DrawTarget;
use embedded_graphics::geometry::{OriginDimensions, Point, Size};
use embedded_graphics::pixelcolor::Rgb888;
use embedded_graphics::Pixel;
use embedded_graphics::prelude::RgbColor;
use embedded_graphics::Drawable;

use nostd::string::String;
use alloc::collections::VecDeque;
use anyhow::Result;

pub struct ScrollbackBuffer {
    lines: VecDeque<String>,
    current_line: String,
    max_lines: usize,
}

impl ScrollbackBuffer {
    pub fn new(max_lines: usize) -> Self {
        Self {
            lines: VecDeque::new(),
            current_line: String::new(),
            max_lines,
        }
    }

    pub fn lines(&self) -> impl Iterator<Item = &String> {
        self.lines.iter().chain(core::iter::once(&self.current_line))
    }
}

impl Write for ScrollbackBuffer {
     fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        
        for c in s.chars() {
            if c == '\n' {
                self.lines.push_back(core::mem::take(&mut self.current_line));
                if self.lines.len() > self.max_lines {
                    self.lines.pop_front();
                }
            } else {
                self.current_line.push(c);
            }
        }
        Ok(())
    }
}

pub enum Screen {
    BootloaderScreen(BootloaderScreen),
    VGAScreen1280x800x256(VGAScreen1280x800x256),
}

pub struct BootloaderScreen {
    frame_buffer: FrameBuffer,
    offscreen_frame_buffer: Vec<u8>,
}

impl BootloaderScreen {
    pub fn new(frame_buffer: FrameBuffer) -> Self {
        let fb_info = frame_buffer.info();
        Self {
            frame_buffer,
            offscreen_frame_buffer: vec![0; (fb_info.byte_len) as usize], // Assuming 4 bytes per pixel
        }
    }
}

impl OriginDimensions for BootloaderScreen {
    fn size(&self) -> Size {
        let fb_info = self.frame_buffer.info();
        Size::new(fb_info.width as u32, fb_info.height as u32)
    }
}

/// A Console type which keeps track of dimensional and address data for the
/// FrameBuffer provided by UEFI
pub struct Console {
    screens: Vec<Screen>,
    /// The text buffer to be rendered
    scrollback_buffer: ScrollbackBuffer,
}

#[derive(Debug)]
pub enum ConsoleError {
    BoundsError,
    UnsupportedPixelFormat,
}

    
impl Console {
    pub fn new() -> Self {
        Console {
            screens: Vec::new(),
            scrollback_buffer: ScrollbackBuffer::new(70), // 50 lines of scrollback
        }
    }

    pub fn add_screen(&mut self, screen: Screen) {
        self.screens.push(screen);
    }
     
    pub fn clear_screen(&mut self) -> Result<(), ConsoleError> {
        for screen in &mut self.screens {
            match screen {
                Screen::BootloaderScreen(s) => s.clear(Rgb888::BLACK)?,
                Screen::VGAScreen1280x800x256(s) => s.clear(Rgb888::BLACK)?,
            }
        }
        Ok(())
    }

    fn redraw(&mut self) -> Result<(), ConsoleError> {
        let text_style = MonoTextStyle::new(&FONT_6X10, Rgb888::WHITE);
        let mut y = text_style.line_height() as i32; // Start a bit down from the top
        for line in &mut self.scrollback_buffer.lines() {
             let drawable_text = Text::new(line, Point::new(0, y), text_style);
             y += text_style.line_height() as i32; // Move down for the next line
             for screen in &mut self.screens {
                match screen {
                    Screen::BootloaderScreen(s) => drawable_text.draw(s)?,
                    Screen::VGAScreen1280x800x256(s) => drawable_text.draw(s)?,
                };
            };
        }
        Ok(())
    }
}


impl DrawTarget for BootloaderScreen {
    /// Code is simplified (for now) by statically setting the Color to Rgb888
    type Color = Rgb888;
    type Error = ConsoleError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {

        let fb_info: bootloader_api::info::FrameBufferInfo = self.frame_buffer.info();
        let fb = self.frame_buffer.buffer_mut();

        const RGB_SETTER: fn(&mut [u8], usize, Rgb888) = |fb: &mut [u8], offset: usize, color: Rgb888| {
            (fb[offset], fb[offset + 1], fb[offset + 2]) = (color.r(), color.g(), color.b());
        };

        const BGR_SETTER: fn(&mut [u8], usize, Rgb888) = |fb: &mut [u8], offset: usize, color: Rgb888| {
            (fb[offset], fb[offset + 1], fb[offset + 2]) = (color.b(), color.g(), color.r());
        };

        let pixel_setter = match fb_info.pixel_format {
            PixelFormat::Rgb => RGB_SETTER,
            PixelFormat::Bgr => BGR_SETTER,
            _ => return Err(ConsoleError::UnsupportedPixelFormat),
        };

        for Pixel(Point { x: px, y: py }, color) in pixels {
            // Convert point positions to usize
            let x = px as usize;
            let y = py as usize;

            if (x > fb_info.width as usize) || (y > fb_info.height as usize) {
                return Err(ConsoleError::BoundsError);
            }

            /* Calculate offset into framebuffer */
            let offset = (y * (fb_info.stride * fb_info.bytes_per_pixel)) + (x * fb_info.bytes_per_pixel);
            pixel_setter(fb, offset, color);
        }
        Ok(())
    }

    fn fill_solid(&mut self, area: &Rectangle, color: Self::Color) -> Result<(), Self::Error> {
        // Clamp area to drawable part of the display target
        let drawable_area = area.intersection(&self.bounding_box());
        // Check that there are visible pixels to be drawn
        if drawable_area.size != Size::zero() {
            let fb_info = self.frame_buffer.info();
            let fbsize: usize = fb_info.stride * fb_info.height as usize * fb_info.bytes_per_pixel as usize;

            const RGB_SETTER: fn(&mut [u8], usize, Rgb888) = |fb: &mut [u8], offset: usize, color: Rgb888| {
                (fb[offset], fb[offset + 1], fb[offset + 2]) = (color.r(), color.g(), color.b());
            };

            const BGR_SETTER: fn(&mut [u8], usize, Rgb888) = |fb: &mut [u8], offset: usize, color: Rgb888| {
                (fb[offset], fb[offset + 1], fb[offset + 2]) = (color.b(), color.g(), color.r());
            };

            let pixel_setter = match fb_info.pixel_format {
                PixelFormat::Rgb => RGB_SETTER,
                PixelFormat::Bgr => BGR_SETTER,
                _ => return Err(ConsoleError::UnsupportedPixelFormat),
            };

            for y in area.top_left.y as usize..(area.top_left.y + area.size.height as i32) as usize {
                for x in area.top_left.x as usize..(area.top_left.x + area.size.width as i32) as usize {
                    /* Calculate offset into framebuffer */
                    let offset = (y * (fb_info.stride * fb_info.bytes_per_pixel)) + (x * fb_info.bytes_per_pixel);
                    pixel_setter(&mut self.offscreen_frame_buffer, offset, color);
                }
            }
            
            self.frame_buffer.buffer_mut().copy_from_slice(&self.offscreen_frame_buffer);
        } 

        Ok(())
    }

 
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
pub fn _print(args: Arguments) {
    interrupts::without_interrupts(|| {
        SERIAL_PORT_COM1.lock().write_fmt(args).unwrap();
    });
}

impl Write for Console {
    fn write_str(&mut self, s: &str) -> Result<(), core::fmt::Error> {
        print!("{}", s);
        write!(self.scrollback_buffer, "{}", s)?;
        self.clear_screen().unwrap();
        self.redraw().unwrap();
        Ok(())
    }
}

use vga::writers::Graphics1280x800x256;
use vga::writers::GraphicsWriter;

use crate::serial::SERIAL_PORT_COM1;

pub struct VGAScreen1280x800x256 {
    graphics_writer: Graphics1280x800x256,
}

impl VGAScreen1280x800x256 {
    pub fn new() -> Self {
        let graphics_writer = Graphics1280x800x256::new();
        //raphics_writer.set_mode();
        Self {
            graphics_writer,
        }
    }
}

impl DrawTarget for VGAScreen1280x800x256 {
    type Color = Rgb888;
    type Error = ConsoleError;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>,
    {
        for Pixel(Point { x, y }, color) in pixels {
            self.graphics_writer.set_pixel(x as usize, y as usize, 
                (color.r() as u32) << 16 | (color.g() as u32) << 8 | (color.b() as u32));
        }
        Ok(())
    }
}

impl OriginDimensions for VGAScreen1280x800x256 {
    fn size(&self) -> Size {
        Size::new(1280, 800)
    }
}