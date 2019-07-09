use crate::device::Device;
use crate::sys;

#[allow(dead_code)]
pub struct Swapchain {
    swapchain: sys::dxgi::Swapchain,
    backbuffer: sys::dxgi::BackbufferRaw,
    render_target: sys::direct2d::Bitmap,
}

impl Swapchain {
    pub fn create_from_hwnd(device: &Device, hwnd: winapi::shared::windef::HWND) -> Self {
        let swapchain = sys::dxgi::Swapchain::create_from_hwnd(&device.d3d11_device, hwnd);
        let backbuffer = swapchain.get_backbuffer();
        let render_target = device.create_bitmap_from_backbuffer(&backbuffer);
        device.set_target(&render_target);

        Swapchain {
            swapchain,
            backbuffer,
            render_target,
        }
    }

    pub fn present(&self) {
        self.swapchain.present();
    }
}
