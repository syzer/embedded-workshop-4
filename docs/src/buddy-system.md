# Let's write an embassy project: Buddy System

## What is the buddy system?
Basically: Have an MCU and an embedded Linux system work together.
They each do what they can do best and communicate to keep in sync.

As an example, think of a Raspberry Pi ("big buddy") connected to an ESP32 MCU ("tiny buddy") via some serial bus (USB, UART, etc.).
The MCU does what it does best: Real-time control jobs with deterministic timing, low-power operation, etc.
The Raspberry Pi also does what it does best: Run expensive algorithms, render a UI on a display, manage secure networking, etc.
They talk to each other via the serial connection.

## Why choose the buddy system?
You may ask yourself one or more of these questions:
- Isn't that more expensive to produce?
- Doesn't that lead to a more complex codebase?
- This seems dumb

Yes, the hardware costs are higher, but for low production runs, the R&D costs usually far outweigh the cost of a Raspberry Pi chip or similar.

Yes, this requires two builds for the firmware and software, but we have embedded Rust now and can write it all in one codebase.
In this project you'll see the MCU and the host literally use the same common enum to de-/serialize messages.

Also, OTA (over-the-air) updates on an MCU are scary!
Screw one up and you'll be soldering an USB connector to your hardware to re-flash the firmware.
Compare that to just doing a deployment on a Linux system that also flashes the firmware on the MCU via it's USB connection and can restart if it fails.

## Postcard
[Postcard](https://docs.rs/crate/postcard) is a protocol designed for communication in constrained environments.
It integrates flawlessly with Serde and is resource efficient enough to run on our tiny buddy.

Postcard comes with a lot of features and even with an [RPC](https://crates.io/crates/postcard-rpc) library.
Today we'll mostly use it's de-/serializing and COBS ([Consistent Overhead Byte Stuffing](https://en.wikipedia.org/wiki/Consistent_Overhead_Byte_Stuffing) helps frame the packets sent on the serial bus.) implementation:

```rust
#[derive(Debug, Serialize, Deserialize, Format, Clone, Copy)]
pub enum Message {
    Button(bool),
}

// ...

let message: Vec<u8, 128> = postcard::to_vec_cobs(&Message::Button(button_state))?;

// ...

let message: Message = postcard::from_bytes_cobs::<Message>(&mut buffer)?;
```

## USB CDC on the ESP-C3
The ESP32-C3 has an USB controller that supports USB CDC serial communication (and acts as a JTAG probe):
```rust
use esp_hal::usb_serial_jtag::UsbSerialJtag;

let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
let (mut usb_rx, mut usb_tx) = usb_serial.split();

_ = tx.write(&buffer);
_ = tx.flush_tx();
```

## serialport on the big buddy
On the host side we'll use the [serialport](https://crates.io/crates/serialport) library:
```rust
let ports = serialport::available_ports()?;

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
        let mut reader = BufReader::new(&mut port);
        let mut buffer = Vec::new();
        reader.read_until(0x00, &mut buffer)?;
    }
}
```
