use smithay::{
    delegate_xdg_shell,
    desktop::{space::SpaceElement, PopupKind},
    reexports::wayland_server::protocol::{wl_seat, wl_surface::WlSurface},
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
    fn toplevel_destroyed(&mut self, surface: ToplevelSurface) {
        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface.wl_surface())
            .cloned()
        else {
            return;
        };
        self.handle_dead_window(&window);
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

    pub fn handle_popup_commit(&self, surface: &WlSurface) {
        let Some(popup) = self.popups.find_popup(surface) else {
            return;
        };
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
            state.size = Some((w, h).into());
        });
        let mut fin_window = window.clone();
        fin_window.set_inited();
        fin_window.toplevel().send_configure();
        fin_window.set_output_size(current_screen.size);
        fin_window.set_element_size(current_screen.size);
        fin_window.set_origin_pos(loc);
        self.space.map_element(fin_window, loc, true);

        Some(())
    }

    fn map_with_split(&mut self, surface: &WlSurface, windowpre: WindowElement) -> Option<()> {
        let current_screen = self.current_screen_rectangle()?;
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
            state.size = Some(size);
        });
        window.toplevel().send_configure();

        let mut fin_window = window.clone();
        fin_window.set_inited();
        fin_window.set_element_size(size);
        fin_window.set_output_size(current_screen.size);
        fin_window.set_origin_pos(point);
        self.space.map_element(fin_window, point, false);

        let mut window_pre = windowpre.clone();
        window_pre.toplevel().with_pending_state(|state| {
            state.size = Some(size);
        });
        window_pre.toplevel().send_configure();
        window_pre.set_output_size(current_screen.size);
        window_pre.set_element_size(size);
        window_pre.remap_element(&mut self.space);

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

    // TODO:?
    fn current_screen_rectangle(&self) -> Option<Rectangle<i32, Logical>> {
        let output = self
            .space
            .output_under(self.pointer.current_location())
            .next()?;
        self.space.output_geometry(output)
    }

    // TODO: very base
    fn handle_dead_window(&mut self, window: &WindowElement) {
        let Some(current_screen) = self.current_screen_rectangle() else {
            return;
        };
        let screen_size = current_screen.size;
        let Some(pos) = self.space.element_location(window) else {
            self.space.unmap_elem(&window);
            return;
        };
        let (x, y) = pos.into();
        let (w, h) = window.geometry().size.into();
        let (rb_x, rb_y) = (x + w, y + h);
        self.space.unmap_elem(&window);
        if let Some(mut elements) = self.find_up_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(&element) else {
                    continue;
                };
                let (ow, oh) = element.geometry().size.into();
                let newsize: Size<i32, Logical> = (ow, oh + h).into();
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_origin_pos(ori_pos);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
                element.remap_element(&mut self.space);
            }
            return;
        }
        if let Some(mut elements) = self.find_down_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(&element) else {
                    continue;
                };
                let (o_x, _) = ori_pos.into();
                let (ow, oh) = element.geometry().size.into();
                let newsize: Size<i32, Logical> = (ow, oh + h).into();
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
                self.space.map_element(element.clone(), (o_x, rb_y), true);
            }
            return;
        }
        if let Some(mut elements) = self.find_left_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(&element) else {
                    continue;
                };
                let (ow, oh) = element.geometry().size.into();
                let newsize: Size<i32, Logical> = (ow + w, oh).into();
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_origin_pos(ori_pos);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
                element.remap_element(&mut self.space);
            }
            return;
        }
        if let Some(mut elements) = self.find_right_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(&element) else {
                    continue;
                };
                let (_, o_y) = ori_pos.into();
                let (ow, oh) = element.geometry().size.into();
                let newsize: Size<i32, Logical> = (ow + w, oh).into();
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_origin_pos(ori_pos);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
                self.space.map_element(element.clone(), (x, o_y), true);
            }
            return;
        }
    }
}

impl SmallCage {
    fn find_up_element(
        &self,
        (start_x, start_y): (i32, i32),
        (end_x, _end_y): (i32, i32),
    ) -> Option<Vec<WindowElement>> {
        let elements: Vec<WindowElement> = self
            .space
            .elements()
            .filter(|w| {
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, h) = w.geometry().size.into();
                x >= start_x && x + w <= end_x && (y + h - start_y).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { x, .. }) = self.space.element_location(w) else {
                    return false;
                };
                (x - start_x).abs() < 5
            })
            .is_some();
        let has_end_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { x, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, _) = w.geometry().size.into();
                (x + w - end_x).abs() < 5
            })
            .is_some();
        if !(has_start_pos && has_end_pos) {
            return None;
        }
        Some(elements)
    }

    fn find_down_element(
        &self,
        (start_x, _start_y): (i32, i32),
        (end_x, end_y): (i32, i32),
    ) -> Option<Vec<WindowElement>> {
        let elements: Vec<WindowElement> = self
            .space
            .elements()
            .filter(|w| {
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, _) = w.geometry().size.into();
                x >= start_x && x + w <= end_x && (y - end_y).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { x, .. }) = self.space.element_location(w) else {
                    return false;
                };
                (x - start_x).abs() < 5
            })
            .is_some();
        let has_end_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { x, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, _) = w.geometry().size.into();
                (x + w - end_x).abs() < 5
            })
            .is_some();
        if !(has_start_pos && has_end_pos) {
            return None;
        }
        Some(elements)
    }

    fn find_left_element(
        &self,
        (start_x, start_y): (i32, i32),
        (_end_x, end_y): (i32, i32),
    ) -> Option<Vec<WindowElement>> {
        let elements: Vec<WindowElement> = self
            .space
            .elements()
            .filter(|w| {
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, h) = w.geometry().size.into();
                y >= start_y && y + h <= end_y && (x + w - start_x).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                (y - start_y).abs() < 5
            })
            .is_some();
        let has_end_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (_, h) = w.geometry().size.into();
                (y + h - end_y).abs() < 5
            })
            .is_some();
        if !(has_start_pos && has_end_pos) {
            return None;
        }
        Some(elements)
    }

    fn find_right_element(
        &self,
        (_start_x, start_y): (i32, i32),
        (end_x, end_y): (i32, i32),
    ) -> Option<Vec<WindowElement>> {
        let elements: Vec<WindowElement> = self
            .space
            .elements()
            .filter(|w| {
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (_, h) = w.geometry().size.into();
                y >= start_y && y + h <= end_y && (x - end_x).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                (y - start_y).abs() < 5
            })
            .is_some();
        let has_end_pos = elements
            .iter()
            .find(|w| {
                let Some(Point { y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (_, h) = w.geometry().size.into();
                (y + h - end_y).abs() < 5
            })
            .is_some();
        if !(has_start_pos && has_end_pos) {
            return None;
        }
        Some(elements)
    }
}
