mod png;

use pixel_canvas::Color;
use png::PNG;
use std::fs;

fn main() {
    let bytes = fs::read("/home/lapepega/Downloads/rgb.png").expect("couldn't read png");
    assert_eq!(
        bytes[..8],
        vec![137, 80, 78, 71, 13, 10, 26, 10],
        "Png signature is invalid"
    );

    let chunk_bytes: Vec<u8> = bytes[8..].into();
    let image = PNG::from_bytes(chunk_bytes);
    for (i, c) in image.chunks.iter().enumerate() {
        println!("Chunk {}:\tType:{}\tLen:{}", i, c.type_str(), c.length);
    }

    println!("{:#?}", image.header);

    let canvas =
        pixel_canvas::Canvas::new(image.header.width as usize, image.header.height as usize)
            .title("PNG");

    canvas.render(move |state, img| {
        for (rgb, clr) in image
            .image
            .0
            .iter()
            .rev()
            .zip(img.chunks_mut(image.header.width as usize))
        {
            for (rgb_px, clr_px) in rgb.iter().zip(clr.iter_mut()) {
                *clr_px = Color {
                    r: rgb_px.0,
                    g: rgb_px.1,
                    b: rgb_px.2,
                }
            }
        }
    });
}
