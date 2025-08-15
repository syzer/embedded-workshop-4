#![no_std]
#![no_main]

use embassy_time::{Duration, Timer};
use esp_hal::{
    gpio::{Level, Output, OutputConfig},
    timer::timg::TimerGroup,
    uart::{Uart, Config, UartRx},
    Async,
};
use esp_println::println;
use heapless::Vec;
use esp_backtrace as _;

// Protocol constants
const START_BYTE: u8 = 0xAA;
const END_BYTE: u8 = 0x55;
const SENSOR_HUMIDITY: u8 = 0x02;

// Maximum frame size
const MAX_FRAME_SIZE: usize = 32;

// Received sensor data
#[derive(Clone, Copy, Debug)]
struct SensorReading {
    sensor_id: u8,
    value: f32,
    timestamp: u32,
}

// Frame parser state machine
#[derive(Debug, PartialEq)]
enum ParseState {
    WaitingForStart,
    ReadingSensorId,
    ReadingLength,
    ReadingData,
    ReadingChecksum,
    ReadingEnd,
}

struct FrameParser {
    state: ParseState,
    buffer: Vec<u8, MAX_FRAME_SIZE>,
    sensor_id: u8,
    data_length: u8,
    bytes_read: usize,
    expected_checksum: u8,
}

impl FrameParser {
    fn new() -> Self {
        Self {
            state: ParseState::WaitingForStart,
            buffer: Vec::new(),
            sensor_id: 0,
            data_length: 0,
            bytes_read: 0,
            expected_checksum: 0,
        }
    }
    
    fn reset(&mut self) {
        self.state = ParseState::WaitingForStart;
        self.buffer.clear();
        self.sensor_id = 0;
        self.data_length = 0;
        self.bytes_read = 0;
        self.expected_checksum = 0;
    }
    
    fn process_byte(&mut self, byte: u8) -> Option<SensorReading> {
        match self.state {
            ParseState::WaitingForStart => {
                if byte == START_BYTE {
                    self.buffer.clear();
                    self.state = ParseState::ReadingSensorId;
                }
            }
            
            ParseState::ReadingSensorId => {
                self.sensor_id = byte;
                self.expected_checksum = byte; // Start checksum calculation
                self.state = ParseState::ReadingLength;
            }
            
            ParseState::ReadingLength => {
                self.data_length = byte;
                self.expected_checksum ^= byte;
                self.bytes_read = 0;
                self.buffer.clear();
                
                if self.data_length > 0 && self.data_length <= 20 {
                    self.state = ParseState::ReadingData;
                } else {
                    println!("Invalid data length: {}", self.data_length);
                    self.reset();
                }
            }
            
            ParseState::ReadingData => {
                if self.buffer.push(byte).is_err() {
                    println!("Buffer overflow during data reading");
                    self.reset();
                    return None;
                }
                
                self.expected_checksum ^= byte;
                self.bytes_read += 1;
                
                if self.bytes_read >= self.data_length as usize {
                    self.state = ParseState::ReadingChecksum;
                }
            }
            
            ParseState::ReadingChecksum => {
                if byte == self.expected_checksum {
                    self.state = ParseState::ReadingEnd;
                } else {
                    println!("Checksum mismatch: got {:#02X}, expected {:#02X}", 
                            byte, self.expected_checksum);
                    self.reset();
                }
            }
            
            ParseState::ReadingEnd => {
                if byte == END_BYTE {
                    // Frame complete, parse the data
                    let reading = self.parse_sensor_data();
                    self.reset();
                    return reading;
                } else {
                    println!("Invalid end byte: {:#02X}", byte);
                    self.reset();
                }
            }
        }
        
        None
    }
    
    fn parse_sensor_data(&self) -> Option<SensorReading> {
        if self.buffer.len() < 9 {
            println!("Insufficient data in buffer: {} bytes", self.buffer.len());
            return None;
        }
        
        // Data format: [sensor_id][value(4 bytes)][timestamp(4 bytes)]
        let received_sensor_id = self.buffer[0];
        
        if received_sensor_id != self.sensor_id {
            println!("Sensor ID mismatch in data");
            return None;
        }
        
        // Extract value (f32, little endian)
        let mut value_bytes = [0u8; 4];
        value_bytes.copy_from_slice(&self.buffer[1..5]);
        let value = f32::from_le_bytes(value_bytes);
        
        // Extract timestamp (u32, little endian)
        let mut timestamp_bytes = [0u8; 4];
        timestamp_bytes.copy_from_slice(&self.buffer[5..9]);
        let timestamp = u32::from_le_bytes(timestamp_bytes);
        
        Some(SensorReading {
            sensor_id: received_sensor_id,
            value,
            timestamp,
        })
    }
}

