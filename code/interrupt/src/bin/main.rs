#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use core::cell::RefCell;
use critical_section::Mutex;
use defmt::info;
use esp_hal::{
    clock::CpuClock,
    delay::Delay,
    gpio::{Event, Input, InputConfig, Io, Level, Output, OutputConfig},
    handler, main,
};

#[panic_handler]
fn panic(_: &core::panic::PanicInfo) -> ! {
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Interrupt flag
static BUTTON: Mutex<RefCell<Option<Input>>> = Mutex::new(RefCell::new(None));

#[main]
fn main() -> ! {
    // generator version: 0.5.0

    rtt_target::rtt_init_defmt!();

    let config = esp_hal::Config::default().with_cpu_clock(CpuClock::max());
    let peripherals = esp_hal::init(config);

    let mut io = Io::new(peripherals.IO_MUX);

    // set the interrupt_handler here - i.e. callback
    io.set_interrupt_handler(my_interrupt);

    // Our boot button as Input button
    let mut button = Input::new(peripherals.GPIO9, InputConfig::default());

    critical_section::with(|cs| {
        button.listen(Event::FallingEdge);
        BUTTON.borrow_ref_mut(cs).replace(button)
    });

    let delay = Delay::new();
    loop {
        delay.delay_millis(500u32);
    }
}

#[handler]
fn my_interrupt() {
    critical_section::with(|cs| {
        info!("Button Pressed");
        BUTTON
            .borrow_ref_mut(cs)
            .as_mut()
            .unwrap()
            .clear_interrupt();
    });
}
