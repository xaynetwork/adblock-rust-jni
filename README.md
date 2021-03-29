# Rust Adblocker JNI bindings for Android

[![](https://img.shields.io/github/v/tag/xaynetwork/adblock-rust-jni.svg?label=version)](https://github.com/xaynetwork/adblock-rust-jni/packages)

### install




### Preparation 

You need Java, the Android SDK the Android NDK (version 21.3.6528147) and rust (tested on 1.47.0).
Then install the following toolchains


Mandatory for the AAR
```
rustup target add armv7-linux-androideabi   # for arm
rustup target add i686-linux-android        # for x86
rustup target add x86_64-linux-android      # for x86_64
rustup target add aarch64-linux-android     # for arm64
```

One of the following need to be installed to run tests on the host system
```
# For Linux testing
rustup target add x86_64-unknown-linux-gnu  # for linux-x86-64
# For MacOS testing
rustup target add x86_64-apple-darwin       # for darwin (macOS)
# For Windows testing
rustup target add x86_64-pc-windows-gnu     # for win32-x86-64-gnu
```

### Building

To create a debug (alt: `buildRelease` for release) aar run:

```bash
cd aar
./gradlew build
```

### Testing

```bash
cd aar
./gradlew test
```

## TODO

- Add licence
- Add more documenation on the Adblocker interface
- create tests for tags and resources
- create benchmark
- add android example project with webview 
- add verification on CI
- fix not throwing exceptions
- add proguard rules to keep jni names

 