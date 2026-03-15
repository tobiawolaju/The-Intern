#[cfg(windows)]
use enigo::{Axis, Button, Coordinate, Direction, Enigo, Key, Keyboard, Mouse, Settings};

pub struct Input {
    #[cfg(windows)]
    enigo: Enigo,
}

impl Input {
    pub fn new() -> Self {
        Self {
            #[cfg(windows)]
            enigo: Enigo::new(&Settings::default()).expect("failed to init enigo"),
        }
    }

    pub fn dispatch(&mut self, instruction: &str) -> String {
        // Fallback for unrecognized instructions.
        format!("dispatched: {}", instruction)
    }

    pub fn move_mouse(&mut self, x: i32, y: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            return Ok(format!("mouse_move_to({}, {})", x, y));
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y);
            Err("mouse_move_to is only supported on Windows".to_string())
        }
    }

    pub fn click(&mut self, x: i32, y: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            return Ok(format!("click({}, {})", x, y));
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y);
            Err("click is only supported on Windows".to_string())
        }
    }

    pub fn double_click(&mut self, x: i32, y: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Click)
                .map_err(|e| e.to_string())?;
            return Ok(format!("doubleclick({}, {})", x, y));
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y);
            Err("doubleclick is only supported on Windows".to_string())
        }
    }

    pub fn mouse_down(&mut self, x: i32, y: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Press)
                .map_err(|e| e.to_string())?;
            return Ok(format!("mousedown({}, {})", x, y));
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y);
            Err("mousedown is only supported on Windows".to_string())
        }
    }

    pub fn mouse_up(&mut self, x: i32, y: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .move_mouse(x, y, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Release)
                .map_err(|e| e.to_string())?;
            return Ok(format!("mouseup({}, {})", x, y));
        }
        #[cfg(not(windows))]
        {
            let _ = (x, y);
            Err("mouseup is only supported on Windows".to_string())
        }
    }

    pub fn drag(&mut self, x1: i32, y1: i32, x2: i32, y2: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .move_mouse(x1, y1, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Press)
                .map_err(|e| e.to_string())?;
            self.enigo
                .move_mouse(x2, y2, Coordinate::Abs)
                .map_err(|e| e.to_string())?;
            self.enigo
                .button(Button::Left, Direction::Release)
                .map_err(|e| e.to_string())?;
            return Ok(format!("drag({}, {}, {}, {})", x1, y1, x2, y2));
        }
        #[cfg(not(windows))]
        {
            let _ = (x1, y1, x2, y2);
            Err("drag is only supported on Windows".to_string())
        }
    }

    pub fn scroll(&mut self, delta: i32) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo
                .scroll(delta, Axis::Vertical)
                .map_err(|e| e.to_string())?;
            return Ok(format!("scroll({})", delta));
        }
        #[cfg(not(windows))]
        {
            let _ = delta;
            Err("scroll is only supported on Windows".to_string())
        }
    }

    pub fn type_text(&mut self, text: &str) -> Result<String, String> {
        #[cfg(windows)]
        {
            self.enigo.text(text).map_err(|e| e.to_string())?;
            return Ok(format!("type_text({})", text));
        }
        #[cfg(not(windows))]
        {
            let _ = text;
            Err("type_text is only supported on Windows".to_string())
        }
    }

    pub fn key_press(&mut self, key_name: &str) -> Result<String, String> {
        #[cfg(windows)]
        {
            let key = map_key(key_name);
            self.enigo
                .key(key, Direction::Click)
                .map_err(|e| e.to_string())?;
            return Ok(format!("key_press({})", key_name));
        }
        #[cfg(not(windows))]
        {
            let _ = key_name;
            Err("key_press is only supported on Windows".to_string())
        }
    }

    pub fn hotkey(&mut self, combo: &str) -> Result<String, String> {
        #[cfg(windows)]
        {
            let mut parts: Vec<&str> = combo.split('+').map(|s| s.trim()).filter(|s| !s.is_empty()).collect();
            if parts.is_empty() {
                return Err("hotkey requires a combo; use: hotkey: CTRL+S".to_string());
            }

            let main_key = parts.pop().unwrap();
            for p in &parts {
                match p.to_ascii_uppercase().as_str() {
                    "CTRL" | "CONTROL" => {
                        self.enigo.key(Key::Control, Direction::Press).map_err(|e| e.to_string())?
                    }
                    "SHIFT" => {
                        self.enigo.key(Key::Shift, Direction::Press).map_err(|e| e.to_string())?
                    }
                    "ALT" => self.enigo.key(Key::Alt, Direction::Press).map_err(|e| e.to_string())?,
                    "WIN" | "META" | "SUPER" => {
                        self.enigo.key(Key::Meta, Direction::Press).map_err(|e| e.to_string())?
                    }
                    _ => return Err(format!("unsupported modifier: {}", p)),
                }
            }

            let key = map_key(main_key);
            self.enigo
                .key(key, Direction::Click)
                .map_err(|e| e.to_string())?;

            for p in parts.into_iter().rev() {
                match p.to_ascii_uppercase().as_str() {
                    "CTRL" | "CONTROL" => {
                        let _ = self.enigo.key(Key::Control, Direction::Release);
                    }
                    "SHIFT" => {
                        let _ = self.enigo.key(Key::Shift, Direction::Release);
                    }
                    "ALT" => {
                        let _ = self.enigo.key(Key::Alt, Direction::Release);
                    }
                    "WIN" | "META" | "SUPER" => {
                        let _ = self.enigo.key(Key::Meta, Direction::Release);
                    }
                    _ => {}
                }
            }

            return Ok(format!("hotkey({})", combo));
        }
        #[cfg(not(windows))]
        {
            let _ = combo;
            Err("hotkey is only supported on Windows".to_string())
        }
    }
}

#[cfg(windows)]
fn map_key(name: &str) -> Key {
    match name.to_ascii_uppercase().as_str() {
        "ENTER" => Key::Return,
        "TAB" => Key::Tab,
        "ESC" | "ESCAPE" => Key::Escape,
        "SPACE" => Key::Space,
        "BACKSPACE" => Key::Backspace,
        "DELETE" => Key::Delete,
        "UP" => Key::UpArrow,
        "DOWN" => Key::DownArrow,
        "LEFT" => Key::LeftArrow,
        "RIGHT" => Key::RightArrow,
        other => Key::Unicode(other.chars().next().unwrap_or(' ')),
    }
}
