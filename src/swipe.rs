use std::f64::consts::PI;

use anyhow::Result;
use tracing::{debug, error};

use crate::commands::SwaypedCommand;

const SWIPE_DIST_THRESHOLD: f64 = 100.0;

#[derive(Debug)]
enum SwaypedSwipeDir {
    Up,
    Down,
    Left,
    Right,
}

impl SwaypedSwipeDir {
    fn process_command(self, _finger_count: i32) -> Result<()> {
        use SwaypedCommand::*;
        use SwaypedSwipeDir::*;

        let cmd = match self {
            Up => WorkspaceNew,
            Down => WorkspaceBackAndForth,
            Left => WorkspacePrev,
            Right => WorkspaceNext,
        };
        cmd.process_command()
    }

    fn display(&self) -> String {
        use SwaypedSwipeDir::*;

        match self {
            Up => "UP".to_string(),
            Down => "DOWN".to_string(),
            Left => "LEFT".to_string(),
            Right => "RIGHT".to_string(),
        }
    }
}

// convert swipe coordinates to user event
pub fn swipe_process(dx: f64, dy: f64, finger_count: i32) {
    use SwaypedSwipeDir::*;

    let mut swipe: Option<SwaypedSwipeDir> = None;
    let mut ratio: f64 = PI / 8.0;
    ratio = ratio.tan();

    debug!(?dx, ?dy, ?finger_count, "swipe_process");

    if dx.abs() >= SWIPE_DIST_THRESHOLD && dy.abs() >= SWIPE_DIST_THRESHOLD {
        if (dx.abs() / dy.abs()) > (dy.abs() / dx.abs() + ratio) {
            swipe = if dx > 0.0 { Some(Right) } else { Some(Left) }
        } else if (dy.abs() / dx.abs()) > (dx.abs() / dy.abs() + ratio) {
            swipe = if dy > 0.0 { Some(Down) } else { Some(Up) }
        }
    } else if dx.abs() > SWIPE_DIST_THRESHOLD {
        swipe = if dx > 0.0 { Some(Right) } else { Some(Left) }
    } else if dy.abs() > SWIPE_DIST_THRESHOLD {
        swipe = if dy > 0.0 { Some(Down) } else { Some(Up) }
    }

    let Ok(_) = swipe_detected(swipe, finger_count) else {
        error!("Error in swipe detection");
        return;
    };
}

fn swipe_detected(o: Option<SwaypedSwipeDir>, finger_count: i32) -> Result<()> {
    if let Some(sw) = o {
        debug!(swipe = ?sw.display(), ?finger_count, "swipe_detected!");
        sw.process_command(finger_count)?;
    }
    Ok(())
}
