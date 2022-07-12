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

#[test]
fn test_qrcode_version() {
    let qrcode: QRcode = QRcode::new(
        "https://youtu.be/dQw4w9WgXcQ",
        1,
        config::ErrorLevel::H,
        config::Mask::_100,
    );
    assert_eq!(qrcode.rs.version, 4);
}
