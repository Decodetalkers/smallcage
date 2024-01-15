use smithay::{
    delegate_xdg_shell,
    desktop::{space::SpaceElement, PopupKind},
    reexports::{
        wayland_protocols::xdg::{decoration as xdg_decoration, shell::server::xdg_toplevel},
        wayland_server::protocol::{wl_seat, wl_surface::WlSurface},
    },
    utils::{Logical, Point, Rectangle, Serial, Size},
    wayland::{
        compositor::with_states,
        shell::xdg::{
            Configure, PopupSurface, PositionerState, SurfaceCachedState, ToplevelSurface,
            XdgPopupSurfaceData, XdgShellHandler, XdgShellState, XdgToplevelSurfaceData,
        },
    },
};

use crate::{
    shell::{ElementState, WindowElement},
    state::SplitState,
    SmallCage,
};

use super::HEADER_BAR_HEIGHT;

impl XdgShellHandler for SmallCage {
    fn xdg_shell_state(&mut self) -> &mut XdgShellState {
        &mut self.xdg_shell_state
    }

    fn new_toplevel(&mut self, surface: ToplevelSurface) {
        let window = WindowElement::new(surface);
        self.space.map_element(window, (0, 0), false);
    }

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

    #[allow(unused)]
    fn resize_request(
        &mut self,
        surface: ToplevelSurface,
        seat: wl_seat::WlSeat,
        serial: Serial,
        edges: xdg_toplevel::ResizeEdge,
    ) {
        // TODO:
    }
    fn reposition_request(
        &mut self,
        _surface: PopupSurface,
        _positioner: PositionerState,
        _token: u32,
    ) {
        // TODO
    }

    fn ack_configure(&mut self, surface: WlSurface, configure: Configure) {
        let Configure::Toplevel(configure) = configure else {
            return;
        };
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;

        let Some(window) = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == &surface)
        else {
            return;
        };
        let is_ssd = configure
            .state
            .decoration_mode
            .map(|mode| mode == Mode::ServerSide)
            .unwrap_or(false);
        window.set_ssd(is_ssd);
    }
}

// Xdg Shell
delegate_xdg_shell!(SmallCage);

/// Should be called on `WlSurface::commit`
impl SmallCage {
    pub fn handle_xdg_commit(&mut self, surface: &WlSurface) -> Option<()> {
        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)?;

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

        let max_size = with_states(surface, |states| {
            states.cached_state.pending::<SurfaceCachedState>().max_size
        });

        let min_size = with_states(surface, |states| {
            states.cached_state.pending::<SurfaceCachedState>().min_size
        });

        let is_fixed_size = (max_size == min_size) && max_size != (0, 0).into();

        if window.has_pedding_size() {
            window.set_pedding_size(None);
        }

        if !initial_configure_sent {
            if is_fixed_size {
                window.set_is_fixed_window();
            }
            window.toplevel().send_configure();
        } else if isconfigured {
            if !window.is_init() {
                if window.is_fixed_window() {
                    self.map_untitled_element(surface);
                } else {
                    self.resize_element_commit(surface);
                }
            } else {
                let need_state_change = window.need_state_change();
                if need_state_change {
                    let current_window_state = window.current_window_state().clone();
                    match current_window_state {
                        ElementState::TileToUnTile => {
                            window.change_state();
                            self.handle_dead_window(&(window.clone()));
                            self.map_untitled_element(surface);
                        }
                        ElementState::UnTileToTile => {
                            window.change_state();
                            self.resize_element_commit(surface);
                        }
                        _ => {}
                    }
                }
            }
        }
        self.raise_untiled_elements();

