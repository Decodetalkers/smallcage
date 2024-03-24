mod handlers;

mod drawing;
mod grabs;
mod input;
mod render;
mod shell;
mod state;
mod udev;
mod winit;

static POSSIBLE_BACKENDS: &[&str] = &[
    "--winit : Run anvil as a X11 or Wayland client using winit.",
    "--tty-udev : Run anvil as a tty udev client (requires root if without logind).",
    "--x11 : Run anvil as an X11 client.",
];

use smithay::reexports::wayland_server::DisplayHandle;

use state::{Backend, SmallCageState};

pub struct CalloopData<BackendData: Backend + 'static> {
    state: SmallCageState<BackendData>,
    display_handle: DisplayHandle,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    if let Ok(env_filter) = tracing_subscriber::EnvFilter::try_from_default_env() {
        tracing_subscriber::fmt().with_env_filter(env_filter).init();
    } else {
        tracing_subscriber::fmt().init();
    }

    let arg = ::std::env::args().nth(1);
    match arg.as_ref().map(|s| &s[..]) {
        Some("--winit") => {
            tracing::info!("Start with winit backend");
            winit::run_winit()?;
        }
        Some(other) => {
            tracing::error!("Unknown backend: {}", other);
        }
        None => {
            #[allow(clippy::disallowed_macros)]
            {
                println!("USAGE: anvil --backend");
                println!();
                println!("Possible backends are:");
                for b in POSSIBLE_BACKENDS {
                    println!("\t{}", b);
                }
            }
        }
    }

    Ok(())
}
