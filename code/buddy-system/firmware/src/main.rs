#![no_std]
#![no_main]

use common::Message;
use defmt::info;
use embassy_executor::Spawner;
use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Input, InputConfig, Pull},
    timer::systimer::SystemTimer,
    usb_serial_jtag::{UsbSerialJtag, UsbSerialJtagTx},
    Blocking, Config,
};
use esp_hal_embassy::main;
use heapless::Vec;
use panic_rtt_target as _;

#[main]
async fn main(_spawner: Spawner) {
    rtt_target::rtt_init_defmt!();

    let peripherals = esp_hal::init(Config::default());
    let timer0 = SystemTimer::new(peripherals.SYSTIMER);
    esp_hal_embassy::init(timer0.alarm0);

    info!("buddy");

    let usb_serial = UsbSerialJtag::new(peripherals.USB_DEVICE);
    let (mut _usb_rx, mut usb_tx) = usb_serial.split();

    let config = InputConfig::default().with_pull(Pull::Up);
    let button = Input::new(peripherals.GPIO9, config);
    let mut button_state = false;

    loop {
        if !button_state && button.is_low() {
            // button pressed
            info!("button pressed");
            button_state = true;
            send_state(&mut usb_tx, button_state);
        } else if button_state && button.is_high() {
            // button released
            info!("button released");
            button_state = false;
            send_state(&mut usb_tx, button_state);
        }

        Timer::after(Duration::from_millis(10)).await;
    }
}

fn send_state(usb_tx: &mut UsbSerialJtagTx<Blocking>, state: bool) {
    let message: Vec<u8, 128> =
        postcard::to_vec_cobs(&Message::Button(state)).expect("Couldn't serialize button state");
    _ = usb_tx.write(&message);
    _ = usb_tx.flush_tx();
}
