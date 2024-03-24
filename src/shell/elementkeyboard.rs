use super::WindowElement;
use crate::{state::Backend, SmallCageState};
use smithay::{desktop::WindowSurface, input::keyboard::KeyboardTarget};

impl<BackendData: Backend + 'static> KeyboardTarget<SmallCageState<BackendData>> for WindowElement {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        keys: Vec<smithay::input::keyboard::KeysymHandle<'_>>,
        serial: smithay::utils::Serial,
    ) {
        if let WindowSurface::Wayland(w) = self.window.underlying_surface() {
            KeyboardTarget::enter(w.wl_surface(), seat, data, keys, serial)
        }
    }
    fn modifiers(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        if let WindowSurface::Wayland(w) = self.window.underlying_surface() {
            KeyboardTarget::modifiers(w.wl_surface(), seat, data, modifiers, serial)
        }
    }
    fn leave(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        serial: smithay::utils::Serial,
    ) {
        if let WindowSurface::Wayland(w) = self.window.underlying_surface() {
            KeyboardTarget::leave(w.wl_surface(), seat, data, serial)
        }
    }
    fn key(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        key: smithay::input::keyboard::KeysymHandle<'_>,
        state: smithay::backend::input::KeyState,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        if let WindowSurface::Wayland(w) = self.window.underlying_surface() {
            KeyboardTarget::key(w.wl_surface(), seat, data, key, state, serial, time)
        }
    }
    // add code here
}
