#![no_std]
#![no_main]

use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
    uart::{Uart, Config, UartTx},
    Async,
};
use esp_println::println;
use heapless::Vec;
use rand::{RngCore, SeedableRng};
use rand::rngs::SmallRng;
use esp_backtrace as _;
use embedded_io_async::Write;

// Protocol constants
const START_BYTE: u8 = 0xAA;
const END_BYTE: u8 = 0x55;
const SENSOR_HUMIDITY: u8 = 0x02;

// Sensor data structure
#[derive(Clone, Copy)]
struct SensorReading {
    sensor_id: u8,
    value: f32,
    timestamp: u32,
}

// Protocol frame structure
struct DataFrame {
    data: Vec<u8, 32>,
}

impl DataFrame {
    fn new() -> Self {
        Self {
            data: Vec::new(),
        }
    }

    fn build_frame(&mut self, reading: &SensorReading) -> Result<(), ()> {
        self.data.clear();
        
        // Convert f32 to bytes
        let value_bytes = reading.value.to_le_bytes();
        let timestamp_bytes = reading.timestamp.to_le_bytes();
        
        // Calculate data length (1 + 4 + 4 = 9 bytes)
        let data_length = 9u8;
        
        // Build frame
        self.data.push(START_BYTE).map_err(|_| ())?;
        self.data.push(reading.sensor_id).map_err(|_| ())?;
        self.data.push(data_length).map_err(|_| ())?;
        
        // Add sensor ID again in data section
        self.data.push(reading.sensor_id).map_err(|_| ())?;
        
        // Add value bytes
        for byte in value_bytes.iter() {
            self.data.push(*byte).map_err(|_| ())?;
        }
        
        // Add timestamp bytes
        for byte in timestamp_bytes.iter() {
            self.data.push(*byte).map_err(|_| ())?;
        }
        
        // Calculate XOR checksum
        let mut checksum = 0u8;
        for i in 1..self.data.len() {
            checksum ^= self.data[i];
        }
        
        self.data.push(checksum).map_err(|_| ())?;
        self.data.push(END_BYTE).map_err(|_| ())?;
        
        Ok(())
    }
    
    fn get_bytes(&self) -> &[u8] {
        &self.data
    }
}

// Mock sensor that generates realistic readings
struct MockSensor {
    rng: SmallRng,
    base_humidity: f32,
    timestamp: u32,
}

impl MockSensor {
    fn new() -> Self {
        Self {
            rng: SmallRng::seed_from_u64(12345), // Fixed seed for reproducible testing
            base_humidity: 45.0,  // Base humidity in %
            timestamp: 0,
        }
    }
    
    fn read_humidity(&mut self) -> SensorReading {
        self.timestamp += 1;
        // Generate humidity between 30-70% with some noise
        let noise = (self.rng.next_u32() % 1000) as f32 / 1000.0 - 0.5;
        let value = self.base_humidity + noise * 20.0; // ±10% variation
        let value = value.max(0.0).min(100.0); // Clamp to valid range
        
        SensorReading {
            sensor_id: SENSOR_HUMIDITY,
            value,
            timestamp: self.timestamp,
        }
    }
}

#[esp_hal_embassy::main]
async fn main(spawner: embassy_executor::Spawner) {
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    println!("ESP32-C3 Sensor Board Starting...");
    
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    // Configure indicator LED on GPIO2
    let led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());

    // Configure UART for communication with custom pins
    // TX: GPIO4, RX: GPIO5
    let config = Config::default().with_baudrate(115200);
    
    let uart = Uart::new(peripherals.UART1, config).unwrap()
        .with_tx(peripherals.GPIO4)
        .with_rx(peripherals.GPIO5)
        .into_async();
    
    println!("UART configured on GPIO4(TX)/GPIO5(RX) at 115200 baud");
    
    let (_rx, tx) = uart.split();
    
    spawner.must_spawn(sensor_main_task(tx, led));
}

#[embassy_executor::task]
async fn sensor_main_task(
    uart_tx: UartTx<'static, Async>,
    led: Output<'static>
) {
    sensor_task(uart_tx, led).await;
}

async fn sensor_task(
    mut uart_tx: UartTx<'static, Async>,
    mut led: Output<'static>
) {
    let mut sensor = MockSensor::new();
    let mut frame = DataFrame::new();
    
    println!("Starting sensor reading task...");
    
    loop {
        let humidity_reading = sensor.read_humidity();
        if frame.build_frame(&humidity_reading).is_ok() {
            send_frame(&mut uart_tx, &frame, &mut led).await;
            println!("Sent fake humidity: {:.1}%", humidity_reading.value);
        }
        
        Timer::after(Duration::from_millis(1000)).await;
    }
}

async fn send_frame(
    uart_tx: &mut UartTx<'static, Async>,
    frame: &DataFrame,
    indicator_led: &mut Output<'static>
) {
    indicator_led.set_high();
    
    let bytes = frame.get_bytes();
    uart_tx.write_all(bytes).await.unwrap();

    Timer::after(Duration::from_millis(20)).await;

    indicator_led.set_low();
}