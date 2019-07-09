use super::d3d11;
use super::dwrite::{TextFormat, TextLayout};
use super::dxgi::BackbufferRaw;
use std::ops::Deref;
use std::ops::Range;
use std::ptr;
use winapi::shared::{dxgi, dxgiformat::*};
use winapi::um::{d2d1, d2d1_1, d2d1_3, dcommon};
use winapi::Interface;
use wio::com::ComPtr;
use wio::wide::ToWide;

pub type TransformRaw = [[f32; 3]; 2];
#[derive(Copy, Clone, Debug)]
pub struct Transform(TransformRaw);

impl Transform {
    pub fn identity() -> Self {
        Transform([[1.0, 0.0, 0.0], [0.0, 1.0, 0.0]])
    }
}

impl Deref for Transform {
    type Target = TransformRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type Point = [f32; 2];

fn point_to_d2d(p: Point) -> d2d1::D2D1_POINT_2F {
    d2d1::D2D1_POINT_2F { x: p[0], y: p[1] }
}

pub type Color = [f32; 4];

pub struct Extent {
    pub width: f32,
    pub height: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct GradientStop {
    pub position: f32,
    pub color: Color,
}

pub type GradientStopCollectionRaw = ComPtr<d2d1::ID2D1GradientStopCollection>;
pub struct GradientStopCollection(GradientStopCollectionRaw);

impl Deref for GradientStopCollection {
    type Target = GradientStopCollectionRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl Rect {
    fn as_rect_f(&self) -> d2d1::D2D1_RECT_F {
        d2d1::D2D1_RECT_F {
            left: self.x,
            right: self.x + self.width,
            top: self.y,
            bottom: self.y + self.height,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct RoundedRect {
    pub rect: Rect,
    pub radius_x: f32,
    pub radius_y: f32,
}

#[derive(Copy, Clone, Debug)]
pub struct Ellipse {
    pub x: f32,
    pub y: f32,
    pub radius_x: f32,
    pub radius_y: f32,
}

pub type DeviceRaw = ComPtr<d2d1_3::ID2D1Device4>;
pub struct Device(DeviceRaw);

impl Device {
    pub fn create_context(&self) -> DeviceContext {
        unsafe {
            let mut context = ptr::null_mut();
            let _hr = self.CreateDeviceContext(
                d2d1_1::D2D1_DEVICE_CONTEXT_OPTIONS_NONE,
                &mut context as *mut _,
            );

            DeviceContext(DeviceContextRaw::from_raw(context))
        }
    }
}

impl Deref for Device {
    type Target = DeviceRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type BitmapRaw = ComPtr<d2d1_1::ID2D1Bitmap1>;
pub struct Bitmap(BitmapRaw);

impl Deref for Bitmap {
    type Target = BitmapRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type DeviceContextRaw = ComPtr<d2d1_3::ID2D1DeviceContext4>;
pub struct DeviceContext(DeviceContextRaw);

impl DeviceContext {
    pub fn create_bitmap(&self, extent: Extent, data: &[u8], pitch: u32) -> Bitmap {
        let size = d2d1::D2D1_SIZE_U {
            width: extent.width as _,
            height: extent.height as _,
        };
        let properties = d2d1_1::D2D1_BITMAP_PROPERTIES1 {
            pixelFormat: dcommon::D2D1_PIXEL_FORMAT {
                format: DXGI_FORMAT_R8G8B8A8_UNORM,
                alphaMode: dcommon::D2D1_ALPHA_MODE_IGNORE,
            },
            dpiX: 96.0,
            dpiY: 96.0,
            bitmapOptions: d2d1_1::D2D1_BITMAP_OPTIONS_NONE,
            colorContext: ptr::null(),
        };

        unsafe {
            let mut bitmap = ptr::null_mut();
            let _hr = self.CreateBitmap(
                size,
                data.as_ptr() as *const _,
                pitch,
                &properties,
                &mut bitmap as *mut _,
            );
            Bitmap(BitmapRaw::from_raw(bitmap))
        }
    }

    pub fn create_bitmap_from_backbuffer(&self, backbuffer: &BackbufferRaw) -> Bitmap {
        let surface = backbuffer.cast::<dxgi::IDXGISurface>().unwrap();
        unsafe {
            let mut bitmap = ptr::null_mut();
            let _hr = self.CreateBitmapFromDxgiSurface(
                surface.as_raw(),
                ptr::null(),
                &mut bitmap as *mut _,
            );
            Bitmap(BitmapRaw::from_raw(bitmap))
        }
    }

    pub fn set_target(&self, bitmap: &Bitmap) {
        let image = bitmap.cast::<d2d1::ID2D1Image>().unwrap();
        unsafe {
            self.SetTarget(image.as_raw());
        }
    }

    pub fn create_solid_brush(&self, c: Color, opacity: f32, transform: Transform) -> SolidBrush {
        let properties = d2d1::D2D1_BRUSH_PROPERTIES {
            opacity,
            transform: d2d1::D2D1_MATRIX_3X2_F {
                matrix: [
                    [transform[0][0], transform[1][0]],
                    [transform[0][1], transform[1][1]],
                    [transform[0][2], transform[1][2]],
                ],
            },
        };
        unsafe {
            let mut brush = ptr::null_mut();
            let _hr = self.CreateSolidColorBrush(
                &d2d1::D2D1_COLOR_F {
                    r: c[0],
                    g: c[1],
                    b: c[2],
                    a: c[3],
                },
                &properties,
                &mut brush as *mut _,
            );
            SolidBrush(SolidBrushRaw::from_raw(brush))
        }
    }

    pub fn create_gradient_stop_collection(
        &self,
        stops: &[GradientStop],
    ) -> GradientStopCollection {
        let stops = stops
            .iter()
            .map(|stop| d2d1::D2D1_GRADIENT_STOP {
                position: stop.position,
                color: d2d1::D2D1_COLOR_F {
                    r: stop.color[0],
                    g: stop.color[1],
                    b: stop.color[2],
                    a: stop.color[3],
                },
            })
            .collect::<Vec<_>>();
        unsafe {
            let mut collection = ptr::null_mut();
            let _hr = self
                .cast::<d2d1::ID2D1RenderTarget>()
                .unwrap()
                .CreateGradientStopCollection(
                    stops.as_ptr(),
                    stops.len() as _,
                    d2d1::D2D1_GAMMA_1_0,
                    d2d1::D2D1_EXTEND_MODE_CLAMP,
                    &mut collection as *mut _,
                );
            GradientStopCollection(GradientStopCollectionRaw::from_raw(collection))
        }
    }

    pub fn create_linear_gradient_brush(
        &self,
        line: Range<Point>,
        stops: &GradientStopCollection,
        opacity: f32,
        transform: Transform,
    ) -> LinearGradientBrush {
        let brush_properties = d2d1::D2D1_BRUSH_PROPERTIES {
            opacity,
            transform: d2d1::D2D1_MATRIX_3X2_F {
                matrix: [
                    [transform[0][0], transform[1][0]],
                    [transform[0][1], transform[1][1]],
                    [transform[0][2], transform[1][2]],
                ],
            },
        };

        let properties = d2d1::D2D1_LINEAR_GRADIENT_BRUSH_PROPERTIES {
            startPoint: point_to_d2d(line.start),
            endPoint: point_to_d2d(line.end),
        };
        unsafe {
            let mut brush = ptr::null_mut();
            let _hr = self.CreateLinearGradientBrush(
                &properties,
                &brush_properties,
                stops.as_raw(),
                &mut brush as *mut _,
            );
            LinearGradientBrush(LinearGradientBrushRaw::from_raw(brush))
        }
    }

    pub fn begin_draw(&self) {
        unsafe {
            self.BeginDraw();
        }
    }

    pub fn clear(&self, c: Color) {
        unsafe {
            self.Clear(&d2d1::D2D1_COLOR_F {
                r: c[0],
                g: c[1],
                b: c[2],
                a: c[3],
            });
        }
    }

    pub fn draw_bitmap(&self, bitmap: &Bitmap, dst: Rect, opacity: f32, src: Rect) {
        unsafe {
            let upper = bitmap.cast::<d2d1::ID2D1Bitmap>().unwrap();
            let dst = dst.as_rect_f();
            let src = src.as_rect_f();
            self.DrawBitmap(
                upper.as_raw(),
                &dst,
                opacity,
                d2d1_1::D2D1_INTERPOLATION_MODE_LINEAR,
                &src,
                ptr::null(),
            );
        }
    }

    pub fn fill_rectangle(&self, brush: &impl Brush, rect: Rect) {
        let r = rect.as_rect_f();
        unsafe {
            self.FillRectangle(&r, brush.as_brush());
        }
    }

    pub fn fill_rounded_rectangle(&self, brush: &impl Brush, rect: RoundedRect) {
        let r = d2d1::D2D1_ROUNDED_RECT {
            rect: rect.rect.as_rect_f(),
            radiusX: rect.radius_x,
            radiusY: rect.radius_y,
        };
        unsafe {
            self.cast::<d2d1::ID2D1RenderTarget>()
                .unwrap()
                .FillRoundedRectangle(&r, brush.as_brush());
        }
    }

    pub fn fill_ellipse(&self, brush: &impl Brush, ellipse: Ellipse) {
        let e = d2d1::D2D1_ELLIPSE {
            point: point_to_d2d([ellipse.x, ellipse.y]),
            radiusX: ellipse.radius_x,
            radiusY: ellipse.radius_y,
        };

        unsafe {
            self.cast::<d2d1::ID2D1RenderTarget>()
                .unwrap()
                .FillEllipse(&e, brush.as_brush());
        }
    }

    pub fn draw_rounded_rectangle(&self, brush: &impl Brush, rect: RoundedRect, stroke_width: f32) {
        let r = d2d1::D2D1_ROUNDED_RECT {
            rect: rect.rect.as_rect_f(),
            radiusX: rect.radius_x,
            radiusY: rect.radius_y,
        };
        unsafe {
            self.cast::<d2d1::ID2D1RenderTarget>()
                .unwrap()
                .DrawRoundedRectangle(&r, brush.as_brush(), stroke_width, ptr::null_mut());
        }
    }

    pub fn draw_line(&self, brush: &impl Brush, line: Range<Point>, stroke_width: f32) {
        unsafe {
            self.DrawLine(
                point_to_d2d(line.start),
                point_to_d2d(line.end),
                brush.as_brush(),
                stroke_width,
                ptr::null_mut(), // style
            );
        }
    }

    pub fn draw_text(
        &self,
        text: &str,
        text_format: &TextFormat,
        layout: Rect,
        brush: &impl Brush,
    ) {
        let text = text.to_wide();
        let r = layout.as_rect_f();
        unsafe {
            self.cast::<d2d1::ID2D1RenderTarget>().unwrap().DrawText(
                text.as_ptr(),
                text.len() as _,
                text_format.as_raw(),
                &r,
                brush.as_brush(),
                d2d1::D2D1_DRAW_TEXT_OPTIONS_NONE,
                dcommon::DWRITE_MEASURING_MODE_NATURAL,
            );
        }
    }

    pub fn draw_text_layout(&self, origin: Point, text_layout: &TextLayout, brush: &impl Brush) {
        unsafe {
            self.DrawTextLayout(
                point_to_d2d(origin),
                text_layout.as_raw(),
                brush.as_brush(),
                ptr::null_mut(), // TODO
                0,               // TODO
                d2d1::D2D1_DRAW_TEXT_OPTIONS_ENABLE_COLOR_FONT,
            );
        }
    }

    pub fn end_draw(&self) {
        unsafe {
            self.EndDraw(ptr::null_mut(), ptr::null_mut());
        }
    }

    pub fn create_layer(&self, extent: Extent) -> Layer {
        let size = d2d1::D2D1_SIZE_F {
            width: extent.width as _,
            height: extent.height as _,
        };
        unsafe {
            let mut layer = ptr::null_mut();
            let _hr = self.CreateLayer(&size, &mut layer as *mut _);
            Layer(LayerRaw::from_raw(layer))
        }
    }

    pub fn fill_geometry(&self, geometry: &PathGeometry, brush: &impl Brush) {
        unsafe {
            self.FillGeometry(
                geometry.as_raw() as *mut _,
                brush.as_brush(),
                ptr::null_mut(),
            );
        }
    }

    pub fn draw_geometry(&self, geometry: &PathGeometry, brush: &impl Brush, stroke_width: f32) {
        unsafe {
            self.DrawGeometry(
                geometry.as_raw() as *mut _,
                brush.as_brush(),
                stroke_width,
                ptr::null_mut(), // TODO
            );
        }
    }
}

impl Deref for DeviceContext {
    type Target = DeviceContextRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type FactoryRaw = ComPtr<d2d1_3::ID2D1Factory5>;
pub struct Factory(FactoryRaw);

impl Factory {
    pub fn new() -> Self {
        let options = d2d1::D2D1_FACTORY_OPTIONS {
            debugLevel: d2d1::D2D1_DEBUG_LEVEL_INFORMATION,
        };

        unsafe {
            let mut factory = ptr::null_mut();
            let _hr = d2d1::D2D1CreateFactory(
                d2d1::D2D1_FACTORY_TYPE_SINGLE_THREADED,
                &d2d1_1::ID2D1Factory1::uuidof(),
                &options,
                &mut factory as *mut _ as *mut *mut _,
            );

            Factory(FactoryRaw::from_raw(factory))
        }
    }

    pub fn create_device(&self, d3d11_device: &d3d11::Device) -> Device {
        let dxgi = d3d11_device.cast::<dxgi::IDXGIDevice>().unwrap();
        unsafe {
            let mut device = ptr::null_mut();
            let _hr = self.CreateDevice(dxgi.as_raw(), &mut device as *mut _);

            Device(DeviceRaw::from_raw(device))
        }
    }

    pub fn create_path_geometry(&self) -> PathGeometry {
        unsafe {
            let mut geometry = ptr::null_mut();
            let _hr = self.CreatePathGeometry(&mut geometry as *mut _);
            PathGeometry(PathGeometryRaw::from_raw(geometry))
        }
    }
}

impl Deref for Factory {
    type Target = FactoryRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type RenderTargetRaw = ComPtr<d2d1::ID2D1RenderTarget>;
pub struct RenderTarget(RenderTargetRaw);

impl Deref for RenderTarget {
    type Target = RenderTargetRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub trait Brush {
    fn as_brush(&self) -> *mut d2d1::ID2D1Brush;
}

pub type SolidBrushRaw = ComPtr<d2d1::ID2D1SolidColorBrush>;
pub struct SolidBrush(SolidBrushRaw);

impl Brush for SolidBrush {
    fn as_brush(&self) -> *mut d2d1::ID2D1Brush {
        self.as_raw() as *mut _
    }
}

impl Deref for SolidBrush {
    type Target = SolidBrushRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type LinearGradientBrushRaw = ComPtr<d2d1::ID2D1LinearGradientBrush>;
pub struct LinearGradientBrush(LinearGradientBrushRaw);

impl Brush for LinearGradientBrush {
    fn as_brush(&self) -> *mut d2d1::ID2D1Brush {
        self.as_raw() as *mut _
    }
}

impl Deref for LinearGradientBrush {
    type Target = LinearGradientBrushRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type LayerRaw = ComPtr<d2d1::ID2D1Layer>;
pub struct Layer(LayerRaw);

impl Deref for Layer {
    type Target = LayerRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

pub type GeometrySinkRaw = ComPtr<d2d1::ID2D1GeometrySink>;
pub struct GeometrySink(GeometrySinkRaw);

impl Deref for GeometrySink {
    type Target = GeometrySinkRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl GeometrySink {
    pub fn begin_figure(&mut self, start: Point) {
        // TODO
        unsafe {
            self.BeginFigure(point_to_d2d(start), d2d1::D2D1_FIGURE_BEGIN_FILLED);
        }
    }

    pub fn add_line(&mut self, point: Point) {
        unsafe {
            self.AddLine(point_to_d2d(point));
        }
    }

    pub fn add_quadratic_bezier(&mut self, p0: Point, p1: Point) {
        unsafe {
            self.AddQuadraticBezier(&d2d1::D2D1_QUADRATIC_BEZIER_SEGMENT {
                point1: point_to_d2d(p0),
                point2: point_to_d2d(p1),
            });
        }
    }

    pub fn add_bezier(&mut self, p0: Point, p1: Point, p2: Point) {
        unsafe {
            self.AddBezier(&d2d1::D2D1_BEZIER_SEGMENT {
                point1: point_to_d2d(p0),
                point2: point_to_d2d(p1),
                point3: point_to_d2d(p2),
            });
        }
    }

    pub fn end_figure(&mut self) {
        // TODO
        unsafe {
            self.EndFigure(d2d1::D2D1_FIGURE_END_CLOSED);
        }
    }

    pub fn close(&mut self) {
        unsafe {
            self.Close();
        }
    }
}

pub type PathGeometryRaw = ComPtr<d2d1_1::ID2D1PathGeometry1>;
pub struct PathGeometry(PathGeometryRaw);

impl Deref for PathGeometry {
    type Target = PathGeometryRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl PathGeometry {
    pub fn open(&self) -> GeometrySink {
        unsafe {
            let mut sink = ptr::null_mut();
            let _hr = self.Open(&mut sink as *mut _);
            GeometrySink(GeometrySinkRaw::from_raw(sink))
        }
    }
}
