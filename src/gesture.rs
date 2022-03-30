use crate::swipe::swipe_process;
use input::event::{
    gesture::{
        GestureEventCoordinates, GestureEventTrait, GestureSwipeEndEvent, GestureSwipeEvent,
        GestureSwipeUpdateEvent,
    },
    GestureEvent,
};

#[allow(dead_code)]
pub enum SwaypedGesture {
    Swipe { dx: f64, dy: f64, finger_count: i32 },
    Hold,
}

impl SwaypedGesture {
    fn begin(&mut self) {
        match self {
            SwaypedGesture::Swipe {
                dx,
                dy,
                finger_count: _,
            } => {
                *dx = 0.0;
                *dy = 0.0;
            }
            _ => (),
        }
    }

    fn update(&mut self, event: GestureSwipeUpdateEvent) {
        match self {
            SwaypedGesture::Swipe {
                dx,
                dy,
                finger_count,
            } => {
                *dx += event.dx();
                *dy += event.dy();
                *finger_count = event.finger_count();
            }
            _ => (),
        }
    }

    fn end(&self, _event: GestureSwipeEndEvent) {
        match self {
            SwaypedGesture::Swipe {
                dx,
                dy,
                finger_count,
            } => swipe_process(*dx, *dy, *finger_count),
            _ => (),
        }
    }

    fn abort(&self) {
        match self {
            SwaypedGesture::Swipe {
                dx,
                dy,
                finger_count,
            } => {
                println!("abort: dx {} dy {} finger_count {}", dx, dy, finger_count);
            }
            _ => (),
        }
    }
}

fn gesture_handle_swipe_event(event: GestureSwipeEvent, gesture: &mut Option<SwaypedGesture>) {
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
                GestureSwipeEvent::Begin(_) => sg.begin(),
                GestureSwipeEvent::Update(u) => sg.update(u),
                GestureSwipeEvent::End(e) => sg.end(e),
                _ => (),
            };
        }
    };
}

pub fn gesture_handle_event(event: GestureEvent, gesture: &mut Option<SwaypedGesture>) {
    match event {
        // swipe event
        GestureEvent::Swipe(swipe_event) => gesture_handle_swipe_event(swipe_event, gesture),

        // hold event: abort pending gesture
        GestureEvent::Hold(_) => {
            match gesture {
                None => (),
                Some(sg) => sg.abort(),
            };
        }

        _ => (),
    }
}
