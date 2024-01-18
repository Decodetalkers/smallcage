use crate::{shell::WindowElement, state::ClientState, SmallCage};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_shm,
    reexports::{
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            protocol::{wl_buffer, wl_surface::WlSurface},
            Client,
        },
    },
    utils::{Logical, Point, SERIAL_COUNTER},
    wayland::{
        buffer::BufferHandler,
        compositor::{
            get_parent, is_sync_subsurface, CompositorClientState, CompositorHandler,
            CompositorState,
        },
        selection::data_device::set_data_device_focus,
        selection::primary_selection::set_primary_focus,
        shm::{ShmHandler, ShmState},
    },
};

impl CompositorHandler for SmallCage {
    fn compositor_state(&mut self) -> &mut CompositorState {
        &mut self.compositor_state
    }

    fn client_compositor_state<'a>(&self, client: &'a Client) -> &'a CompositorClientState {
        &client.get_data::<ClientState>().unwrap().compositor_state
    }

    fn commit(&mut self, surface: &WlSurface) {
        on_commit_buffer_handler::<Self>(surface);
        if !is_sync_subsurface(surface) {
            let mut root = surface.clone();
            while let Some(parent) = get_parent(&root) {
                root = parent;
            }
            if let Some(window) = self
                .space
                .elements()
                .find(|w| w.toplevel().wl_surface() == &root)
            {
                window.on_commit();
            }
        };

        self.handle_xdg_commit(surface);
        self.handle_popup_commit(surface);
        self.popups.commit(surface);
    }
}

impl BufferHandler for SmallCage {
    fn buffer_destroyed(&mut self, _buffer: &wl_buffer::WlBuffer) {}
}

impl ShmHandler for SmallCage {
    fn shm_state(&self) -> &ShmState {
        &self.shm_state
    }
}

delegate_compositor!(SmallCage);
delegate_shm!(SmallCage);

impl SmallCage {
    pub fn find_current_select_surface(&self) -> Option<(WindowElement, Point<i32, Logical>)> {
        self.surface_under_pointer(&self.pointer)
    }

    pub fn find_current_focus_window(&self) -> Option<&WindowElement> {
        self.space.elements().find(|w| {
            w.toplevel()
                .current_state()
                .states
                .contains(xdg_toplevel::State::Activated)
        })
    }

    pub fn handle_focus_change(&mut self) -> Option<()> {
        if let Some(window_focus) = self.find_current_focus_window() {
            if window_focus.is_untiled_window() {
                return None;
            }
        }
        let (window, _) = self.find_current_select_surface()?;
        if window.is_untiled_window() {
            return Some(());
        }
        let dh = &self.display_handle;
        let client = dh.get_client(window.id()).ok();
        set_data_device_focus(dh, &self.seat, client.clone());
        set_primary_focus(dh, &self.seat, client);
        let keyboard = self.seat.get_keyboard().unwrap();
        let serial = SERIAL_COUNTER.next_serial();

        self.space.raise_element(&window, true);
        self.space.elements().for_each(|window| {
            window.toplevel().send_pending_configure();
        });
        self.raise_untiled_elements();

        keyboard.set_focus(self, Some(window), serial);
        Some(())
    }
}
