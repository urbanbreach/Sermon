# ASIO Support in Sermon

## Overview

Sermon supports ASIO (Audio Stream Input/Output) for low-latency, high-fidelity audio playback. ASIO is the industry standard for professional audio interfaces on Windows and is often required for native DSD bitstreaming.

## Build Prerequisites

The ASIO backend in Sermon is implemented using the `asio-sys` crate, which generates Rust bindings for the Steinberg ASIO SDK using `bindgen`. This process requires **LLVM/Clang** to be installed on your build machine.

### LLVM/Clang Installation (Windows)

Choose one of the following methods to install LLVM/Clang:

#### Method 1: Chocolatey (Recommended)

If you use [Chocolatey](https://chocolatey.org/), run the following command in an Administrator shell:

```powershell
choco install llvm
```

#### Method 2: Manual Download

1. Download the LLVM installer from the [LLVM GitHub Releases](https://github.com/llvm/llvm-project/releases) page (e.g., `LLVM-xx.x.x-win64.exe`).
2. Run the installer and ensure you select the option **"Add LLVM to the system PATH for all users"** (or for the current user).

#### Method 3: Winget

```powershell
winget install LLVM.LLVM
```

## Environment Setup

In most cases, the build system will automatically find LLVM if it is in your system `PATH`. However, if you encounter `bindgen` errors during build, you may need to set the `LIBCLANG_PATH` environment variable.

1. Find the path to your LLVM `bin` directory (e.g., `C:\Program Files\LLVM\bin`).
2. Set the `LIBCLANG_PATH` environment variable to this directory:

```powershell
$env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
```

To make this change permanent, add it to your system Environment Variables.

## ASIO SDK

Sermon uses a GPLv3-friendly approach to the Steinberg ASIO SDK. The `asio-sys` crate automatically downloads the necessary SDK headers during the build process. No manual download of the Steinberg SDK is required.

## Verification

To verify that your build environment is correctly set up for ASIO support, run a build of the `audio-engine` crate:

```bash
cargo build -p audio-engine
```

If the build completes successfully, `bindgen` was able to find Clang and generate the necessary bindings.

## Troubleshooting

### "libclang.dll not found" or "unable to find libclang"

This is the most common error when LLVM/Clang is not correctly installed or not in the `PATH`.
- **Check PATH**: Run `clang --version` in your terminal. If it's not found, LLVM is not in your `PATH`.
- **Set LIBCLANG_PATH**: Explicitly set the path to the `bin` directory of your LLVM installation as described in [Environment Setup](#environment-setup).

### ASIO SDK Download Fails

The build process requires an active internet connection to download the ASIO SDK from Steinberg's servers. If you are behind a proxy, ensure your `http_proxy` and `https_proxy` environment variables are correctly configured for Cargo.

## See Also

- [ADR 0011: ASIO Strategy](adr/0011-asio-strategy.md)
- [ADR 0006: Audio Output (WASAPI)](adr/0006-audio-output-wasapi.md)
