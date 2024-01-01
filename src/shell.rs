use std::time::Duration;

use smithay::{
    backend::renderer::{
        element::{
            solid::SolidColorRenderElement, surface::WaylandSurfaceRenderElement, AsRenderElements,
        },
        ImportAll, ImportMem, Renderer,
    },
    desktop::{space::SpaceElement, Space, Window, WindowSurfaceType},
    output::Output,
    reexports::wayland_server::protocol::wl_surface,
    render_elements,
    utils::{IsAlive, Logical, Physical, Point, Rectangle, Scale, Size},
    wayland::{compositor::SurfaceData, seat::WaylandFocus, shell::xdg::ToplevelSurface},
};

#[derive(Debug, Clone)]
pub struct WindowElement {
    window: Window,
    pub is_init: bool,
    output_size: Size<i32, Logical>,
    element_size: Size<i32, Logical>,
    origin_pos: Point<i32, Logical>,
}

impl PartialEq for WindowElement {
    fn eq(&self, other: &Self) -> bool {
        self.window == other.window
    }
}

impl WindowElement {
    pub fn remap_element(&self, space: &mut Space<Self>) {
        let Some(position) = space.element_location(self) else {
            return;
        };
        space.map_element(self.clone(), position, true);
    }

    pub fn set_inited(&mut self) {
        self.is_init = true;
    }

    pub fn output_size(&self) -> Size<i32, Logical> {
        self.output_size
    }

    pub fn element_size(&self) -> Size<i32, Logical> {
        self.element_size
    }

    pub fn origin_pos(&self) -> Point<i32, Logical> {
        self.origin_pos
    }

    pub fn set_output_size(&mut self, size: Size<i32, Logical>) {
        self.output_size = size;
    }

    pub fn set_element_size(&mut self, size: Size<i32, Logical>) {
        self.element_size = size;
    }
    pub fn set_origin_pos(&mut self, point: Point<i32, Logical>) {
        self.origin_pos = point
    }
}

impl WindowElement {
    pub fn new(surface: ToplevelSurface) -> Self {
        WindowElement {
            window: Window::new(surface),
            is_init: false,
            output_size: Default::default(),
            element_size: Default::default(),
            origin_pos: Default::default()
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
        SpaceElement::geometry(&self.window)
    }

    fn bbox(&self) -> Rectangle<i32, smithay::utils::Logical> {
        SpaceElement::bbox(&self.window)
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
    R: Renderer + ImportAll,
    <R as Renderer>::TextureId: 'static,
{
    type RenderElement = WaylandSurfaceRenderElement<R>;

    #[profiling::function]
    fn render_elements<C: From<WaylandSurfaceRenderElement<R>>>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<C> {
        self.window
            .render_elements(renderer, location, scale, alpha)
    }
}
