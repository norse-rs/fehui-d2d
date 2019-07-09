use std::mem;
use std::ops::Deref;
use std::ptr;
use winapi::um::dwrite;
use winapi::Interface;
use wio::com::ComPtr;
use wio::wide::ToWide;

pub type FactoryRaw = ComPtr<dwrite::IDWriteFactory>;
pub struct Factory(FactoryRaw);

#[repr(u32)]
#[derive(Copy, Clone, Debug)]
pub enum FontWeight {
    Thin = dwrite::DWRITE_FONT_WEIGHT_THIN,
    UltraLight = dwrite::DWRITE_FONT_WEIGHT_ULTRA_LIGHT,
    Light = dwrite::DWRITE_FONT_WEIGHT_LIGHT,
    SemiLeght = dwrite::DWRITE_FONT_WEIGHT_SEMI_LIGHT,
    Normal = dwrite::DWRITE_FONT_WEIGHT_NORMAL,
    Medium = dwrite::DWRITE_FONT_WEIGHT_MEDIUM,
    SemiBold = dwrite::DWRITE_FONT_WEIGHT_SEMI_BOLD,
    Bold = dwrite::DWRITE_FONT_WEIGHT_BOLD,
    UltraBold = dwrite::DWRITE_FONT_WEIGHT_ULTRA_BOLD,
    Black = dwrite::DWRITE_FONT_WEIGHT_BLACK,
    UltraBlack = dwrite::DWRITE_FONT_WEIGHT_ULTRA_BLACK,
}

impl Factory {
    pub fn new() -> Self {
        unsafe {
            let mut factory = ptr::null_mut();
            let _hr = dwrite::DWriteCreateFactory(
                dwrite::DWRITE_FACTORY_TYPE_SHARED,
                &dwrite::IDWriteFactory::uuidof(),
                &mut factory as *mut _ as *mut *mut _,
            );

            Factory(FactoryRaw::from_raw(factory))
        }
    }

    pub fn create_text_format(
        &self,
        font_family: &str,
        size: f32,
        font_weight: FontWeight,
    ) -> TextFormat {
        let font_family = font_family.to_wide_null();
        let locale = "en-GB".to_wide_null(); // TODO
        unsafe {
            let mut text_format = ptr::null_mut();
            let _hr = self.CreateTextFormat(
                font_family.as_ptr(),
                ptr::null_mut(),
                font_weight as _,
                dwrite::DWRITE_FONT_STYLE_NORMAL,   // TODO
                dwrite::DWRITE_FONT_STRETCH_NORMAL, // TODO
                size,
                locale.as_ptr(),
                &mut text_format as *mut _,
            );

            TextFormat(TextFormatRaw::from_raw(text_format))
        }
    }

    pub fn create_text_layout(
        &self,
        text: &str,
        format: &TextFormat,
        width: f32,
        height: f32,
    ) -> TextLayout {
        let text = text.to_wide_null();
        unsafe {
            let mut layout = ptr::null_mut();
            let _hr = self.CreateTextLayout(
                text.as_ptr(),
                text.len() as _,
                format.as_raw(),
                width,
                height,
                &mut layout as *mut _,
            );

            TextLayout(TextLayoutRaw::from_raw(layout))
        }
    }
}

impl Deref for Factory {
    type Target = FactoryRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type TextFormatRaw = ComPtr<dwrite::IDWriteTextFormat>;
pub struct TextFormat(TextFormatRaw);

impl Deref for TextFormat {
    type Target = TextFormatRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct OverhangMetrics {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct TextMetrics {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub width_incl_trailing_whitespaces: f32,
    pub height: f32,
    pub layout_width: f32,
    pub layout_height: f32,
    pub max_bidi_reordering_depth: u32,
    pub line_count: u32,
}

pub type TextLayoutRaw = ComPtr<dwrite::IDWriteTextLayout>;
pub struct TextLayout(TextLayoutRaw);

impl TextLayout {
    pub fn get_metrics(&self) -> TextMetrics {
        unsafe {
            let mut metrics = mem::zeroed();
            let _hr = self.GetMetrics(&mut metrics);
            TextMetrics {
                left: metrics.left,
                top: metrics.top,
                width: metrics.width,
                width_incl_trailing_whitespaces: metrics.widthIncludingTrailingWhitespace,
                height: metrics.height,
                layout_width: metrics.layoutWidth,
                layout_height: metrics.layoutHeight,
                max_bidi_reordering_depth: metrics.maxBidiReorderingDepth,
                line_count: metrics.lineCount,
            }
        }
    }

    pub fn get_overhang_metrics(&self) -> OverhangMetrics {
        unsafe {
            let mut metrics = mem::zeroed();
            let _hr = self.GetOverhangMetrics(&mut metrics);
            OverhangMetrics {
                left: metrics.left,
                right: metrics.right,
                top: metrics.top,
                bottom: metrics.bottom,
            }
        }
    }
}

impl Deref for TextLayout {
    type Target = TextLayoutRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
