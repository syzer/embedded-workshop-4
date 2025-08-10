#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::timer::systimer::SystemTimer;

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

#[embassy_executor::task]
async fn my_led_blink_task() {
    loop {
        info!("On");
        Timer::after_millis(500).await;
        info!("Off");
        Timer::after_millis(500).await;
    }
}

#[embassy_executor::task]
async fn my_other_things() {
    loop {
        info!("Other GPIO HIGH");
        Timer::after_millis(1000).await;
        info!("Other GPIO LOW");
        Timer::after_millis(1000).await;
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: Spawner) {
    // generator version: 0.5.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);
    info!("Embassy initialized!");

    spawner
        .spawn(my_led_blink_task())
        .expect("Could not spawn LED task");
    spawner
        .spawn(my_other_things())
        .expect("Could not spawn my other task")
}
