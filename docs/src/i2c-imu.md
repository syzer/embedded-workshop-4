# Let's write an embassy project: I2C IMU

Setup

```sh
esp-generate --chip esp32c3 --headless -o probe-rs -o defmt -o embassy -o unstable-hal i2c_imu
```

<details>
<summary>If you use an ESP32s3:</summary>
<br>

[Source](https://github.com/esp-rs/espup?tab=readme-ov-file#quickstart)

```sh
esp-generate --chip esp32s3 --headless -o probe-rs -o defmt -o embassy -o unstable-hal i2c_imu
```

Install Xtensa toolchain

```sh
espup install --targets=esp32s3
```

Source the toolchain into your environment

```sh
source ~/export-esp.sh
```
</details>

Build and flashing

```sh
cargo run --release
```

## TODO: Explain i2c - and its wiring

## TODO: Find correct driver for given IMU

```sh
cargo add mpu6050
```

## TODO: Let participants play/implement on their own

- Netherless give hints: E.g. how to init an I2C device
