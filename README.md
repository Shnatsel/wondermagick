# Wondermagick

A memory-safe replacement for [imagemagick](https://en.wikipedia.org/wiki/ImageMagick). It's also [really fast](BENCHMARKS.md)!

## Why?

Vulnerabilities in image processing are devastating, because image processing is *everywhere.* And while the share of memory safety vulnerabilities [across all software is 70%](https://alexgaynor.net/2020/may/27/science-on-memory-unsafety-and-security/), in image manipulation it is closer to 100%. Binary format parsing is notorious for these kinds of vulnerabilities.

It is not a theoretical concern. Every C image parsing library has a history of serious memory safety vulnerabilities. Imagemagick itself has had [many more](https://www.cvedetails.com/vulnerability-list/vendor_id-1749/Imagemagick.html). And they are being [exploited in the wild](https://chromereleases.googleblog.com/2023/09/stable-channel-update-for-desktop_11.html).

Trying to secure massive memory-unsafe codebases is [untenable](https://www.usenix.org/conference/enigma2021/presentation/gaynor), but migrating to memory safe languages [works](https://security.googleblog.com/2022/12/memory-safe-languages-in-android-13.html).

Thanks to Rust, we can now eradicate these vulnerabilities once and for all, without sacrificing performance!

## Current status

The underlying image format decoders and encoders are very mature. The Rust community has been developing them and using them in production for years. They have been tested on millions of real-world images.

`wondermagick` itself is in the early stages of development. We are currently focusing on converting and resizing images, which is the most common workload for `imagemagick`.

## Contributing

You can help by:

1. Funding this project, so that we could advance both `wondermagick` and the Rust [`image`](https://github.com/image-rs/image/) library
1. Funding https://www.memorysafety.org/, a registered non-profit, so they could complete [`rav1d`](https://github.com/memorysafety/rav1d) which will enable us to decode AVIF images (encoding works already)
1. Contributing to the libraries `wondermagick` relies on. We will publish a list of issues affecting us soon. TODO: publish a list
1. Implementing more `imagemagick` commands in `wondermagick`. See [CONTRIBUTING.md](CONTRIBUTING.md) for details on code contributions.
1. Making bindings to Rust [`image`](https://github.com/image-rs/image/) for your favourite language. Making drop-in replacements for other memory-unsafe systems. Not the whole world runs on `imagemagick`.
