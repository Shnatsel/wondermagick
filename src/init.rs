//! Initialization that needs to be done on startup

/// Performs any global state initialization that needs to be done before performing image operations
pub fn init() {
    #[cfg(feature = "jxl")]
    jxl_oxide::integration::register_image_decoding_hook();
    #[cfg(feature = "jpeg2000")]
    hayro_jpeg2000::integration::register_decoding_hook();
    #[cfg(any(
        feature = "otb",
        feature = "pcx",
        feature = "sgi",
        feature = "wbmp",
        feature = "xbm",
        feature = "xpm"
    ))]
    image_extras::register();
}
