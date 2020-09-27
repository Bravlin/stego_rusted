use std::{fs, error::Error, convert::TryFrom, fmt};

const K1: f64 = 0.01;
const K2: f64 = 0.03;
const L: f64 = 255.;

#[derive(Debug)]
pub struct NotComparableError;

impl fmt::Display for NotComparableError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        write!(f, "Provided BMPs are not comparable.")
    }
}

impl Error for NotComparableError {}

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

    pub fn mean_squared_error(bmp1: &Self, bmp2: &Self) -> Result<f64, NotComparableError> {
        if !Self::comparable(bmp1, bmp2) {
            Err(NotComparableError)
        } else {
            let (width, height) = (bmp1.width, bmp1.height.abs());
            let (mut aux1, mut aux2, mut index);

            aux2 = 0;
            for i in 0..height {
                aux1 = 0;
                for j in 0..width {
                    index = (i*width + j) as usize;
                    aux1 += (
                        bmp1.pixel_as_usize(index) as isize - bmp2.pixel_as_usize(index) as isize
                    ).pow(2);
                }
                aux2 += aux1;
            }

            Ok(aux2 as f64 / (height*width) as f64)
        }
    }

    pub fn peak_signal_noise_ratio(bmp1: &Self, bmp2: &Self) -> Result<f64, NotComparableError> {
        if !Self::comparable(bmp1, bmp2) {
            Err(NotComparableError)
        } else {
            let mut actual_max_value = 0;
            let max_value = 2_usize.pow(bmp1.pixel_size as u32) - 1;
            let image_size = (bmp1.width * bmp1.height.abs()) as usize;
            let mut pixel_usize;

            let mut i = 0;
            while i < image_size && actual_max_value < max_value {
                pixel_usize = bmp1.pixel_as_usize(i);
                if pixel_usize > actual_max_value {
                    actual_max_value = pixel_usize;
                }
                i += 1;
            }

            let mse = Self::mean_squared_error(bmp1, bmp2).unwrap();
            Ok(10. * (actual_max_value.pow(2) as f64 / mse).log10())
        }
    }

    pub fn structural_similarity(bmp1: &Self, bmp2: &Self) -> Result<f64, NotComparableError> {
        if !Self::comparable(bmp1, bmp2) {
            Err(NotComparableError)
        } else {
            let (c1, c2) = ((L*K1).powi(2), (L*K2).powi(2));
            let c3 = c2/2.;
            let image_size = (bmp1.width * bmp1.height.abs()) as usize;
            let (mut xmu, mut xvar, mut ymu, mut yvar, mut covar) = (0., 0., 0., 0., 0.);
            let (mut xdelta, mut ydelta);

            for i in 0..image_size {
                xmu += bmp1.pixel_as_usize(i) as f64;
                ymu += bmp2.pixel_as_usize(i) as f64;
            }
            xmu /= image_size as f64;
            ymu /= image_size as f64;

            for i in 0..image_size {
                xdelta = bmp1.pixel_as_usize(i) as f64 - xmu;
                ydelta = bmp2.pixel_as_usize(i) as f64 - ymu;
                xvar += xdelta.powi(2);
                yvar += ydelta.powi(2);
                covar += xdelta * ydelta;
            }
            xvar = (xvar/(image_size as f64 - 1.)).sqrt();
            yvar = (yvar/(image_size as f64 - 1.)).sqrt();
            covar /= image_size as f64;

            let luminance = (2.*xmu*ymu + c1) / (c1 + xmu.powi(2) + ymu.powi(2));
            let contrast = (2.*xvar*yvar + c2) / (c2 + xvar.powi(2) + yvar.powi(2));
            let structure = (covar + c3) / (xvar*yvar + c3);

            Ok(luminance * contrast * structure)
        }
    }

    fn comparable(bmp1: &Self, bmp2: &Self) -> bool {
        bmp1.width == bmp2.width
        && bmp1.height == bmp2.height
        && bmp1.pixel_size == bmp2.pixel_size
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

    fn pixel_as_usize(&self, index: usize) -> usize {
        let pixel = self.pixel(index);
        let mut aux = 0;
        for byte in pixel.iter().rev() {
            aux = aux << 8 | *byte as usize;
        }
        aux
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