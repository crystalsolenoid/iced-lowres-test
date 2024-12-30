use iced::advanced::renderer::{self, Quad};
use iced::advanced::{text, Text};
use iced::{Background, Color, Pixels, Point, Rectangle, Transformation};
use iced_wgpu::graphics::Compositor;

pub struct LowRes {}

impl renderer::Renderer for LowRes {
    fn start_layer(&mut self, bounds: Rectangle) {}
    fn end_layer(&mut self) {}
    fn start_transformation(&mut self, transformation: Transformation) {}
    fn end_transformation(&mut self) {}
    fn fill_quad(&mut self, quad: Quad, background: impl Into<Background>) {}
    fn clear(&mut self) {}
}

impl text::Renderer for LowRes {
    type Font = iced::Font;
    //    type Paragraph = dyn iced::advanced::text::Paragraph<Font = Self::Font>;
    type Paragraph = ();
    type Editor = ();
    //    type Editor = dyn iced::advanced::text::Editor;

    const ICON_FONT: Self::Font = iced::Font::DEFAULT;
    const CHECKMARK_ICON: char = '0';
    const ARROW_DOWN_ICON: char = '0';

    fn default_font(&self) -> Self::Font {
        todo!();
    }
    fn default_size(&self) -> Pixels {
        todo!();
    }
    fn fill_paragraph(
        &mut self,
        text: &Self::Paragraph,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
    }
    fn fill_editor(
        &mut self,
        editor: &Self::Editor,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
    }
    fn fill_text(
        &mut self,
        text: Text<String, Self::Font>,
        position: Point,
        color: Color,
        clip_bounds: Rectangle,
    ) {
    }
}

// TODO: https://github.com/iced-rs/iced/blob/e8f8216ea1f9deef7f2d02fa2600a0b4e247f8fa/src/program.rs#L630
/*
impl iced::program::Renderer for LowRes {}
*/
pub struct LowResCompositor {}

impl Compositor for LowResCompositor {
    type Renderer = LowRes;

    type Surface = iced_wgpu::wgpu::Surface<'static>;

    async fn with_backend<W: iced_wgpu::graphics::compositor::Window + Clone>(
        _settings: iced_wgpu::graphics::Settings,
        _compatible_window: W,
        _backend: Option<&str>,
    ) -> Result<Self, iced_wgpu::graphics::Error> {
        todo!()
    }

    fn create_renderer(&self) -> Self::Renderer {
        todo!()
    }

    fn create_surface<W: iced_wgpu::graphics::compositor::Window + Clone>(
        &mut self,
        window: W,
        width: u32,
        height: u32,
    ) -> Self::Surface {
        todo!()
    }

    fn configure_surface(&mut self, surface: &mut Self::Surface, width: u32, height: u32) {
        todo!()
    }

    fn fetch_information(&self) -> iced_wgpu::graphics::compositor::Information {
        todo!()
    }

    fn present<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &iced_wgpu::graphics::Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Result<(), iced_wgpu::graphics::compositor::SurfaceError> {
        todo!()
    }

    fn screenshot<T: AsRef<str>>(
        &mut self,
        renderer: &mut Self::Renderer,
        surface: &mut Self::Surface,
        viewport: &iced_wgpu::graphics::Viewport,
        background_color: Color,
        overlay: &[T],
    ) -> Vec<u8> {
        todo!()
    }
}

impl iced::advanced::graphics::compositor::Default for LowRes {
    type Compositor = LowResCompositor;
}
