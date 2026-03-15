#[cfg(windows)]
use screenshots::Screen;
#[cfg(windows)]
use screenshots::image::codecs::png::PngEncoder;
#[cfg(windows)]
use screenshots::image::ColorType;
#[cfg(windows)]
use screenshots::image::ImageEncoder;
#[cfg(windows)]
use base64::Engine;
#[cfg(windows)]
use std::path::PathBuf;

#[derive(Default)]
pub struct ScreenCapture;

impl ScreenCapture {
    pub fn new() -> Self {
        Self
    }

    pub fn peek(&self) -> String {
        "screen_ok".to_string()
    }

    pub fn capture_to_file(&self, path: &str) -> Result<ScreenCaptureResult, String> {
        #[cfg(windows)]
        {
            let output_path = normalize_output_path(path)?;
            if let Some(parent) = output_path.parent() {
                if !parent.as_os_str().is_empty() {
                    std::fs::create_dir_all(parent)
                        .map_err(|e| format!("create dir error: {e}"))?;
                }
            }
            let screens = Screen::all().map_err(|e| format!("screen list error: {e}"))?;
            let screen = screens
                .first()
                .ok_or_else(|| "no screens found".to_string())?;
            let image = screen.capture().map_err(|e| format!("capture error: {e}"))?;
            let mut png_bytes: Vec<u8> = Vec::new();
            let encoder = PngEncoder::new(&mut png_bytes);
            encoder
                .write_image(
                    image.as_raw(),
                    image.width(),
                    image.height(),
                    ColorType::Rgba8,
                )
                .map_err(|e| format!("encode error: {e}"))?;
            std::fs::write(&output_path, &png_bytes).map_err(|e| format!("save error: {e}"))?;
            let data_url = format!(
                "data:image/png;base64,{}",
                base64::engine::general_purpose::STANDARD.encode(&png_bytes)
            );
            return Ok(ScreenCaptureResult {
                detail: format!("screenshot_saved: {}", output_path.display()),
                screenshot_path: Some(output_path.display().to_string()),
                screenshot_data_url: Some(data_url),
            });
        }
        #[cfg(not(windows))]
        {
            let _ = path;
            Err("capture_to_file is only supported on Windows".to_string())
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct ScreenCaptureResult {
    pub detail: String,
    pub screenshot_path: Option<String>,
    pub screenshot_data_url: Option<String>,
}

#[cfg(windows)]
fn normalize_output_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err("screenshot requires a path; use: screenshot <path>".to_string());
    }
    let mut output_path = PathBuf::from(trimmed);
    match output_path.extension().and_then(|ext| ext.to_str()) {
        None | Some("") => {
            output_path.set_extension("png");
        }
        Some(ext) if ext.eq_ignore_ascii_case("png") => {}
        Some(_) => {
            return Err("unsupported screenshot format; use .png".to_string());
        }
    }
    Ok(output_path)
}
