import "dotenv/config";
import { WebSocket, WebSocketServer } from "ws";

const PORT = Number(process.env.BACKEND_PORT || 8787);
const LOCAL_AGENT_WS = process.env.LOCAL_AGENT_WS || "ws://127.0.0.1:8765";
const GEMINI_API_KEY = process.env.GEMINI_API_KEY || "";
const GEMINI_MODEL = process.env.GEMINI_MODEL || "gemini-flash-latest";
const SCREENSHOT_PATH =
  process.env.SCREENSHOT_PATH || "C:\\temp\\intern-proof.png";
const LOG_LEVEL = (process.env.LOG_LEVEL || "info").toLowerCase();

type LogLevel = "debug" | "info" | "warn" | "error";
const LOG_LEVELS: Record<LogLevel, number> = {
  debug: 10,
  info: 20,
  warn: 30,
  error: 40
};

function log(level: LogLevel, message: string, meta?: unknown) {
  if (LOG_LEVELS[level] < (LOG_LEVELS[LOG_LEVEL as LogLevel] || 20)) return;
  const stamp = new Date().toISOString();
  if (meta !== undefined) {
    console.log(`[${stamp}] [${level.toUpperCase()}] ${message}`, meta);
  } else {
    console.log(`[${stamp}] [${level.toUpperCase()}] ${message}`);
  }
}

if (!GEMINI_API_KEY) {
  log("warn", "Missing GEMINI_API_KEY; LLM calls will fail until set.");
}

const wss = new WebSocketServer({ port: PORT });
log("info", `Backend listening on ws://127.0.0.1:${PORT}`);

let localAgent: WebSocket | null = null;
let localAgentReady = false;
let reconnectTimer: NodeJS.Timeout | null = null;

const frontendClients = new Set<WebSocket>();

function connectLocalAgent() {
  if (localAgent && (localAgent.readyState === WebSocket.OPEN || localAgent.readyState === WebSocket.CONNECTING)) {
    return;
  }

  localAgentReady = false;
  localAgent = new WebSocket(LOCAL_AGENT_WS);

  localAgent.on("open", () => {
    localAgentReady = true;
    log("info", `Connected to local agent at ${LOCAL_AGENT_WS}`);
    broadcast(makeEvent("agent_connected", "ok", `Connected to ${LOCAL_AGENT_WS}`));
  });

  localAgent.on("close", () => {
    localAgentReady = false;
    log("warn", "Local agent disconnected");
    broadcast(makeEvent("agent_disconnected", "error", "Local agent disconnected"));
    scheduleReconnect();
  });

  localAgent.on("error", (err) => {
    localAgentReady = false;
    log("error", "Local agent error", err);
    broadcast(makeEvent("agent_error", "error", "Local agent error"));
    scheduleReconnect();
  });

  localAgent.on("message", (data) => {
    let payload: unknown;
    try {
      payload = JSON.parse(data.toString());
    } catch {
      payload = { type: "raw", payload: data.toString() };
    }
    log("debug", "Local agent -> backend", payload);
    broadcast(payload);

    if (isScreenshotEvent(payload)) {
      broadcast(makeEvent("done", "ok", "Done. Screenshot attached."));
    }
  });
}

function scheduleReconnect() {
  if (reconnectTimer) return;
  reconnectTimer = setTimeout(() => {
    reconnectTimer = null;
    connectLocalAgent();
  }, 1000);
}

connectLocalAgent();

