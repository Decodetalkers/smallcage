mod elementkeyboard;
mod elementpoint;

use std::{
    cell::{Ref, RefCell, RefMut},
    time::Duration,
};

use smithay::{
    backend::renderer::{
        element::{
            solid::SolidColorRenderElement, surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        ImportAll, ImportMem, Renderer,
    },
    desktop::{space::SpaceElement, Window, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::{backend::ObjectId, protocol::wl_surface, Resource},
    render_elements,
    utils::{user_data::UserDataMap, IsAlive, Logical, Physical, Point, Rectangle, Scale, Size},
    wayland::{
        compositor::{with_states, SurfaceData},
        seat::WaylandFocus,
        shell::xdg::{SurfaceCachedState, ToplevelSurface, XdgToplevelSurfaceData},
    },
};

use crate::handlers::{HeaderBar, HEADER_BAR_HEIGHT};

#[derive(Debug, Default, Clone)]
pub enum ElementState {
    #[default]
    Tile,
    TileToUnTile,
    Untile,
    UnTileToTile,
}

impl ElementState {
    fn is_change_state(&self) -> bool {
        matches!(
            self,
            ElementState::TileToUnTile | ElementState::UnTileToTile
        )
    }

    fn is_untiled_state(&self) -> bool {
        matches!(self, ElementState::Untile)
    }

    fn change_state(&mut self) {
        match self {
            ElementState::UnTileToTile => *self = ElementState::Tile,
            ElementState::TileToUnTile => *self = ElementState::Untile,
            ElementState::Untile => *self = ElementState::UnTileToTile,
            ElementState::Tile => *self = ElementState::TileToUnTile,
        }
    }
}

#[derive(Debug, Default, Clone, Copy)]
pub enum SsdResizeState {
    #[default]
    Nothing,
    Top,
    Bottom,
    Left,
    Right,
}

#[derive(Debug, Default, Clone)]
pub struct WindowState {
    pub is_ssd: bool,
    pub ptr_entered_window: bool,
    pub is_fixed_window: bool,
    pub element_state: ElementState,
    pub output_size: Size<i32, Logical>,
    pub element_size: Size<i32, Logical>,
    pub origin_pos: Point<i32, Logical>,
    pub pedding_size: Option<Size<i32, Logical>>,
    pub header_bar: HeaderBar,
    pub ssd_resize_state: SsdResizeState,
}

#[derive(Debug, Clone)]
pub struct WindowElement {
    window: Window,
    is_init: bool,
}

impl PartialEq for WindowElement {
    fn eq(&self, other: &Self) -> bool {
        self.window == other.window
    }
}

impl WindowElement {
    pub fn id(&self) -> ObjectId {
        self.window.toplevel().wl_surface().id()
    }
    pub fn is_init(&self) -> bool {
        self.is_init
    }

    pub fn to_untile_property_size(&self) -> Size<i32, Logical> {
        if self.is_fixed_window() {
            return self.max_size();
        }
        self.geometry().size
    }

    pub fn set_inited(&mut self) {
        self.is_init = true;
    }

    #[allow(unused)]
    pub fn max_size(&self) -> Size<i32, Logical> {
        with_states(self.toplevel().wl_surface(), |states| {
            states.cached_state.pending::<SurfaceCachedState>().max_size
        })
    }

    #[allow(unused)]
    pub fn min_size(&self) -> Size<i32, Logical> {
        with_states(self.toplevel().wl_surface(), |states| {
            states.cached_state.pending::<SurfaceCachedState>().min_size
        })
    }

    #[allow(unused)]
    pub fn title(&self) -> Option<String> {
        with_states(self.toplevel().wl_surface(), |states| {
            states
                .data_map
                .get::<XdgToplevelSurfaceData>()
                .unwrap()
                .lock()
                .unwrap()
                .title
                .clone()
        })
    }

    pub fn user_data(&self) -> &UserDataMap {
        self.window.user_data()
    }
}

impl WindowElement {
    pub fn window_state(&self) -> Ref<'_, WindowState> {
        self.user_data()
            .insert_if_missing(|| RefCell::new(WindowState::default()));
        self.user_data()
            .get::<RefCell<WindowState>>()
            .unwrap()
            .borrow()
    }

    fn window_state_mut(&self) -> RefMut<'_, WindowState> {
        self.user_data()
            .insert_if_missing(|| RefCell::new(WindowState::default()));
        self.user_data()
            .get::<RefCell<WindowState>>()
            .unwrap()
            .borrow_mut()
    }

    pub fn is_ssd(&self) -> bool {
        self.window_state().is_ssd
    }

    pub fn set_ssd(&self, ssd: bool) {
        self.window_state_mut().is_ssd = ssd
    }

    pub fn is_untiled_window(&self) -> bool {
        self.window_state().element_state.is_untiled_state()
    }

    pub fn need_state_change(&self) -> bool {
        self.window_state().element_state.is_change_state()
    }

    pub fn change_state(&self) {
        if self.window_state().is_fixed_window {
            return;
        }
        self.window_state_mut().element_state.change_state();
    }

    pub fn current_window_state(&self) -> ElementState {
        self.window_state().element_state.clone()
    }

    pub fn is_fixed_window(&self) -> bool {
        self.window_state().is_fixed_window
    }

    pub fn set_is_fixed_window(&self) {
        self.window_state_mut().is_fixed_window = true;
        self.window_state_mut().element_state = ElementState::Untile;
    }

    pub fn output_size(&self) -> Size<i32, Logical> {
        self.window_state().output_size
    }

    pub fn element_size(&self) -> Size<i32, Logical> {
        self.window_state().element_size
    }

    pub fn origin_pos(&self) -> Point<i32, Logical> {
        self.window_state().origin_pos
    }

    pub fn set_output_size(&self, size: Size<i32, Logical>) {
        self.window_state_mut().output_size = size;
    }

    pub fn set_element_size(&self, size: Size<i32, Logical>) {
        self.window_state_mut().element_size = size;
    }

    pub fn has_pedding_size(&self) -> bool {
        self.window_state_mut().pedding_size.is_some()
    }

    pub fn set_pedding_size(&self, pedding_size: Option<Size<i32, Logical>>) {
        self.window_state_mut().pedding_size = pedding_size;
    }

    pub fn get_pedding_size(&self) -> Size<i32, Logical> {
        self.window_state()
            .pedding_size
            .unwrap_or(self.geometry().size)
    }

    pub fn set_origin_pos(&self, point: Point<i32, Logical>) {
        self.window_state_mut().origin_pos = point
    }

    #[allow(unused)]
    pub fn get_ssd_resize_state(&self) -> SsdResizeState {
        self.window_state().ssd_resize_state
    }

    #[allow(unused)]
    pub fn set_ssd_resize_state(&self, state: SsdResizeState) {
        self.window_state_mut().ssd_resize_state = state;
    }
}

