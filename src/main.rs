use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    use image::io::Reader as ImageReader;
    let input = std::env::args().nth(1).unwrap();
    let output = std::env::args().nth(2).unwrap();
    let img = ImageReader::open(input)?.with_guessed_format()?.decode()?;
    
    img.save(output)?;
    Ok(())
}