// Statistics tracking
struct Statistics {
    humidity_count: u32,
    total_frames: u32,
    last_humidity: f32,
}

impl Statistics {
    fn new() -> Self {
        Self {
            humidity_count: 0,
            total_frames: 0,
            last_humidity: 0.0,
        }
    }
    
    fn update(&mut self, reading: &SensorReading) {
        self.total_frames += 1;
        
        match reading.sensor_id {
            SENSOR_HUMIDITY => {
                self.humidity_count += 1;
                self.last_humidity = reading.value;
            }
            _ => {}
        }
    }
    
    fn print_summary(&self) {
        println!("\n=== SENSOR STATISTICS ===");
        println!("Total frames received: {}", self.total_frames);
        println!("Humidity readings: {} (last: {:.1}%)", self.humidity_count, self.last_humidity);
        println!("========================\n");
    }
}

#[esp_hal_embassy::main]
async fn main(_spawner: embassy_executor::Spawner) {
    println!("Welcome to board2 main");
    let peripherals = esp_hal::init(esp_hal::Config::default());
    
    println!("ESP32-C3 Display Board Starting...");
    
    let timg0 = TimerGroup::new(peripherals.TIMG0);
    esp_hal_embassy::init(timg0.timer0);
    
    // Configure LED on GPIO2
    let led = Output::new(peripherals.GPIO2, Level::Low, OutputConfig::default());
    
    // Configure UART for communication
    // RX: GPIO5, TX: GPIO4 (reverse of Board 1)
    let config = Config::default().with_baudrate(115200);
    
    let uart = Uart::new(peripherals.UART1, config).unwrap()
        .with_rx(peripherals.GPIO5)
        .with_tx(peripherals.GPIO4)
        .into_async();
    
    println!("UART configured on GPIO5(RX)/GPIO4(TX) at 115200 baud");
    println!("Waiting for sensor data...\n");
    
    // Split for reception
    let (rx, _tx) = uart.split();
    
    // Run the async task
    uart_receive_task(rx, led).await;
}

async fn uart_receive_task(
    mut uart_rx: UartRx<'static, Async>,
    mut led: Output<'static>
) {
    let mut parser = FrameParser::new();
    let mut stats = Statistics::new();
    
    println!("UART receiver task started");
    
    let mut buffer = [0u8; 1];
    
    loop {
        // Read byte from UART
        match uart_rx.read(&mut buffer) {
            Ok(_) => {
                let byte = buffer[0];
                if let Some(reading) = parser.process_byte(byte) {
                    // Valid frame received - blink LED
                    led.set_high();
                    
                    stats.update(&reading);
                    display_sensor_reading(&reading);
                    
                    // Print statistics every 10 frames
                    if stats.total_frames % 10 == 0 {
                        stats.print_summary();
                    }
                    
                    // Keep LED on for a brief moment
                    Timer::after(Duration::from_millis(50)).await;
                    led.set_low();
                }
            }
            Err(e) => {
                println!("UART read error: {:?}", e);
                Timer::after(Duration::from_millis(100)).await;
            }
        }
    }
}


fn display_sensor_reading(reading: &SensorReading) {
    let sensor_name = match reading.sensor_id {
        SENSOR_HUMIDITY => "Humidity",
        _ => "Unknown",
    };
    
    let unit = match reading.sensor_id {
        SENSOR_HUMIDITY => "%",
        _ => "",
    };
    
    let icon = match reading.sensor_id {
        SENSOR_HUMIDITY => "💧",
        _ => "❓",
    };
    
    println!(
        "{} {} {}: {:.2} {} [timestamp: {}]",
        icon,
        sensor_name,
        reading.sensor_id,
        reading.value,
        unit,
        reading.timestamp
    );
}