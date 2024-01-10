use crate::{state::ClientState, SmallCage};
use smithay::{
    backend::renderer::utils::on_commit_buffer_handler,
    delegate_compositor, delegate_shm,
    reexports::wayland_server::{
        protocol::{wl_buffer, wl_surface::WlSurface},
        Client, Resource,
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

#[allow(unused)]
impl SmallCage {
    pub fn find_current_select_surface(&self) -> Option<(WlSurface, Point<i32, Logical>)> {
        self.surface_under_pointer(&self.pointer)
    }
    pub fn handle_focus_change(&mut self) -> Option<()> {
        let (surface, _) = self.find_current_select_surface()?;
        let dh = &self.display_handle;
        let client = dh.get_client(surface.id()).ok();
        set_data_device_focus(dh, &self.seat, client.clone());
        set_primary_focus(dh, &self.seat, client);
        let keyboard = self.seat.get_keyboard().unwrap();
        let serial = SERIAL_COUNTER.next_serial();
        keyboard.set_focus(self, Some(surface), serial);
        self.raise_untiled_elements();
        Some(())
    }
}
