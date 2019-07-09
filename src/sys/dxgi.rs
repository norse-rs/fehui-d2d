use super::d3d11;
use std::ops::Deref;
use std::ptr;
use winapi::shared::{dxgi, dxgi1_2, dxgiformat::*, dxgitype};
use winapi::um::d3d11::ID3D11Texture2D;
use winapi::Interface;
use wio::com::ComPtr;

pub type AdapterRaw = ComPtr<dxgi::IDXGIAdapter>;
pub type FactoryRaw = ComPtr<dxgi1_2::IDXGIFactory2>;
pub type BackbufferRaw = ComPtr<ID3D11Texture2D>;

pub type SwapchainRaw = ComPtr<dxgi1_2::IDXGISwapChain1>;
pub struct Swapchain(SwapchainRaw);

impl Swapchain {
    pub fn create_from_hwnd(device: &d3d11::Device, hwnd: winapi::shared::windef::HWND) -> Self {
        let dxgi = device.cast::<dxgi::IDXGIDevice>().unwrap();

        let adapter = unsafe {
            let mut adapter = ptr::null_mut();
            let _hr = dxgi.GetAdapter(&mut adapter as *mut _);
            AdapterRaw::from_raw(adapter)
        };
        let factory = unsafe {
            let mut factory = ptr::null_mut();
            let _hr = adapter.GetParent(
                &dxgi1_2::IDXGIFactory2::uuidof(),
                &mut factory as *mut _ as *mut *mut _,
            );
            FactoryRaw::from_raw(factory)
        };

        let desc = dxgi1_2::DXGI_SWAP_CHAIN_DESC1 {
            Width: 0,
            Height: 0,
            Format: DXGI_FORMAT_B8G8R8A8_UNORM,
            Stereo: 0,
            SampleDesc: dxgitype::DXGI_SAMPLE_DESC {
                Count: 1,
                Quality: 0,
            },
            BufferUsage: dxgitype::DXGI_USAGE_RENDER_TARGET_OUTPUT,
            BufferCount: 2,
            Scaling: dxgi1_2::DXGI_SCALING_NONE,
            SwapEffect: dxgi::DXGI_SWAP_EFFECT_FLIP_SEQUENTIAL,
            AlphaMode: dxgi1_2::DXGI_ALPHA_MODE_UNSPECIFIED,
            Flags: 0,
        };

        unsafe {
            let mut swapchain = ptr::null_mut();
            let _hr = factory.CreateSwapChainForHwnd(
                device.as_raw() as *mut _,
                hwnd as *mut _,
                &desc,
                ptr::null(), // windowed
                ptr::null_mut(),
                &mut swapchain,
            );
            Swapchain(SwapchainRaw::from_raw(swapchain))
        }
    }

    pub fn get_backbuffer(&self) -> BackbufferRaw {
        unsafe {
            let mut buffer = ptr::null_mut();
            let _hr = self.GetBuffer(
                0,
                &ID3D11Texture2D::uuidof(),
                &mut buffer as *mut _ as *mut *mut _,
            );
            BackbufferRaw::from_raw(buffer)
        }
    }

    pub fn present(&self) {
        unsafe {
            self.Present(1, 0);
        }
    }
}

impl Deref for Swapchain {
    type Target = SwapchainRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
