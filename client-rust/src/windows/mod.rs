pub mod audio;
pub mod input;
pub mod screen;

use crate::command::CommandItem;
use std::thread;
use std::time::Duration;

pub struct LocalBody {
    screen: screen::ScreenCapture,
    input: input::Input,
    audio: audio::Audio,
}

impl LocalBody {
    pub fn new() -> Self {
        Self {
            screen: screen::ScreenCapture::new(),
            input: input::Input::new(),
            audio: audio::Audio::new(),
        }
    }

    pub fn execute(&mut self, cmd: &CommandItem) -> CommandResult {
        // Minimal action parser + Windows-backed actions.
        let _ = self.screen.peek();
        let _ = self.audio.peek();

        let instruction = cmd.instruction.trim();
        let ok_result = |detail: String| CommandResult {
            index: cmd.index,
            tag: cmd.tag.clone(),
            instruction: cmd.instruction.clone(),
            status: "ok".to_string(),
            detail,
            screenshot_path: None,
            screenshot_data_url: None,
        };
        let outcome = if let Some(rest) = instruction.strip_prefix("click ") {
            parse_two_ints(rest)
                .ok_or_else(|| "invalid click format; use: click <x> <y>".to_string())
                .and_then(|(x, y)| self.input.click(x, y))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("doubleclick ") {
            parse_two_ints(rest)
                .ok_or_else(|| "invalid doubleclick format; use: doubleclick <x> <y>".to_string())
                .and_then(|(x, y)| self.input.double_click(x, y))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("move ") {
            parse_two_ints(rest)
                .ok_or_else(|| "invalid move format; use: move <x> <y>".to_string())
                .and_then(|(x, y)| self.input.move_mouse(x, y))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("mousedown ") {
            parse_two_ints(rest)
                .ok_or_else(|| "invalid mousedown format; use: mousedown <x> <y>".to_string())
                .and_then(|(x, y)| self.input.mouse_down(x, y))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("mouseup ") {
            parse_two_ints(rest)
                .ok_or_else(|| "invalid mouseup format; use: mouseup <x> <y>".to_string())
                .and_then(|(x, y)| self.input.mouse_up(x, y))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("drag ") {
            parse_four_ints(rest)
                .ok_or_else(|| "invalid drag format; use: drag <x1> <y1> <x2> <y2>".to_string())
                .and_then(|(x1, y1, x2, y2)| self.input.drag(x1, y1, x2, y2))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("scroll ") {
            rest.trim()
                .parse::<i32>()
                .map_err(|_| "invalid scroll format; use: scroll <delta>".to_string())
                .and_then(|delta| self.input.scroll(delta))
                .map(ok_result)
        } else if let Some(rest) = instruction.strip_prefix("type:") {
            let text = rest.trim();
            if text.is_empty() {
                Err("type requires text; use: type: your text".to_string())
            } else {
                self.input.type_text(text).map(ok_result)
            }
        } else if let Some(rest) = instruction.strip_prefix("key:") {
            let key = rest.trim();
            if key.is_empty() {
                Err("key requires a name; use: key: ENTER".to_string())
            } else {
                self.input.key_press(key).map(ok_result)
            }
        } else if let Some(rest) = instruction.strip_prefix("hotkey:") {
            let combo = rest.trim();
            if combo.is_empty() {
                Err("hotkey requires a combo; use: hotkey: CTRL+S".to_string())
            } else {
                self.input.hotkey(combo).map(ok_result)
            }
        } else if let Some(rest) = instruction.strip_prefix("screenshot ") {
            let path = rest.trim();
            if path.is_empty() {
                Err("screenshot requires a path; use: screenshot <path>".to_string())
            } else {
                self.screen
                    .capture_to_file(path)
                    .map(|result| CommandResult {
                        index: cmd.index,
                        tag: cmd.tag.clone(),
                        instruction: cmd.instruction.clone(),
                        status: "ok".to_string(),
                        detail: result.detail,
                        screenshot_path: result.screenshot_path,
                        screenshot_data_url: result.screenshot_data_url,
                    })
            }
        } else if let Some(rest) = instruction.strip_prefix("wait ") {
            match rest.trim().parse::<u64>() {
                Ok(ms) => {
                    thread::sleep(Duration::from_millis(ms));
                    Ok(ok_result(format!("wait({})", ms)))
                }
                Err(_) => Err("invalid wait format; use: wait <ms>".to_string()),
            }
        } else if instruction.eq_ignore_ascii_case("noop") {
            Ok(ok_result("noop".to_string()))
        } else {
            Ok(ok_result(self.input.dispatch(instruction)))
        };

        match outcome {
            Ok(result) => result,
            Err(err) => CommandResult {
                index: cmd.index,
                tag: cmd.tag.clone(),
                instruction: cmd.instruction.clone(),
                status: "error".to_string(),
                detail: err,
                screenshot_path: None,
                screenshot_data_url: None,
            },
        }
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct CommandResult {
    pub index: u32,
    pub tag: String,
    pub instruction: String,
    pub status: String,
    pub detail: String,
    pub screenshot_path: Option<String>,
    pub screenshot_data_url: Option<String>,
}

fn parse_two_ints(input: &str) -> Option<(i32, i32)> {
    let mut parts = input.split_whitespace();
    let x = parts.next()?.parse::<i32>().ok()?;
    let y = parts.next()?.parse::<i32>().ok()?;
    Some((x, y))
}

fn parse_four_ints(input: &str) -> Option<(i32, i32, i32, i32)> {
    let mut parts = input.split_whitespace();
    let x1 = parts.next()?.parse::<i32>().ok()?;
    let y1 = parts.next()?.parse::<i32>().ok()?;
    let x2 = parts.next()?.parse::<i32>().ok()?;
    let y2 = parts.next()?.parse::<i32>().ok()?;
    Some((x1, y1, x2, y2))
}
