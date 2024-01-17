use std::{ffi::OsString, sync::Arc};

use smithay::{
    delegate_input_method_manager, delegate_text_input_manager, delegate_virtual_keyboard_manager,
    delegate_xdg_activation, delegate_xdg_decoration,
    desktop::{space::SpaceElement, PopupKind, PopupManager, Space, WindowSurfaceType},
    input::{pointer::PointerHandle, Seat, SeatState},
    reexports::{
        calloop::{
            generic::Generic, EventLoop, Interest, LoopHandle, LoopSignal, Mode, PostAction,
        },
        wayland_protocols::xdg::decoration::{
            self as xdg_decoration,
            zv1::server::zxdg_toplevel_decoration_v1::Mode as DecorationMode,
        },
        wayland_protocols::xdg::shell::server::xdg_toplevel,
        wayland_server::{
            backend::{ClientData, ClientId, DisconnectReason},
            protocol::wl_surface::WlSurface,
            Display, DisplayHandle, Resource,
        },
    },
    utils::{Logical, Physical, Point, Rectangle, Size},
    wayland::{
        compositor::{with_states, CompositorClientState, CompositorState},
        input_method::{InputMethodHandler, InputMethodManagerState, PopupSurface},
        output::OutputManagerState,
        selection::{data_device::DataDeviceState, primary_selection::PrimarySelectionState},
        shell::xdg::{
            decoration::{XdgDecorationHandler, XdgDecorationState},
            ToplevelSurface, XdgShellState, XdgToplevelSurfaceData,
        },
        shm::ShmState,
        socket::ListeningSocketSource,
        text_input::TextInputManagerState,
        virtual_keyboard::VirtualKeyboardManagerState,
        xdg_activation::{XdgActivationHandler, XdgActivationState},
    },
};

use crate::shell::WindowElement;
use crate::CalloopData;

#[derive(Debug, Default, Clone, Copy)]
pub enum SplitState {
    #[default]
    HSplit,
    VSplit,
}

pub struct SmallCage {
    pub start_time: std::time::Instant,
    pub socket_name: OsString,

    pub display_handle: DisplayHandle,

    pub space: Space<WindowElement>,
    pub popups: PopupManager,
    pub loop_signal: LoopSignal,

    // Smithay State
    pub compositor_state: CompositorState,
    pub xdg_shell_state: XdgShellState,
    pub shm_state: ShmState,
    pub output_manager_state: OutputManagerState,
    pub seat_state: SeatState<SmallCage>,
    pub primary_selection_state: PrimarySelectionState,
    pub data_device_state: DataDeviceState,
    pub xdg_activation_state: XdgActivationState,
    pub xdg_decoration_state: XdgDecorationState,

    pub seat: Seat<Self>,
    pub pointer: PointerHandle<Self>,

    pub handle: LoopHandle<'static, CalloopData>,

    pub splitstate: SplitState,
}

impl SmallCage {
    pub fn new(event_loop: &mut EventLoop<'static, CalloopData>, display: Display<Self>) -> Self {
        let start_time = std::time::Instant::now();

        let dh = display.handle();

        let compositor_state = CompositorState::new::<Self>(&dh);
        let xdg_shell_state = XdgShellState::new::<Self>(&dh);
        let shm_state = ShmState::new::<Self>(&dh, vec![]);
        let output_manager_state = OutputManagerState::new_with_xdg_output::<Self>(&dh);
        let mut seat_state = SeatState::new();
        let primary_selection_state = PrimarySelectionState::new::<Self>(&dh);
        let data_device_state = DataDeviceState::new::<Self>(&dh);

        let xdg_activation_state = XdgActivationState::new::<Self>(&dh);
        let xdg_decoration_state = XdgDecorationState::new::<Self>(&dh);

        TextInputManagerState::new::<Self>(&dh);
        InputMethodManagerState::new::<Self, _>(&dh, |_| true);
        VirtualKeyboardManagerState::new::<Self, _>(&dh, |_| true);

        // A seat is a group of keyboards, pointer and touch devices.
        // A seat typically has a pointer and maintains a keyboard focus and a pointer focus.
        let mut seat: Seat<Self> = seat_state.new_wl_seat(&dh, "winit");

        // Notify clients that we have a keyboard, for the sake of the example we assume that keyboard is always present.
        // You may want to track keyboard hot-plug in real compositor.
        seat.add_keyboard(Default::default(), 200, 200).unwrap();

        // Notify clients that we have a pointer (mouse)
        // Here we assume that there is always pointer plugged in
        let pointer = seat.add_pointer();

        // A space represents a two-dimensional plane. Windows and Outputs can be mapped onto it.
        //
        // Windows get a position and stacking order through mapping.
        // Outputs become views of a part of the Space and can be rendered via Space::render_output.
        let space = Space::default();

        let socket_name = Self::init_wayland_listener(display, event_loop);

        // Get the loop signal, used to stop the event loop
        let loop_signal = event_loop.get_signal();

        Self {
            start_time,

            display_handle: dh,

            popups: PopupManager::default(),
            space,
            loop_signal,

            socket_name,

            compositor_state,
            xdg_shell_state,
            shm_state,
            output_manager_state,
            seat_state,
            primary_selection_state,
            data_device_state,
            xdg_activation_state,
            xdg_decoration_state,

            seat,
            pointer,

            handle: event_loop.handle(),

            splitstate: SplitState::default(),
        }
    }

