use smithay::{
    backend::renderer::{
        element::{
            solid::{SolidColorBuffer, SolidColorRenderElement},
            AsRenderElements, Kind,
        },
        Renderer,
    },
    input::Seat,
    utils::{Logical, Point, Serial},
};

use crate::{shell::WindowElement, state::SmallCage};

#[derive(Debug, Clone, Default)]
pub struct HeaderBar {
    pub pointer_loc: Option<Point<f64, Logical>>,
    pub width: u32,
    pub state_button_hover: bool,
    pub close_button_hover: bool,
    pub min_button_hover: bool,
    pub background: SolidColorBuffer,
    pub state_button: SolidColorBuffer,
    pub close_button: SolidColorBuffer,
    pub fullscreen_button: SolidColorBuffer,
}

const BG_COLOR: [f32; 4] = [0.75f32, 0.9f32, 0.78f32, 1f32];
const FULLSCREEN_COLOR: [f32; 4] = [1f32, 0.965f32, 0.71f32, 1f32];
const STATE_CHANGE_COLOR: [f32; 4] = [0.85f32, 0.665f32, 0.71f32, 1f32];
const CLOSE_COLOR: [f32; 4] = [1f32, 0.66f32, 0.612f32, 1f32];
const FULLSCREEN_COLOR_HOVER: [f32; 4] = [0.71f32, 0.424f32, 0f32, 1f32];
const STATE_CHANGE_COLOR_HOVER: [f32; 4] = [0.71f32, 0.624f32, 0f32, 1f32];
const CLOSE_COLOR_HOVER: [f32; 4] = [0.75f32, 0.11f32, 0.016f32, 1f32];

pub const HEADER_BAR_HEIGHT: i32 = 25;
const BUTTON_HEIGHT: u32 = HEADER_BAR_HEIGHT as u32;
const BUTTON_WIDTH: u32 = 25;

impl HeaderBar {
    pub fn pointer_enter(&mut self, loc: Point<f64, Logical>) {
        self.pointer_loc = Some(loc);
    }

    pub fn pointer_leave(&mut self) {
        self.pointer_loc = None;
    }

    pub fn clicked(
        &mut self,
        seat: &Seat<SmallCage>,
        state: &mut SmallCage,
        window: &WindowElement,
        serial: Serial,
    ) {
        match self.pointer_loc.as_ref() {
            Some(loc) if loc.x > (self.width - BUTTON_WIDTH) as f64 => {
                window.toplevel().send_close();
            }
            Some(loc) if loc.x <= BUTTON_WIDTH as f64 => {
                let window = window.clone();
                state.handle.insert_idle(move |data| {
                    data.state.handle_element_state_change(&window);
                });
            }
            Some(_) => {
                let seat = seat.clone();
                let toplevel = window.toplevel().clone();
                state
                    .handle
                    .insert_idle(move |data| data.state.move_request_xdg(&toplevel, seat, serial));
            }
            _ => {}
        }
    }

    pub fn redraw(&mut self, width: u32) {
        if width == 0 {
            self.width = 0;
            return;
        }

        self.background
            .update((width as i32, HEADER_BAR_HEIGHT), BG_COLOR);

        let mut needs_redraw_buttons = false;
        if width != self.width {
            needs_redraw_buttons = true;
            self.width = width;
        }

        if self
            .pointer_loc
            .as_ref()
            .map(|l| l.x <= BUTTON_WIDTH as f64)
            .unwrap_or(false)
            && (needs_redraw_buttons || !self.state_button_hover)
        {
            self.state_button.update(
                (BUTTON_WIDTH as i32, BUTTON_HEIGHT as i32),
                STATE_CHANGE_COLOR_HOVER,
            );
            self.state_button_hover = true;
        } else if !self
            .pointer_loc
            .as_ref()
            .map(|l| l.x <= BUTTON_WIDTH as f64)
            .unwrap_or(false)
            && (needs_redraw_buttons || self.state_button_hover)
        {
            self.state_button.update(
                (BUTTON_WIDTH as i32, BUTTON_HEIGHT as i32),
                STATE_CHANGE_COLOR,
            );
            self.state_button_hover = false;
        }

        if self
            .pointer_loc
            .as_ref()
            .map(|l| l.x >= (width - BUTTON_WIDTH) as f64)
            .unwrap_or(false)
            && (needs_redraw_buttons || !self.close_button_hover)
        {
            self.close_button.update(
                (BUTTON_WIDTH as i32, BUTTON_HEIGHT as i32),
                CLOSE_COLOR_HOVER,
            );
            self.close_button_hover = true;
        } else if !self
            .pointer_loc
            .as_ref()
            .map(|l| l.x >= (width - BUTTON_WIDTH) as f64)
            .unwrap_or(false)
            && (needs_redraw_buttons || self.close_button_hover)
        {
            self.close_button
                .update((BUTTON_WIDTH as i32, BUTTON_HEIGHT as i32), CLOSE_COLOR);
            self.close_button_hover = false;
        }

        if self
            .pointer_loc
            .as_ref()
            .map(|l| {
                l.x >= (width - BUTTON_WIDTH * 2) as f64 && l.x <= (width - BUTTON_WIDTH) as f64
            })
            .unwrap_or(false)
            && (needs_redraw_buttons || !self.min_button_hover)
        {
            self.fullscreen_button.update(
                (BUTTON_WIDTH as i32, BUTTON_HEIGHT as i32),
                FULLSCREEN_COLOR_HOVER,
            );
            self.min_button_hover = true;
        } else if !self
            .pointer_loc
            .as_ref()
            .map(|l| {
                l.x >= (width - BUTTON_WIDTH * 2) as f64 && l.x <= (width - BUTTON_WIDTH) as f64
            })
            .unwrap_or(false)
            && (needs_redraw_buttons || self.min_button_hover)
        {
            self.fullscreen_button.update(
                (BUTTON_WIDTH as i32, BUTTON_HEIGHT as i32),
                FULLSCREEN_COLOR,
            );
            self.min_button_hover = false;
        }
    }
}

impl<R: Renderer> AsRenderElements<R> for HeaderBar {
    type RenderElement = SolidColorRenderElement;

    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        _renderer: &mut R,
        location: Point<i32, smithay::utils::Physical>,
        scale: smithay::utils::Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        let header_end_offset: Point<i32, Logical> = Point::from((self.width as i32, 0));
        let button_offset: Point<i32, Logical> = Point::from((BUTTON_WIDTH as i32, 0));

        vec![
            SolidColorRenderElement::from_buffer(
                &self.state_button,
                location,
                scale,
                alpha,
                Kind::Unspecified,
            )
            .into(),
            SolidColorRenderElement::from_buffer(
                &self.close_button,
                location + (header_end_offset - button_offset).to_physical_precise_round(scale),
                scale,
                alpha,
                Kind::Unspecified,
            )
            .into(),
            SolidColorRenderElement::from_buffer(
                &self.fullscreen_button,
                location
                    + (header_end_offset - button_offset.upscale(2))
                        .to_physical_precise_round(scale),
                scale,
                alpha,
                Kind::Unspecified,
            )
            .into(),
            SolidColorRenderElement::from_buffer(
                &self.background,
                location,
                scale,
                alpha,
                Kind::Unspecified,
            )
            .into(),
        ]
    }
}
