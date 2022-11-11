//Read in the png and output the data as a text array
extern crate png;

use std::fs::File;
use std::io::prelude::*;

const IMAGE_FILENAME: &'static str = "../../assets/gfx.png";
// for testing
// const IMAGE_FILENAME: &'static str = "assets/pallete.png";

const OUTPUT_FILENAME: &'static str = "../../libs/assets/src/gfx.in";
// for testing
// const OUTPUT_FILENAME: &'static str = "out.txt";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let decoder = png::Decoder::new(File::open(IMAGE_FILENAME)?);
    let (info, mut reader) = decoder.read_info()?;
    println!(
        "{} : {:?}",
        IMAGE_FILENAME,
        (
            info.width,
            info.height,
            info.color_type,
            info.bit_depth,
            info.line_size
        )
    );
    // Allocate the output buffer.
    let mut buf = vec![0; info.buffer_size()];
    // Read the next frame. Currently this function should only called once.
    // The default options
    reader.next_frame(&mut buf)?;

    let output_filename = OUTPUT_FILENAME;

    let mut file = File::create(output_filename)?;

    use png::ColorType::*;
    let pixel_width = match info.color_type {
        RGBA => 4,
        _ => unimplemented!(
            "This program cannot handle {:?} images (yet.)",
            info.color_type
        ),
    };

    let mut pixels = Vec::with_capacity(buf.len() / pixel_width);

    for colour in buf.chunks(pixel_width) {
        let argb =
        ((colour[3] as u32) << 24)
        | ((colour[0] as u32) << 16)
        | ((colour[1] as u32) << 8)
        | ((colour[2] as u32));

        pixels.push(argb);
    }

    let mut output = String::with_capacity(
        pixels.len() * "0xFFFFFFFF, ".len()
        // Newlines for each row
        + 1024
        // Extra for start and end of array
        + 8
    );
    output.push_str("[\n");
    for chunk in pixels.chunks(512) {
        for colour in chunk.iter() {
            output.push_str(&format!("0x{colour:08X}, "));
        }
        output.push('\n');
    }
    output.push_str("]\n");

    file.write_all(output.as_bytes())?;

    println!("overwrote {}", output_filename);

    Ok(())
}
