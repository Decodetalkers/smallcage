mod handlers;

mod grabs;
mod input;
mod shell;
mod state;
mod winit;

use smithay::reexports::{
    calloop::EventLoop,
    wayland_server::{Display, DisplayHandle},
};

use state::SmallCage;

pub struct CalloopData {
    state: SmallCage,
    display_handle: DisplayHandle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let mut event_loop: EventLoop<CalloopData> = EventLoop::try_new()?;

    let display: Display<SmallCage> = Display::new()?;
    let display_handle = display.handle();
    let state = SmallCage::new(&mut event_loop, display);

    let mut data = CalloopData {
        state,
        display_handle,
    };

    crate::winit::init_winit(&mut event_loop, &mut data)?;

    let mut args = std::env::args().skip(1);
    let flag = args.next();
    let arg = args.next();

    match (flag.as_deref(), arg) {
        (Some("-c") | Some("--command"), Some(command)) => {
            std::process::Command::new(command).spawn().ok();
        }
        _ => {
            std::process::Command::new("wezterm").spawn().ok();
        }
    }

    event_loop.run(
        Some(std::time::Duration::from_secs(1)),
        &mut data,
        move |w| {
            w.state.handle_focus_change();
        },
    )?;

    Ok(())
}
