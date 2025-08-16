#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::info;
use embassy_executor::Spawner;
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::timer::systimer::SystemTimer;

#[panic_handler]
fn panic(info: &core::panic::PanicInfo) -> ! {
    defmt::error!("panic: {}", defmt::Debug2Format(info));
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

    // Configure GPIO9 as input with pull-up resistor
    let config = InputConfig::default().with_pull(Pull::Up);
    let button = Input::new(peripherals.GPIO9, config);

    spawner
        .spawn(my_interrupt_awaiting_task(button))
        .expect("Could not spawn this task");
}

#[embassy_executor::task]
async fn my_interrupt_awaiting_task(mut input_button: Input<'static>) {
    loop {
        info!("Waiting for a button press");
        // I.E.: When we press the button, the edge will fall
        input_button.wait_for_falling_edge().await;
        info!("I got woken up!")
    }
}
