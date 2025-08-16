use std::{
    error::Error,
    io::{BufRead, BufReader},
    time::Duration,
};

use common::Message;
use serialport::SerialPortType;

fn main() -> Result<(), Box<dyn Error>> {
    let ports = serialport::available_ports().expect("No ports found");

    println!("{} ports available", ports.len());

    for port in ports {
        if let SerialPortType::UsbPort(info) = &port.port_type
            && info.vid == 0x303A
            && info.pid == 0x1001
        {
            println!(
                "Using {} at {}",
                info.product
                    .clone()
                    .unwrap_or_else(|| "[No Product Name]".to_string()),
                port.port_name
            );

            let mut port = serialport::new(port.port_name, 115_200)
                .timeout(Duration::MAX)
                .open()?;
            loop {
                let mut reader = BufReader::new(&mut port);
                let mut buffer = Vec::new();
                reader.read_until(0x00, &mut buffer)?;
                if let Ok(message) = postcard::from_bytes_cobs::<Message>(&mut buffer) {
                    println!("{message:?} ({buffer:?})");
                } else {
                    println!("Failed to decode message");
                }
            }
        }
    }

    Ok(())
}
