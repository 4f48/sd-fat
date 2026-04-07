// SPDX-License-Identifier: GPL-3.0-or-later
// Copyright (C) 2026 Olivér Pirger

use embedded_hal::digital::OutputPin;
use embedded_hal_async::spi::SpiBus;

use crate::{Error, Result, block_device::BlockDevice};

/// SPI driver for SD card storage devices. Does not support SD Ultra Capacity (SDUC) cards.
pub struct SdCard<SPI: SpiBus, CS: OutputPin> {
    spi: SPI,
    cs: CS,
    card_type: SdCardType,
}

/// Differentiate between byte-addressed (SDSC) and block-addressed (**SDHC**/SDXC) SD cards.
enum SdCardType {
    /// SD Standard Capacity (SDSC) - max size 2 GB
    SDSC,

    /// Support for higher capacity SD cards (SDHC/SDXC). Does not support SDUC cards.
    SDHC,
}

impl<SPI: SpiBus, CS: OutputPin> SdCard<SPI, CS> {
    /// Initialize the SD card for use with SPI.
    pub async fn new(mut spi: SPI, mut cs: CS) -> Result<Self> {
        deselect(&mut spi, &mut cs).await?;
        let mut delay = [0xFF; 10];
        spi.transfer_in_place(&mut delay)
            .await
            .map_err(|_| Error::TransferError)?;

        let cmd0 = make_cmd(0, 0);
        cs.set_low().map_err(|_| Error::CsError)?;
        spi.write(&cmd0).await.map_err(|_| Error::WriteError)?;
        let r1 = read_r1(&mut spi).await?;
        deselect(&mut spi, &mut cs).await?;
        if r1 != 0x01 {
            return Err(Error::NotFound);
        }

        let cmd8 = make_cmd(8, 0x000001AA);
        cs.set_low().map_err(|_| Error::CsError)?;
        spi.write(&cmd8).await.map_err(|_| Error::WriteError)?;
        let r1 = read_r1(&mut spi).await?;
        let mut r7 = [0xFF; 4];
        spi.transfer_in_place(&mut r7)
            .await
            .map_err(|_| Error::TransferError)?;
        deselect(&mut spi, &mut cs).await?;
        let is_v2 = r1 == 0x01 && r7[3] == 0xAA;

        let acmd41_arg = if is_v2 { 0x40000000 } else { 0x00000000 };
        loop {
            cs.set_low().map_err(|_| Error::CsError)?;
            let cmd55 = make_cmd(55, 0);
            spi.write(&cmd55).await.map_err(|_| Error::WriteError)?;
            read_r1(&mut spi).await?;
            deselect(&mut spi, &mut cs).await?;

            let cmd41 = make_cmd(41, acmd41_arg);
            cs.set_low().map_err(|_| Error::CsError)?;
            spi.write(&cmd41).await.map_err(|_| Error::WriteError)?;
            if read_r1(&mut spi).await? == 0x00 {
                deselect(&mut spi, &mut cs).await?;
                break;
            }
            deselect(&mut spi, &mut cs).await?;
        }

        let cmd58 = make_cmd(58, 0);
        cs.set_low().map_err(|_| Error::CsError)?;
        spi.write(&cmd58).await.map_err(|_| Error::WriteError)?;
        read_r1(&mut spi).await?;
        let mut ocr = [0xFF; 4];
        spi.transfer_in_place(&mut ocr)
            .await
            .map_err(|_| Error::TransferError)?;
        deselect(&mut spi, &mut cs).await?;

        let card_type = if is_v2 && (ocr[0] & 0x40 != 0) {
            SdCardType::SDHC
        } else {
            SdCardType::SDSC
        };

        if matches!(card_type, SdCardType::SDSC) {
            cs.set_low().map_err(|_| Error::CsError)?;
            let cmd16 = make_cmd(16, 512);
            spi.write(&cmd16).await.map_err(|_| Error::WriteError)?;
            read_r1(&mut spi).await?;
            deselect(&mut spi, &mut cs).await?;
        }

        Ok(Self { spi, cs, card_type })
    }
}

impl<SPI: SpiBus, CS: OutputPin> BlockDevice for SdCard<SPI, CS> {
    async fn read(&mut self, i: u32, buf: &mut [u8; 512]) -> crate::Result<()> {
        let address = match self.card_type {
            SdCardType::SDHC => i,
            SdCardType::SDSC => i * 512,
        };

        self.cs.set_low().map_err(|_| Error::CsError)?;
        let cmd17 = make_cmd(17, address);
        self.spi
            .write(&cmd17)
            .await
            .map_err(|_| Error::WriteError)?;
        let r1 = read_r1(&mut self.spi).await?;
        if r1 != 0x00 {
            deselect(&mut self.spi, &mut self.cs).await?;
            return Err(Error::TransferError);
        }

        loop {
            let mut b = [0xFF];
            self.spi
                .transfer_in_place(&mut b)
                .await
                .map_err(|_| Error::TransferError)?;
            if b[0] == 0xFE {
                break;
            }
        }

        buf.fill(0xFF);
        self.spi
            .transfer_in_place(buf)
            .await
            .map_err(|_| Error::TransferError)?;

        let mut crc = [0xFF; 2];
        self.spi
            .transfer_in_place(&mut crc)
            .await
            .map_err(|_| Error::TransferError)?;

        deselect(&mut self.spi, &mut self.cs).await?;

        Ok(())
    }

    async fn write(&mut self, _i: u32, _buf: &[u8; 512]) -> crate::Result<()> {
        todo!();
    }
}

/// Utility for creating SPI commands.
fn make_cmd(index: u8, arg: u32) -> [u8; 6] {
    let crc = match index {
        0 => 0x95,
        8 => 0x87,
        _ => 0xFF,
    };
    [
        0x40 | index,
        (arg >> 24) as u8,
        (arg >> 16) as u8,
        (arg >> 8) as u8,
        arg as u8,
        crc,
    ]
}

/// Utility to check SD card's R1 response 8 times before timing out. Returns 0xFF on timeout.
async fn read_r1<SPI: SpiBus>(spi: &mut SPI) -> Result<u8> {
    for _ in 0..8 {
        let mut b = [0xFF];
        spi.transfer_in_place(&mut b)
            .await
            .map_err(|_| Error::TransferError)?;
        if b[0] & 0x80 == 0 {
            return Ok(b[0]);
        }
    }
    Ok(0xFF)
}

/// Wait some time after each transaction and give the SD card some time to ready itself for the next.
async fn deselect<SPI: SpiBus, CS: OutputPin>(spi: &mut SPI, cs: &mut CS) -> Result<()> {
    cs.set_high().map_err(|_| Error::CsError)?;
    spi.write(&[0xFF]).await.map_err(|_| Error::WriteError)?;
    Ok(())
}
