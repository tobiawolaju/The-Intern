#[derive(Default)]
pub struct Audio;

impl Audio {
    pub fn new() -> Self {
        Self
    }

    pub fn peek(&self) -> String {
        // Stub: return placeholder status.
        "audio_ok".to_string()
    }
}
