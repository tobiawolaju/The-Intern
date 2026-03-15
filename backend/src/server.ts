import "dotenv/config";
import { WebSocket, WebSocketServer } from "ws";

const PORT = Number(process.env.BACKEND_PORT || 8787);
const LOCAL_AGENT_WS = process.env.LOCAL_AGENT_WS || "ws://127.0.0.1:8765";
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

/* ── WebSocket server ─────────────────────────────── */

const wss = new WebSocketServer({ port: PORT });
log("info", `Backend listening on ws://127.0.0.1:${PORT}`);

let localAgent: WebSocket | null = null;
let localAgentReady = false;
let reconnectTimer: NodeJS.Timeout | null = null;

const frontendClients = new Set<WebSocket>();

/* ── Local agent connection ───────────────────────── */

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

/* ── Frontend connection handler ──────────────────── */

wss.on("connection", (ws) => {
  frontendClients.add(ws);
  ws.send(JSON.stringify(makeEvent("backend_ready", "ok", "Backend connected")));
  log("info", "Frontend connected");

  ws.on("close", () => {
    frontendClients.delete(ws);
    log("info", "Frontend disconnected");
  });

  ws.on("message", (data) => {
    let msg: any;
    try {
      msg = JSON.parse(data.toString());
    } catch {
      ws.send(JSON.stringify({ type: "error", error: "Invalid JSON" }));
      log("warn", "Invalid JSON from frontend", data.toString());
      return;
    }

    // Ping/pong keepalive
    if (msg.type === "ping") {
      ws.send(JSON.stringify({ type: "pong", ts_ms: Date.now() }));
      return;
    }

    // Expect the frontend to send commands directly
    // Supported shapes:
    //   { type: "commands", commands: [ {index, instruction, tag}, ... ] }
    //   { instruction: "...", tag: "..." }   (single command shorthand)
    //   { text: "..." }                      (raw text → forwarded as-is)

    if (!localAgentReady || !localAgent) {
      ws.send(JSON.stringify(makeEvent("agent_offline", "error", "Local agent not connected")));
      return;
    }

    if (msg.type === "commands" && Array.isArray(msg.commands)) {
      // Batch of commands from frontend
      log("info", "Received commands from frontend", { count: msg.commands.length, commands: msg.commands });
      sendToAgent(msg.commands);
      ws.send(JSON.stringify(makeEvent("commands_sent", "ok", `Sent ${msg.commands.length} commands to agent`)));
      return;
    }

    if (msg.instruction) {
      // Single command shorthand
      const command = { index: 0, instruction: msg.instruction, tag: msg.tag || "ws" };
      log("info", "Received single command from frontend", command);
      sendToAgent(command);
      ws.send(JSON.stringify(makeEvent("commands_sent", "ok", `Sent command: ${msg.instruction}`)));
      return;
    }

    if (msg.text) {
      // Raw text — wrap as a single instruction and forward
      const command = { index: 0, instruction: msg.text, tag: "ws" };
      log("info", "Received text from frontend, forwarding to agent", { text: msg.text });
      sendToAgent(command);
      ws.send(JSON.stringify(makeEvent("commands_sent", "ok", `Forwarded: ${msg.text}`)));
      return;
    }

    ws.send(JSON.stringify({ type: "error", error: "Unknown message format" }));
    log("warn", "Unknown message format from frontend", msg);
  });
});

/* ── Helpers ──────────────────────────────────────── */

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

function sendToAgent(commands: any[] | object) {
  if (!localAgentReady || !localAgent) {
    broadcast(makeEvent("agent_offline", "error", "Local agent not connected"));
    return;
  }
  const payload = {
    index: 0,
    instruction: JSON.stringify(commands),
    tag: "ws"
  };
  log("debug", "Backend -> local agent", payload);
  localAgent.send(JSON.stringify(payload));
}