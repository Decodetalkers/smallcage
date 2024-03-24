use smithay::{
    backend::{
        drm::{DrmNode, NodeType},
        libinput::{LibinputInputBackend, LibinputSessionInterface},
        session::{libseat::LibSeatSession, Session},
        udev::{all_gpus, primary_gpu, UdevBackend},
    },
    reexports::{calloop::EventLoop, input::Libinput, wayland_server::Display},
};

use crate::{
    state::{Backend, SmallCageState},
    CalloopData,
};

pub struct UdevData {
    pub session: LibSeatSession,
    primary_gpu: DrmNode,
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
    tracing::info!("Using {} as primary gpu.", primary_gpu);

    let data = UdevData {
        session,
        primary_gpu,
    };

    let mut state = SmallCageState::init(&mut event_loop, display, data);

    let udev_backend = UdevBackend::new(&state.backend_data.seat_name())?;

    let mut libinput_context = Libinput::new_with_udev::<LibinputSessionInterface<LibSeatSession>>(
        state.backend_data.session.clone().into(),
    );
    libinput_context
        .udev_assign_seat(&state.backend_data.seat_name())
        .unwrap();

    let libinput_backend = LibinputInputBackend::new(libinput_context.clone());

    Ok(())
}
