mod commands;
mod gesture;
mod pointer;
mod swipe;

use std::fs::{File, OpenOptions};
use std::os::unix::prelude::AsRawFd;
use std::os::unix::{
    fs::OpenOptionsExt,
    io::{FromRawFd, IntoRawFd, RawFd},
};
use std::path::Path;

use input::event::Event::Gesture;
use input::event::Event::Pointer;
use input::{Libinput, LibinputInterface};
use libc::{O_RDWR, O_WRONLY};
use nix::poll::{poll, PollFd, PollFlags};

use log::LevelFilter;
use syslog::{BasicLogger, Facility, Formatter3164};
use anyhow::{Result, Context, bail};

use gesture::*;
use pointer::*;

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read(flags & O_RDWR != 0)
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into_raw_fd())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: RawFd) {
        unsafe {
            File::from_raw_fd(fd);
        }
    }
}

fn syslog_config() -> Result<()> {
    let formatter = Formatter3164 {
        facility: Facility::LOG_USER,
        hostname: None,
        process: "swayped".into(),
        pid: 0,
    };

    let Ok(logger) = syslog::unix(formatter) else {
            bail!("Failed to connect to syslog");
    };

    log::set_boxed_logger(Box::new(BasicLogger::new(logger)))
        .map(|()| log::set_max_level(LevelFilter::Debug))?;

    Ok(())
}

fn main() -> Result<()> {
    let mut input = Libinput::new_with_udev(Interface);
    let Ok(_) = input.udev_assign_seat("seat0") else {
        return Ok(());
    };

    syslog_config().context("Failed to initialize syslog")?;

    let pollfd = PollFd::new(input.as_raw_fd(), PollFlags::POLLIN);

    let mut gesture: Option<SwaypedGesture> = None;

    while poll(&mut [pollfd], -1).is_ok() {
        let Ok (_) = input.dispatch() else {
            continue;
        };

        for event in &mut input {
            match event {
                Gesture(gesture_event) => gesture_handle_event(gesture_event, &mut gesture),
                Pointer(pointer_event) => pointer_handle_event(pointer_event),
                _ => Ok(()),
            }?;
        }
    }

    Ok(())
}
