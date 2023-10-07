use bootloader_api::info::{FrameBuffer, PixelFormat};
use core::{fmt, ptr};
use noto_sans_mono_bitmap::{
    get_raster, get_raster_width, FontWeight, RasterHeight, RasterizedChar,
};

const LINE_SPACING: usize = 2;
const LETTER_SPACING: usize = 0;
const BORDER_PADDING: usize = 1;
const CHAR_RASTER_HEIGHT: RasterHeight = RasterHeight::Size16;
const CHAR_RASTER_WDITH: usize = get_raster_width(FontWeight::Regular, CHAR_RASTER_HEIGHT);
const BACKUP_CHAR: char = '�';
const FONT_WEIGHT: FontWeight = FontWeight::Regular;

fn get_char_raster(c: char) -> RasterizedChar {
    fn get(c: char) -> Option<RasterizedChar> {
        get_raster(c, FONT_WEIGHT, CHAR_RASTER_HEIGHT)
    }

    get(c).unwrap_or_else(|| get(BACKUP_CHAR).expect("Should have been able to get backup char"))
}

pub struct FrameBufferWriter {
    framebuffer: &'static mut FrameBuffer,
    x_pos: usize,
    y_pos: usize,
}

impl FrameBufferWriter {
    pub fn new(framebuffer: &'static mut FrameBuffer) -> Self {
        let mut writer = Self {
            framebuffer,
            x_pos: 0,
            y_pos: 0,
        };
        writer.clear();
        writer
    }

    fn newline(&mut self) {
        self.y_pos += CHAR_RASTER_HEIGHT.val() + LINE_SPACING;
        self.carrriage_return();
    }

    fn carrriage_return(&mut self) {
        self.x_pos = BORDER_PADDING;
    }

    fn width(&self) -> usize {
        self.framebuffer.info().width
    }

    fn height(&self) -> usize {
        self.framebuffer.info().height
    }

    pub fn clear(&mut self) {
        self.x_pos = BORDER_PADDING;
        self.y_pos = BORDER_PADDING;
        self.framebuffer.buffer_mut().fill(0);
    }

    fn write_char(&mut self, c: char) {
        match c {
            '\n' => self.newline(),
            '\r' => self.carrriage_return(),
            c => {
                let new_xpos = self.x_pos + CHAR_RASTER_WDITH;
                if new_xpos >= self.width() {
                    self.newline();
                }
                let new_ypos = self.y_pos + CHAR_RASTER_HEIGHT.val() + BORDER_PADDING;
                if new_ypos >= self.height() {
                    self.clear();
                }

                self.write_rendered_char(get_char_raster(c));
            }
        }
    }

    fn write_rendered_char(&mut self, rendered_char: RasterizedChar) {
        for (y, row) in rendered_char.raster().iter().enumerate() {
            for (x, byte) in row.iter().enumerate() {
                self.write_pixel(self.x_pos + x, self.y_pos + y, *byte);
            }
        }
        self.x_pos += rendered_char.width() + LETTER_SPACING;
    }

    fn write_pixel(&mut self, x: usize, y: usize, intensity: u8) {
        let info = self.framebuffer.info();

        let pixel_offset = y * info.stride + x;
        let color = match info.pixel_format {
            PixelFormat::Bgr => [intensity, intensity, intensity / 2, 0],
            PixelFormat::Rgb => [intensity / 2, intensity, intensity, 0],
            PixelFormat::U8 => [if intensity > 200 { 0xf } else { 0 }, 0, 0, 0],
            _ => [0xff, 0xff, 0xff, 0],
        };

        let bytes_pre_pixel = info.bytes_per_pixel;
        let byte_offset = pixel_offset * bytes_pre_pixel;

        self.framebuffer.buffer_mut()[byte_offset..(byte_offset + bytes_pre_pixel)]
            .copy_from_slice(&color[..bytes_pre_pixel]);

        let _ = unsafe { ptr::read_volatile(&self.framebuffer.buffer()[byte_offset]) };
    }
}

unsafe impl Send for FrameBufferWriter {}
unsafe impl Sync for FrameBufferWriter {}

impl fmt::Write for FrameBufferWriter {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c);
        }
        Ok(())
    }
}
