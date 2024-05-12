use image::{ImageBuffer, Luma};
use log::info;
use rppal::gpio::{Gpio, InputPin, OutputPin, Level::*};
use rppal::spi::{self, Spi, Bus, SlaveSelect};

use crate::args::DisplayMode;

use super::util::*;


#[derive(Debug)]
pub enum DriverError {
    Gpio(rppal::gpio::Error),
    Spi(rppal::spi::Error),
    WrongInput(String),
}

impl std::fmt::Display for DriverError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl std::error::Error for DriverError {}

impl From<rppal::gpio::Error> for DriverError {
    fn from(value: rppal::gpio::Error) -> Self {
        Self::Gpio(value)
    }
}

impl From<rppal::spi::Error> for DriverError {
    fn from(value: rppal::spi::Error) -> Self {
        Self::Spi(value)
    }
}


pub type DisplayImagePixel = Luma<u8>;
pub type DisplayImage = ImageBuffer<DisplayImagePixel, Vec<u8>>;

pub const BLACK: DisplayImagePixel = Luma([u8::MIN]);
pub const WHITE: DisplayImagePixel = Luma([u8::MAX]);


#[repr(u8)]
#[derive(Clone, Copy)]
enum ColorGreyscale {
    Black = 0b00,
    Dark  = 0b01,
    Light = 0b10,
    White = 0b11,
}

impl ColorGreyscale {
    const B_FROM: u8 = 0;
    const B_TO: u8   = Self::B_FROM + u8::MAX / 4;

    const D_FROM: u8 = Self::B_TO + 1;
    const D_TO: u8   = Self::D_FROM + u8::MAX / 4;

    const L_FROM: u8 = Self::D_TO + 1;
    const L_TO: u8   = Self::L_FROM + u8::MAX / 4;

    const W_FROM: u8 = Self::L_TO + 1;
    const W_TO: u8   = Self::W_FROM + u8::MAX / 4;

    pub fn new(color: &DisplayImagePixel) -> Self {
        match color.0[0] {
            Self::B_FROM..=Self::B_TO => Self::Black,
            Self::D_FROM..=Self::D_TO => Self::Dark,
            Self::L_FROM..=Self::L_TO => Self::Light,
            Self::W_FROM..=Self::W_TO => Self::White,
        }
    }

    pub fn bit_0011(&self) -> bool {
        let num = *self as u8;
        num & 0b10 > 0
    }

    pub fn bit_0101(&self) -> bool {
        let num = *self as u8;
        num & 0b01 > 0
    }
}


pub struct Display {
    // Output: reset the display
    rst: OutputPin,
    // Output: select whether SPI sends commands or data
    dc: OutputPin,
    // Output: power?
    pwr: OutputPin,

    // Input: whether the display is busy
    busy: InputPin,

    // SPI interface
    spi: Spi,
}


/// Waveshare 2.7 inch e-Paper display
///
/// Documentation: https://www.waveshare.com/wiki/2.7inch_e-Paper_HAT_Manual
/// Specification: https://files.waveshare.com/upload/b/ba/2.7inch_e-Paper_V2_Specification.pdf
///
/// LUT stands for look up table.
/// It stores the Waveform which defines the relation between greyscale, voltage and temperature.
impl Display {

    const WIDTH: u32 = 176;
    const HEIGHT: u32 = 264;

    const OUT_RST: u8 = 17;
    const OUT_DC: u8 = 25;
    const OUT_PWR: u8 = 18;

    const IN_BUSY: u8 = 24;

    // SPI bus to use
    const SPI_BUS: Bus = Bus::Spi0;
    // SPI device to use
    const SPI_DEV: SlaveSelect = SlaveSelect::Ss0;
    // SPI clock speed in Hz to use
    const SPI_CLOCK_HZ: u32 = 4000000;
    // SPI mode to use
    const SPI_MODE: spi::Mode = spi::Mode::Mode0;


    pub fn new() -> Result<Self, DriverError> {
        let gpio = Gpio::new()?;

        let rst = gpio.get(Self::OUT_RST)?.into_output();
        let dc = gpio.get(Self::OUT_DC)?.into_output();
        let pwr = gpio.get(Self::OUT_PWR)?.into_output();

        let busy = gpio.get(Self::IN_BUSY)?.into_input();

        let spi = Spi::new(
            Self::SPI_BUS,
            Self::SPI_DEV,
            Self::SPI_CLOCK_HZ,
            Self::SPI_MODE,
        )?;

        Ok(Self {
            rst,
            dc,
            pwr,
            busy,
            spi,
        })
    }

    /// Get vertical image
    pub fn image_white_v() -> DisplayImage {
        let mut img = DisplayImage::new(Self::WIDTH, Self::HEIGHT);

        for (_x, _y, pixel) in img.enumerate_pixels_mut() {
            *pixel = WHITE;
        }

        return img;
    }

