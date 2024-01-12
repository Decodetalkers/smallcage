use super::WindowElement;
use crate::SmallCage;
use smithay::input::keyboard::KeyboardTarget;

impl KeyboardTarget<SmallCage> for WindowElement {
    fn enter(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        keys: Vec<smithay::input::keyboard::KeysymHandle<'_>>,
        serial: smithay::utils::Serial,
    ) {
        self.window.enter(seat, data, keys, serial)
    }
    fn modifiers(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        modifiers: smithay::input::keyboard::ModifiersState,
        serial: smithay::utils::Serial,
    ) {
        self.window.modifiers(seat, data, modifiers, serial)
    }
    fn leave(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        serial: smithay::utils::Serial,
    ) {
        self.window.leave(seat, data, serial)
    }
    fn key(
        &self,
        seat: &smithay::input::Seat<SmallCage>,
        data: &mut SmallCage,
        key: smithay::input::keyboard::KeysymHandle<'_>,
        state: smithay::backend::input::KeyState,
        serial: smithay::utils::Serial,
        time: u32,
    ) {
        self.window.key(seat, data, key, state, serial, time)
    }
    // add code here
}
