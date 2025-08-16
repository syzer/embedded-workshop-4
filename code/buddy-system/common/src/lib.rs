#![no_std]

use defmt::Format;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize, Format, Clone, Copy)]
pub enum Message {
    Button(bool),
}