    /// Get horizontal image
    pub fn image_white_h() -> DisplayImage {
        let mut img = DisplayImage::new(Self::HEIGHT, Self::WIDTH);

        for (_x, _y, pixel) in img.enumerate_pixels_mut() {
            *pixel = WHITE;
        }

        return img;
    }

    fn image_size_bytes() -> (usize, usize) {
        // Pad width so that each row takes a whole number of bytes
        let width = match Self::WIDTH % 8 {
            0 => Self::WIDTH / 8,
            _ => Self::WIDTH / 8 + 1,
        } as usize;

        // Height stays unchanged
        let height = Self::HEIGHT as usize;

        return (width, height);
    }

    fn buffer_white() -> Vec<u8> {
        let (width, height) = Self::image_size_bytes();
        let mut buffer = Vec::with_capacity(width * height);
        for _ in 0..(width * height) {
            buffer.push(0b11111111);
        }
        return buffer;
    }


    fn reset(&mut self) {
        info!("reset");

        self.rst.write(High);
        sleep_ms(200);
        self.rst.write(Low);
        sleep_ms(2);
        self.rst.write(High);
        sleep_ms(200);
    }

    fn send_command(&mut self, command: &[u8]) -> Result<(), DriverError> {
        self.dc.write(Low);
        self.spi.write(command)?;
        Ok(())
    }

    fn send_data(&mut self, data: &[u8]) -> Result<(), DriverError> {
        self.dc.write(High);
        self.spi.write(data)?;
        Ok(())
    }

    fn wait_not_busy(&mut self) {
        while self.busy.read() == High {
            sleep_ms(20);
        }
    }

    fn show(&mut self, mode: DisplayMode) -> Result<(), DriverError>  {
        info!("{} show", mode);

        // Display update control
        self.send_command(&[0x22])?;
        match mode {
            // Load temperature value, Display with mode 1
            DisplayMode::Full => self.send_data(&[0xF7])?,
            // Display with mode 1
            DisplayMode::Fast | DisplayMode::Grey => self.send_data(&[0xC7])?,
        }

        // Execute the selected update sequence
        self.send_command(&[0x20])?;
        self.wait_not_busy();

        Ok(())
    }


    pub fn init(&mut self, mode: DisplayMode) -> Result<(), DriverError> {
        info!("{} init", mode);

        self.pwr.write(High);

        self.reset();
        self.wait_not_busy();

        // SWRESET
        self.send_command(&[0x12])?;
        self.wait_not_busy();

        if let DisplayMode::Fast = mode {
            // Select temperature sensor
            self.send_command(&[0x18])?;
            // Internal sensor
            self.send_data(&[0x80])?;

            // Display update control
            self.send_command(&[0x22])?;
            // Load temperature value, Load LUT (display mode 1)
            self.send_data(&[0xB1])?;

            // Execute the selected update sequence
            self.send_command(&[0x20])?;
            self.wait_not_busy();
        }

        if let DisplayMode::Grey = mode {
            // Set analog block control
            self.send_command(&[0x74])?;
            self.send_data(&[0x54])?;
            // Set digital block control
            self.send_command(&[0x7E])?;
            self.send_data(&[0x3B])?;

            // Driver output control
            self.send_command(&[0x01])?;
            self.send_data(&[0x07])?;
            self.send_data(&[0x01])?;
            self.send_data(&[0x00])?;
        }

        // Data entry mode
        self.send_command(&[0x11])?;
        // Y increment, X increment, counter updated in X direction
        self.send_data(&[0b0000_0011])?;

        // Set RAM Y address start/end position
        let [y_byte_1, y_byte_2] = (Self::HEIGHT as u16 - 1).to_le_bytes();
        self.send_command(&[0x45])?;
        self.send_data(&[0x00, 0x00, y_byte_1, y_byte_2])?;

        if let DisplayMode::Grey = mode {
            // Don't draw border
            self.send_command(&[0x3C])?;
            self.send_data(&[0x00])?;

            // VCOM Voltage
            self.send_command(&[0x2C])?;
            self.send_data(&[LUT_2BIT[158]])?; // 0x1C

            // EOPQ
            self.send_command(&[0x3F])?;
            self.send_data(&[LUT_2BIT[153]])?;

            // VGH
            self.send_command(&[0x03])?;
            self.send_data(&[LUT_2BIT[154]])?;

            // Something
            self.send_command(&[0x04])?;
            self.send_data(&[LUT_2BIT[155]])?; // VSH1
            self.send_data(&[LUT_2BIT[156]])?; // VSH2
            self.send_data(&[LUT_2BIT[157]])?; // VSL

            // LUT
            self.send_command(&[0x32])?;
            self.send_data(&LUT_2BIT[0..159])?;

            self.wait_not_busy();
        }

        Ok(())
    }

    pub fn clear(&mut self, mode: DisplayMode) -> Result<(), DriverError> {
        info!("{} clear", mode);
        self.display(Self::image_white_v(), mode)?;
        Ok(())
    }

