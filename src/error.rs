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

/// Like `format!`, but returns a `MagickError` instead of a `String`,
/// and records the source code location where it was called.
/// We use it to imitate the structure of imagemagick's error messages.
#[macro_export]
macro_rules! wm_err {
    ($($arg:tt)*) => {{
        // This is a copy of the implementation of `format!`
        let res = std::fmt::format(std::format_args!($($arg)*));
        // Now that we've lowered our arguments to a string, add the prefix and source code location
        MagickError(format!(
            "wondermagick: {} @ {}/{}/{}.",
            res,
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
    }};
}

/// Similar to the `try!` macro and the `?` operator, but always converts the result to `MagickError`.
/// It is implemented as a macro to record the source code location where it is called.
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
                // The type ID check is to avoid appending the line numbers twice by accident
                // by converting MagickError into itself.
                // I haven't figured out how to complain about that at compile time
                // with a helpful error message, so we just prevent it at runtime.
                // This only happens a handful of times per execution and only when we're bailing out anyway,
                // so we can easily afford the slight runtime overhead of copying a string.
                let magick_type_id = TypeId::of::<$crate::error::MagickError>();
                if get_type_id(&err) == magick_type_id {
                    // Even though we know *at runtime* that we are dealing with a MagickError,
                    // we still need this to compile for any type.
                    // We achieve this by using `to_string()` from the `Display` trait
                    // because anything that implements `Error` also implements `Display`.
                    return std::result::Result::Err(MagickError(err.to_string()));
                } else {
                    // Convert the foreign error into our format
                    return std::result::Result::Err(wm_err!("{}", err));
                };
            }
        }
    }};
}
