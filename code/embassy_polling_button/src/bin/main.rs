#![no_std]
#![no_main]
#![deny(
    clippy::mem_forget,
    reason = "mem::forget is generally not safe to do with esp_hal types, especially those \
    holding buffers for the duration of a data transfer."
)]

use defmt::{error, info};
use embassy_executor::Spawner;
use embassy_sync::channel::Channel;
use embassy_time::{Duration, Timer};
use esp_hal::clock::CpuClock;
use esp_hal::gpio::{Input, InputConfig, Pull};
use esp_hal::timer::systimer::SystemTimer;

#[panic_handler]
fn panic(err: &core::panic::PanicInfo) -> ! {
    error!("Error happenend: {}", err);
    loop {}
}

// This creates a default app-descriptor required by the esp-idf bootloader.
// For more information see: <https://docs.espressif.com/projects/esp-idf/en/stable/esp32/api-reference/system/app_image_format.html#application-description>
esp_bootloader_esp_idf::esp_app_desc!();

// Channel for communication between button task and handler task
static BUTTON_CHANNEL: Channel<
    embassy_sync::blocking_mutex::raw::CriticalSectionRawMutex,
    ButtonEvent,
    10,
> = Channel::new();

#[derive(Clone, Copy, defmt::Format)]
enum ButtonEvent {
    Pressed,
    Released,
}

// Button polling task
#[embassy_executor::task]
async fn button_task(button: Input<'static>) {
    let sender = BUTTON_CHANNEL.sender();
    let mut last_state = button.is_high();

    info!("Button task started, monitoring GPIO9");

    loop {
        let current_state = button.is_high();

        // We only send an event, if the button state changed
        if last_state != current_state {
            if current_state {
                info!("Button released");
                sender.send(ButtonEvent::Released).await;
            } else {
                info!("Button pressed");
                sender.send(ButtonEvent::Pressed).await;
            }
            last_state = current_state;
        }

        // Small delay to debounce and avoid excessive polling
        Timer::after(Duration::from_millis(50)).await;
    }
}

// Button event handler task
#[embassy_executor::task]
async fn button_handler_task() {
    let receiver = BUTTON_CHANNEL.receiver();

    info!("Button handler task started, waiting for button events");

    loop {
        info!("I am idle and waiting for an event");
        let event = receiver.receive().await;

        match event {
            ButtonEvent::Pressed => {
                info!("🔴 Button handler: Received PRESSED event! Doing some work...");
                // Simulate some work
                Timer::after(Duration::from_millis(100)).await;
                info!("✅ Button handler: Work completed!");
            }
            ButtonEvent::Released => {
                info!("🟢 Button handler: Received RELEASED event!");
            }
        }
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

    // Configure GPIO9 as input with pull-up resistor
    let config = InputConfig::default().with_pull(Pull::Up);
    let button = Input::new(peripherals.GPIO9, config);

    // Spawn the button polling task
    spawner.spawn(button_task(button)).unwrap();

    // Spawn the button handler task
    spawner.spawn(button_handler_task()).unwrap();
    info!("Both tasks spawned successfully!");
}
