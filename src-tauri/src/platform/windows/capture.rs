//! Single-frame capture with Windows.Graphics.Capture (monitor or window) → BGRA `Frame`.
//! The same device/frame-pool setup is the basis of the Stage 4 recorder.

use super::win_err;
use crate::core::{
    errors::{AppError, AppResult},
    image::Frame,
};
use std::{sync::mpsc, time::Duration};
use windows::{
    core::{factory, Interface},
    Foundation::TypedEventHandler,
    Graphics::{
        Capture::{Direct3D11CaptureFrame, Direct3D11CaptureFramePool, GraphicsCaptureItem},
        DirectX::{Direct3D11::IDirect3DDevice, DirectXPixelFormat},
    },
    Win32::{
        Foundation::{HMODULE, HWND},
        Graphics::{
            Direct3D::{D3D_DRIVER_TYPE, D3D_DRIVER_TYPE_HARDWARE, D3D_DRIVER_TYPE_WARP},
            Direct3D11::{
                D3D11CreateDevice, ID3D11Device, ID3D11Texture2D, D3D11_CPU_ACCESS_READ,
                D3D11_CREATE_DEVICE_BGRA_SUPPORT, D3D11_MAPPED_SUBRESOURCE, D3D11_MAP_READ,
                D3D11_SDK_VERSION, D3D11_TEXTURE2D_DESC, D3D11_USAGE_STAGING,
            },
            Dxgi::IDXGIDevice,
            Gdi::HMONITOR,
        },
        System::{
            Com::{CoInitializeEx, COINIT_MULTITHREADED},
            WinRT::{
                Direct3D11::{CreateDirect3D11DeviceFromDXGIDevice, IDirect3DDxgiInterfaceAccess},
                Graphics::Capture::IGraphicsCaptureItemInterop,
            },
        },
    },
};

fn create_device(kind: D3D_DRIVER_TYPE) -> windows::core::Result<ID3D11Device> {
    let mut device = None;
    // SAFETY: standard device creation with out-pointer.
    unsafe {
        D3D11CreateDevice(
            None,
            kind,
            HMODULE::default(),
            D3D11_CREATE_DEVICE_BGRA_SUPPORT,
            None,
            D3D11_SDK_VERSION,
            Some(&mut device),
            None,
            None,
        )?;
    }
    device.ok_or_else(|| windows::core::Error::from_hresult(windows::Win32::Foundation::E_FAIL))
}

/// Hardware device, falling back to WARP (software) on machines/VMs without a usable GPU.
pub(super) fn d3d_device() -> AppResult<ID3D11Device> {
    create_device(D3D_DRIVER_TYPE_HARDWARE)
        .or_else(|_| create_device(D3D_DRIVER_TYPE_WARP))
        .map_err(|e| win_err("D3D11CreateDevice", e))
}

pub(super) fn winrt_device(d3d: &ID3D11Device) -> AppResult<IDirect3DDevice> {
    let dxgi: IDXGIDevice = d3d.cast().map_err(|e| win_err("IDXGIDevice", e))?;
    // SAFETY: valid DXGI device.
    let inspectable = unsafe { CreateDirect3D11DeviceFromDXGIDevice(&dxgi) }
        .map_err(|e| win_err("CreateDirect3D11DeviceFromDXGIDevice", e))?;
    inspectable.cast().map_err(|e| win_err("IDirect3DDevice", e))
}

pub fn item_for_monitor(monitor: HMONITOR) -> AppResult<GraphicsCaptureItem> {
    let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .map_err(|e| win_err("capture interop", e))?;
    // SAFETY: valid monitor handle.
    unsafe { interop.CreateForMonitor(monitor) }.map_err(|e| win_err("CreateForMonitor", e))
}

pub fn item_for_window(hwnd: HWND) -> AppResult<GraphicsCaptureItem> {
    let interop = factory::<GraphicsCaptureItem, IGraphicsCaptureItemInterop>()
        .map_err(|e| win_err("capture interop", e))?;
    // SAFETY: window handle from our own enumeration; the API validates it.
    unsafe { interop.CreateForWindow(hwnd) }.map_err(|e| win_err("CreateForWindow", e))
}

