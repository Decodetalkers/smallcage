use super::WindowElement;
use crate::{handlers::HEADER_BAR_HEIGHT, SmallCage};
use smithay::{backend::input::ButtonState, input::pointer::PointerTarget};

impl PointerTarget<SmallCage> for WindowElement {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        let mut state = self.window_state_mut();
        if state.is_ssd {
            if event.location.y < HEADER_BAR_HEIGHT as f64 {
                state.header_bar.pointer_enter(event.location);
            } else {
                state.header_bar.pointer_leave();
                let mut event = event.clone();
                event.location.y -= HEADER_BAR_HEIGHT as f64;
                self.window.enter(seat, data, &event);
                state.ptr_entered_window = true;
            }
            return;
        }
        state.ptr_entered_window = true;
        self.window.enter(seat, data, event)
    }

    fn motion(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        let mut state = self.window_state_mut();
        if state.is_ssd {
            if event.location.y < HEADER_BAR_HEIGHT as f64 {
                self.window.motion(seat, data, event);
                state.ptr_entered_window = false;
                state.header_bar.pointer_enter(event.location);
            } else {
                state.ptr_entered_window = true;
                state.header_bar.pointer_leave();
                let mut event = event.clone();
                event.location.y -= HEADER_BAR_HEIGHT as f64;
                self.window.enter(seat, data, &event);
            }
            return;
        }
        self.window.motion(seat, data, event)
    }

    fn leave(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        let mut state = self.window_state_mut();
        if state.is_ssd {
            state.header_bar.pointer_leave();
            if state.ptr_entered_window {
                self.window.leave(seat, data, serial, time);
                state.ptr_entered_window = false
            }
        } else {
            self.window.leave(seat, data, serial, time);
            state.ptr_entered_window = false;
        }
    }

    fn button(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        let mut state = self.window_state_mut();
        if state.is_ssd {
            if state.ptr_entered_window {
                self.window.button(seat, data, event)
            } else {
                if state.header_bar.clicked(seat, data, self, event.serial)
                    && event.state == ButtonState::Released
                {
                    state.element_state.change_state();
                    self.window.toplevel().send_configure();
                }
            }
            return;
        }
        self.window.button(seat, data, event)
    }

    fn relative_motion(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.relative_motion(seat, data, event)
        }
    }

    fn gesture_hold_end(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureHoldEndEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_hold_end(seat, data, event);
        }
    }

    fn gesture_swipe_end(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureSwipeEndEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_swipe_end(seat, data, event)
        }
    }

    fn gesture_swipe_begin(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureSwipeBeginEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_swipe_begin(seat, data, event)
        }
    }

    fn gesture_hold_begin(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureHoldBeginEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_hold_begin(seat, data, event)
        }
    }

    fn gesture_pinch_end(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GesturePinchEndEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_pinch_end(seat, data, event)
        }
    }

    fn gesture_swipe_update(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GestureSwipeUpdateEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_swipe_update(seat, data, event)
        }
    }

    fn gesture_pinch_update(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GesturePinchUpdateEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_pinch_update(seat, data, event)
        }
    }

    fn gesture_pinch_begin(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        event: &smithay::input::pointer::GesturePinchBeginEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_pinch_begin(seat, data, event)
        }
    }

    fn frame(&self, seat: &smithay::input::Seat<SmallCage>, data: &mut SmallCage) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.frame(seat, data)
        }
    }

    fn axis(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        frame: smithay::input::pointer::AxisFrame,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.axis(seat, data, frame)
        }
    }
}
