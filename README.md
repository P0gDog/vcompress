
# vcompress

A simple command line video compression utillity

[![MIT License](https://img.shields.io/badge/License-MIT-green.svg)](https://choosealicense.com/licenses/mit/)
![Rust](https://img.shields.io/badge/rust-%23000000.svg?style=for-the-badge&logo=rust&logoColor=white)
![Linux](https://img.shields.io/badge/Linux-FCC624?style=for-the-badge&logo=linux&logoColor=black)
![FFmpeg](https://shields.io/badge/FFmpeg-%23171717.svg?logo=ffmpeg&style=for-the-badge&labelColor=171717&logoColor=5cb85c)
## Features

- MP4 support
- Easy to use
- Customizable audio and video quality
- Perfect for Discord Nitro limits


## Usage/Example

```
  -o, --output <OUTPUT>          output
  -t, --target-mb <TARGET_MB>    targ. size [default: 9]
  -a, --audio-kbps <AUDIO_KBPS>  audio bitrate [default: 128]
  -h, --help                     Print help
  -V, --version                  Print version
```
Example use:
```
vcompress example.mp4
```
Compressed files are output as example-compressed.mp4
## Installation
<small>Now Supports Windows!</small>
### Via crates.io

Requires the Rust toolchain and `ffmpeg` installed on your system.

    cargo install vcompress

This puts the `vcompress` binary in `~/.cargo/bin`. Please make sure that's on your `$PATH`
(`export PATH="$HOME/.cargo/bin:$PATH"` in your `.bashrc`/`.zshrc` if it isn't already).

Arch:

    sudo pacman -S ffmpeg

Debian/Ubuntu:

    sudo apt install ffmpeg

macOS:

    brew install ffmpeg

### Windows
In PowerShell, run:

    cargo install vcompress
    winget install ffmpeg

`cargo install` puts the binary in `%USERPROFILE%\.cargo\bin`, which the Rust
installer adds to `PATH` automatically.

No Rust toolchain? Grab `vcompress.exe` directly from the
[latest release](https://github.com/P0gDog/vcompress/releases/latest) instead.
### AUR

AUR uploads are currently suspended Arch-wide due to a security incident.
Until it reopens, please build manually:

    git clone https://github.com/P0gDog/vcompress.git
    cd vcompress
    makepkg -sic -p packaging/arch/PKGBUILD