impl WindowElement {
    pub fn new(surface: ToplevelSurface) -> Self {
        WindowElement {
            window: Window::new(surface),
            is_init: false,
        }
    }

    pub fn toplevel(&self) -> &ToplevelSurface {
        self.window.toplevel()
    }

    pub fn surface_under<P>(
        &self,
        point: P,
        surface_type: WindowSurfaceType,
    ) -> Option<(wl_surface::WlSurface, Point<i32, Logical>)>
    where
        P: Into<Point<f64, Logical>>,
    {
        self.window.surface_under(point, surface_type)
    }

    pub fn on_commit(&self) {
        self.window.on_commit()
    }

    pub fn set_activated(&self, active: bool) -> bool {
        self.window.set_activated(active)
    }

    pub fn send_frame<T, F>(
        &self,
        output: &Output,
        time: T,
        throttle: Option<Duration>,
        primary_scan_out_output: F,
    ) where
        T: Into<Duration>,
        F: FnMut(&wl_surface::WlSurface, &SurfaceData) -> Option<Output> + Copy,
    {
        self.window
            .send_frame(output, time, throttle, primary_scan_out_output)
    }
    #[allow(unused)]
    pub fn wl_surface(&self) -> Option<wl_surface::WlSurface> {
        self.window.wl_surface()
    }
}

