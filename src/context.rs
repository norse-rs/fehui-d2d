use crate::sys::direct2d::Brush as D2DBrush;
use crate::{device::Device, sys, text};
use kurbo::{Affine, PathEl, Rect, Shape, Point};
use piet::{Color, Error, FillRule, Gradient, ImageFormat, InterpolationMode, RoundInto, StrokeStyle};
use winapi::um::d2d1;

pub struct Image(sys::direct2d::Bitmap);

pub enum Brush {
    Solid(sys::direct2d::SolidBrush),
    LinearGradient(sys::direct2d::LinearGradientBrush),
}

impl D2DBrush for Brush {
    fn as_brush(&self) -> *mut d2d1::ID2D1Brush {
        match *self {
            Brush::Solid(ref brush) => brush.as_brush(),
            Brush::LinearGradient(ref brush) => brush.as_brush(),
        }
    }
}

fn byte_to_frac(byte: u32) -> f32 {
    ((byte & 0xFF) as f32) * (1.0 / 255.0)
}

impl piet::RenderContext for Device {
    type Brush = Brush;
    type Text = text::Text;
    type TextLayout = text::TextLayout;

    type Image = Image;

    /// Report an internal error.
    ///
    /// Drawing operations may cause internal errors, which may also occur
    /// asynchronously after the drawing command was issued. This method reports
    /// any such error that has been detected.
    fn status(&mut self) -> Result<(), Error> {
        Ok(())
    }

    fn solid_brush(&mut self, rgba: Color) -> Self::Brush {
        let rgba = rgba.as_rgba32();
        Brush::Solid(self.create_solid_brush(
            [
                byte_to_frac(rgba >> 24),
                byte_to_frac(rgba >> 16),
                byte_to_frac(rgba >> 8),
                byte_to_frac(rgba),
            ],
            1.0,
            sys::direct2d::Transform::identity(),
        ))
    }

    /// Create a new gradient brush.
    fn gradient(&mut self, gradient: Gradient) -> Result<Self::Brush, Error> {
        match gradient {
            Gradient::Linear(ref linear) => {
                let dir = linear.end - linear.start;
                let stops = linear
                    .stops
                    .iter()
                    .map(|stop| {
                        let rgba = stop.color.as_rgba32();
                        sys::direct2d::GradientStop {
                        position: stop.pos as _,
                        color: [
                            byte_to_frac(rgba >> 24),
                            byte_to_frac(rgba >> 16),
                            byte_to_frac(rgba >> 8),
                            byte_to_frac(rgba),
                        ],
                    }
                    })
                    .collect::<Box<[_]>>();

                let gradient_stop_collection = self.create_gradient_stop_collection(&stops);
                let linear = self.create_linear_gradient_brush(
                    [linear.start.x as _, linear.start.y as _]
                        ..[linear.end.x as _, linear.end.y as _],
                    &gradient_stop_collection,
                    1.0,
                    sys::direct2d::Transform::identity(),
                );
                Ok(Brush::LinearGradient(linear))
            }
            Gradient::Radial(ref radial) => unimplemented!(),
        }
    }

    /// Clear the canvas with the given color.
    fn clear(&mut self, rgba: Color) {
        let rgba = rgba.as_rgba32();
        sys::direct2d::DeviceContext::clear(
            &self,
            [
                byte_to_frac(rgba >> 24),
                byte_to_frac(rgba >> 16),
                byte_to_frac(rgba >> 8),
                byte_to_frac(rgba),
            ],
        );
    }

    /// Stroke a shape.
    fn stroke(
        &mut self,
        shape: impl Shape,
        brush: &Self::Brush,
        width: f64,
        _style: Option<&StrokeStyle>,
    ) {
        assert!(_style.is_none()); // unimplemented!()

        if let Some(circle) = shape.as_circle() {
            unimplemented!()
        } else if let Some(rect) = shape.as_rect() {
            unimplemented!()
        } else if let Some(rounded) = shape.as_rounded_rect() {
            let origin = rounded.origin();
            let radius = rounded.radius();
            self.draw_rounded_rectangle(
                brush,
                sys::direct2d::RoundedRect {
                    rect: sys::direct2d::Rect {
                        x: origin.x as _,
                        y: origin.y as _,
                        width: rounded.width() as _,
                        height: rounded.height() as _,
                    },
                    radius_x: radius as _,
                    radius_y: radius as _,
                },
                width.round_into(),
            );
        } else {
            // TODO: move into function
            let path = shape.to_bez_path(1e-3);
            let path_geometry = self.d2d_factory.create_path_geometry();
            {
                let mut active_figure = false;
                let mut sink = path_geometry.open();
                for elem in path {
                    match elem {
                        PathEl::MoveTo(p) => {
                            if active_figure {
                                sink.end_figure();
                            }

                            sink.begin_figure([p.x as _, p.y as _]);
                            active_figure = true;
                        }
                        PathEl::LineTo(p) => {
                            sink.add_line([p.x as _, p.y as _]);
                        }
                        PathEl::QuadTo(p0, p1) => {
                            sink.add_quadratic_bezier(
                                [p0.x as _, p0.y as _],
                                [p1.x as _, p1.y as _],
                            );
                        }
                        PathEl::CurveTo(p0, p1, p2) => {
                            sink.add_bezier(
                                [p0.x as _, p0.y as _],
                                [p1.x as _, p1.y as _],
                                [p2.x as _, p2.y as _],
                            );
                        }
                        PathEl::ClosePath => {
                            sink.end_figure();
                            active_figure = false;
                        }
                        _ => unimplemented!(),
                    }
                }

                if active_figure {
                    sink.end_figure();
                }

                sink.close();
            }
            self.draw_geometry(&path_geometry, brush, width.round_into());
        }
    }

