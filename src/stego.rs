use crate::bitmap::BMP;

const MAX_TEXT_SIZE: u16 = 256;
const BYTE_SIZE: u8 = 8;

fn padding_jump(prev_byte: u8, k: u8, padding_left: usize) -> u8 {
    let xor = padding_left as u8 ^ prev_byte;

    if xor & 0x01 != 0x00 {
        (xor%(BYTE_SIZE*k) + 1) | (padding_left + 1) as u8
    } else {
        0
    }
}

fn hide_byte(byte: u8, k: u8, bmp: &mut BMP, pixel_number: &mut usize) {
    let mut shift = BYTE_SIZE;
    let mut pixel;

    while shift > k {
        shift -= k;
        pixel = bmp.pixel_as_mut(*pixel_number);
        pixel[0] &= 0xFF << k;
        pixel[0] |= (byte >> shift) & !(0xFF << k);
        *pixel_number += 1;
    }
    pixel = bmp.pixel_as_mut(*pixel_number);
    let mask = 0xFFu8.checked_shl(shift as u32).unwrap_or(0);
    pixel[0] &= mask;
    pixel[0] |= byte & !(mask);
    *pixel_number += 1;
}

fn get_byte(bmp: &BMP, k: u8, pixel_number: &mut usize) -> u8 {
    let mut mask = 0xFFu8;
    let mut shift = BYTE_SIZE;
    let mut res = 0;

    while shift > k {
        shift -= k;
        res |= (bmp.pixel(*pixel_number)[0] << shift) & mask;
        mask >>= k;
        *pixel_number += 1;
    }
    res |= bmp.pixel(*pixel_number)[0] & !(0xFFu8.checked_shl(shift as u32).unwrap_or(0));
    *pixel_number += 1;

    res
}

pub fn hide_text(bmp: &mut BMP, text: &str, k: u8) {
    let usable_pixels = (bmp.width() * bmp.height().abs() - 3) as usize;

    assert!(k > 0);
    assert!(k <= BYTE_SIZE);
    assert!(text.len() > 0);
    assert!(text.len() as u16 <= MAX_TEXT_SIZE);
    assert!(usable_pixels > 0);
    {
        let bits_available = usable_pixels * k as usize;
        let bits_to_hide = (text.len() + 1) * BYTE_SIZE as usize;
        assert!(bits_available > bits_to_hide);
    }

    let mut pixel_number = 0;
    
    for i in (0..=2).rev() {
        let pixel = bmp.pixel_as_mut(pixel_number);
        pixel[0] &= 0xFE;
        pixel[0] |= ((k - 1) >> i) & 0x01;
        pixel_number += 1;
    }

    hide_byte(text.len() as u8, k, bmp, &mut pixel_number);

    let mut jump;
    let pixels_per_char = {
        let mut aux = BYTE_SIZE/k;
        if BYTE_SIZE%k != 0 {
            aux += 1;
        }
        aux
    };
    let mut padding_left = usable_pixels - (text.len() + 1) * pixels_per_char as usize;

    for byte in text.bytes() {
        if padding_left > 0 {
            jump = padding_jump(bmp.pixel(pixel_number - 1)[0], k, padding_left) as usize;
            pixel_number += jump;
            padding_left -= jump;
        }
        hide_byte(byte, k, bmp, &mut pixel_number);
    }
}

pub fn get_text(bmp: &BMP) -> String {
    assert!(bmp.pixel_array_size() > 0);

    let mut k = 0;
    let mut pixel_number = 0;

    // Recovers k from the first 3 pixels
    for i in (0..=2).rev() {
        k |= (bmp.pixel(pixel_number)[0] & 0x01) << i;
        pixel_number += 1;
    }
    k += 1;

    let text_size = get_byte(bmp, k, &mut pixel_number);

    let pixels_per_char = {
        let mut aux = BYTE_SIZE/k;
        if BYTE_SIZE%k != 0 {
            aux += 1;
        }
        aux
    };
    let usable_pixels = (bmp.width() * bmp.height().abs() - 3) as usize;
    let mut padding_left = usable_pixels - ((text_size + 1) * pixels_per_char) as usize;
    let mut text: Vec<u8> = Vec::with_capacity(text_size as usize);
    let mut jump;

    for _ in 0..text_size {
        if padding_left > 0 {
            jump = padding_jump(bmp.pixel(pixel_number - 1)[0], k, padding_left) as usize;
            pixel_number += jump;
            padding_left -= jump;
        }
        text.push(get_byte(bmp, k, &mut pixel_number));
    }

    String::from_utf8(text).unwrap()
}

#[cfg(test)]
mod tests {
    use crate::stego;
    use crate::bitmap::BMP;
    use std::fs;

    #[test]
    fn hide_and_get() {
        let mut img = BMP::new("example_images/tiger.bmp").unwrap();
        stego::hide_text(&mut img, "Hello, World!", 2);
        assert!(img.save_as("example_images/tiger.bmp.stego").is_ok());
        
        img = BMP::new("example_images/tiger.bmp.stego").unwrap();
        let hidden_text= stego::get_text(&img);
        assert_eq!("Hello, World!", hidden_text);
        assert!(fs::remove_file("example_images/tiger.bmp.stego").is_ok())
    }
}