    pub fn sleep(&mut self) -> Result<(), DriverError> {
        info!("sleep");

        self.send_command(&[0x10])?;
        self.send_data(&[0x01])?;

        sleep_ms(2000);

        self.rst.write(Low);
        self.dc.write(Low);
        self.pwr.write(Low);

        Ok(())
    }

    pub fn display(&mut self, img: DisplayImage, mode: DisplayMode) -> Result<(), DriverError> {
        info!("{} display", mode);

        // Set RAM Y address count to 0
        self.send_command(&[0x4F])?;
        self.send_data(&[0x00, 0x00])?;

        let (width, ..) = Self::image_size_bytes();

        let horizontal = match (img.width(), img.height()) {
            (Self::WIDTH, Self::HEIGHT) => false,
            (Self::HEIGHT, Self::WIDTH) => true,
            _ => return Err(DriverError::WrongInput(format!(
                "Image dimensions do not match screen size. Image is {}x{}. Screen is {}, {}",
                img.width(), img.height(),
                Self::WIDTH, Self::HEIGHT,
            ))),
        };

        if let DisplayMode::Grey = mode {
            return self.display_greyscale(img, horizontal);
        }

        // Note: how images are moved into a buffer.
        //
        // When the image is vertical, it is transferred byte to bit as is.
        // The default orientation is such that the flexible connector of the screen is on the bottom.
        // - (0, 0) of the image corresponds to (0, 0) of the screen
        //
        // When the image is horizontal, a transformation is necessary.
        // The orientation is such that the flexible connector of the screen is on the left.
        // Therefore:
        // - (0, 0)           -> (0, ScreenH-1)
        // - (ImgW-1, 0)      -> (0, 0)
        // - (0, ImgH-1)      -> (ScreenW-1, ScreenH-1)
        // - (ImgW-1, ImgH-1) -> (ScreenW-1, 0)
        //
        // TODO: figure out how to do this using memory addressing settings

        let mut buffer = Self::buffer_white();

        // Convert the image data to be used in the buffer
        for (x, y, pixel) in img.enumerate_pixels() {
            let black = pixel.0[0] <= u8::MAX / 2;

            let (x, y) = if horizontal {
                (y as usize, (Self::HEIGHT - x - 1) as usize)
            } else {
                (x as usize, y as usize)
            };

            let mask = 0b1000_0000 >> (x % 8);
            // Just flip (xor) it, there should be no duplicates
            buffer[x / 8 + y * width] ^= if black { mask } else { 0 };
        }

        self.send_command(&[0x24])?;
        self.send_data(buffer.as_slice())?;

        self.show(mode)?;

        Ok(())
    }

    pub fn display_greyscale(&mut self, img: DisplayImage, horizontal: bool) -> Result<(), DriverError> {
        let (width, ..) = Self::image_size_bytes();

        let mut buffer_0011 = Self::buffer_white();
        let mut buffer_0101 = Self::buffer_white();

        // TODO: this pretty much copies the general `display()` method, would be nice to deduplicate
        for (x, y, pixel) in img.enumerate_pixels() {
            let color = ColorGreyscale::new(pixel);
            let bit_0011 = color.bit_0011();
            let bit_0101 = color.bit_0101();

            let (x, y) = if horizontal {
                (y as usize, (Self::HEIGHT - x - 1) as usize)
            } else {
                (x as usize, y as usize)
            };

            let mask = 0b1000_0000 >> (x % 8);
            // Just flip (xor) it, there should be no duplicates
            buffer_0011[x / 8 + y * width] ^= if bit_0011 { mask } else { 0 };
            buffer_0101[x / 8 + y * width] ^= if bit_0101 { mask } else { 0 };
        }

        self.send_command(&[0x24])?;
        self.send_data(buffer_0101.as_slice())?;

        self.send_command(&[0x26])?;
        self.send_data(buffer_0011.as_slice())?;

        self.show(DisplayMode::Grey)?;

        Ok(())
    }
}


const LUT_2BIT: &'static [u8] = &[
    0x40, 0x48, 0x80, 0x0,  0x0,  0x0,  0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x8,  0x48, 0x10, 0x0,  0x0,  0x0,  0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x2,  0x48, 0x4,  0x0,  0x0,  0x0,  0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x20, 0x48, 0x1,  0x0,  0x0,  0x0,  0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0, 0x0, 0x0, 0x0, 0x0, 0x0,
    0xA,  0x19, 0x0,  0x3,  0x8,  0x0,  0x0,
    0x14, 0x1,  0x0,  0x14, 0x1,  0x0,  0x3,
    0xA,  0x3,  0x0,  0x8,  0x19, 0x0,  0x0,
    0x1,  0x0,  0x0,  0x0,  0x0,  0x0,  0x1,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x0,  0x0,  0x0,  0x0,  0x0,  0x0,  0x0,
    0x22, 0x22, 0x22, 0x22, 0x22, 0x22, 0x0, 0x0, 0x0,
    0x22, 0x17, 0x41, 0x0,  0x32, 0x1C,
];