    /// Fill a shape.
    fn fill(&mut self, shape: impl Shape, brush: &Self::Brush, _fill_rule: FillRule) {
        if let Some(circle) = shape.as_circle() {
            self.fill_ellipse(
                brush,
                sys::direct2d::Ellipse {
                    x: circle.center.x as _,
                    y: circle.center.y as _,
                    radius_x: circle.radius as _,
                    radius_y: circle.radius as _,
                },
            );
        } else if let Some(rect) = shape.as_rect() {
            self.fill_rectangle(
                brush,
                sys::direct2d::Rect {
                    x: rect.x0 as _,
                    y: rect.y0 as _,
                    width: (rect.x1 - rect.x0) as _,
                    height: (rect.y1 - rect.y0) as _,
                },
            )
        } else if let Some(rounded) = shape.as_rounded_rect() {
            let origin = rounded.origin();
            let radius = rounded.radius();
            self.fill_rounded_rectangle(
                brush,
                sys::direct2d::RoundedRect {
                    rect: sys::direct2d::Rect {
                        x: origin.x as _,
                        y: origin.y as _,
                        width: rounded.width() as _,
                        height: rounded.height() as _,
                    },
                    radius_x: radius as _,
                    radius_y: radius as _,
                },
            );
        } else {
            let path = shape.to_bez_path(1e-3);
            let path_geometry = self.d2d_factory.create_path_geometry();
            {
                let mut active_figure = false;
                let mut sink = path_geometry.open();
                for elem in path {
                    match elem {
                        PathEl::MoveTo(p) => {
                            if active_figure {
                                sink.end_figure();
                            }

                            sink.begin_figure([p.x as _, p.y as _]);
                            active_figure = true;
                        }
                        PathEl::LineTo(p) => {
                            sink.add_line([p.x as _, p.y as _]);
                        }
                        PathEl::QuadTo(p0, p1) => {
                            sink.add_quadratic_bezier(
                                [p0.x as _, p0.y as _],
                                [p1.x as _, p1.y as _],
                            );
                        }
                        PathEl::CurveTo(p0, p1, p2) => {
                            sink.add_bezier(
                                [p0.x as _, p0.y as _],
                                [p1.x as _, p1.y as _],
                                [p2.x as _, p2.y as _],
                            );
                        }
                        PathEl::ClosePath => {
                            sink.end_figure();
                            active_figure = false;
                        }
                        _ => unimplemented!(),
                    }
                }

                if active_figure {
                    sink.end_figure();
                }

                sink.close();
            }
            self.fill_geometry(&path_geometry, brush);
        }
    }

    /// Clip to a shape.
    ///
    /// All subsequent drawing operations up to the next [`restore`](#method.restore)
    /// are clipped by the shape.
    fn clip(&mut self, shape: impl Shape, fill_rule: FillRule) {
        unimplemented!()
    }

    fn text(&mut self) -> &mut Self::Text {
        &mut self.dwrite_factory
    }

    /// Draw a text layout.
    ///
    /// The `pos` parameter specifies the baseline of the left starting place of
    /// the text. Note: this is true even if the text is right-to-left.
    fn draw_text(
        &mut self,
        layout: &Self::TextLayout,
        pos: impl Into<Point>,
        brush: &Self::Brush,
    ) {
        let pos: Point = pos.into();
        self.draw_text_layout([pos.x as _, pos.y as _], &layout.0, brush);
    }

    /// Save the context state.
    ///
    /// Pushes the current context state onto a stack, to be popped by
    /// [`restore`](#method.restore).
    ///
    /// Prefer [`with_save`](#method.with_save) if possible, as that statically
    /// enforces balance of save/restore pairs.
    ///
    /// The context state currently consists of a clip region and an affine
    /// transform, but is expected to grow in the near future.
    fn save(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    /// Restore the context state.
    ///
    /// Pop a context state that was pushed by [`save`](#method.save). See
    /// that method for details.
    fn restore(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    /// Finish any pending operations.
    ///
    /// This will generally be called by a shell after all user drawing
    /// operations but before presenting. Not all back-ends will handle this
    /// the same way.
    fn finish(&mut self) -> Result<(), Error> {
        unimplemented!()
    }

    /// Apply a transform.
    ///
    /// Apply an affine transformation. The transformation remains in effect
    /// until a [`restore`](#method.restore) operation.
    fn transform(&mut self, transform: Affine) {
        unimplemented!()
    }

    /// Create a new image from a pixel buffer.
    fn make_image(
        &mut self,
        width: usize,
        height: usize,
        buf: &[u8],
        format: ImageFormat,
    ) -> Result<Self::Image, Error> {
        // TODO: format
        Ok(Image(self.create_bitmap(
            sys::direct2d::Extent {
                width: width as _,
                height: height as _,
            },
            buf,
            width as u32 * 4,
        )))
    }

    /// Draw an image.
    ///
    /// The image is scaled to the provided `rect`. It will be squashed if
    /// aspect ratios don't match.
    fn draw_image(
        &mut self,
        image: &Self::Image,
        rect: impl Into<Rect>,
        interp: InterpolationMode,
    ) {
        // TODO: interp
        let size = unsafe { image.0.GetSize() };
        let rect: kurbo::Rect = rect.into();

        self.draw_bitmap(
            &image.0,
            sys::direct2d::Rect {
                x: rect.x0 as _,
                y: rect.y0 as _,
                width: (rect.x1 - rect.x0) as _,
                height: (rect.y1 - rect.y0) as _,
            },
            1.0,
            sys::direct2d::Rect {
                x: 0.0,
                y: 0.0,
                width: size.width as _,
                height: size.height as _,
            },
        );
    }
}
