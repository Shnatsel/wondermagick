use std::ffi::OsString;
use wondermagick::{
    decode::decode, encode::encode, error::MagickError, operations::composite, wm_err, wm_try,
};

fn main() {
    let arguments: Vec<_> = std::env::args_os().collect();

    if let Err(e) = real_main(arguments) {
        eprintln!("{}", e);
        std::process::exit(1);
    }
}

fn real_main(args: Vec<OsString>) -> Result<(), MagickError> {
    if args.len() != 4 {
        return Err(wm_err!("Usage: wm-composite <input> <input> <output>"));
    }

    let input1 = &args[1];
    let input2 = &args[2];
    let output = &args[3];

    let mut img1 = wm_try!(decode(input1, None));
    let img2 = wm_try!(decode(input2, None));

    wm_try!(composite::composite(&mut img1, &img2));
    wm_try!(encode(&mut img1, output, img2.format, &Default::default()));

    Ok(())
}
