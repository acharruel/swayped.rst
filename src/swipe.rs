use std::f64::consts::PI;

use log::debug;

use crate::commands::SwaypedCommand;

const SWIPE_DIST_THRESHOLD: f64 = 100.0;

enum SwaypedSwipeDir {
    Up,
    Down,
    Left,
    Right,
}

impl SwaypedSwipeDir {
    fn process_command(self, _finger_count: i32) {
        let cmd: SwaypedCommand = match self {
            SwaypedSwipeDir::Up => SwaypedCommand::WorkspaceNew,
            SwaypedSwipeDir::Down => SwaypedCommand::WorkspaceBackAndForth,
            SwaypedSwipeDir::Left => SwaypedCommand::WorkspacePrev,
            SwaypedSwipeDir::Right => SwaypedCommand::WorkspaceNext,
        };
        cmd.process_command()
    }

    fn display(&self) -> String {
        match self {
            SwaypedSwipeDir::Up => "UP".to_string(),
            SwaypedSwipeDir::Down => "DOWN".to_string(),
            SwaypedSwipeDir::Left => "LEFT".to_string(),
            SwaypedSwipeDir::Right => "RIGHT".to_string(),
        }
    }
}

// convert swipe coordinates to user event
pub fn swipe_process(dx: f64, dy: f64, finger_count: i32) {
    let mut swipe: Option<SwaypedSwipeDir> = None;
    let mut ratio: f64 = PI / 8.0;
    ratio = ratio.tan();

    debug!(
        "swipe_process: dx {} dy {} finger_count {}",
        dx, dy, finger_count
    );

    if dx.abs() >= SWIPE_DIST_THRESHOLD && dy.abs() >= SWIPE_DIST_THRESHOLD {
        if (dx.abs() / dy.abs()) > (dy.abs() / dx.abs() + ratio) {
            swipe = if dx > 0.0 {
                Some(SwaypedSwipeDir::Right)
            } else {
                Some(SwaypedSwipeDir::Left)
            }
        } else if (dy.abs() / dx.abs()) > (dx.abs() / dy.abs() + ratio) {
            swipe = if dy > 0.0 {
                Some(SwaypedSwipeDir::Down)
            } else {
                Some(SwaypedSwipeDir::Up)
            }
        }
    } else if dx.abs() > SWIPE_DIST_THRESHOLD {
        swipe = if dx > 0.0 {
            Some(SwaypedSwipeDir::Right)
        } else {
            Some(SwaypedSwipeDir::Left)
        }
    } else if dy.abs() > SWIPE_DIST_THRESHOLD {
        swipe = if dy > 0.0 {
            Some(SwaypedSwipeDir::Down)
        } else {
            Some(SwaypedSwipeDir::Up)
        }
    }

    swipe_detected(swipe, finger_count);
}

fn swipe_detected(o: Option<SwaypedSwipeDir>, finger_count: i32) {
    if let Some(sw) = o {
        debug!(
            "swipe_detected! {:?} finger count {}",
            sw.display(),
            finger_count
        );
        sw.process_command(finger_count);
    }
}
