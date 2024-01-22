mod compositor;
mod ssd;
mod xdg_shell;

use crate::shell::WindowElement;
use crate::SmallCage;
pub use ssd::{HeaderBar, HEADER_BAR_HEIGHT};

//
// Wl Seat
//

use smithay::input::{SeatHandler, SeatState};
use smithay::wayland::selection::data_device::{
    set_data_device_focus, ClientDndGrabHandler, DataDeviceHandler, ServerDndGrabHandler,
};
use smithay::wayland::selection::primary_selection::{
    PrimarySelectionHandler, PrimarySelectionState,
};
use smithay::wayland::selection::SelectionHandler;
use smithay::{delegate_data_device, delegate_output, delegate_primary_selection, delegate_seat};

impl SeatHandler for SmallCage {
    type KeyboardFocus = WindowElement;
    type PointerFocus = WindowElement;

    fn seat_state(&mut self) -> &mut SeatState<SmallCage> {
        &mut self.seat_state
    }

    fn cursor_image(
        &mut self,
        _seat: &smithay::input::Seat<Self>,
        image: smithay::input::pointer::CursorImageStatus,
    ) {
        *self.cursor_status.lock().unwrap() = image;
    }

    fn focus_changed(
        &mut self,
        seat: &smithay::input::Seat<Self>,
        focused: Option<&WindowElement>,
    ) {
        let dh = &self.display_handle;
        let client = focused.and_then(|s| dh.get_client(s.id()).ok());
        set_data_device_focus(dh, seat, client)
    }
}

impl PrimarySelectionHandler for SmallCage {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection_state
    }
}

delegate_seat!(SmallCage);
delegate_primary_selection!(SmallCage);

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
