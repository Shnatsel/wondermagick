// I'm sorry, no. Just no.
// There was going to be a limit to bug compatibility and this is it.
// I can't. No. Please. 
// *sobbing*
// You see, for JPEG you can set negative quality and it maps to 100.
// Or quality that exceeds even 64-bit int and it's also 100. Even on overflow.
// So far so good.
// But PNG! Oh, PNG has an entirely different parser!
// So like, the last digit matters, and negative qualities get rejected but not at parser level,
// it just says 
// convert-im6.q16: bad parameters to zlib `/home/shnatsel/out.png' @ error/png.c/MagickPNGErrorHandler/1642.
// and values over 100 are parsed as digits instead and and and
// and other formats are going to have their own ludicrous quirks aren't they
// *wailing*