wss.on("connection", (ws) => {
  frontendClients.add(ws);
  ws.send(JSON.stringify(makeEvent("backend_ready", "ok", "Backend connected")));
  log("info", "Frontend connected");

  ws.on("close", () => {
    frontendClients.delete(ws);
    log("info", "Frontend disconnected");
  });

  ws.on("message", async (data) => {
    let msg: any;
    try {
      msg = JSON.parse(data.toString());
    } catch {
      ws.send(JSON.stringify({ type: "error", error: "Invalid JSON" }));
      log("warn", "Invalid JSON from frontend", data.toString());
      return;
    }
    log("debug", "Frontend -> backend", msg);

    if (msg.type === "ping") {
      ws.send(JSON.stringify({ type: "pong", ts_ms: Date.now() }));
      return;
    }

    const text = (msg.text || msg.instruction || "").toString().trim();
    if (!text) {
      ws.send(JSON.stringify({ type: "error", error: "Missing text" }));
      log("warn", "Missing text from frontend", msg);
      return;
    }

    if (!localAgentReady || !localAgent) {
      ws.send(JSON.stringify(makeEvent("agent_offline", "error", "Local agent not connected")));
      return;
    }

    try {
      log("info", "Generating commands", { text });
      const commands = await generateCommands(text);
      const withScreenshot = appendScreenshot(commands);

      ws.send(JSON.stringify(makeEvent("ai_plan", "ok", `Generated ${commands.length} commands`)));

      const payload = {
        index: 0,
        instruction: JSON.stringify(withScreenshot),
        tag: "ws"
      };

      log("debug", "Backend -> local agent (batch)", payload);
      localAgent.send(JSON.stringify(payload));
    } catch (err: any) {
      ws.send(JSON.stringify(makeEvent("ai_error", "error", err?.message || "LLM error")));
      log("error", "LLM error", err?.message || err);
    }
  });
});

function broadcast(event: unknown) {
  const message = JSON.stringify(event);
  for (const client of frontendClients) {
    if (client.readyState === WebSocket.OPEN) {
      client.send(message);
    }
  }
}

function makeEvent(instruction: string, status: "ok" | "error", detail: string) {
  return {
    ts_ms: Date.now(),
    duration_ms: 0,
    index: 0,
    tag: "ai",
    instruction,
    status,
    detail
  };
}

function isScreenshotEvent(payload: any): boolean {
  return Boolean(
    payload &&
      payload.instruction &&
      typeof payload.instruction === "string" &&
      payload.instruction.startsWith("screenshot ") &&
      payload.status === "ok" &&
      payload.screenshot_data_url
  );
}

async function generateCommands(userText: string) {
  const system =
    "You are a command generator for a Windows automation client. " +
    "Return ONLY a JSON array. Each item must have: index (number), instruction (string), tag (string). " +
    "Allowed instructions: move, click, doubleclick, mousedown, mouseup, drag, scroll, type:, key:, hotkey:, wait. " +
    "Use hotkey: CTRL+ESC to open Start (not WIN). " +
    "Use waits (500-800ms) between UI steps. " +
    "Example for 'open notepad' should open Start, type notepad, press enter.";

  const prompt = `User request: ${userText}`;

  const body = {
    systemInstruction: { parts: [{ text: system }] },
    contents: [{ role: "user", parts: [{ text: prompt }] }],
    generationConfig: {
      temperature: 0.2,
      maxOutputTokens: 512
    }
  };

  const url = `https://generativelanguage.googleapis.com/v1beta/models/${GEMINI_MODEL}:generateContent?key=${GEMINI_API_KEY}`;
  log("debug", "Gemini request", { model: GEMINI_MODEL });
  const res = await fetch(url, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body)
  });

  if (!res.ok) {
    const text = await res.text();
    throw new Error(`Gemini API error: ${res.status} ${text}`);
  }

  const json = await res.json();
  const text = json?.candidates?.[0]?.content?.parts?.[0]?.text || "";
  log("debug", "Gemini response text", text);
  const parsed = parseJsonArray(text);
  validateCommands(parsed);
  return parsed;
}

function parseJsonArray(text: string) {
  const trimmed = text.trim();
  if (trimmed.startsWith("[")) {
    return JSON.parse(trimmed);
  }

  const start = trimmed.indexOf("[");
  const end = trimmed.lastIndexOf("]");
  if (start !== -1 && end !== -1 && end > start) {
    const slice = trimmed.slice(start, end + 1);
    return JSON.parse(slice);
  }

  throw new Error("LLM did not return a JSON array");
}

function validateCommands(commands: any[]) {
  if (!Array.isArray(commands) || commands.length === 0) {
    throw new Error("No commands generated");
  }
  for (const cmd of commands) {
    if (!cmd || typeof cmd.instruction !== "string") {
      throw new Error("Invalid command shape from LLM");
    }
  }
}

function appendScreenshot(commands: any[]) {
  const index = commands.length + 1;
  return [
    ...commands,
    { index, instruction: `screenshot ${SCREENSHOT_PATH}`, tag: "ws" }
  ];
}
