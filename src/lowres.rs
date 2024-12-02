use iced::advanced::renderer::{self, Quad};
use iced::advanced::{text, Text};
use iced::{Background, Color, Pixels, Point, Rectangle, Transformation};

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
