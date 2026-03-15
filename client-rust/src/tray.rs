#[cfg(windows)]
use std::fs;
#[cfg(windows)]
use std::path::PathBuf;
#[cfg(windows)]
use base64::{engine::general_purpose::STANDARD, Engine as _};
#[cfg(windows)]
use tray_item::TrayItem;
#[cfg(windows)]
use screenshots::image;
#[cfg(windows)]
use screenshots::image::codecs::png::PngEncoder;
#[cfg(windows)]
use screenshots::image::ColorType;
#[cfg(windows)]
use screenshots::image::ImageEncoder;
#[cfg(windows)]
use screenshots::image::imageops::{overlay, resize, FilterType};

#[cfg(windows)]
use windows::Win32::System::Console::GetConsoleWindow;
#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{ShowWindow, SW_HIDE};
#[cfg(windows)]
use windows_sys::Win32::UI::WindowsAndMessaging::{
    LoadImageW as LoadImageW_sys, HICON as HICON_sys, IMAGE_ICON as IMAGE_ICON_sys,
    LR_DEFAULTSIZE as LR_DEFAULTSIZE_sys, LR_LOADFROMFILE as LR_LOADFROMFILE_sys,
};

#[cfg(windows)]
pub struct TrayHandle {
    _tray: TrayItem,
}

#[cfg(windows)]
pub fn init_tray() -> Result<TrayHandle, String> {
    hide_console();

    let icon_path = write_temp_icon()?;
    let icon = load_icon(&icon_path)?;
    let mut tray =
        TrayItem::new("Intern Local", tray_item::IconSource::RawIcon(icon)).map_err(|e| e.to_string())?;

    tray.add_label("Intern Local running")
        .map_err(|e| e.to_string())?;

    tray.add_menu_item("Quit", || {
        std::process::exit(0);
    })
    .map_err(|e| e.to_string())?;

    Ok(TrayHandle { _tray: tray })
}

#[cfg(windows)]
fn hide_console() {
    unsafe {
        let hwnd = GetConsoleWindow();
        if hwnd.0 != 0 {
            let _ = ShowWindow(hwnd, SW_HIDE);
        }
    }
}

#[cfg(windows)]
fn write_temp_icon() -> Result<PathBuf, String> {
    let icon_bytes = build_ico_bytes()?;
    let mut path = std::env::temp_dir();
    path.push("intern_local.ico");
    fs::write(&path, icon_bytes).map_err(|e| e.to_string())?;
    Ok(path)
}

#[cfg(windows)]
fn build_ico_bytes() -> Result<Vec<u8>, String> {
    let png_bytes = build_png_icon_bytes()?;
    let mut ico = Vec::with_capacity(22 + png_bytes.len());
    // ICONDIR
    ico.extend_from_slice(&0u16.to_le_bytes()); // reserved
    ico.extend_from_slice(&1u16.to_le_bytes()); // type
    ico.extend_from_slice(&1u16.to_le_bytes()); // count

    // ICONDIRENTRY
    ico.push(0); // width (0 == 256)
    ico.push(0); // height (0 == 256)
    ico.push(0); // color count
    ico.push(0); // reserved
    ico.extend_from_slice(&0u16.to_le_bytes()); // planes
    ico.extend_from_slice(&0u16.to_le_bytes()); // bitcount
    ico.extend_from_slice(&(png_bytes.len() as u32).to_le_bytes()); // size
    ico.extend_from_slice(&22u32.to_le_bytes()); // offset

    ico.extend_from_slice(&png_bytes);
    Ok(ico)
}

#[cfg(windows)]
fn build_png_icon_bytes() -> Result<Vec<u8>, String> {
    const TARGET_SIZE: u32 = 256;
    const FALLBACK_PNG: &str = "iVBORw0KGgoAAAANSUhEUgAAAAEAAAABCAQAAAC1HAwCAAAAC0lEQVR4nGNgYAAAAAMAAWgmWQ0AAAAASUVORK5CYII=";

    let source_bytes: &[u8] = include_bytes!(concat!(env!("CARGO_MANIFEST_DIR"), "/logo.jpg"));
    let decoded = image::load_from_memory(source_bytes)
        .map_err(|e| format!("icon decode error: {e}"))?;

    let rgba = decoded.to_rgba8();
    let (src_w, src_h) = rgba.dimensions();
    if src_w == 0 || src_h == 0 {
        let png_bytes = STANDARD.decode(FALLBACK_PNG).unwrap_or_default();
        return Ok(png_bytes);
    }

    let scale = (TARGET_SIZE as f32 / src_w as f32)
        .min(TARGET_SIZE as f32 / src_h as f32);
    let new_w = (src_w as f32 * scale).round().max(1.0) as u32;
    let new_h = (src_h as f32 * scale).round().max(1.0) as u32;

    let resized = resize(&rgba, new_w, new_h, FilterType::Lanczos3);
    let mut canvas = image::RgbaImage::from_pixel(TARGET_SIZE, TARGET_SIZE, image::Rgba([0, 0, 0, 0]));
    let offset_x = ((TARGET_SIZE - new_w) / 2) as i64;
    let offset_y = ((TARGET_SIZE - new_h) / 2) as i64;
    overlay(&mut canvas, &resized, offset_x, offset_y);

    let mut png_bytes: Vec<u8> = Vec::new();
    let encoder = PngEncoder::new(&mut png_bytes);
    encoder
        .write_image(
            canvas.as_raw(),
            TARGET_SIZE,
            TARGET_SIZE,
            ColorType::Rgba8,
        )
        .map_err(|e| format!("icon encode error: {e}"))?;
    Ok(png_bytes)
}

#[cfg(windows)]
fn load_icon(path: &PathBuf) -> Result<HICON_sys, String> {
    use std::os::windows::ffi::OsStrExt;
    let wide: Vec<u16> = path.as_os_str().encode_wide().chain(std::iter::once(0)).collect();
    let hicon = unsafe {
        LoadImageW_sys(
            0,
            wide.as_ptr(),
            IMAGE_ICON_sys,
            0,
            0,
            LR_LOADFROMFILE_sys | LR_DEFAULTSIZE_sys,
        )
    };
    if hicon == 0 {
        return Err("failed to load tray icon".to_string());
    }
    Ok(hicon)
}

#[cfg(not(windows))]
pub struct TrayHandle;

#[cfg(not(windows))]
pub fn init_tray() -> Result<TrayHandle, String> {
    Ok(TrayHandle)
}
