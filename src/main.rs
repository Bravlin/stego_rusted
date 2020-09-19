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
            if bmp.save_as(&format!("stego_{}", file)).is_err() {
                eprintln!("An error ocurred while saving the modified image.");
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
        _ => {
            eprintln!("Invalid command provided");
            process::exit(1);
        }
    }
}