impl IsAlive for WindowElement {
    fn alive(&self) -> bool {
        self.window.alive()
    }
}

impl SpaceElement for WindowElement {
    fn geometry(&self) -> Rectangle<i32, smithay::utils::Logical> {
        let mut geo = SpaceElement::geometry(&self.window);
        if self.window_state().is_ssd {
            geo.size.h += HEADER_BAR_HEIGHT;
        }
        geo
    }

    fn bbox(&self) -> Rectangle<i32, smithay::utils::Logical> {
        let mut bbox = SpaceElement::bbox(&self.window);
        if self.is_ssd() {
            bbox.size.h += HEADER_BAR_HEIGHT;
        }
        bbox
    }

    fn is_in_input_region(&self, point: &Point<f64, smithay::utils::Logical>) -> bool {
        SpaceElement::is_in_input_region(&self.window, point)
    }

    fn z_index(&self) -> u8 {
        SpaceElement::z_index(&self.window)
    }

    fn set_activate(&self, activated: bool) {
        self.window.set_activate(activated)
    }

    fn output_enter(&self, output: &Output, overlap: Rectangle<i32, smithay::utils::Logical>) {
        SpaceElement::output_enter(&self.window, output, overlap)
    }

    fn output_leave(&self, output: &Output) {
        SpaceElement::output_leave(&self.window, output)
    }

    fn refresh(&self) {
        SpaceElement::refresh(&self.window)
    }
}

impl WaylandFocus for WindowElement {
    fn wl_surface(&self) -> Option<wl_surface::WlSurface> {
        Some(self.toplevel().wl_surface().clone())
    }
}

render_elements!(
    pub WindowRenderElement<R> where R: ImportAll + ImportMem;
    Window=WaylandSurfaceRenderElement<R>,
    Decoration=SolidColorRenderElement,
);

impl<R: Renderer> std::fmt::Debug for WindowRenderElement<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Window(arg0) => f.debug_tuple("Window").field(arg0).finish(),
            Self::Decoration(arg0) => f.debug_tuple("Decoration").field(arg0).finish(),
            Self::_GenericCatcher(arg0) => f.debug_tuple("_GenericCatcher").field(arg0).finish(),
        }
    }
}

impl<R> AsRenderElements<R> for WindowElement
where
    R: Renderer + ImportAll + ImportMem,
    <R as Renderer>::TextureId: 'static,
{
    type RenderElement = WindowRenderElement<R>;

    #[profiling::function]
    fn render_elements<C: From<Self::RenderElement>>(
        &self,
        renderer: &mut R,
        mut location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        let window_bbox = self.window.bbox();
        if !self.window_state().is_ssd || window_bbox.is_empty() {
            return self
                .window
                .render_elements(renderer, location, scale, alpha)
                .into_iter()
                .map(C::from)
                .collect();
        }
        let window_geo = self.window.geometry();
        let mut state = self.window_state_mut();
        let width = window_geo.size.w;
        state.header_bar.redraw(width as u32);
        let mut vec = AsRenderElements::<R>::render_elements::<WindowRenderElement<R>>(
            &state.header_bar,
            renderer,
            location,
            scale,
            alpha,
        );

        location.y += (scale.y * HEADER_BAR_HEIGHT as f64) as i32;

        let window_elements = AsRenderElements::<R>::render_elements::<WindowRenderElement<R>>(
            &self.window,
            renderer,
            location,
            scale,
            alpha,
        );
        vec.extend(window_elements);
        vec.into_iter().map(C::from).collect()
    }
}
