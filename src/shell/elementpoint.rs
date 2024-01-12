use super::WindowElement;
use crate::SmallCage;
use smithay::input::pointer::PointerTarget;
impl PointerTarget<SmallCage> for WindowElement {
    fn leave(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        self.window.leave(seat, data, serial, time)
    }
    fn gesture_hold_end(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureHoldEndEvent,
    ) {
        self.window.gesture_hold_end(seat, data, event)
    }
    fn gesture_swipe_end(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureSwipeEndEvent,
    ) {
        self.window.gesture_swipe_end(seat, data, event)
    }
    fn gesture_swipe_begin(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureSwipeBeginEvent,
    ) {
        self.window.gesture_swipe_begin(seat, data, event)
    }
    fn gesture_hold_begin(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureHoldBeginEvent,
    ) {
        self.window.gesture_hold_begin(seat, data, event)
    }
    fn gesture_pinch_end(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GesturePinchEndEvent,
    ) {
        self.window.gesture_pinch_end(seat, data, event)
    }
    fn gesture_swipe_update(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureSwipeUpdateEvent,
    ) {
        self.window.gesture_swipe_update(seat, data, event)
    }
    fn gesture_pinch_update(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GesturePinchUpdateEvent,
    ) {
        self.window.gesture_pinch_update(seat, data, event)
    }
    fn gesture_pinch_begin(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GesturePinchBeginEvent,
    ) {
        self.window.gesture_pinch_begin(seat, data, event)
    }
    fn relative_motion(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        self.window.relative_motion(seat, data, event)
    }
    fn frame(&self, seat: &smithay::input::Seat<SmallCage>, data: &mut SmallCage) {
        self.window.frame(seat, data)
    }
    fn axis(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        frame: smithay::input::pointer::AxisFrame,
    ) {
        self.window.axis(seat, data, frame)
    }
    fn button(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        self.window.button(seat, data, event)
    }
    fn motion(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        self.window.motion(seat, data, event)
    }
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        self.window.enter(seat, data, event)
    }
}
