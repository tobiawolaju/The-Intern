<script>
  import { onMount, onDestroy } from "svelte";

  let ws;
  let wsUrl = "ws://127.0.0.1:8765";
  let connected = false;
  let lastError = "";
  let reconnectTimer;
  let reconnectAttempts = 0;
  let shouldReconnect = true;

  let instruction = "";
  let tag = "ws";
  let index = 1;

  let events = [];
  let feedEl;
  let showMobilePanel = false;

  const connect = () => {
    if (ws && (ws.readyState === WebSocket.OPEN || ws.readyState === WebSocket.CONNECTING)) {
      return;
    }

    shouldReconnect = true;
    lastError = "";
    ws = new WebSocket(wsUrl);

    ws.addEventListener("open", () => {
      connected = true;
      reconnectAttempts = 0;
    });

    ws.addEventListener("close", () => {
      connected = false;
      scheduleReconnect();
    });

    ws.addEventListener("error", () => {
      lastError = "WebSocket connection failed";
      scheduleReconnect();
    });

    ws.addEventListener("message", (event) => {
      try {
        const data = JSON.parse(event.data);
        events = [...events, data].slice(-200);
        tickScroll();
      } catch (err) {
        events = [...events, { type: "raw", payload: event.data }].slice(-200);
        tickScroll();
      }
    });
  };

  const disconnect = () => {
    shouldReconnect = false;
    if (reconnectTimer) {
      clearTimeout(reconnectTimer);
      reconnectTimer = null;
    }
    if (ws) {
      ws.close();
      ws = null;
    }
  };

  const sendCommand = () => {
    if (!connected || !ws) return;
    if (!instruction.trim()) return;

    const payload = {
      index,
      instruction: instruction.trim(),
      tag: tag.trim() || "ws"
    };

    events = [
      ...events,
      {
        type: "outgoing",
        instruction: payload.instruction,
        tag: payload.tag,
        ts_ms: Date.now()
      }
    ].slice(-200);
    tickScroll();

    ws.send(JSON.stringify(payload));
    index += 1;
    instruction = "";
  };

  const sendPing = () => {
    if (!connected || !ws) return;
    ws.send(JSON.stringify({ type: "ping" }));
  };

  const scheduleReconnect = () => {
    if (!shouldReconnect) return;
    if (reconnectTimer) return;
    const delay = Math.min(10000, 500 * (reconnectAttempts + 1));
    reconnectAttempts += 1;
    reconnectTimer = setTimeout(() => {
      reconnectTimer = null;
      connect();
    }, delay);
  };

  const tickScroll = () => {
    requestAnimationFrame(() => {
      if (feedEl) {
        feedEl.scrollTop = feedEl.scrollHeight;
      }
    });
  };


  const commandTemplates = [
    "move <x> <y>",
    "click <x> <y>",
    "doubleclick <x> <y>",
    "mousedown <x> <y>",
    "mouseup <x> <y>",
    "drag <x1> <y1> <x2> <y2>",
    "scroll <delta>",
    "type: <text>",
    "key: <NAME>",
    "hotkey: CTRL+SHIFT+S",
    "wait <ms>",
    "screenshot <path>",
    "noop"
  ];

  $: showSuggestions = instruction.trim().startsWith("/");
  $: suggestionQuery = showSuggestions ? instruction.trim().slice(1).toLowerCase() : "";
  $: filteredTemplates = showSuggestions
    ? commandTemplates.filter((cmd) => cmd.toLowerCase().includes(suggestionQuery))
    : [];
  $: hasInput = instruction.trim().length > 0;

  const openPanel = () => {
    if (window.matchMedia && window.matchMedia("(max-width: 960px)").matches) {
      showMobilePanel = true;
    }
  };

  const closePanel = () => {
    showMobilePanel = false;
  };

  onMount(() => {
    connect();
  });

  onDestroy(() => {
    disconnect();
  });
