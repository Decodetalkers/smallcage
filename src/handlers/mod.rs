mod compositor;
mod xdg_shell;

use crate::SmallCage;

//
// Wl Seat
//

use smithay::input::{SeatHandler, SeatState};
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::selection::data_device::{
    ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler,
};
use smithay::wayland::selection::SelectionHandler;
use smithay::{delegate_data_device, delegate_output, delegate_seat};

impl SeatHandler for SmallCage {
    type KeyboardFocus = WlSurface;
    type PointerFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<SmallCage> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &smithay::input::Seat<Self>,
        _image: smithay::input::pointer::CursorImageStatus,
    ) {
    }
    fn focus_changed(&mut self, _seat: &smithay::input::Seat<Self>, _focused: Option<&WlSurface>) {}
}

delegate_seat!(SmallCage);

//
// Wl Data Device
//

impl SelectionHandler for SmallCage {
    type SelectionUserData = ();
}

impl DataDeviceHandler for SmallCage {
    fn data_device_state(&self) -> &smithay::wayland::selection::data_device::DataDeviceState {
        &self.data_device_state
    }
}

impl ClientDndGrabHandler for SmallCage {}
impl ServerDndGrabHandler for SmallCage {}

delegate_data_device!(SmallCage);

//
// Wl Output & Xdg Output
//

delegate_output!(SmallCage);
