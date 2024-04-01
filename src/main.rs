#![no_std]
#![no_main]

mod params;
mod flow;

use embassy_executor::Spawner;
use embassy_stm32::init;
use embassy_time::{Duration, Timer};
use {defmt_rtt as _, panic_probe as _};
use bounded_integer::BoundedI8;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    let _p = init(Default::default());
    loop {
        defmt::info!("ping");
        Timer::after(Duration::from_millis(100)).await;
    }
}
