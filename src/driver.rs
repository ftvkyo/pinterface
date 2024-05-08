use image::{ImageBuffer, Luma};
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
        let mut image = DisplayImage::new(Self::WIDTH, Self::HEIGHT);

        for x in 0..Self::WIDTH {
            for y in 0..Self::HEIGHT {
                image.put_pixel(x, y, Luma([u8::MAX]));
            }
        }

        return image;
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

    fn display_on(&mut self) -> Result<(), DriverError>  {
        // Display Update Control
        self.send_command(&[0x22])?;
        self.send_data(&[0xF7])?;

        // Activate Display Update Sequence
        self.send_command(&[0x20])?;
        self.wait_not_busy();

        Ok(())
    }


    pub fn init(&mut self) -> Result<(), DriverError> {
        self.pwr.write(High);

        self.reset();
        self.wait_not_busy();

        // SWRESET
        self.send_command(&[0x12])?;

        // Set RAM Y address start/end position
        let [byte1, byte2] = (Self::HEIGHT as u16 - 1).to_le_bytes();
        self.send_command(&[0x45])?;
        self.send_data(&[0x00, 0x00, byte1, byte2])?;

        // Set RAM Y address count to 0
        self.send_command(&[0x4F])?;
        self.send_data(&[0x00, 0x00])?;

        // Data entry mode
        self.send_command(&[0x11])?;
        self.send_data(&[0x03])?;

        Ok(())
    }

    pub fn clear(&mut self) -> Result<(), DriverError> {
        self.display(Self::image_white())?;
        Ok(())
    }

    pub fn display(&mut self, image: DisplayImage) -> Result<(), DriverError> {
        let (width, ..) = Self::image_size_bytes();
        let mut buffer = Self::buffer_white();

        // Convert the image data to be used in the buffer
        for (x, y, pixel) in image.enumerate_pixels() {
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

        self.display_on()?;

        Ok(())
    }

    pub fn sleep(&mut self) -> Result<(), DriverError> {
        self.send_command(&[0x10])?;
        self.send_data(&[0x01])?;

        sleep_ms(2000);

        self.rst.write(Low);
        self.dc.write(Low);
        self.pwr.write(Low);

        Ok(())
    }
}
