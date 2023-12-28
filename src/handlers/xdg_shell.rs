use smithay::{
    delegate_xdg_shell,
    desktop::{PopupKind, Window},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{wl_seat, wl_surface::WlSurface},
            Resource,
        },
    },
    utils::Serial,
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgPopupSurfaceData, XdgShellHandler,
            XdgShellState, XdgToplevelSurfaceData,
        },
    },
};

use crate::SmallCage;

impl XdgShellHandler for SmallCage {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = Window::new(surface);
        self.space.map_element(window, (0, 0), true);
    }

    fn new_popup(&mut self, surface: PopupSurface, positioner: PositionerState) {
        // TODO: Popup handling using PopupManager
        surface.with_pending_state(|state| {
            // NOTE: This is not really necessary as the default geometry
            // is already set the same way, but for demonstrating how
            // to set the initial popup geometry this code is left as
            // an example
            state.geometry = positioner.get_geometry();
        });
        if let Err(err) = self.popups.track_popup(PopupKind::from(surface)) {
            tracing::warn!("Failed to track popup: {}", err);
        }
    }

    fn grab(&mut self, _surface: PopupSurface, _seat: wl_seat::WlSeat, _serial: Serial) {
        // TODO popup grabs
    }

    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        // TODO
    }
}

// Xdg Shell
delegate_xdg_shell!(SmallCage);

/// Should be called on `WlSurface::commit`
impl SmallCage {
    pub fn handle_commit(&self, surface: &WlSurface) -> Option<()> {
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()?;

        let initial_configure_sent = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        let isconfigured = with_states(surface, |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .configured
        });
        if !initial_configure_sent {
            window.toplevel().send_configure();
        } else if isconfigured {
            self.full_screen_commit(surface);
        }

        Some(())
    }

    pub fn full_screen_commit(&self, surface: &WlSurface) {
        let output = self.space.outputs().next().unwrap();
        let geometry = self.space.output_geometry(output).unwrap();
        let window = self.space.elements().next().unwrap();
        let toplevelsurface = window.toplevel();

        let client = self.display_handle.get_client(surface.id()).unwrap();

        let Some(wl_output) = output.client_outputs(&client).into_iter().next() else {
            return;
        };

        toplevelsurface.with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Fullscreen);
            state.size = Some(geometry.size);
            state.fullscreen_output = Some(wl_output);
        });
        toplevelsurface.send_configure();
    }
    pub fn handle_popup_commit(&self, surface: &WlSurface) {
        if let Some(popup) = self.popups.find_popup(surface) {
            // TODO: input method
            let PopupKind::Xdg(ref popup) = popup else {
                return;
            };
            let initial_configure_sent = with_states(surface, |states| {
                states
                    .data_map
                    .get::<XdgPopupSurfaceData>()
                    .unwrap()
                    .lock()
                    .unwrap()
                    .initial_configure_sent
            });
            if !initial_configure_sent {
                // NOTE: This should never fail as the initial configure is always
                // allowed.
                popup.send_configure().expect("initial configure failed");
            }
        };
    }
}
