use std::ops::Deref;
use std::ptr;
use winapi::um::{d3d11, d3dcommon};
use wio::com::ComPtr;

pub type DeviceRaw = ComPtr<d3d11::ID3D11Device>;
pub struct Device(DeviceRaw);

pub type DeviceContextRaw = ComPtr<d3d11::ID3D11DeviceContext>;
pub struct DeviceContext(DeviceContextRaw);

impl Device {
    pub fn new() -> (Device, DeviceContext) {
        let mut feature_level = d3dcommon::D3D_FEATURE_LEVEL_11_0;
        unsafe {
            let mut device = ptr::null_mut();
            let mut device_context = ptr::null_mut();
            let _hr = d3d11::D3D11CreateDevice(
                ptr::null_mut(), // default adapter
                d3dcommon::D3D_DRIVER_TYPE_HARDWARE,
                ptr::null_mut(),
                d3d11::D3D11_CREATE_DEVICE_BGRA_SUPPORT, // required
                ptr::null(),
                0,
                d3d11::D3D11_SDK_VERSION,
                &mut device as *mut _,
                &mut feature_level,
                &mut device_context as *mut _,
            );

            (
                Device(DeviceRaw::from_raw(device)),
                DeviceContext(DeviceContextRaw::from_raw(device_context)),
            )
        }
    }
}

impl Deref for Device {
    type Target = DeviceRaw;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
