//! Mostly copied here: https://github.com/barafael/mpu6050-dmp-rs/blob/master/examples/src/basic_async.rs

#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_time::Timer;
use esp_hal::clock::CpuClock;
use esp_hal::i2c::master::{Config, I2c};
use esp_hal::time::Rate;
use esp_hal::timer::systimer::SystemTimer;

use embassy_time::Delay;
use mpu6050_dmp::sensor_async::Mpu6050;
use mpu6050_dmp::{address::Address, calibration::CalibrationParameters};

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    error!("{}", info);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("Embassy initialized!");

    // TODO: Spawn some tasks
    let _ = spawner;

    info!("Init i2c");
    let i2c_config = Config::default().with_frequency(Rate::from_khz(400));
    let i2c = I2c::new(peripherals.I2C0, i2c_config)
        .expect("Failed to initialize I2C")
        .with_sda(peripherals.GPIO5)
        .with_scl(peripherals.GPIO6)
        .into_async();

    info!("Init i2c finished");

    let mut sensor = Mpu6050::new(i2c, Address::default())
        .await
        .expect("Could not create MPU6050 Sensor");
    info!("Init Sensor finished");

    let mut delay = Delay;

    info!("Init DMP");
    sensor.initialize_dmp(&mut delay).await.unwrap();
    info!("DMP finished");

    let calibration_params = CalibrationParameters::new(
        mpu6050_dmp::accel::AccelFullScale::G2,
        mpu6050_dmp::gyro::GyroFullScale::Deg2000,
        mpu6050_dmp::calibration::ReferenceGravity::ZN,
    );

    info!("Calibrating Sensor");
    sensor
        .calibrate(&mut delay, &calibration_params)
        .await
        .unwrap();
    info!("Sensor Calibrated");

    loop {
        let (accel, gyro, temp) = (
            sensor.accel().await.unwrap(),
            sensor.gyro().await.unwrap(),
            sensor.temperature().await.unwrap().celsius(),
        );
        info!("Sensor Readings:");
        info!(
            "  Accelerometer [mg]: x={}, y={}, z={}",
            accel.x() as i32,
            accel.y() as i32,
            accel.z() as i32
        );
        info!(
            "  Gyroscope [deg/s]: x={}, y={}, z={}",
            gyro.x() as i32,
            gyro.y() as i32,
            gyro.z() as i32
        );
        info!("  Temperature: {}°C", temp);
        Timer::after_millis(1000).await;
    }
}
