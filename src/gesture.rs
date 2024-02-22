use crate::swipe::swipe_process;
use anyhow::Result;
use input::event::{
    gesture::{
        GestureEventCoordinates, GestureEventTrait, GestureSwipeEndEvent, GestureSwipeEvent,
        GestureSwipeUpdateEvent,
    },
    GestureEvent,
};
use log::debug;

#[allow(dead_code)]
pub enum SwaypedGesture {
    Swipe { dx: f64, dy: f64, finger_count: i32 },
    Hold,
}

impl SwaypedGesture {
    fn begin(&mut self) {
        if let SwaypedGesture::Swipe {
            dx,
            dy,
            finger_count: _,
        } = self
        {
            *dx = 0.0;
            *dy = 0.0;
        }
    }

    fn update(&mut self, event: &GestureSwipeUpdateEvent) {
        if let SwaypedGesture::Swipe {
            dx,
            dy,
            finger_count,
        } = self
        {
            *dx += event.dx();
            *dy += event.dy();
            *finger_count = event.finger_count();
        }
    }

    fn end(&self, _event: &GestureSwipeEndEvent) {
        if let SwaypedGesture::Swipe {
            dx,
            dy,
            finger_count,
        } = self
        {
            swipe_process(*dx, *dy, *finger_count)
        }
    }

    fn abort(&self) {
        if let SwaypedGesture::Swipe {
            dx,
            dy,
            finger_count,
        } = self
        {
            debug!("abort: dx {} dy {} finger_count {}", dx, dy, finger_count)
        }
    }
}

fn gesture_handle_swipe_event(
    event: &GestureSwipeEvent,
    gesture: &mut Option<SwaypedGesture>,
) -> Result<()> {
    use GestureSwipeEvent::*;

    // make sure gesture is a valid option, create one if needed
    match gesture {
        None => {
            *gesture = Some(SwaypedGesture::Swipe {
                dx: 0.0,
                dy: 0.0,
                finger_count: 0,
            });
        }
        Some(_) => (),
    };

    // handle gesture operation
    match gesture {
        None => (),
        Some(sg) => {
            match event {
                Begin(_) => sg.begin(),
                Update(u) => sg.update(u),
                End(e) => sg.end(e),
                _ => (),
            };
        }
    };

    Ok(())
}

pub fn gesture_handle_event(
    event: &GestureEvent,
    gesture: &mut Option<SwaypedGesture>,
) -> Result<()> {
    use GestureEvent::*;

    match event {
        // swipe event
        Swipe(swipe_event) => gesture_handle_swipe_event(swipe_event, gesture),

        // hold event: abort pending gesture
        Hold(_) => {
            match gesture {
                None => (),
                Some(sg) => sg.abort(),
            };
            Ok(())
        }

        _ => Ok(()),
    }
}
