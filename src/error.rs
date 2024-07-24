use std::fmt::{Debug, Display};
pub struct MagickError(pub String);

impl Display for MagickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

impl Debug for MagickError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("MagickError").field(&self.0).finish()
    }
}

impl std::error::Error for MagickError {}

/// Returns a `MagickError` with the specified string, and also records the source code location where it was called.
/// We use it to imitate the structure of imagemagick's error messages.
#[macro_export]
macro_rules! wm_err {
    ($msg:expr) => {
        MagickError(format!(
            "wondermagick: {} @ {}/{}/{}.",
            $msg,
            file!(),
            // Get the function name.
            // Adapted from https://stackoverflow.com/a/40234666/585725
            {
                fn f() {}
                fn type_name_of<T>(_: T) -> &'static str {
                    std::any::type_name::<T>()
                }
                let name = type_name_of(f);
                // transform the full path that ends with "::f" to indicate a function
                // into the name of the function
                &name.rsplit("::").nth(1).unwrap_or("unknown")
            },
            line!(),
        ))
    };
}

/// Similar to the `try!` macro and the `?` operator, but also
/// records the source code location where it was called.
#[macro_export]
macro_rules! wm_try {
    ($expr:expr $(,)?) => {{
        use std::any::TypeId;

        // gets the type ID of a *value*, as opposed to a *type* that `TypeId::of` operates on
        fn get_type_id<T: std::any::Any>(_: &T) -> TypeId {
            TypeId::of::<T>()
        }

        match $expr {
            std::result::Result::Ok(val) => val,
            std::result::Result::Err(err) => {
                // Avoid appending the line numbers twice by accident
                let magick_type_id = TypeId::of::<$crate::error::MagickError>();
                if get_type_id(&err) == magick_type_id {
                    // Even though we know *at runtime* that we are dealing with a MagickError,
                    // we still need this to compile for any type.
                    // We achieve this by using `to_string()` from the `Display` trait
                    // because anything that implements `Error` also implements `Display`.
                    return std::result::Result::Err(MagickError(err.to_string()));
                } else {
                    // Convert the foreign error into our format
                    return std::result::Result::Err(wm_err!(err));
                };
            }
        }
    }};
}
