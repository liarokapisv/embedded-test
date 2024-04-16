#![no_std]
#![no_main]
#![feature(const_fn_floating_point_arithmetic)]
#![feature(ptr_metadata)]
#![feature(type_alias_impl_trait)]
#![feature(effects)]
#![feature(never_type)]
#![feature(future_join)]
#![allow(dead_code)]

mod adc;
mod board;
mod bounded;
mod display;
mod ext_memory;
mod parameter_controllers;
mod params;
mod user_inputs;

use embassy_stm32::init;
use embassy_stm32::spi;
use embassy_time::Delay;
use embassy_time::Duration;
use embassy_time::Timer;
use w25qxx;

use embassy_executor::Spawner;
use embassy_stm32::gpio::{Level, Output, Speed};
use embedded_hal_bus::spi::ExclusiveDevice;

use crate::board::ExtMemory as _;
use {defmt_rtt as _, panic_probe as _};

#[embassy_executor::main]
async fn main(mut _spawner: Spawner) {
    let p = init(Default::default());

    let spi_bus = spi::Spi::new(
        p.SPI1,
        p.PA5,
        p.PA7,
        p.PA6,
        p.DMA1_CH4,
        p.DMA2_CH3,
        spi::Config::default(),
    );

    let memory_device =
        ExclusiveDevice::new(spi_bus, Output::new(p.PA8, Level::Low, Speed::High), Delay);

    let mut memory = ext_memory::Driver::new(memory_device)
        .await
        .expect("driver creation failed");

    let mut buffer = [0 as u8; 4096];
    let mut counter: u8 = 0;

    loop {
        memory.write(counter, &buffer).await.expect("write failed");
        memory
            .read(counter, &mut buffer)
            .await
            .expect("read failed");
        counter += 1;

        Timer::after(Duration::from_millis(20)).await;
    }
}
