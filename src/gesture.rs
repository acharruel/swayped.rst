use anyhow::Result;
use input::event::gesture::GestureSwipeEvent::{Begin, End, Update};
use input::event::gesture::{
    GestureEventCoordinates, GestureEventTrait, GestureSwipeBeginEvent, GestureSwipeEndEvent,
    GestureSwipeEvent, GestureSwipeUpdateEvent,
};
use std::f64::consts::PI;
use tracing::{debug, trace};

use crate::commands::{CommandDesc, InputCommand};

const SWIPE_DIST_THRESHOLD: f64 = 100.0;

pub struct SwaypedGesture<'a> {
    dx: f64,
    dy: f64,
    finger_count: i32,
    cmd_desc: &'a CommandDesc,
}

#[derive(Debug)]
enum SwaypedSwipeDir {
    Up,
    Down,
    Left,
    Right,
}

impl<'a> SwaypedGesture<'a> {
    pub fn new(cmd_desc: &'a CommandDesc) -> Self {
        SwaypedGesture {
            dx: 0.0,
            dy: 0.0,
            finger_count: 0,
            cmd_desc,
        }
    }

    pub fn reset(&mut self) -> Result<()> {
        self.dx = 0.0;
        self.dy = 0.0;
        self.finger_count = 0;
        Ok(())
    }

    fn begin(&mut self, event: &GestureSwipeBeginEvent) -> Result<()> {
        trace!(finger_count = ?event.finger_count(), "begin gesture");
        self.reset()?;
        Ok(())
    }

    fn update(&mut self, event: &GestureSwipeUpdateEvent) -> Result<()> {
        trace!(dx = ?event.dx(), dy = ?event.dy(), finger_count = ?event.finger_count(), "update gesture");
        self.dx += event.dx();
        self.dy += event.dy();
        self.finger_count = event.finger_count();
        Ok(())
    }

    async fn terminate(&self, event: &GestureSwipeEndEvent) -> Result<()> {
        trace!(finger_count = ?event.finger_count(), "terminate gesture");
        trace!(?self.dx, ?self.dy, ?self.finger_count, "terminate gesture");
        self.process_swipe().await?;
        Ok(())
    }

    pub async fn handle_event(&mut self, event: &GestureSwipeEvent) -> Result<()> {
        match event {
            Begin(event) => self.begin(event)?,
            Update(event) => self.update(event)?,
            End(event) => self.terminate(event).await?,
            &_ => (),
        }
        Ok(())
    }

    async fn process_swipe(&self) -> Result<()> {
        use SwaypedSwipeDir::*;

        let dx = self.dx;
        let dy = self.dy;
        let finger_count = self.finger_count;

        let mut ratio: f64 = PI / 8.0;
        ratio = ratio.tan();

        let mut swipe: Option<SwaypedSwipeDir> = None;
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

        debug!(?swipe, ?finger_count, "swipe_process");

        let cmd = match swipe {
            Some(Up) => InputCommand::SwipeUp(finger_count),
            Some(Down) => InputCommand::SwipeDown(finger_count),
            Some(Left) => InputCommand::SwipeLeft(finger_count),
            Some(Right) => InputCommand::SwipeRight(finger_count),
            None => return Ok(()),
        };

        self.cmd_desc.send(cmd).await?;

        Ok(())
    }
}

#[cfg(test)]
mod test {
    use tokio::sync::mpsc;

    use super::*;
    use crate::{commands::CommandDesc, config::TomlConfig};

    #[test]
    fn test_swayped_gesture_new() {
        let (tx, _) = mpsc::channel(1);
        let config = TomlConfig { mappings: vec![] };

        let cmd_desc = CommandDesc::new(false, config, tx);

        let gesture = SwaypedGesture::new(&cmd_desc);

        assert_eq!(gesture.dx, 0.0);
        assert_eq!(gesture.dy, 0.0);
        assert_eq!(gesture.finger_count, 0);
    }

    #[tokio::test]
    async fn test_swayped_gesture_reset() {
        let (tx, _) = mpsc::channel(1);
        let config = TomlConfig { mappings: vec![] };

        let cmd_desc = CommandDesc::new(false, config, tx);

        let mut gesture = SwaypedGesture::new(&cmd_desc);

        gesture.dx = 10.0;
        gesture.dy = 10.0;
        gesture.finger_count = 3;

        let res = gesture.reset();
        assert_eq!(res.is_ok(), true);
        assert_eq!(gesture.dx, 0.0);
        assert_eq!(gesture.dy, 0.0);
        assert_eq!(gesture.finger_count, 0);
    }

    #[tokio::test]
    async fn test_swayped_gesture_swipe_process() {
        let (tx, _) = mpsc::channel(1);
        let config = TomlConfig { mappings: vec![] };

        let cmd_desc = CommandDesc::new(false, config, tx);

        let mut gesture = SwaypedGesture::new(&cmd_desc);

        gesture.dx = 100.0;
        gesture.dy = 0.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = 0.0;
        gesture.dy = 100.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = -100.0;
        gesture.dy = 0.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = 0.0;
        gesture.dy = -100.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = 100.0;
        gesture.dy = 100.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = -100.0;
        gesture.dy = 100.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = 100.0;
        gesture.dy = -100.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);

        gesture.dx = -100.0;
        gesture.dy = -100.0;
        gesture.finger_count = 3;

        let res = gesture.process_swipe().await;
        assert_eq!(res.is_ok(), true);
    }
}
