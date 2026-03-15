<div class="app-shell">
  <header class="app-header">
    <div class="app-title">
      <h2>Download Windows Local Agent</h2>
      <p>Install the local body to give The Intern hands on your PC.</p>
    </div>
    <a class="btn ghost" href="/index.html">Back to Home</a>
  </header>

  <section class="panel download-card">
    <div>
      <h3>intern-local.exe</h3>
      <p>
        This executable runs the Windows local body. It listens on
        <code>ws://127.0.0.1:8765</code> by default and accepts command messages
        from the chat interface.
      </p>
      <p class="note">
        Place the built executable in <code>frontend/public/downloads</code> to
        enable direct download from this page.
      </p>
    </div>

    <div class="download-cta">
      <a class="btn primary" href="/downloads/intern-local.exe" download>
        Download Windows Agent
      </a>
      <span class="note">If the file is missing, build it and copy it into the downloads folder.</span>
    </div>
  </section>

  <section class="panel" style="margin-top: 24px;">
    <h3>Quick Start</h3>
    <p class="note">Run the agent, then open the chat interface:</p>
    <div class="control-row">
      <span class="note">1) Double-click <code>intern-local.exe</code></span>
      <span class="note">2) Open <code>chat.html</code> and send commands</span>
    </div>
  </section>
</div>
