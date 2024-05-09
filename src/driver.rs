use image::{ImageBuffer, Luma};
use log::info;
use rppal::gpio::{Gpio, InputPin, OutputPin, Level::*};
use rppal::spi::{self, Spi, Bus, SlaveSelect};

use super::util::*;


#[derive(Debug)]
pub enum DriverError {
    Gpio(rppal::gpio::Error),
    Spi(rppal::spi::Error),
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


#[derive(Clone, Copy, Debug)]
pub enum DisplayMode {
    Full,
    Fast,
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Full => "🐢",
            Self::Fast => "🐇",
        };
        write!(f, "{}", s)
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
/// It stores the Waveform which defines the relation between grayscale, voltage and temperature.
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

    pub fn image_white() -> DisplayImage {
        let mut img = DisplayImage::new(Self::WIDTH, Self::HEIGHT);

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
        info!("show ({})", mode);

        // Display update control
        self.send_command(&[0x22])?;
        match mode {
            // Load temperature value, Display with mode 1
            DisplayMode::Full => self.send_data(&[0xF7])?,
            // Display with mode 1
            DisplayMode::Fast => self.send_data(&[0xC7])?,
        }

        // Execute the selected update sequence
        self.send_command(&[0x20])?;
        self.wait_not_busy();

        Ok(())
    }


    pub fn init(&mut self, mode: DisplayMode) -> Result<(), DriverError> {
        info!("init ({})", mode);

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

        // Set RAM Y address start/end position
        let [y_byte_1, y_byte_2] = (Self::HEIGHT as u16 - 1).to_le_bytes();
        self.send_command(&[0x45])?;
        self.send_data(&[0x00, 0x00, y_byte_1, y_byte_2])?;

        // Data entry mode
        self.send_command(&[0x11])?;
        // Y increment, X increment, counter updated in X direction
        self.send_data(&[0b0000_0011])?;

        Ok(())
    }

    pub fn clear(&mut self, mode: DisplayMode) -> Result<(), DriverError> {
        info!("clear ({})", mode);
        self.display(Self::image_white(), mode)?;
        Ok(())
    }

    pub fn display(&mut self, img: DisplayImage, mode: DisplayMode) -> Result<(), DriverError> {
        info!("display ({})", mode);

        // Set RAM Y address count to 0
        self.send_command(&[0x4F])?;
        self.send_data(&[0x00, 0x00])?;

        let (width, ..) = Self::image_size_bytes();
        let mut buffer = Self::buffer_white();

        // Convert the image data to be used in the buffer
        for (x, y, pixel) in img.enumerate_pixels() {
            let black = pixel.0[0] <= u8::MAX / 2;

            // Need to make the bit black?
            if black {
                let (x, y) = (x as usize, y as usize);
                let mask = 0b10000000 >> (x % 8);
                // Just flip (xor) it, there should be no duplicates
                buffer[x / 8 + y * width] ^= mask;
            }
        }

        self.send_command(&[0x24])?;
        self.send_data(buffer.as_slice())?;

        self.show(mode)?;

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
}
