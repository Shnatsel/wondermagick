# Wondermagick security properties

Wondermagick focuses on preventing [arbitrary code execution](https://en.wikipedia.org/wiki/Arbitrary_code_execution) vulnerabilities. They are by far the most insidious and devastating.

The vast majority of the code in `wondermagick` is memory-safe and cannot have memory corruption bugs by construction. Our PNG decoding library will never have a single code execution CVE, while libpng has an [ever-growing list](https://cve.mitre.org/cgi-bin/cvekey.cgi?keyword=libpng).

However, it is *not* a priority to prevent things such as:

1. Abruptly terminating the process
1. Allocating an unbounded amount of memory
1. Taking a very long time to perform an operation, or even entering an infinite loop

We will still strive to avoid and fix such issues, but they will not be considered security vulnerabilities, and will not have a high priority.

Unlike code execution vulnerabilities, these issues are easy to prevent using external tools. For example, on Linux save this shell script and name it `convert`:

```bash
#!/bin/bash
ulimit -v 2000000 # 2GB memory limit
timeout 10s wm-convert "$@" # 10 seconds timeout
```

This is far more robust than any limits we could implement inside the `wondermagick` binary.

## Exceptions

We allow potentially memory-unsafe code in certain low-risk scenarios:

1. **Explicit SIMD.** Sadly the [portable SIMD API](https://doc.rust-lang.org/stable/std/simd/index.html) requires a nightly Rust compiler, so we allow unsafe SIMD intrinsics for now. This kind of code tends to be straightforward and easy to audit. We also support SIMD implemented in inline assembly as an opt-in feature, disabled by default.
1. **Format encoders.** Image processing CVEs are overwhelmingly found in decoders, not encoders. Because of that the Rust ecosystem has been primarily focused on implementing decoders. The vast majority of our encoders are memory-safe as well, but we may use C implementations of encoders for certain formats when no comparable Rust implementation is available.

Not yet implemented, but planned: an ability to opt out of even this unsafe code.

## Exploit mitigations

As a defense-in-depth measure against the remaining small amount of memory-unsafe code, the `wondermagick` binary ships with a [hardened memory allocator](https://github.com/microsoft/mimalloc?tab=readme-ov-file#secure-mode), trading a small amount of runtime performance for enhanced security.
