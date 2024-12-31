use core::fmt::Write;

use fonts::psf::RawFont;

use crate::{color::Color, error::FrameBufferError, raw::RawFrameBuffer};

const PAGE_SIZE: usize = 0x1000;

#[derive(Debug)]
pub struct RawWriter {
    row: usize,
    col: usize,
    fg_color: Color,
    bg_color: Color,
    framebuffer: RawFrameBuffer,
    font: RawFont,
}

impl RawWriter {
    pub fn new(
        font: RawFont,
        framebuffer: RawFrameBuffer,
        fg_color: Color,
        bg_color: Color,
    ) -> RawWriter {
        RawWriter {
            row: 0,
            col: 0,
            fg_color,
            bg_color,
            font,
            framebuffer,
        }
    }
}

impl RawWriter {
    /// Get current foreground and background color
    pub fn colors(&self) -> (Color, Color) {
        (self.fg_color, self.bg_color)
    }

    pub fn set_colors(&mut self, fg_color: Color, bg_color: Color) {
        self.fg_color = fg_color;
        self.bg_color = bg_color;
    }

    /// Retrieve a mutable reference of writer's font data
    pub fn font(&mut self) -> &mut RawFont {
        &mut self.font   
    }

    /// Retrieve framebuffer metadata (base, page_count)
    pub fn fb_meta(&self) -> (u64, usize) {
        let fb = self.framebuffer.ptr();
        assert_eq!(fb.len() & (PAGE_SIZE - 1), 0, "framebuffer size in bytes must be page aligned");

        (fb as *mut u8 as u64, fb.len() / PAGE_SIZE)
    }
}

impl RawWriter {
    pub fn write_char(&mut self, character: char) {
        let mut x = self.col;
        let mut y = self.row;

        match character {
            '\n' => {
                if (y + 1) * self.font.glyph_height() >= self.framebuffer.height {
                    // looping terminal
                    self.framebuffer.fill(self.bg_color);
                    y = 0;
                } else {
                    y += 1
                }
                x = 0;
            }
            character => {
                if x * self.font.glyph_width() >= self.framebuffer.width {
                    if (y + 1) * self.font.glyph_height() >= self.framebuffer.height {
                        // looping terminal
                        self.framebuffer.fill(self.bg_color);
                        y = 0;
                    } else {
                        y += 1
                    }
                    x = 0;
                }

                if let Err(err) = self.framebuffer.draw_char(
                    character,
                    x * self.font.glyph_width(),
                    y * self.font.glyph_height(),
                    self.fg_color,
                    self.bg_color,
                    self.font,
                ) {
                    match err {
                        // print ? instead
                        FrameBufferError::InvalidCharacter => {
                            self.framebuffer
                                .draw_char(
                                    '?',
                                    x * self.font.glyph_width(),
                                    y * self.font.glyph_height(),
                                    self.fg_color,
                                    self.bg_color,
                                    self.font,
                                )
                                .unwrap();
                        }
                        FrameBufferError::CoordinatesOutOfBounds(_, _) => {
                            panic!("Writer out of bounds!")
                        }
                    }
                }
                x += 1;
            }
        }
        self.col = x;
        self.row = y;
    }

    fn _write_str(&mut self, s: &str) {
        for character in s.chars() {
            self.write_char(character);
        }
    }
}

impl Write for RawWriter {
    fn write_str(&mut self, s: &str) -> core::fmt::Result {
        self._write_str(s);
        Ok(())
    }
}
