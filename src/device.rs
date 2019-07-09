use crate::{sys, text};
use std::ops::Deref;

#[allow(dead_code)]
pub struct Device {
    pub(crate) d3d11_device: sys::d3d11::Device,
    pub d2d_factory: sys::direct2d::Factory,
    d2d_device: sys::direct2d::Device,
    d2d_context: sys::direct2d::DeviceContext,
    pub(crate) dwrite_factory: text::Text,
    d3d11_context: sys::d3d11::DeviceContext,
}

impl Device {
    pub fn create() -> Self {
        let d2d_factory = sys::direct2d::Factory::new();
        let (d3d11_device, d3d11_context) = sys::d3d11::Device::new();
        let dwrite_factory = text::Text(sys::dwrite::Factory::new());
        let d2d_device = d2d_factory.create_device(&d3d11_device);
        let d2d_context = d2d_device.create_context();

        Device {
            d2d_factory,
            d2d_device,
            d2d_context,
            dwrite_factory,
            d3d11_device,
            d3d11_context,
        }
    }
}

impl Deref for Device {
    type Target = sys::direct2d::DeviceContext;
    fn deref(&self) -> &Self::Target {
        &self.d2d_context
    }
}
