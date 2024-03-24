use smithay::{
    backend::{
        drm::{DrmDeviceFd, DrmNode, NodeType},
        egl::context::ContextPriority,
        input::{Device, InputEvent},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        renderer::{gles::GlesRenderer, multigpu::{gbm::GbmGlesBackend, GpuManager}},
        session::{libseat::LibSeatSession, Event as SessionEvent, Session},
        udev::{all_gpus, primary_gpu, UdevBackend},
    },
    reexports::{
        calloop::EventLoop,
        input::{DeviceCapability, Libinput},
        wayland_server::{Display, DisplayHandle},
    },
};

use crate::{
    state::{Backend, SmallCageState},
    CalloopData,
};

pub struct UdevData {
    pub session: LibSeatSession,
    dh: DisplayHandle,
    primary_gpu: DrmNode,
    //gpus: GpuManager<GbmGlesBackend<GlesRenderer, DrmDeviceFd>>,
}

impl Backend for UdevData {
    fn seat_name(&self) -> String {
        self.session.seat()
    }
}

pub fn run_udev() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop: EventLoop<'_, CalloopData<UdevData>> = EventLoop::try_new()?;
    let display: Display<SmallCageState<UdevData>> = Display::new()?;
    let mut display_handle = display.handle();

    let (session, notifier) = LibSeatSession::new()?;

    let primary_gpu = if let Ok(var) = std::env::var("ANVIL_DRM_DEVICE") {
        DrmNode::from_path(var).expect("Invalid drm device path")
    } else {
        primary_gpu(session.seat())
            .unwrap()
            .and_then(|x| {
                DrmNode::from_path(x)
                    .ok()?
                    .node_with_type(NodeType::Render)?
                    .ok()
            })
            .unwrap_or_else(|| {
                all_gpus(session.seat())
                    .unwrap()
                    .into_iter()
                    .find_map(|x| DrmNode::from_path(x).ok())
                    .expect("No GPU!")
            })
    };
    //let gpus =
    //    GpuManager::new(GbmGlesBackend::with_context_priority(ContextPriority::High)).unwrap();

    tracing::info!("Using {} as primary gpu.", primary_gpu);

    let data = UdevData {
        session,
        dh: display_handle.clone(),
        primary_gpu,
    };

    let mut state = SmallCageState::init(&mut event_loop, display, data);

    let udev_backend = UdevBackend::new(&state.seat_name())?;

    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        state.backend_data.session.clone().into(),
    );
    libinput_context
        .udev_assign_seat(&state.backend_data.seat_name())
        .unwrap();

    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    // NOTE: input listen
    event_loop
        .handle()
        .insert_source(libinput_backend, move |mut event, _, data| {
            //let dh = data.state.display_handle.clone();
            if let InputEvent::DeviceAdded { device } = &mut event {
                if device.has_capability(DeviceCapability::Keyboard) {}
            } else if let InputEvent::DeviceRemoved { ref device } = event {
                if device.has_capability(DeviceCapability::Keyboard) {}
            }
            data.state.process_input_event(event);
        })
        .unwrap();

    let handle = event_loop.handle();

    // NOTE: lession to session
    event_loop
        .handle()
        .insert_source(notifier, move |event, &mut (), data| match event {
            SessionEvent::PauseSession => {}
            SessionEvent::ActivateSession => {}
        })
        .unwrap();

    //state.shm_state.update_formats(state.backend_data.)

    Ok(())
}