/// Starts a capture session, waits for the first frame and copies it to CPU memory.
pub fn grab(item: &GraphicsCaptureItem) -> AppResult<Frame> {
    // SAFETY: COM must be initialized on this worker thread; "already initialized" is fine.
    let _ = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
    let d3d = d3d_device()?;
    let device = winrt_device(&d3d)?;
    let size = item.Size().map_err(|e| win_err("item size", e))?;
    let pool = Direct3D11CaptureFramePool::CreateFreeThreaded(
        &device,
        DirectXPixelFormat::B8G8R8A8UIntNormalized,
        1,
        size,
    )
    .map_err(|e| win_err("frame pool", e))?;
    let session = pool.CreateCaptureSession(item).map_err(|e| win_err("capture session", e))?;
    // Optional properties: not available on every Windows 10 build — ignore if unsupported.
    let _ = session.SetIsCursorCaptureEnabled(false);
    let _ = session.SetIsBorderRequired(false);

    let (tx, rx) = mpsc::sync_channel::<Direct3D11CaptureFrame>(1);
    let handler =
        TypedEventHandler::<Direct3D11CaptureFramePool, windows::core::IInspectable>::new(move |pool, _| {
            if let Some(pool) = pool.as_ref() {
                if let Ok(frame) = pool.TryGetNextFrame() {
                    let _ = tx.try_send(frame);
                }
            }
            Ok(())
        });
    let token = pool.FrameArrived(&handler).map_err(|e| win_err("FrameArrived", e))?;
    session.StartCapture().map_err(|e| win_err("StartCapture", e))?;
    let frame = rx.recv_timeout(Duration::from_secs(3));
    let _ = pool.RemoveFrameArrived(token);
    let _ = session.Close();
    let frame =
        frame.map_err(|_| AppError::Platform("no frame received from Windows.Graphics.Capture".into()))?;
    let result = copy_to_cpu(&d3d, &frame);
    let _ = frame.Close();
    let _ = pool.Close();
    result
}

fn copy_to_cpu(d3d: &ID3D11Device, frame: &Direct3D11CaptureFrame) -> AppResult<Frame> {
    let surface = frame.Surface().map_err(|e| win_err("frame surface", e))?;
    let access: IDirect3DDxgiInterfaceAccess = surface.cast().map_err(|e| win_err("surface access", e))?;
    // SAFETY: all D3D calls use valid interfaces and properly sized descriptors; the mapped
    // memory is read only between Map and Unmap.
    unsafe {
        let texture: ID3D11Texture2D = access.GetInterface().map_err(|e| win_err("surface texture", e))?;
        let mut desc = D3D11_TEXTURE2D_DESC::default();
        texture.GetDesc(&mut desc);
        desc.Usage = D3D11_USAGE_STAGING;
        desc.BindFlags = 0;
        desc.CPUAccessFlags = D3D11_CPU_ACCESS_READ.0 as u32;
        desc.MiscFlags = 0;
        let mut staging = None;
        d3d.CreateTexture2D(&desc, None, Some(&mut staging)).map_err(|e| win_err("staging texture", e))?;
        let staging = staging.ok_or_else(|| AppError::Platform("staging texture missing".into()))?;
        let ctx = d3d.GetImmediateContext().map_err(|e| win_err("device context", e))?;
        ctx.CopyResource(&staging, &texture);
        let mut mapped = D3D11_MAPPED_SUBRESOURCE::default();
        ctx.Map(&staging, 0, D3D11_MAP_READ, 0, Some(&mut mapped)).map_err(|e| win_err("Map", e))?;
        let stride = mapped.RowPitch as usize;
        let data = std::slice::from_raw_parts(mapped.pData as *const u8, stride * desc.Height as usize);
        let result = Frame::from_strided(desc.Width, desc.Height, stride, data);
        ctx.Unmap(&staging, 0);
        result
    }
}
