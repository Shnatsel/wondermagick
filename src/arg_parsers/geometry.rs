// Fun fact: the geometry documentation at https://www.imagemagick.org/Magick++/Geometry.html is a lie.
//
// It says things like
// > Offsets must be given as pairs; in other words, in order to specify either xoffset or yoffset both must be present.
// but this works:
// `convert rose: -crop 50x+0 crop_half.gif`
//
// It also says:
// > Extended geometry strings should *only* be used when *resizing an image.*
// but this works:
// `convert rose: -crop 50% crop_half.gif`
// (but maybe -crop is just magical, and other places where geometry can appear aren't like that?)
//
// So we just rely on observing the actual behavior of `convert` instead.
// Note that this isn't targeting any single particular command yet.
// That is a problem, and this should be changed to adhere to something specific.

use std::fmt::Display;
use std::str;

#[derive(Default, Copy, Clone, PartialEq)]
pub struct Geometry {
    width: Option<f64>,
    height: Option<f64>,
    xoffset: Option<f64>,
    yoffset: Option<f64>,
}

impl Display for Geometry {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(w) = self.width {
            write!(f, "{w}")?;
        }
        if let Some(h) = self.height {
            write!(f, "x{h}")?;
        }
        match (self.xoffset, self.yoffset) {
            (Some(x), Some(y)) => write!(f, "{x:+}{y:+}"),
            (Some(x), None) => write!(f, "{x:+}"),
            (None, Some(y)) => write!(f, "+0{y:+}"),
            (None, None) => Ok(()),
        }
    }
}

fn read_positive_float(input: &mut &[u8]) -> Option<f64> {
    let mut count = count_leading_digits(input);
    if count == 0 {
        return None;
    }
    if input.get(count) == Some(&b'.') {
        // imagemagick permits having a trailing dot with no digits following it
        count += 1;
        count += count_leading_digits(&input[..count]);
    }
    let (number, remainder) = input.split_at(count);
    let float = str::from_utf8(number).unwrap().parse::<f64>().unwrap();
    *input = remainder;
    Some(float)
}

fn count_leading_digits(input: &[u8]) -> usize {
    input
        .iter()
        .copied()
        .take_while(|b| b.is_ascii_digit())
        .count()
}
