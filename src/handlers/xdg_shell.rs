use smithay::{
    delegate_xdg_shell,
    desktop::{space::SpaceElement, PopupKind},
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{wl_seat, wl_surface::WlSurface},
            Resource,
        },
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        compositor::with_states,
        shell::xdg::{
            PopupSurface, PositionerState, ToplevelSurface, XdgPopupSurfaceData, XdgShellHandler,
            XdgShellState, XdgToplevelSurfaceData,
        },
    },
};

use crate::{shell::WindowElement, state::SplitState, SmallCage};

impl XdgShellHandler for SmallCage {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = WindowElement::new(surface);
        self.space.map_element(window, (0, 0), false);
    }

    // TODO: this need to record the place window is destoried
    // and place other windows
    #[allow(unused)]
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {}
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
    pub fn handle_commit(&mut self, surface: &WlSurface) -> Option<()> {
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
            self.resize_element_commit(surface);
        }

        Some(())
    }

    fn resize_element_commit(&mut self, surface: &WlSurface) -> Option<()> {
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)?;
        if window.is_init {
            return None;
        }
        match self.current_activewindow_rectangle(surface) {
            Some(rec) => self.map_with_split(surface, rec),
            None => self.map_one_element(surface),
        }
    }

    // this should commit when full is here
    #[allow(unused)]
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

// This is the logic of tile, here need to find current surface under pointer
// with the split direction, split the space for new window
//
// TODO: I need a new element to mark if it is just init
impl SmallCage {
    fn map_one_element(&mut self, surface: &WlSurface) -> Option<()> {
        let current_screen = self.current_screen_rectangle()?;
        let loc = current_screen.loc;
        let (w, h) = current_screen.size.into();
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()?;
        window.toplevel().with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some((w, h).into());
        });
        let mut fin_window = window.clone();
        fin_window.set_inited();
        fin_window.toplevel().send_configure();
        self.space.map_element(fin_window, loc, true);

        Some(())
    }

    fn map_with_split(&mut self, surface: &WlSurface, windowpre: WindowElement) -> Option<()> {
        let rec = windowpre.geometry();
        let (x, y) = self.space.element_location(&windowpre)?.into();
        let (w, h) = rec.size.into();

        let (point, size): (Point<i32, Logical>, Size<i32, Logical>) = match self.splitstate {
            SplitState::H => {
                let width = w / 2;
                let height = h;
                ((x + width, y).into(), (width, height).into())
            }
            SplitState::V => {
                let width = w;
                let height = h / 2;
                ((x, y + height).into(), (width, height).into())
            }
        };

        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()?;

        window.toplevel().with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some(size);
        });
        window.toplevel().send_configure();
        let mut fin_window = window.clone();
        fin_window.set_inited();
        self.space.map_element(fin_window, point, false);

        windowpre.toplevel().with_pending_state(|state| {
            state.states.set(xdg_toplevel::State::Resizing);
            state.size = Some(size);
        });
        windowpre.toplevel().send_configure();

        Some(())
    }
    fn find_current_select_window(&self) -> Option<&WindowElement> {
        let pos = self.pointer.current_location();
        Some(self.space.element_under(pos)?.0)
    }
    fn current_activewindow_rectangle(&self, surface: &WlSurface) -> Option<WindowElement> {
        let window = self.find_current_select_window()?;
        if window.toplevel().wl_surface() == surface {
            return None;
        }
        Some(window.clone())
    }

    fn current_screen_rectangle(&self) -> Option<Rectangle<i32, Logical>> {
        let output = self
            .space
            .output_under(self.pointer.current_location())
            .next()?;
        self.space.output_geometry(output)
    }
}