    fn init_wayland_listener(
        display: Display<SmallCage>,
        event_loop: &mut EventLoop<CalloopData>,
    ) -> OsString {
        // Creates a new listening socket, automatically choosing the next available `wayland` socket name.
        let listening_socket = ListeningSocketSource::new_auto().unwrap();

        // Get the name of the listening socket.
        // Clients will connect to this socket.
        let socket_name = listening_socket.socket_name().to_os_string();

        let handle = event_loop.handle();

        event_loop
            .handle()
            .insert_source(listening_socket, move |client_stream, _, state| {
                // Inside the callback, you should insert the client into the display.
                //
                // You may also associate some data with the client when inserting the client.
                state
                    .display_handle
                    .insert_client(client_stream, Arc::new(ClientState::default()))
                    .unwrap();
            })
            .expect("Failed to init the wayland event source.");
        // You also need to add the display itself to the event loop, so that client events will be processed by wayland-server.
        handle
            .insert_source(
                Generic::new(display, Interest::READ, Mode::Level),
                |_, display, state| {
                    // Safety: we don't drop the display
                    unsafe {
                        display
                            .get_mut()
                            .dispatch_clients(&mut state.state)
                            .unwrap();
                    }
                    Ok(PostAction::Continue)
                },
            )
            .unwrap();
        // You also need to add the display itself to the event loop, so that client events will be processed by wayland-server.

        socket_name
    }

    pub fn surface_under_pointer(
        &self,
        pointer: &PointerHandle<Self>,
    ) -> Option<(WindowElement, Point<i32, Logical>)> {
        let pos = pointer.current_location();
        self.space
            .element_under(pos)
            .and_then(|(window, location)| {
                window
                    .surface_under(pos - location.to_f64(), WindowSurfaceType::TOPLEVEL)
                    .map(|(_, p)| (window.clone(), p + location))
            })
    }

    // FIXME: it is not good enough
    pub fn resize_elements(&mut self, after_size: Size<i32, Physical>) {
        let after_w = after_size.w;
        let after_h = after_size.h;
        let windows: Vec<WindowElement> = self.space.elements().cloned().collect();
        for winit in windows {
            if winit
                .toplevel()
                .current_state()
                .states
                .contains(xdg_toplevel::State::Fullscreen)
            {
                self.full_screen_commit(winit.toplevel().wl_surface());
                continue;
            }
            if winit.is_untiled_window() {
                continue;
            }
            let (origin_x, origin_y) = winit.origin_pos().into();
            let (out_w, out_h) = winit.output_size().into();
            let (w_w, w_h) = winit.element_size().into();
            let newsize: Size<i32, Logical> = (w_w * after_w / out_w, w_h * after_h / out_h).into();
            let newpoint: Point<i32, Logical> =
                (after_w * origin_x / out_w, after_h * origin_y / out_h).into();
            winit.toplevel().with_pending_state(|state| {
                state.size = Some(newsize);
            });
            winit.toplevel().send_configure();
            self.space.map_element(winit, newpoint, false);
        }
        self.raise_untiled_elements();
    }

    // this should commit when full is here
    // TODO: very basic
    fn full_screen_commit(&self, surface: &WlSurface) {
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
}
impl XdgActivationHandler for SmallCage {
    fn activation_state(&mut self) -> &mut XdgActivationState {
        &mut self.xdg_activation_state
    }
    fn request_activation(
        &mut self,
        _token: smithay::wayland::xdg_activation::XdgActivationToken,
        _token_data: smithay::wayland::xdg_activation::XdgActivationTokenData,
        _surface: WlSurface,
    ) {
    }
}

delegate_xdg_activation!(SmallCage);

#[derive(Default)]
pub struct ClientState {
    pub compositor_state: CompositorClientState,
}

impl ClientData for ClientState {
    fn initialized(&self, _client_id: ClientId) {}
    fn disconnected(&self, _client_id: ClientId, _reason: DisconnectReason) {}
}

delegate_text_input_manager!(SmallCage);
delegate_virtual_keyboard_manager!(SmallCage);
delegate_xdg_decoration!(SmallCage);
impl XdgDecorationHandler for SmallCage {
    fn new_decoration(&mut self, toplevel: ToplevelSurface) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        // Set the default to client side
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ClientSide);
        });
    }
    fn request_mode(&mut self, toplevel: ToplevelSurface, mode: DecorationMode) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;

        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(match mode {
                DecorationMode::ServerSide => Mode::ServerSide,
                _ => Mode::ClientSide,
            });
        });

        let initial_configure_sent = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if initial_configure_sent {
            toplevel.send_pending_configure();
        }
    }
    fn unset_mode(&mut self, toplevel: ToplevelSurface) {
        use xdg_decoration::zv1::server::zxdg_toplevel_decoration_v1::Mode;
        toplevel.with_pending_state(|state| {
            state.decoration_mode = Some(Mode::ClientSide);
        });
        let initial_configure_sent = with_states(toplevel.wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .initial_configure_sent
        });
        if initial_configure_sent {
            toplevel.send_pending_configure();
        }
    }
}

impl InputMethodHandler for SmallCage {
    fn new_popup(&mut self, surface: PopupSurface) {
        if let Err(err) = self.popups.track_popup(PopupKind::from(surface)) {
            tracing::warn!("Failed to track popup: {}", err);
        }
    }

    fn dismiss_popup(&mut self, surface: PopupSurface) {
        if let Some(parent) = surface.get_parent().map(|parent| parent.surface.clone()) {
            let _ = PopupManager::dismiss_popup(&parent, &PopupKind::from(surface));
        }
    }

    fn parent_geometry(&self, parent: &WlSurface) -> Rectangle<i32, smithay::utils::Logical> {
        self.space
            .elements()
            .find_map(|window| {
                (window.wl_surface().as_ref() == Some(parent)).then(|| window.geometry())
            })
            .unwrap_or_default()
    }
}
delegate_input_method_manager!(SmallCage);
