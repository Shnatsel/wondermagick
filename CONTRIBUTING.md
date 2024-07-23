# Wondermagick contribution policy

## Upstream first

The goal of `wondermagick` is to advance the Rust imaging ecosystem as a whole. We want to enable people to migrate other imaging libraries to Rust backends.  Making `wondermagick` bypass the common libraries with bespoke solutions is *not* the goal.

To that end we have a policy of **upstream-first** contributions. That is, if whatever you want to implement can be implemented in a dependency crate, such as `image`, it should be done there and not in `wondermagick`. To facilitate that, `wondermagick` may depend on unreleased versions of dependency crates (pre-releases or versions from Git).

## No unsafe code

We don't want any bespoke unsafe code in `wondermagick`. We declare `#![forbid(unsafe_code)]` to make the compiler yell at you if you try to add any.

We are also cautious with pulling in dependencies that use unsafe code. Any unsafe code that enhances performance should be disabled, with the exception of SIMD intrinsics. Any unsafe code that does get included despite the previous restriction, should be fuzzed with [`cargo fuzz`](https://github.com/rust-fuzz/cargo-fuzz) or an equally capable fuzzer (e.g. AFL) and a fuzzing harness contributed upstream. The project must also have a test suite and run it in CI under Address Sanitizer or [miri](https://github.com/rust-lang/miri). If it does not, please add such a CI setup upstream before adding the dependency to `wondermagick`.
