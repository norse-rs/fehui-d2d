use crate::sys;
use piet::{Error, RoundInto};

pub struct Text(pub(crate) sys::dwrite::Factory);

impl piet::Text for Text {
    type Font = Font;
    type FontBuilder = FontBuilder;
    type TextLayout = TextLayout;

    type TextLayoutBuilder = TextLayoutBuilder;

    fn new_font_by_name(
        &mut self,
        name: &str,
        size: f64,
    ) -> Result<Self::FontBuilder, Error> {
        Ok(FontBuilder {
            text_format: self.0.create_text_format(
                name,
                size.round_into(),
                sys::dwrite::FontWeight::Normal,
            ),
        })
    }

    fn new_text_layout(
        &mut self,
        font: &Self::Font,
        text: &str,
    ) -> Result<Self::TextLayoutBuilder, Error> {
        Ok(TextLayoutBuilder {
            text_layout: self.0.create_text_layout(text, &font.0, 1e6, 1e6), // hmm no widht/height?
        })
    }
}

pub struct TextLayout(pub(crate) sys::dwrite::TextLayout);
impl piet::TextLayout for TextLayout {
    fn width(&self) -> f64 {
        unimplemented!()
    }
}

pub struct Font(sys::dwrite::TextFormat);

impl piet::Font for Font {}

pub struct FontBuilder {
    text_format: sys::dwrite::TextFormat,
}

impl piet::FontBuilder for FontBuilder {
    type Out = Font;

    fn build(self) -> Result<Self::Out, Error> {
        Ok(Font(self.text_format))
    }
}

pub struct TextLayoutBuilder {
    text_layout: sys::dwrite::TextLayout,
}

impl piet::TextLayoutBuilder for TextLayoutBuilder {
    type Out = TextLayout;

    fn build(self) -> Result<Self::Out, Error> {
        Ok(TextLayout(self.text_layout))
    }
}
