#![allow(dead_code)] // enum variants kept for completness and readability
use core::{
    hint::spin_loop,
    sync::atomic::{AtomicU64, Ordering},
};

use crate::io::{io_wait, outb};

const CHANNEL_0_DATA: u16 = 0x40;
const COMMAND_REGISTER: u16 = 0x43;

const MAX_DIVISOR: u16 = 65535;
const DIVISOR: u16 = (1193182 / FREQUENCY as u64) as u16;

const BASE_CLOCK: u64 = 1193182;

pub(crate) const FREQUENCY: u16 = 100;

static TICK_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Initializes the programmable interval timer
///
/// # Safety Requires IO privileges.
pub(crate) unsafe fn initialize() {
    let divisor = DIVISOR;
    let config = ConfigurationByte::new(
        false,
        OperatingMode::RateGenerator,
        AccessMode::LoByteHiByte,
        ChannelSelector::Interrupts,
    );

    // set mode 2 (rate generator)
    outb(COMMAND_REGISTER, config.0);
    io_wait();
    // send lower half of divisor
    outb(CHANNEL_0_DATA, (divisor & 0x00ff) as u8);
    io_wait();
    // send higher half of divisor
    outb(CHANNEL_0_DATA, ((divisor & 0xff00) >> 8) as u8);
    io_wait();
}

#[inline]
pub(crate) fn tick() {
    TICK_COUNTER.fetch_add(1, Ordering::Relaxed);
}
pub(crate) fn sleep(millis: u64) {
    let frequency = BASE_CLOCK / DIVISOR as u64;
    let ticks_to_sleep = (millis * frequency) / 1000;
    let start_ticks = TICK_COUNTER.load(Ordering::Relaxed);
    let target_ticks = start_ticks + ticks_to_sleep;

    while TICK_COUNTER.load(Ordering::Relaxed) < target_ticks {
        spin_loop();
    }
}

#[repr(transparent)]
#[derive(Copy, Clone, Debug)]
struct ConfigurationByte(u8);

impl ConfigurationByte {
    const fn new(
        bcd: bool,
        operating_mode: OperatingMode,
        access_mode: AccessMode,
        channel: ChannelSelector,
    ) -> ConfigurationByte {
        ConfigurationByte(
            (bcd as u8)
                | ((operating_mode as u8) << 1)
                | ((access_mode as u8) << 4)
                | ((channel as u8) << 6),
        )
    }
}

/// Byte-encoding to use for the PIT.
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum EncodingMode {
    /// four-digit biinary-coded decimal
    Bcd,
    /// binary mode
    Binary,
}

/// PIT mode to use for the specific channel
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum OperatingMode {
    /// 0 0 0 = Mode 0 (interrupt on terminal count)
    TerminalCount,
    /// 0 0 1 = Mode 1 (hardware re-triggerable one-shot)
    HadwareReTriggerableOneShot,
    /// 0 1 0 = Mode 2 (rate generator)
    RateGenerator,
    /// 0 1 1 = Mode 3 (square wave generator)
    SquareWaveGenerator,
    /// 1 0 0 = Mode 4 (software triggered strobe)
    SoftwareTriggeredStrobe,
    /// 1 0 1 = Mode 5 (hardware triggered strobe)
    HardwareTriggeredStrobe,
    /// 1 1 0 = Mode 2 (rate generator, same as 010b)
    /// same as `OperatingMode::RateGenerator`
    RateGenerator2,
    /// 1 1 1 = Mode 3 (square wave generator, same as 011b)
    /// same as `OperatingMode::SquareWaveGenerator`
    SquareWaveGenerator2,
}

/// Access mode for the channel
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum AccessMode {
    /// 0 0 = Latch count value command
    LatchCountValueCommand,
    /// 0 1 = Access mode: lobyte only
    LoByteOnly,
    /// 1 0 = Access mode: hibyte only
    HiByteOnly,
    /// 1 1 = Access mode: lobyte/hibyte
    LoByteHiByte,
}

/// Channel to use
#[repr(u8)]
#[derive(Copy, Clone, Debug)]
enum ChannelSelector {
    /// 0 0 = Channel 0 (generates interrupts)
    Interrupts,
    /// 0 1 = Channel 1 (used to refresh the (D)RAM)
    RefreshRam,
    /// 1 0 = Channel 2 (connected to the PC speaker)
    PcSpeaker,
    /// 1 1 = Read-back command (8254 only)
    ReadBack,
}
