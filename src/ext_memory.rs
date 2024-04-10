use crate::board::ExtMemory;
use crate::w25qxx::{PAGE_SIZE, SECTOR_SIZE, W25Q80};
use embedded_hal_async::spi::SpiDevice;

pub struct Driver<T> {
    device: W25Q80<T>,
}

type NewError<T: SpiDevice> = crate::w25qxx::Error<T::Error>;

impl<T: SpiDevice> Driver<T> {
    pub async fn new(device: T) -> Result<Self, NewError<T>> {
        Ok(Self {
            device: W25Q80::new(device).await?,
        })
    }
}

impl<T: SpiDevice> ExtMemory for Driver<T> {
    type Error = T::Error;

    async fn write(&mut self, sector_id: u8, data: &[u8; SECTOR_SIZE]) -> Result<(), T::Error> {
        let sector_address = sector_id as u32 * SECTOR_SIZE as u32;
        let delay = 100;
        self.device.write_enable().await?;
        self.device.erase_sector(sector_address, delay).await?;
        for page in 0..(SECTOR_SIZE / PAGE_SIZE) {
            self.device.write_enable().await?;
            self.device
                .write(
                    sector_address + page as u32 * PAGE_SIZE as u32,
                    &data[page * PAGE_SIZE..(page + 1) * PAGE_SIZE],
                    delay,
                )
                .await?;
        }
        Ok(())
    }

    async fn read(&mut self, sector_id: u8, data: &mut [u8; SECTOR_SIZE]) -> Result<(), T::Error> {
        let sector_address = sector_id as u32 * SECTOR_SIZE as u32;
        self.device.read(sector_address as u32, data).await?;
        Ok(())
    }
}
