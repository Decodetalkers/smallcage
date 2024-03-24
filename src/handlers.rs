mod compositor;
mod ssd;
mod xdg_shell;

use crate::shell::WindowElement;
use crate::state::Backend;
use crate::SmallCageState;
use smithay::reexports::wayland_server::protocol::wl_surface::WlSurface;
use smithay::wayland::output::OutputHandler;
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

impl<BackendData: Backend + 'static> SeatHandler for SmallCageState<BackendData> {
    type KeyboardFocus = WindowElement;
    type PointerFocus = WindowElement;
    // TODO:
    type TouchFocus = WlSurface;

    fn seat_state(&mut self) -> &mut SeatState<SmallCageState<BackendData>> {
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

impl<BackendData: Backend + 'static> PrimarySelectionHandler for SmallCageState<BackendData> {
    fn primary_selection_state(&self) -> &PrimarySelectionState {
        &self.primary_selection_state
    }
}

delegate_seat!(@<BackendData: Backend + 'static> SmallCageState<BackendData>);
delegate_primary_selection!(@<BackendData: Backend + 'static> SmallCageState<BackendData>);

//
// Wl Data Device
//

impl<BackendData: Backend + 'static> SelectionHandler for SmallCageState<BackendData> {
    type SelectionUserData = ();
}

impl<BackendData: Backend + 'static> DataDeviceHandler for SmallCageState<BackendData> {
    fn data_device_state(&self) -> &smithay::wayland::selection::data_device::DataDeviceState {
        &self.data_device_state
    }
}

impl<BackendData: Backend + 'static> ClientDndGrabHandler for SmallCageState<BackendData> {}
impl<BackendData: Backend + 'static> ServerDndGrabHandler for SmallCageState<BackendData> {}

delegate_data_device!(@<BackendData: Backend + 'static>SmallCageState<BackendData>);

//
// Wl Output & Xdg Output
//

impl<BackendData: Backend + 'static> OutputHandler for SmallCageState<BackendData> {}

delegate_output!(@<BackendData: Backend + 'static> SmallCageState<BackendData>);
