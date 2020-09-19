use std::fs;
use std::error::Error;
use std::convert::TryFrom;

#[derive(Clone)]
pub struct BMP {
    width: i32,
    height: i32,
    pixel_size: u16,
    pixel_array_offset: u32,
    contents: Vec<u8>,
}

impl BMP {
    pub fn new(file: &str) -> Result<Self, Box<dyn Error>> {
        let contents = fs::read(file)?;
        assert_eq!(b"BM", &contents[..2]);
        
        let pixel_array_offset = u32::from_le_bytes(<[u8; 4]>::try_from(&contents[10..14])?);

        let width = i32::from_le_bytes(<[u8; 4]>::try_from(&contents[18..22])?);

        let height = i32::from_le_bytes(<[u8; 4]>::try_from(&contents[22..26])?);

        let pixel_size = u16::from_le_bytes(<[u8; 2]>::try_from(&contents[28..30])?);

        assert!(pixel_size >= 8);

        Ok(Self { width, height, pixel_size, pixel_array_offset, contents })
    }

    pub fn width(&self) -> i32 {
        self.width
    }

    pub fn height(&self) -> i32 {
        self.height
    }

    pub fn pixel_size(&self) -> u16 {
        self.pixel_size
    }

    pub fn bytes_per_pixel(&self) -> u8 {
        (self.pixel_size() / 8) as u8
    }

    pub fn padding_per_row(&self) -> u8 {
        let aux = (self.width * self.bytes_per_pixel() as i32 % 4) as u8;
        if aux == 0 { 0 } else { 4 - aux }
    }

    pub fn row_size(&self) -> u32 {
        (self.bytes_per_pixel() as u32 * self.width as u32 + 31) / 32 * 4
    }

    pub fn pixel_array_size(&self) -> usize {
        self.row_size() as usize * self.height.abs() as usize
    }

    pub fn pixel_as_mut(&mut self, index: usize) -> &mut [u8] {
        let mut start = index + self.pixel_array_offset as usize;
        if self.padding_per_row() > 0 {
            start += index / self.width() as usize;
        }
        let end = start + self.bytes_per_pixel() as usize;
        &mut self.contents[start..end]
    }

    pub fn pixel(&self, index: usize) -> &[u8] {
        let mut start = index + self.pixel_array_offset as usize;
        if self.padding_per_row() > 0 {
            start += index / self.width() as usize;
        }
        let end = start + self.bytes_per_pixel() as usize;
        &self.contents[start..end]
    }

    pub fn save_as(&self, path: &str) -> Result<(), Box<dyn Error>> {
        fs::write(path, &self.contents[..])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::BMP;
    use std::fs;

    #[test]
    fn copy() {
        let img = BMP::new("example_images/tiger.bmp").unwrap();
        assert!(img.save_as("example_images/tiger_copy.bmp").is_ok());
        assert!(fs::remove_file("example_images/tiger_copy.bmp").is_ok());
    }

    #[test]
    fn dimensions() {
        let img = BMP::new("example_images/tiger.bmp").unwrap();
        assert_eq!(630, img.width());
        assert_eq!(354, img.height().abs());
    }
}