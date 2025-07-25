#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Level, Output, OutputConfig};
use esp_hal::main;
use esp_hal::time::{Duration, Instant};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

esp_bootloader_esp_idf::esp_app_desc!();

fn sleep(millis: u64) {
    let delay_start = Instant::now();
    while delay_start.elapsed() < Duration::from_millis(millis) {}
}

#[main]
fn main() -> ! {
    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    // GPIO 10 is labeled D10 on the Seed Xiao board
    let mut led = Output::new(peripherals.GPIO10, Level::Low, OutputConfig::default());

    loop {
        led.set_high();
        sleep(500);

        led.set_low();
        sleep(500);
    }
}
