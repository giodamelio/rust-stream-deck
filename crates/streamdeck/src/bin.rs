use std::io::Read;

use pretty_hex::*;
use streamdeck::*;

fn write_to_tmp_file(name: String, data: Vec<u8>) {
    let path = format!(
        "C:\\Users\\giodamelio\\projects\\rust-stream-deck\\crates\\streamdeck\\tmp\\{}.txt",
        name,
    );
    std::fs::write(path, &data).unwrap();
}

fn main() {
    let device = get_first_device();

    // println!("{:?}", device.firmware_version());
    // println!("{:?}", device.serial_number());
    // println!("About to start reading bytes");
    // for byte in device.bytes() {
    //     println!("{}", byte.unwrap());
    // }
    for n in 0x0..=0xFF {
        match device.get_feature_report::<32>(n) {
            Err(_err) => {
                // println!("Feature report error {:#02X?}: {:?}", n, err);
            }
            Ok(report) => {
                // write_to_tmp_file(n.to_string(), report.clone());

                println!(
                    "Feature report response {:#02X?}: {:?}",
                    n,
                    report.hex_dump(),
                );
            }
        }
    }

    loop {}
}
