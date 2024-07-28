use std::fmt::Display;

#[derive(Default, Copy, Clone, Eq, PartialEq)]
pub struct Geometry {
    width: Option<u32>,
    height: Option<u32>,
    xoffset: Option<u32>,
    yoffset: Option<u32>,
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
            (Some(x), Some(y)) => write!(f, "{x:+}{y:+}"), // TODO: explicit sign
            (Some(x), None) => write!(f, "{x:+}"),
            (None, Some(y)) => write!(f, "+0{y:+}"),
            (None, None) => Ok(()),
        }
    }
}
