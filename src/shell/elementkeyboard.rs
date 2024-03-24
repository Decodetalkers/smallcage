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
        let WindowSurface::Wayland(surface) = self.window.underlying_surface();
        KeyboardTarget::enter(surface.wl_surface(), seat, data, keys, serial)
    }
    fn modifiers(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        let WindowSurface::Wayland(surface) = self.window.underlying_surface();
        KeyboardTarget::modifiers(surface.wl_surface(), seat, data, modifiers, serial)
    }
    fn leave(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        serial: smithay::utils::Serial,
    ) {
        let WindowSurface::Wayland(surface) = self.window.underlying_surface();
        KeyboardTarget::leave(surface.wl_surface(), seat, data, serial)
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
        let WindowSurface::Wayland(surface) = self.window.underlying_surface();
        KeyboardTarget::key(surface.wl_surface(), seat, data, key, state, serial, time)
    }
    // add code here
}
