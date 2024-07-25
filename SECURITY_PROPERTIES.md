# Wondermagick security properties

Wondermagick focuses on preventing [arbitrary code execution](https://en.wikipedia.org/wiki/Arbitrary_code_execution) vulnerabilities. They are by far the most insidious and devastating.

It is *not* a priority to prevent things such as:

1. Abruptly terminating the process
1. Allocating an unbounded amount of memory (memory limits are not yet implemented)
1. Taking a very long time to perform an operation, or even entering an infinite loop

We will still strive to avoid and fix such issues, but they will not be considered security vulnerabilities, and will not have a high priority.

That's because unlike code execution vulnerabilities, these issues are easy to prevent using external tools. For example, on Linux save this shell script and name it `convert`:

```bash
#!/bin/bash
ulimit -v 2000000 # 2GB memory limit
timeout 10s wm-convert "$@" # 10 seconds timeout
```

## How

All the image decoders and image processing operations are written in safe Rust.

Memory safety of our code is guaranteed by the Rust compiler. That's why our PNG decoder will never have a single code execution CVE, while libpng has an [ever-growing laundry list](https://cve.mitre.org/cgi-bin/cvekey.cgi?keyword=libpng). And our WebP decoder will never have [a code execution vulnerability exploited in the wild](https://blog.cloudflare.com/uncovering-the-hidden-webp-vulnerability-cve-2023-4863).

Really, making image processing *secure* is not the hard part. We've had memory-safe languages for decades! The problem is that image processing written in Java or Python will never have satisfactory performance. Rust enables code that is *secure and fast at the same time.*

## Exceptions

We allow potentially memory-unsafe code in certain low-risk scenarios:

1. **Explicit SIMD intrinsics.** Sadly the [portable SIMD API](https://doc.rust-lang.org/stable/std/simd/index.html) requires a nightly Rust compiler, so we allow unsafe SIMD intrinsics for now. This kind of code tends to be straightforward and easy to audit.
1. **Format encoders.** Image processing CVEs are overwhelmingly found in decoders, not encoders. Because of that the Rust ecosystem has been primarily focused on implementing decoders. The vast majority of our encoders are memory-safe as well, but we may use C implementations of encoders for certain formats when no comparable Rust implementation is available.

Not yet implemented, but planned: an ability to opt out of even this unsafe code.
