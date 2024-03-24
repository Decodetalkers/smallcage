use super::WindowElement;
use crate::{state::Backend, SmallCageState};
use smithay::input::keyboard::KeyboardTarget;

impl<BackendData: Backend + 'static> KeyboardTarget<SmallCageState<BackendData>> for WindowElement {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        keys: Vec<smithay::input::keyboard::KeysymHandle<'_>>,
        serial: smithay::utils::Serial,
    ) {
        self.window.enter(seat, data, keys, serial)
    }
    fn modifiers(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        self.window.modifiers(seat, data, modifiers, serial)
    }
    fn leave(
        &self,
        seat: &smithay::input::Seat<SmallCageState<BackendData>>,
        data: &mut SmallCageState<BackendData>,
        serial: smithay::utils::Serial,
    ) {
        self.window.leave(seat, data, serial)
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
        self.window.key(seat, data, key, state, serial, time)
    }
    // add code here
}
