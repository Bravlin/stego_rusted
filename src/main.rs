use std::env;
use std::process;

mod bitmap;
use bitmap::BMP;

mod stego;

fn main() {
    let mut arguments = env::args();
    arguments.next();
    let command = arguments.next().expect("No command provided.");
    let file = arguments.next().expect("No path provided.");

    match &command[..] {
        "hide" => {
            let k = match arguments.next().expect("No k provided.").trim().parse() {
                Err(e) => panic!("{}", e),
                Ok(k) => k,
            };
            let text = arguments.next().expect("No text to hide provided.");
            let mut bmp = match BMP::new(&file[..]) {
                Err(e) => panic!("{}", e),
                Ok(bmp) => bmp,
            };
            stego::hide_text(&mut bmp, &text, k);
            if let Err(e) = bmp.save_as(&format!("{}.stego", file)) {
                eprintln!("{}", e);
            }
        },
        "get" => {
            let bmp = match BMP::new(&file[..]) {
                Err(e) => panic!("{}", e),
                Ok(bmp) => bmp,
            };
            let hidden_text = stego::get_text(&bmp);
            println!("{}", hidden_text);
        }
        "compare" => {
            let file2 = arguments.next().expect("No path provided for second file.");
            let bmp1 = match BMP::new(&file[..]) {
                Err(e) => panic!("{}", e),
                Ok(bmp) => bmp,
            };
            let bmp2 = match BMP::new(&file2[..]) {
                Err(e) => panic!("{}", e),
                Ok(bmp) => bmp,
            };
            let mse = BMP::mean_squared_error(&bmp1, &bmp2).expect("Images not comparable");
            let psnr = BMP::peak_signal_noise_ratio(&bmp1, &bmp2).unwrap();
            let ssim = BMP::structural_similarity(&bmp1, &bmp2).unwrap();
            println!("MSE: {}\nPSNR: {}\nSSIM: {}", mse, psnr, ssim);
        }
        _ => {
            eprintln!("Invalid command provided");
            process::exit(1);
        }
    }
}