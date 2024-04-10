use bitflags::bitflags;
use core::{fmt, pin::pin};
use embedded_hal_async::spi;

pub struct HexSlice<T>(pub T)
where
    T: AsRef<[u8]>;

impl<T: AsRef<[u8]>> fmt::Debug for HexSlice<T> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("[")?;
        for (i, byte) in self.0.as_ref().iter().enumerate() {
            if i != 0 {
                f.write_str(", ")?;
            }
            write!(f, "{:02x}", byte)?;
        }
        f.write_str("]")
    }
}

pub struct Identification {
    bytes: [u8; 3],
    continuations: u8,
}

impl Identification {
    /// Build an Identification from JEDEC ID bytes.
    pub fn from_jedec_id(buf: &[u8]) -> Identification {
        // Example response for Cypress part FM25V02A:
        // 7F 7F 7F 7F 7F 7F C2 22 08  (9 bytes)
        // 0x7F is a "continuation code", not part of the core manufacturer ID
        // 0xC2 is the company identifier for Cypress (Ramtron)

        // Find the end of the continuation bytes (0x7F)
        let mut start_idx = 0;
        for i in 0..(buf.len() - 2) {
            if buf[i] != 0x7F {
                start_idx = i;
                break;
            }
        }

        Self {
            bytes: [buf[start_idx], buf[start_idx + 1], buf[start_idx + 2]],
            continuations: start_idx as u8,
        }
    }

    /// The JEDEC manufacturer code for this chip.
    pub fn mfr_code(&self) -> u8 {
        self.bytes[0]
    }

    /// The manufacturer-specific device ID for this chip.
    pub fn device_id(&self) -> &[u8] {
        self.bytes[1..].as_ref()
    }

    /// Number of continuation codes in this chip ID.
    ///
    /// For example the ARM Ltd identifier is `7F 7F 7F 7F 3B` (5 bytes), so
    /// the continuation count is 4.
    pub fn continuation_count(&self) -> u8 {
        self.continuations
    }
}

impl fmt::Debug for Identification {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("Identification")
            .field(&HexSlice(self.bytes))
            .finish()
    }
}

#[repr(u8)]
#[allow(unused)] // TODO support more features
enum Opcode {
    /// Read the 8-bit legacy device ID.
    ReadDeviceId = 0xAB,
    /// Read the 8-bit manufacturer and device IDs.
    ReadMfDId = 0x90,
    /// Read 16-bit manufacturer ID and 8-bit device ID.
    ReadJedecId = 0x9F,
    /// Set the write enable latch.
    WriteEnable = 0x06,
    /// Clear the write enable latch.
    WriteDisable = 0x04,
    /// Read the 8-bit status register.
    ReadStatus = 0x05,
    /// Write the 8-bit status register. Not all bits are writeable.
    WriteStatus = 0x01,
    Read = 0x03,
    PageProg = 0x02, // directly writes to EEPROMs too
    SectorErase = 0x20,
    BlockErase = 0xD8,
    ChipErase = 0xC7,
    PowerDown = 0xB9,
}

bitflags! {
    /// Status register bits.
    pub struct Status: u8 {
        /// Erase or write in progress.
        const BUSY = 1 << 0;
        /// Status of the **W**rite **E**nable **L**atch.
        const WEL = 1 << 1;
        /// The 3 protection region bits.
        const PROT = 0b00011100;
        /// **S**tatus **R**egister **W**rite **D**isable bit.
        const SRWD = 1 << 7;
    }
}

pub const PAGE_SIZE: u16 = 0x100;
pub const SECTOR_SIZE: u32 = 0x1000;
pub const BLOCK_SIZE: u32 = SECTOR_SIZE * 16;

pub struct FlashInfo {
    pub id: u32,
    pub page_count: u32,
    pub sector_count: u32,
    pub block_count: u32,
    pub capacity_kb: u32,
}

impl fmt::Debug for FlashInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("FlashInfo")
            .field(&self.id)
            .field(&format_args!("_KB:_"))
            .field(&self.capacity_kb)
            .finish()
    }
}

impl FlashInfo {
    pub const fn page_to_sector(page_address: &u32) -> u32 {
        return (page_address * (PAGE_SIZE) as u32) / SECTOR_SIZE;
    }

    pub const fn page_to_block(page_address: &u32) -> u32 {
        return (page_address * PAGE_SIZE as u32) / BLOCK_SIZE;
    }

    pub const fn sector_to_block(sector_address: &u32) -> u32 {
        return (sector_address * SECTOR_SIZE as u32) / BLOCK_SIZE;
    }

    pub const fn sector_to_page(sector_address: &u32) -> u32 {
        return (sector_address * SECTOR_SIZE as u32) / PAGE_SIZE as u32;
    }

    pub const fn block_to_page(block_adress: &u32) -> u32 {
        return (block_adress * BLOCK_SIZE as u32) / PAGE_SIZE as u32;
    }
}

pub struct W25Q80<T> {
    spi: T,
}

pub enum Error<E> {
    Spi(E),
    UnexpectedStatus,
}

impl<T> From<T> for Error<T> {
    fn from(value: T) -> Self {
        Self::Spi(value)
    }
}

impl<T: spi::SpiDevice> W25Q80<T> {
    pub async fn new(spi: T) -> Result<Self, Error<T::Error>> {
        let mut new = Self { spi };
        let status = new.read_status().await?;
        if !(status & (Status::BUSY | Status::WEL)).is_empty() {
            return Err(Error::UnexpectedStatus);
        }
        Ok(new)
    }