</script>

<div class="wa-shell">
  <aside class="wa-sidebar">
    <div class="wa-brand">
      <a class="wa-home" href="/index.html" aria-label="Back to home">
        <span class="material-symbols-outlined">home</span>
      </a>
      <div>
        <h3>Home</h3>
        <p>Return to landing</p>
      </div>
    </div>

    <div class="wa-connection">
      <span class="dot" class:online={connected}></span>
      <div>
        <strong>{connected ? "Connected" : "Disconnected"}</strong>
        <small>{connected ? "Listening on 127.0.0.1:8765" : "Waiting for agent"}</small>
        <small>Auto-reconnect: {connected ? "on" : "waiting"}</small>
      </div>
    </div>

    <div class="wa-chatlist">
      <h4>Command Streams</h4>
      <div class="wa-chat-item active">
        <div>
          <strong>Local Agent</strong>
          <span>ws</span>
        </div>
        <p>Live execution events</p>
      </div>
      <div class="wa-chat-item">
        <div>
          <strong>System</strong>
          <span>ping</span>
        </div>
        <p>Health & latency checks</p>
      </div>
    </div>

    <div class="wa-settings">
      <label>
        WebSocket URL
        <input bind:value={wsUrl} placeholder="ws://127.0.0.1:8765" />
      </label>
      <label>
        Tag
        <input bind:value={tag} placeholder="ws" />
      </label>
      <div class="wa-actions">
        <button class="btn ghost" on:click={connect}>Connect</button>
        <button class="btn ghost" on:click={disconnect}>Disconnect</button>
        <button class="btn ghost" on:click={sendPing}>Ping</button>
      </div>
      {#if lastError}
        <p class="note">{lastError}</p>
      {/if}
    </div>
  </aside>

  <main class="wa-main">
    <header class="wa-header">
      <div class="wa-toolbar">
        <div>
          <button type="button" class="wa-title-btn" on:click={openPanel} aria-label="Open agent details">
            <h2>Local Agent</h2>
          </button>
        </div>
      </div>
      <div class="wa-header-actions">
        <button type="button" class="wa-icon-btn" aria-label="Voice call">
          <span class="material-symbols-outlined">call</span>
        </button>
      </div>
    </header>

    <section class="wa-feed" bind:this={feedEl}>
      {#if events.length === 0}
        <div class="wa-empty">
          <h3>No events yet</h3>
          <p>Send a command to see replies from the local body.</p>
        </div>
      {:else}
        {#each events as event}
          <div class="wa-bubble-wrap {event.type === 'outgoing' ? 'from-me' : 'from-body'}">
            <div class="wa-bubble-tag">
              {event.type === "outgoing" ? "User" : "Agent"}
            </div>
            <div
              class="wa-bubble {event.type === 'outgoing' ? 'from-me' : 'from-body'} {event.status === 'error' ? 'error' : 'ok'}"
            >
            {#if event.type === "outgoing"}
              <strong>{event.instruction}</strong>
              <small>Tag: {event.tag}</small>
            {:else if event.type === "pong"}
              <strong>Pong</strong>
              <small>ts_ms: {event.ts_ms}</small>
            {:else if event.type === "error"}
              <strong>Error</strong>
              <small>{event.error}</small>
            {:else if event.type === "raw"}
              <strong>Raw</strong>
              <small>{event.payload}</small>
            {:else}
              <strong>{event.instruction}</strong>
              <small>Status: {event.status} · Tag: {event.tag}</small>
              <small>Detail: {event.detail}</small>
              {#if event.screenshot_path}
                <small>Path: {event.screenshot_path}</small>
              {/if}
              {#if event.screenshot_data_url}
                <img class="wa-bubble-image" src={event.screenshot_data_url} alt="Screenshot" loading="lazy" />
              {/if}
              <small>Duration: {event.duration_ms} ms</small>
            {/if}
            </div>
          </div>
        {/each}
      {/if}
    </section>

    <div class="wa-footer">
      {#if showSuggestions}
        <div class="wa-suggest">
          <div class="wa-suggest-header">
            <span>Command templates</span>
            <small>Type to filter, click to insert</small>
          </div>
          <div class="wa-suggest-list">
            {#if filteredTemplates.length === 0}
              <div class="wa-suggest-empty">No matches</div>
            {:else}
              {#each filteredTemplates as cmd}
                <button
                  class="wa-suggest-item"
                  type="button"
                  on:click={() => (instruction = cmd)}
                >
                  {cmd}
                </button>
              {/each}
            {/if}
          </div>
        </div>
      {/if}

      <form class="wa-input" on:submit|preventDefault={sendCommand}>
      <input
        bind:value={instruction}
        placeholder="Type a command... (e.g. move 400 400)"
      />
        <button type="submit" class="wa-send" disabled={!connected} aria-label={hasInput ? "Send" : "Voice"}>
          {#if hasInput}
            <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
              <path d="M3.4 20.6 21 12 3.4 3.4l-.6 6.7 10.1 1.9-10.1 1.9.6 6.7Z" />
            </svg>
          {:else}
            <svg viewBox="0 0 24 24" aria-hidden="true" focusable="false">
              <path d="M12 14a2.5 2.5 0 0 0 2.5-2.5v-5a2.5 2.5 0 1 0-5 0v5A2.5 2.5 0 0 0 12 14Zm5-2.5a.8.8 0 0 0-1.6 0 3.4 3.4 0 1 1-6.8 0 .8.8 0 0 0-1.6 0 5 5 0 0 0 4.6 5v2.2h-2a.8.8 0 1 0 0 1.6h5.6a.8.8 0 1 0 0-1.6h-2V16.5a5 5 0 0 0 4.8-5Z" />
            </svg>
          {/if}
        </button>
      </form>
    </div>
  </main>
</div>

{#if showMobilePanel}
  <div class="wa-overlay">
    <div class="wa-panel">
      <button type="button" class="wa-panel-close" on:click={closePanel} aria-label="Close panel">
        ×
      </button>
      <div class="wa-brand panel-brand">
        <a class="wa-home" href="/index.html" aria-label="Back to home">
          <span class="material-symbols-outlined">home</span>
        </a>
        <div>
          <h3>Home</h3>
          <p>Return to landing</p>
        </div>
      </div>

      <div class="wa-connection">
        <span class="dot" class:online={connected}></span>
        <div>
          <strong>{connected ? "Connected" : "Disconnected"}</strong>
          <small>{connected ? "Listening on 127.0.0.1:8765" : "Waiting for agent"}</small>
          <small>Auto-reconnect: {connected ? "on" : "waiting"}</small>
        </div>
      </div>

      <div class="wa-chatlist">
        <h4>Command Streams</h4>
        <div class="wa-chat-item active">
          <div>
            <strong>Local Agent</strong>
            <span>ws</span>
          </div>
          <p>Live execution events</p>
        </div>
        <div class="wa-chat-item">
          <div>
            <strong>System</strong>
            <span>ping</span>
          </div>
          <p>Health & latency checks</p>
        </div>
      </div>

      <div class="wa-settings">
        <label>
          WebSocket URL
          <input bind:value={wsUrl} placeholder="ws://127.0.0.1:8765" />
        </label>
        <label>
          Tag
          <input bind:value={tag} placeholder="ws" />
        </label>
        <div class="wa-actions">
          <button class="btn ghost" on:click={connect}>Connect</button>
          <button class="btn ghost" on:click={disconnect}>Disconnect</button>
          <button class="btn ghost" on:click={sendPing}>Ping</button>
        </div>
        {#if lastError}
          <p class="note">{lastError}</p>
        {/if}
      </div>
    </div>
  </div>
{/if}
