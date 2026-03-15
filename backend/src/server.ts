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
let agentDisconnectNotified = false;
let reconnectDelay = 1000;

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
    reconnectDelay = 1000;
    if (agentDisconnectNotified) {
      log("info", `Reconnected to local agent at ${LOCAL_AGENT_WS}`);
    } else {
      log("info", `Connected to local agent at ${LOCAL_AGENT_WS}`);
    }
    agentDisconnectNotified = false;
    broadcast(makeEvent("agent_connected", "ok", `Connected to ${LOCAL_AGENT_WS}`));
  });

  localAgent.on("close", () => {
    localAgentReady = false;
    if (!agentDisconnectNotified) {
      agentDisconnectNotified = true;
      log("warn", "Local agent disconnected. Will keep retrying silently...");
      broadcast(makeEvent("agent_disconnected", "error", "Local agent disconnected"));
    }
    scheduleReconnect();
  });

  localAgent.on("error", () => {
    localAgentReady = false;
    if (!agentDisconnectNotified) {
      agentDisconnectNotified = true;
      log("warn", "Local agent not reachable. Will keep retrying silently...");
      broadcast(makeEvent("agent_disconnected", "error", "Local agent not connected"));
    }
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
  }, reconnectDelay);
  reconnectDelay = Math.min(reconnectDelay * 2, 30000);
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

    if (!localAgentReady || !localAgent) {
      ws.send(JSON.stringify(makeEvent("agent_offline", "error", "Local agent not connected")));
      return;
    }

    // Figure out what commands to send
    let commands: any[] | null = null;

    if (msg.type === "commands" && Array.isArray(msg.commands)) {
      commands = msg.commands;
    } else if (msg.text) {
      // Try parsing text as a JSON array of commands
      try {
        const parsed = JSON.parse(msg.text);
        if (Array.isArray(parsed)) {
          commands = parsed;
        }
      } catch {
        // Not JSON — treat as a single raw instruction
        commands = [{ index: 0, instruction: msg.text, tag: "ws" }];
      }
    } else if (msg.instruction) {
      commands = [{ index: 0, instruction: msg.instruction, tag: msg.tag || "ws" }];
    }

    if (!commands) {
      ws.send(JSON.stringify({ type: "error", error: "Unknown message format" }));
      return;
    }

    log("info", "Forwarding to agent", { count: commands.length, commands });
    sendToAgent(commands);
    ws.send(JSON.stringify(makeEvent("commands_sent", "ok", `Sent ${commands.length} command(s) to agent`)));
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