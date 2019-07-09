use piet::{FontBuilder, ImageFormat, InterpolationMode, RenderContext, Text, TextLayoutBuilder};
use std::path::Path;
use winit::os::windows::WindowExt;

use kurbo::Shape;

fn main() {
    let mut events_loop = winit::EventsLoop::new();
    let (width, height) = (1164, 853);
    let window = winit::WindowBuilder::new()
        .with_dimensions((width, height).into())
        .with_decorations(false)
        .build(&events_loop)
        .unwrap();

    window.set_position((400, 100).into());

    let mut device = fehui_d2d::Device::create();
    let swapchain = fehui_d2d::Swapchain::create_from_hwnd(&device, window.get_hwnd() as *mut _);

    let img = image::open(&Path::new("examples/bg_tokyo_small.png"))
        .unwrap()
        .to_rgba();
    let img_width = img.width();
    let img_height = img.height();
    let img_data = img.into_raw();

    let image = device
        .make_image(
            img_width as _,
            img_height as _,
            &img_data,
            ImageFormat::RgbaSeparate,
        )
        .unwrap();

    let brush = device.solid_brush(piet::Color::rgb(1.0, 1.0, 1.0));
    let gradient = device
        .gradient(piet::Gradient::Linear(piet::LinearGradient {
            start: (100.0, 100.0).into(),
            end: (500.0, 300.0).into(),
            stops: vec![
                piet::GradientStop {
                    pos: 0.0,
                    color: piet::Color::rgb(1.0, 1.0, 1.0),
                },
                piet::GradientStop {
                    pos: 1.0,
                    color: piet::Color::rgb(0.0, 0.0, 0.0),
                },
            ],
        }))
        .unwrap();
    let circle = kurbo::Circle::new((100.0, 100.0), 50.0);
    let rect = kurbo::Rect::new(200.0, 100.0, 400.0, 140.0);
    let rounded_rect = kurbo::RoundedRect::new(200.0, 250.0, 400.0, 340.0, 20.0);

    let mut path = dbg!(kurbo::BezPath::from_vec(
        kurbo::RoundedRect::new(420.0, 250.0, 620.0, 340.0, 20.0)
            .to_bez_path(1e-3)
            .collect(),
    ));
    let mut path2 = dbg!(kurbo::BezPath::from_vec(
        kurbo::RoundedRect::new(420.0, 350.0, 620.0, 440.0, 20.0)
            .to_bez_path(1e-3)
            .collect(),
    ));

    let segoe = device
        .text()
        .new_font_by_name("Segoe", 49.0)
        .unwrap()
        .build()
        .unwrap();
    let text_layout = device
        .text()
        .new_text_layout(&segoe, "hewwo fehui 👌")
        .unwrap()
        .build()
        .unwrap();

    let mut stop = false;
    while !stop {
        events_loop.poll_events(|event| match event {
            winit::Event::WindowEvent { event, .. } => match event {
                winit::WindowEvent::CloseRequested => stop = true,
                _ => (),
            },
            _ => (),
        });

        device.begin_draw(); // non-generic

        device.draw_image(
            &image,
            piet::kurbo::Rect::new(0.0, 0.0, width as f64, height as f64),
            InterpolationMode::Bilinear,
        );

        device.fill(&circle, &brush, piet::FillRule::NonZero);
        device.fill(&rect, &brush, piet::FillRule::NonZero);
        device.fill(&rounded_rect, &brush, piet::FillRule::NonZero);
        device.fill(&path, &brush, piet::FillRule::NonZero);
        device.stroke(&path2, &brush, 3.0, None);

        device.draw_text(&text_layout, (500.5, 100.325), &brush);

        device.end_draw(); // non-generic
        swapchain.present();
    }
}
