use std::f64::consts::PI;

use crate::commands::SwaypedCommand;

const SWIPE_DIST_THRESHOLD: f64 = 100.0;

enum SwaypedSwipeDir {
    UP,
    DOWN,
    LEFT,
    RIGHT,
}

impl SwaypedSwipeDir {
    fn process_command(self, _finger_count: i32) {
        let cmd: SwaypedCommand = match self {
            SwaypedSwipeDir::UP => SwaypedCommand::WorkspaceNew,
            SwaypedSwipeDir::DOWN => SwaypedCommand::WorkspaceBackAndForth,
            SwaypedSwipeDir::LEFT => SwaypedCommand::WorkspacePrev,
            SwaypedSwipeDir::RIGHT => SwaypedCommand::WorkspaceNext,
        };
        cmd.process_command()
    }
}

pub fn swipe_process(dx: f64, dy: f64, finger_count: i32) {
    let mut swipe: Option<SwaypedSwipeDir> = None;
    let mut ratio: f64 = PI / 8.0;
    ratio = ratio.tan();

    if dx.abs() >= SWIPE_DIST_THRESHOLD && dy.abs() >= SWIPE_DIST_THRESHOLD {
        if (dx.abs() / dy.abs()) > (dy.abs() / dx.abs() + ratio) {
            swipe = if dx > 0.0 {
                Some(SwaypedSwipeDir::RIGHT)
            } else {
                Some(SwaypedSwipeDir::LEFT)
            }
        } else if (dy.abs() / dx.abs()) > (dx.abs() / dy.abs() + ratio) {
            swipe = if dy > 0.0 {
                Some(SwaypedSwipeDir::DOWN)
            } else {
                Some(SwaypedSwipeDir::UP)
            }
        }
    } else if dx.abs() > SWIPE_DIST_THRESHOLD {
        swipe = if dx > 0.0 {
            Some(SwaypedSwipeDir::RIGHT)
        } else {
            Some(SwaypedSwipeDir::LEFT)
        }
    } else if dy.abs() > SWIPE_DIST_THRESHOLD {
        swipe = if dy > 0.0 {
            Some(SwaypedSwipeDir::DOWN)
        } else {
            Some(SwaypedSwipeDir::UP)
        }
    }

    swipe_detected(swipe, finger_count);
}

fn swipe_detected(o: Option<SwaypedSwipeDir>, finger_count: i32) {
    match o {
        Some(sw) => sw.process_command(finger_count),
        None => (),
    }
}
