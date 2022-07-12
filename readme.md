## QR code generator in Rust

---

A qr code generator written in Rust without using external libraries.
I created this program for educational purposes. While coding I based on the ISO/IEC18004 standard.
Code in the next versions will be refactored, and there will be documentation describing each operation.
Based on my another project in ts [here](https:github.com/cnuebred/qrcode.ts)

The program allows you to generate qr code with a selected level of error correction LMQH, and in size from 1 to 40 version (21x21 - 177x177 pixels).
It is possible to set the minimum limit of the generator's options.
Currently available static options are mask '100' in byte coding.

```rs
// sample usage
mod config;
mod polynomial;
mod qr_code;
mod reed_solomon;
mod utils;
use crate::qr_code::QRcode;

fn main() {
    let mut qrcode: QRcode = QRcode::new(
        "https://youtu.be/dQw4w9WgXcQ",
        1,
        config::ErrorLevel::H,
        config::Mask::_100,
    );
    qrcode.render();
    println!("{:?}", qrcode);
    println!("{:?}", qrcode.rs.version);
    println!("{:?}", qrcode.rs.error_level);
}

```

![qr code with rick](https://i.imgur.com/6Ajt4B9.png)
