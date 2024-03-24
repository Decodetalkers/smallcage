use super::{SsdResizeState, WindowElement};
use crate::{
    grabs::{ResizeEdge, ResizeSurfaceGrab},
    handlers::HEADER_BAR_HEIGHT,
    state::Backend,
    SmallCageState,
};
#[allow(unused)]
use smithay::{
    backend::input::ButtonState,
    input::pointer::{
        CursorIcon, CursorImageStatus, GrabStartData as PointerGrabStartData, PointerTarget,
    },
};
use smithay::{
    desktop::space::SpaceElement,
    input::{pointer::Focus, Seat},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{protocol::wl_surface::WlSurface, Resource},
    },
    utils::{Rectangle, Serial},
};

// NOTE: if enter, set state, and check position
impl<BackendData: Backend + 'static> PointerTarget<SmallCageState<BackendData>> for WindowElement {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        let (w, h) = self.geometry().size.into();
        let mut state = self.window_state_mut();
        if state.is_ssd {
            'resizeState: {
                if event.location.y < 70. && event.location.y > HEADER_BAR_HEIGHT as f64 {
                    state.ssd_resize_state = SsdResizeState::Top;
                    break 'resizeState;
                }
                if event.location.y > h as f64 - 70. {
                    state.ssd_resize_state = SsdResizeState::Bottom;
                    break 'resizeState;
                }
                if event.location.x < 10. {
                    state.ssd_resize_state = SsdResizeState::Left;
                    break 'resizeState;
                }
                if event.location.x > w as f64 - 10. {
                    state.ssd_resize_state = SsdResizeState::Right;
                    break 'resizeState;
                }
                state.ssd_resize_state = SsdResizeState::Nothing;
            }

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
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::MotionEvent,
    ) {
        let (w, h) = self.geometry().size.into();
        let mut state = self.window_state_mut();
        if state.is_ssd {
            'resizeState: {
                if event.location.y < 70. && event.location.y > HEADER_BAR_HEIGHT as f64 {
                    state.ssd_resize_state = SsdResizeState::Top;
                    break 'resizeState;
                }
                if event.location.y > h as f64 - 70. {
                    state.ssd_resize_state = SsdResizeState::Bottom;
                    break 'resizeState;
                }
                if event.location.x < 10. {
                    state.ssd_resize_state = SsdResizeState::Left;
                    break 'resizeState;
                }
                if event.location.x > w as f64 - 10. {
                    state.ssd_resize_state = SsdResizeState::Right;
                    break 'resizeState;
                }
                state.ssd_resize_state = SsdResizeState::Nothing;
            }
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
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        let mut state = self.window_state_mut();
        if state.is_ssd {
            state.ssd_resize_state = SsdResizeState::Nothing;
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
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::ButtonEvent,
    ) {
        let mut state = self.window_state_mut();
        if state.is_ssd {
            let ssd_resize_state = state.ssd_resize_state;
            let serial = event.serial;
            let window = self.clone();
            if state.element_state.is_untiled_state() {
                data.handle.insert_idle(move |data| {
                    let state = &mut data.state;
                    let edges = match ssd_resize_state {
                        SsdResizeState::Left => ResizeEdge::LEFT,
                        SsdResizeState::Top => ResizeEdge::TOP,
                        SsdResizeState::Right => ResizeEdge::RIGHT,
                        SsdResizeState::Bottom => ResizeEdge::BOTTOM,
                        _ => return,
                    };
                    let seat = &state.seat;
                    let Some(start_data) = check_grab(seat, window.toplevel().wl_surface(), serial)
                    else {
                        return;
                    };
                    let pointer = state.seat.get_pointer().unwrap();
                    let initial_window_location = state.space.element_location(&window).unwrap();
                    let initial_window_size = window.geometry().size;
                    let top_level = window.toplevel();
                    top_level.with_pending_state(|state| {
                        state.states.set(xdg_toplevel::State::Resizing);
                    });
                    top_level.send_pending_configure();
                    let grab = ResizeSurfaceGrab::start(
                        start_data,
                        window.clone(),
                        edges,
                        Rectangle::from_loc_and_size(initial_window_location, initial_window_size),
                    );
                    pointer.set_grab(state, grab, serial, Focus::Clear);
                });
            }
            if state.ptr_entered_window {
                self.window.button(seat, data, event)
            } else if event.state == ButtonState::Pressed {
                state.header_bar.clicked(seat, data, self, event.serial)
            }
            return;
        }
        self.window.button(seat, data, event)
    }

    fn relative_motion(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::RelativeMotionEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.relative_motion(seat, data, event)
        }
    }

    fn gesture_hold_end(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GestureHoldEndEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_hold_end(seat, data, event);
        }
    }

    fn gesture_swipe_end(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GestureSwipeEndEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_swipe_end(seat, data, event)
        }
    }

    fn gesture_swipe_begin(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GestureSwipeBeginEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_swipe_begin(seat, data, event)
        }
    }

    fn gesture_hold_begin(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GestureHoldBeginEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_hold_begin(seat, data, event)
        }
    }

    fn gesture_pinch_end(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GesturePinchEndEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_pinch_end(seat, data, event)
        }
    }

    fn gesture_swipe_update(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GestureSwipeUpdateEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_swipe_update(seat, data, event)
        }
    }

    fn gesture_pinch_update(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GesturePinchUpdateEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_pinch_update(seat, data, event)
        }
    }

    fn gesture_pinch_begin(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        event: &smithay::input::pointer::GesturePinchBeginEvent,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.gesture_pinch_begin(seat, data, event)
        }
    }

    fn frame(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.frame(seat, data)
        }
    }

    fn axis(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        frame: smithay::input::pointer::AxisFrame,
    ) {
        let state = self.window_state();
        if !state.is_ssd || state.ptr_entered_window {
            self.window.axis(seat, data, frame)
        }
    }
}

fn check_grab<BackendData: Backend + 'static>(
    seat: &Seat<SmallCageState<BackendData>>,
    surface: &WlSurface,
    serial: Serial,
) -> Option<PointerGrabStartData<SmallCageState<BackendData>>> {
    let pointer = seat.get_pointer()?;

    // Check that this surface has a click grab.
    if !pointer.has_grab(serial) {
        return None;
    }

    let start_data = pointer.grab_start_data()?;

    let (focus, _) = start_data.focus.as_ref()?;
    // If the focus was for a different surface, ignore the request.
    if !focus.id().same_client_as(&surface.id()) {
        return None;
    }

    Some(start_data)
}
