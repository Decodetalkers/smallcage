use smithay::{
    backend::renderer::{
        element::{
            surface::WaylandSurfaceRenderElement,
            texture::{TextureBuffer, TextureRenderElement},
            utils::{CropRenderElement, RelocateRenderElement, RescaleRenderElement},
            AsRenderElements, Kind, RenderElement, Wrap,
        },
        ImportAll, ImportMem, Renderer, Texture,
    },
    desktop::space::SpaceRenderElements,
    input::pointer::CursorImageStatus,
    render_elements,
    utils::{Physical, Point, Scale},
};

use crate::{render::CustomRenderElements, shell::WindowRenderElement};

pub static CLEAR_COLOR: [f32; 4] = [0.8, 0.8, 0.9, 1.0];

#[allow(unused)]
pub static CLEAR_COLOR_FULLSCREEN: [f32; 4] = [0.0, 0.0, 0.0, 0.0];

pub struct PointerElement<T: Texture> {
    texture: Option<TextureBuffer<T>>,
    status: CursorImageStatus,
}

impl<T: Texture> Default for PointerElement<T> {
    fn default() -> Self {
        Self {
            texture: Default::default(),
            status: CursorImageStatus::default_named(),
        }
    }
}

#[allow(unused)]
impl<T: Texture> PointerElement<T> {
    pub fn set_status(&mut self, status: CursorImageStatus) {
        self.status = status;
    }

    pub fn set_texture(&mut self, texture: TextureBuffer<T>) {
        self.texture = Some(texture);
    }
}

render_elements! {
    pub PointerRenderElement<R> where
        R: ImportAll;
    Surface=WaylandSurfaceRenderElement<R>,
    Texture=TextureRenderElement<<R as Renderer>::TextureId>,
}

impl<R: Renderer> std::fmt::Debug for PointerRenderElement<R> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Surface(arg0) => f.debug_tuple("Surface").field(arg0).finish(),
            Self::Texture(arg0) => f.debug_tuple("Texture").field(arg0).finish(),
            Self::_GenericCatcher(arg0) => f.debug_tuple("_GenericCatcher").field(arg0).finish(),
        }
    }
}

impl<T: Texture + Clone + 'static, R> AsRenderElements<R> for PointerElement<T>
where
    R: Renderer<TextureId = T> + ImportAll,
{
    type RenderElement = PointerRenderElement<R>;
    fn render_elements<E>(
        &self,
        renderer: &mut R,
        location: Point<i32, Physical>,
        scale: Scale<f64>,
        alpha: f32,
    ) -> Vec<E>
    where
        E: From<PointerRenderElement<R>>,
    {
        match &self.status {
            CursorImageStatus::Hidden => vec![],
            // Always render `Default` for a named shape.
            CursorImageStatus::Named(_) => {
                if let Some(texture) = self.texture.as_ref() {
                    vec![PointerRenderElement::<R>::from(
                        TextureRenderElement::from_texture_buffer(
                            location.to_f64(),
                            texture,
                            None,
                            None,
                            None,
                            Kind::Cursor,
                        ),
                    )
                    .into()]
                } else {
                    vec![]
                }
            }
            CursorImageStatus::Surface(surface) => {
                let elements: Vec<PointerRenderElement<R>> =
                    smithay::backend::renderer::element::surface::render_elements_from_surface_tree(
                        renderer,
                        surface,
                        location,
                        scale,
                        alpha,
                        Kind::Cursor,
                    );
                elements.into_iter().map(E::from).collect()
            }
        }
    }
}

smithay::backend::renderer::element::render_elements! {
    pub OutputRenderElements<R, E> where R: ImportAll + ImportMem;
    Space=SpaceRenderElements<R, E>,
    Window=Wrap<E>,
    Custom=CustomRenderElements<R>,
    Preview=CropRenderElement<RelocateRenderElement<RescaleRenderElement<WindowRenderElement<R>>>>,
}

impl<R: Renderer + ImportAll + ImportMem, E: RenderElement<R> + std::fmt::Debug> std::fmt::Debug
    for OutputRenderElements<R, E>
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Space(arg0) => f.debug_tuple("Space").field(arg0).finish(),
            Self::Window(arg0) => f.debug_tuple("Window").field(arg0).finish(),
            Self::Custom(arg0) => f.debug_tuple("Custom").field(arg0).finish(),
            Self::Preview(arg0) => f.debug_tuple("Preview").field(arg0).finish(),
            Self::_GenericCatcher(arg0) => f.debug_tuple("_GenericCatcher").field(arg0).finish(),
        }
    }
}
