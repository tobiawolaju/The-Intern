use crate::command::CommandItem;
use crate::logger::Logger;
use crate::runner::{now_ms, run_command};
use crate::windows::LocalBody;
use serde::Deserialize;
use std::net::TcpListener;
use tungstenite::{accept, Message};

#[derive(Debug, Deserialize)]
struct IncomingMessage {
    #[serde(rename = "type")]
    msg_type: Option<String>,
    index: Option<u32>,
    instruction: Option<String>,
    tag: Option<String>,
}

#[derive(Debug, serde::Serialize)]
struct ErrorMessage {
    #[serde(rename = "type")]
    msg_type: &'static str,
    error: String,
}

#[derive(Debug, serde::Serialize)]
struct PongMessage {
    #[serde(rename = "type")]
    msg_type: &'static str,
    ts_ms: u128,
}

pub fn serve(bind: &str, port: u16, local_body: &mut LocalBody, logger: &mut Logger) -> Result<(), String> {
    let addr = format!("{}:{}", bind, port);
    let server = TcpListener::bind(&addr).map_err(|e| format!("bind error: {e}"))?;
    println!("Intern Local listening on ws://{}", addr);

    for stream in server.incoming() {
        let stream = stream.map_err(|e| format!("accept error: {e}"))?;
        let mut ws = accept(stream).map_err(|e| format!("ws accept error: {e}"))?;
        println!("Client connected");

        loop {
            let msg = match ws.read() {
                Ok(m) => m,
                Err(err) => {
                    println!("Client disconnected: {err}");
                    break;
                }
            };

            match msg {
                Message::Text(text) => {
                    if let Err(e) = handle_text(&text, &mut ws, local_body, logger) {
                        let err_msg = ErrorMessage { msg_type: "error", error: e };
                        let _ = ws.send(Message::Text(serde_json::to_string(&err_msg).unwrap_or_default()));
                    }
                }
                Message::Close(_) => {
                    println!("Client closed connection");
                    break;
                }
                Message::Ping(payload) => {
                    let _ = ws.send(Message::Pong(payload));
                }
                _ => {}
            }
        }
    }

    Ok(())
}

fn handle_text(
    text: &str,
    ws: &mut tungstenite::WebSocket<std::net::TcpStream>,
    local_body: &mut LocalBody,
    logger: &mut Logger,
) -> Result<(), String> {
    let incoming: IncomingMessage = serde_json::from_str(text)
        .map_err(|e| format!("invalid JSON: {e}"))?;

    if let Some(t) = incoming.msg_type.as_deref() {
        if t.eq_ignore_ascii_case("ping") {
            let pong = PongMessage { msg_type: "pong", ts_ms: now_ms() };
            ws.send(Message::Text(serde_json::to_string(&pong).map_err(|e| e.to_string())?))
                .map_err(|e| e.to_string())?;
            return Ok(());
        }
    }

    let instruction = incoming
        .instruction
        .ok_or_else(|| "missing instruction".to_string())?;

    let trimmed = instruction.trim();
    if trimmed.starts_with('[') {
        let batch: Vec<CommandItem> = serde_json::from_str(trimmed)
            .map_err(|e| format!("invalid batch JSON: {e}"))?;
        for cmd in batch {
            let event = run_command(local_body, logger, &cmd)?;
            let payload = serde_json::to_string(&event).map_err(|e| e.to_string())?;
            ws.send(Message::Text(payload)).map_err(|e| e.to_string())?;
        }
        return Ok(());
    }

    let cmd = CommandItem {
        index: incoming.index.unwrap_or(0),
        instruction,
        tag: incoming.tag.unwrap_or_else(|| "ws".to_string()),
    };

    let event = run_command(local_body, logger, &cmd)?;
    let payload = serde_json::to_string(&event).map_err(|e| e.to_string())?;
    ws.send(Message::Text(payload)).map_err(|e| e.to_string())?;

    Ok(())
}