        Some(())
    }

    pub fn raise_untiled_elements(&mut self) {
        let mut elements: Vec<WindowElement> = self
            .space
            .elements()
            .filter(|w| w.is_untiled_window())
            .cloned()
            .collect();
        elements.sort_by(|a, b| a.z_index().partial_cmp(&b.z_index()).unwrap());
        for el in elements {
            self.space.raise_element(&el, true);
        }
    }

    fn resize_element_commit(&mut self, surface: &WlSurface) -> Option<()> {
        match self.current_active_window_rectangle(surface) {
            Some(element) => self.map_with_split(surface, element),
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
    fn map_untitled_element(&mut self, surface: &WlSurface) -> Option<()> {
        let mut window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()?;
        let current_screen = self.current_screen_rectangle()?;
        let max_size = window.to_untile_property_size();
        let mut screen_size = current_screen.size;
        if window.window_state().is_ssd {
            screen_size.h += HEADER_BAR_HEIGHT;
        }
        let (x, y) = (
            (screen_size.w - max_size.w) / 2,
            (screen_size.h - max_size.h) / 2,
        );
        window.set_inited();
        self.space.map_element(window, (x, y), true);
        Some(())
    }

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
        let mut current_screen_size = current_screen.size;
        if window.window_state().is_ssd {
            current_screen_size.h -= HEADER_BAR_HEIGHT;
        }
        let mut fin_window = window.clone();
        fin_window.set_inited();
        fin_window.toplevel().send_configure();
        fin_window.set_output_size(current_screen_size);
        fin_window.set_element_size(current_screen_size);
        fin_window.set_origin_pos(loc);
        self.space.map_element(fin_window, loc, true);

        Some(())
    }

    fn map_with_split(&mut self, surface: &WlSurface, windowpre: WindowElement) -> Option<()> {
        let current_screen = self.current_screen_rectangle()?;
        let (x, y) = self.space.element_location(&windowpre)?.into();
        let (w, h) = windowpre.window_size().into();

        let (point, mut size): (Point<i32, Logical>, Size<i32, Logical>) = match self.splitstate {
            SplitState::HSplit => {
                let width = w / 2;
                let height = h;
                ((x + width, y).into(), (width, height).into())
            }
            SplitState::VSplit => {
                let width = w;
                let height = h / 2;
                ((x, y + height).into(), (width, height).into())
            }
        };

        let mut afterwindowsize = size;

        let window = self
            .space
            .elements()
            .find(|w| w.toplevel().wl_surface() == surface)
            .cloned()?;
        if window.window_state().is_ssd {
            afterwindowsize.h -= HEADER_BAR_HEIGHT;
        }

        window.toplevel().with_pending_state(|state| {
            state.size = Some(afterwindowsize);
        });
        window.toplevel().send_configure();

        let mut fin_window = window.clone();
        fin_window.set_inited();
        fin_window.set_element_size(afterwindowsize);
        fin_window.set_output_size(current_screen.size);
        fin_window.set_origin_pos(point);
        self.space.map_element(fin_window, point, false);

        if windowpre.window_state().is_ssd {
            size.h -= HEADER_BAR_HEIGHT;
        }

        windowpre.toplevel().with_pending_state(|state| {
            state.size = Some(size);
        });
        windowpre.toplevel().send_configure();
        windowpre.set_output_size(current_screen.size);
        windowpre.set_element_size(size);

        Some(())
    }

    #[allow(unused)]
    fn find_current_selected_element(&self, surface: &WlSurface) -> Option<&WindowElement> {
        let point = self.pointer.current_location();
        self.space
            .elements()
            .filter(|e| e.bbox().to_f64().contains(point))
            .find(|w| w.toplevel().wl_surface() != surface)
    }

    fn find_current_focused_element(&self, surface: &WlSurface) -> Option<&WindowElement> {
        self.space.elements().find(|w| {
            w.toplevel()
                .current_state()
                .states
                .contains(xdg_toplevel::State::Activated)
                && w.toplevel().wl_surface() != surface
                && !w.is_untiled_window()
        })
    }

    fn current_active_window_rectangle(&self, surface: &WlSurface) -> Option<WindowElement> {
        match self.find_current_focused_element(surface) {
            None => self
                .space
                .elements()
                .filter(|w| !w.is_untiled_window() && w.toplevel().wl_surface() != surface)
                .last()
                .cloned(),
            value => value.cloned(),
        }
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
            return;
        };
        let (x, y) = pos.into();
        let (w, h) = window.get_pedding_size().into();
        let (rb_x, rb_y) = (x + w, y + h);
        if let Some(mut elements) = self.find_up_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(element) else {
                    continue;
                };
                let (ow, oh) = element.get_pedding_size().into();
                let size: Size<i32, Logical> = (ow, oh + h).into();
                let mut newsize = size;
                if element.window_state().is_ssd {
                    newsize.h -= HEADER_BAR_HEIGHT;
                }
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_pedding_size(Some(size));
                element.set_origin_pos(ori_pos);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
            }
            return;
        }
        if let Some(mut elements) = self.find_down_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(element) else {
                    continue;
                };
                let (o_x, _) = ori_pos.into();
                let (ow, oh) = element.get_pedding_size().into();
                let size: Size<i32, Logical> = (ow, oh + h).into();
                let mut newsize = size;
                if element.window_state().is_ssd {
                    newsize.h -= HEADER_BAR_HEIGHT;
                }
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_pedding_size(Some(size));
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
                self.space.map_element(element.clone(), (o_x, y), true);
            }
            self.raise_untiled_elements();
            return;
        }
        if let Some(mut elements) = self.find_left_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(element) else {
                    continue;
                };
                let (ow, oh) = element.get_pedding_size().into();
                let size: Size<i32, Logical> = (ow + w, oh).into();
                let mut newsize = size;
                if element.window_state().is_ssd {
                    newsize.h -= HEADER_BAR_HEIGHT;
                }
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_pedding_size(Some(size));
                element.set_origin_pos(ori_pos);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
            }
            return;
        }
        if let Some(mut elements) = self.find_right_element((x, y), (rb_x, rb_y)) {
            for element in elements.iter_mut() {
                let Some(ori_pos) = self.space.element_location(element) else {
                    continue;
                };
                let (_, o_y) = ori_pos.into();
                let (ow, oh) = element.get_pedding_size().into();
                let size: Size<i32, Logical> = (ow + w, oh).into();
                let mut newsize = size;
                if element.window_state().is_ssd {
                    newsize.h -= HEADER_BAR_HEIGHT;
                }
                element.set_output_size(screen_size);
                element.set_element_size(newsize);
                element.set_pedding_size(Some(size));
                element.set_origin_pos(ori_pos);
                element.toplevel().with_pending_state(|state| {
                    state.size = Some(newsize);
                });
                element.toplevel().send_configure();
                self.space.map_element(element.clone(), (x, o_y), true);
            }
            self.raise_untiled_elements();
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
                if w.is_untiled_window() {
                    return false;
                }
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, h) = w.get_pedding_size().into();
                x >= start_x - 5 && x + w <= end_x + 5 && (y + h - start_y).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements.iter().any(|w| {
            let Some(Point { x, .. }) = self.space.element_location(w) else {
                return false;
            };
            (x - start_x).abs() < 5
        });
        let has_end_pos = elements.iter().any(|w| {
            let Some(Point { x, .. }) = self.space.element_location(w) else {
                return false;
            };
            let (w, _) = w.get_pedding_size().into();
            (x + w - end_x).abs() < 5
        });
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
                if w.is_untiled_window() {
                    return false;
                }
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, _) = w.get_pedding_size().into();
                x >= start_x - 5 && x + w <= end_x + 5 && (y - end_y).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements.iter().any(|w| {
            let Some(Point { x, .. }) = self.space.element_location(w) else {
                return false;
            };
            (x - start_x).abs() < 5
        });
        let has_end_pos = elements.iter().any(|w| {
            let Some(Point { x, .. }) = self.space.element_location(w) else {
                return false;
            };
            let (w, _) = w.get_pedding_size().into();
            (x + w - end_x).abs() < 5
        });
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
                if w.is_untiled_window() {
                    return false;
                }
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (w, h) = w.get_pedding_size().into();
                y >= start_y - 5 && y + h <= end_y + 5 && (x + w - start_x).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements.iter().any(|w| {
            let Some(Point { y, .. }) = self.space.element_location(w) else {
                return false;
            };
            (y - start_y).abs() < 5
        });
        let has_end_pos = elements.iter().any(|w| {
            let Some(Point { y, .. }) = self.space.element_location(w) else {
                return false;
            };
            let (_, h) = w.get_pedding_size().into();
            (y + h - end_y).abs() < 5
        });
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
                if w.is_untiled_window() {
                    return false;
                }
                let Some(Point { x, y, .. }) = self.space.element_location(w) else {
                    return false;
                };
                let (_, h) = w.get_pedding_size().into();
                y >= start_y - 5 && y + h <= end_y + 5 && (x - end_x).abs() < 5
            })
            .cloned()
            .collect();
        let has_start_pos = elements.iter().any(|w| {
            let Some(Point { y, .. }) = self.space.element_location(w) else {
                return false;
            };
            (y - start_y).abs() < 5
        });
        let has_end_pos = elements.iter().any(|w| {
            let Some(Point { y, .. }) = self.space.element_location(w) else {
                return false;
            };
            let (_, h) = w.get_pedding_size().into();
            (y + h - end_y).abs() < 5
        });
        if !(has_start_pos && has_end_pos) {
            return None;
        }
        Some(elements)
    }
}
