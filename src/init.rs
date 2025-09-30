//! Initialization that needs to be done on startup

/// Performs any global state initialization that needs to be done before performing image operations
pub fn init() {
    #[cfg(feature = "jxl")]
    jxl_oxide::integration::register_image_decoding_hook();
}