    async fn delay(&mut self, delay_ns: u32) -> Result<(), T::Error> {
        let mut buf = [spi::Operation::DelayNs(delay_ns)];
        self.spi.transaction(&mut buf).await?;
        Ok(())
    }

    async fn transfer(&mut self, bytes: &[u8]) -> Result<(), T::Error> {
        self.spi.write(bytes).await?;
        Ok(())
    }

    async fn transfer_in_place(&mut self, bytes: &mut [u8]) -> Result<(), T::Error> {
        self.spi.transfer_in_place(bytes).await?;
        Ok(())
    }

    pub async fn read_status(&mut self) -> Result<Status, T::Error> {
        let mut buf = [Opcode::ReadStatus as u8, 0];
        self.transfer_in_place(&mut buf).await?;
        Ok(Status::from_bits_truncate(buf[1]))
    }

    pub async fn read_jedec_id(&mut self) -> Result<Identification, T::Error> {
        // Optimistically read 12 bytes, even though some identifiers will be shorter
        let mut buf: [u8; 12] = [0; 12];
        buf[0] = Opcode::ReadJedecId as u8;
        self.transfer_in_place(&mut buf).await?;

        // Skip buf[0] (SPI read response byte)
        Ok(Identification::from_jedec_id(&buf[1..]))
    }

    pub async fn get_device_info(&mut self) -> Result<FlashInfo, T::Error> {
        let mut buf: [u8; 12] = [0; 12];
        buf[0] = Opcode::ReadJedecId as u8;
        self.transfer_in_place(&mut buf).await?;

        let full_id: u32 =
            (((buf[1] as u32) << 16) | ((buf[2] as u32) << 8) | (buf[3] as u32)) & 0x000000FF;

        let block_count = match full_id {
            0x20 => 1024, // W25Q512
            0x19 => 512,  // W25Q256
            0x18 => 256,  // W25Q128
            0x17 => 128,  // W25Q64
            0x16 => 64,   // W25Q32
            0x15 => 32,   // W25Q16
            0x14 => 16,   // W25Q80
            0x13 => 8,    // W25Q40
            0x12 => 4,    // W25Q20
            0x11 => 2,    // W25Q10
            33_u32..=u32::MAX => 0,
            0_u32..=16_u32 => 0,
            26_u32..=31_u32 => 0,
        };

        let device_info = FlashInfo {
            id: 0,
            page_size: 256,
            sector_size: 0x1000,
            sector_count: block_count * 16,
            page_count: (block_count * 16 * 0x1000) / 256,
            block_size: 0x1000 * 16,
            block_count,
            capacity_kb: (0x1000 * 16 * block_count) / 1024,
        };
        return Ok(device_info);
    }

    pub async fn write_enable(&mut self) -> Result<(), T::Error> {
        let mut cmd_buf = [Opcode::WriteEnable as u8];
        self.transfer_in_place(&mut cmd_buf).await?;
        Ok(())
    }

    pub async fn wait_done(&mut self, delay_ns: u32) -> Result<(), T::Error> {
        while self.read_status().await?.contains(Status::BUSY) {
            self.delay(delay_ns).await?;
        }
        Ok(())
    }

    pub async fn power_down(&mut self) -> Result<(), T::Error> {
        let buf = [Opcode::PowerDown as u8];
        self.transfer(&buf).await?;
        Ok(())
    }

    pub async fn release_power_down(&mut self) -> Result<(), T::Error> {
        let buf = [Opcode::ReadDeviceId as u8];
        self.transfer(&buf).await?;
        self.delay(3000).await?;

        Ok(())
    }

    pub async fn read(&mut self, addr: u32, buf: &mut [u8]) -> Result<(), T::Error> {
        let cmd_buf = [
            Opcode::Read as u8,
            (addr >> 16) as u8,
            (addr >> 8) as u8,
            addr as u8,
        ];

        let mut ops = [spi::Operation::Write(&cmd_buf), spi::Operation::Read(buf)];

        self.spi.transaction(&mut ops).await?;
        Ok(())
    }

    pub async fn erase_sectors(
        &mut self,
        addr: u32,
        amount: usize,
        delay_ns: u32,
    ) -> Result<(), T::Error> {
        for c in 0..amount {
            self.write_enable().await?;

            let current_addr: u32 = (addr as usize + c * 256).try_into().unwrap();
            let mut cmd_buf = [
                Opcode::SectorErase as u8,
                (current_addr >> 16) as u8,
                (current_addr >> 8) as u8,
                current_addr as u8,
            ];
            self.transfer(&mut cmd_buf).await?;
            self.wait_done(delay_ns).await?;
        }

        Ok(())
    }

    pub async fn erase_block(&mut self, addr: u32, delay_ns: u32) -> Result<(), T::Error> {
        self.write_enable().await?;

        let cmd_buf = [
            Opcode::BlockErase as u8,
            (addr >> 16) as u8,
            (addr >> 8) as u8,
            addr as u8,
        ];

        self.transfer(&cmd_buf).await?;
        self.wait_done(delay_ns).await?;

        Ok(())
    }

    pub async fn erase_all(&mut self, delay_ns: u32) -> Result<(), T::Error> {
        self.write_enable().await?;
        let cmd_buf = [Opcode::ChipErase as u8];
        self.transfer(&cmd_buf).await?;
        self.wait_done(delay_ns).await?;
        Ok(())
    }
}
