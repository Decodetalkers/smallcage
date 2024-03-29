use std::{sync::Mutex, time::Duration};

use smithay::{
    backend::{
        renderer::{
            damage::{Error as OutputDamageTrackerError, OutputDamageTracker},
            element::AsRenderElements,
            gles::{GlesRenderer, GlesTexture},
            ImportEgl,
        },
        winit::{self, WinitEvent},
    },
    input::pointer::{CursorImageAttributes, CursorImageStatus},
    output::{Mode, Output, PhysicalProperties, Subpixel},
    reexports::{calloop::EventLoop, wayland_server::Display},
    utils::{IsAlive, Rectangle, Scale, Transform},
    wayland::compositor,
};

use crate::{
    drawing::PointerElement,
    render::{render_output, CustomRenderElements},
    state::Backend,
    CalloopData, SmallCageState,
};

pub struct WinitData;

impl Backend for WinitData {
    fn seat_name(&self) -> String {
        "winit".to_owned()
    }
}

pub fn run_winit() -> Result<(), Box<dyn std::error::Error>> {
    let mut event_loop = EventLoop::try_new()?;

    let display = Display::new()?;
    let display_handle = display.handle();
    let state = SmallCageState::init(&mut event_loop, display, WinitData);

    let mut data = CalloopData {
        state,
        display_handle,
    };

    init_winit(&mut event_loop, &mut data)?;

    std::process::Command::new("kitty").spawn().ok();

    event_loop.run(
        Some(std::time::Duration::from_secs(1)),
        &mut data,
        move |w| {
            w.state.handle_focus_change();
        },
    )?;

    Ok(())
}

pub fn init_winit(
    event_loop: &mut EventLoop<CalloopData<WinitData>>,
    data: &mut CalloopData<WinitData>,
) -> Result<(), Box<dyn std::error::Error>> {
    let state = &mut data.state;

    let display_handle = &data.display_handle;

    let (mut backend, winit) = winit::init::<GlesRenderer>()?;

    let mode = Mode {
        size: backend.window_size(),
        refresh: 60_000,
    };

    let output = Output::new(
        "winit".to_string(),
        PhysicalProperties {
            size: (0, 0).into(),
            subpixel: Subpixel::Unknown,
            make: "Smithay".into(),
            model: "Winit".into(),
        },
    );
    let _global = output.create_global::<SmallCageState<WinitData>>(display_handle);
    output.change_current_state(
        Some(mode),
        Some(Transform::Flipped180),
        None,
        Some((0, 0).into()),
    );
    output.set_preferred(mode);

    if backend.renderer().bind_wl_display(display_handle).is_ok() {
        tracing::info!("EGL hardware-acceleration enabled");
    };

    state.space.map_output(&output, (0, 0));

    let mut damage_tracker = OutputDamageTracker::from_output(&output);

    std::env::set_var("WAYLAND_DISPLAY", &state.socket_name);

    event_loop
        .handle()
        .insert_source(winit, move |event, _, data| {
            let display = &mut data.display_handle;
            let state = &mut data.state;

            match event {
                WinitEvent::Resized { size, .. } => {
                    output.change_current_state(
                        Some(Mode {
                            size,
                            refresh: 60_000,
                        }),
                        None,
                        None,
                        None,
                    );
                    state.resize_elements(size);
                }
                WinitEvent::Input(event) => state.process_input_event(event),
                WinitEvent::Redraw => {
                    let mut cursor_guard = state.cursor_status.lock().unwrap();

                    let mut pointer_element = PointerElement::<GlesTexture>::default();

                    let mut reset = false;
                    if let CursorImageStatus::Surface(ref surface) = *cursor_guard {
                        reset = !surface.alive();
                    }
                    if reset {
                        *cursor_guard = CursorImageStatus::default_named();
                    }

                    let cursor_visible = !matches!(*cursor_guard, CursorImageStatus::Surface(_));

                    pointer_element.set_status(cursor_guard.clone());

                    let mut elements = Vec::<CustomRenderElements<GlesRenderer>>::new();
                    let scale = Scale::from(output.current_scale().fractional_scale());
                    let cursor_hotspot =
                        if let CursorImageStatus::Surface(ref surface) = *cursor_guard {
                            compositor::with_states(surface, |states| {
                                states
                                    .data_map
                                    .get::<Mutex<CursorImageAttributes>>()
                                    .unwrap()
                                    .lock()
                                    .unwrap()
                                    .hotspot
                            })
                        } else {
                            (0, 0).into()
                        };
                    let cursor_pos = state.pointer.current_location() - cursor_hotspot.to_f64();
                    let cursor_pos_scaled = cursor_pos.to_physical(scale).to_i32_round();
                    let renderer = backend.renderer();
                    elements.extend(pointer_element.render_elements(
                        renderer,
                        cursor_pos_scaled,
                        scale,
                        1.0,
                    ));

                    // TODO: handle result
                    render_output(
                        &output,
                        &state.space,
                        elements,
                        renderer,
                        &mut damage_tracker,
                        0,
                        false,
                    )
                    .map_err(|error| match error {
                        OutputDamageTrackerError::Rendering(err) => err,
                        _ => unreachable!(),
                    })
                    .unwrap();

                    let size = backend.window_size();
                    let damage = Rectangle::from_loc_and_size((0, 0), size);
                    backend.bind().unwrap();

                    backend.submit(Some(&[damage])).unwrap();

                    state.space.elements().for_each(|window| {
                        window.send_frame(
                            &output,
                            state.start_time.elapsed(),
                            Some(Duration::ZERO),
                            |_, _| Some(output.clone()),
                        )
                    });

                    backend.window().set_cursor_visible(cursor_visible);

                    state.space.refresh();
                    let _ = display.flush_clients();

                    // Ask for redraw to schedule new frame.
                    backend.window().request_redraw();
                }
                WinitEvent::CloseRequested => {
                    state.loop_signal.stop();
                }
                _ => (),
            };
        })?;

    Ok(())
}
