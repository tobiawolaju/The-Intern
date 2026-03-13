# The Intern: PC-Embodied AI Agent

**Status:** Prototype / Research  

**Focus:** Autonomous PC-bound AI assistant with real-time UI navigation, voice interaction, and high-level command execution across devices.

**Short Description:**  
`pc-embodied-ai-agent : Autonomous PC-bound AI agent with real-time UI navigation and voice interaction using Gemini APIs.`

---

## **Overview**

**The Intern** is designed to act as your **“eye, ear, and hand”** when you’re away. It can:

- Observe your PC screen and desktop apps.  
- Listen to audio messages or receive text inputs.  
- Execute high-level commands autonomously on any GUI-enabled device.  
- Proactively perform tasks, explore tools, and interact with applications.  
- Capture screenshots, annotate, and send responses without direct supervision.  
- Use AI tools (e.g., code assistants like Codex) to vibe code or automate workflows.  

Think of it like a more intelligent and autonomous “Claw” robot for software — capable of seeing, reasoning, acting, and learning in real-time.  

---

## **Core Capabilities**

1. **UI Navigator (Hands & Eyes)**
   - Observes screens using screenshot capture (`nut.js`).  
   - Understands UI elements via Gemini Multimodal.  
   - Performs mouse/keyboard automation to control apps.  

2. **Live Agent (Voice & Ears)**
   - Listens to audio messages, transcribes using STT, and interacts in real-time.  
   - Interruptible dialogue handling using Gemini Live API.  

3. **Autonomous Planner**
   - Receives high-level commands like: “Check WhatsApp and summarize unread messages.”  
   - Updates `scheduled_tasks.txt` and knowledge base.  
   - Can proactively execute recurring or scheduled tasks.  

4. **Reply & Feedback**
   - Text or annotated screenshot replies.  
   - Can attach visual context (highlighted UI regions) for clarity.  

5. **Multi-Device Capability**
   - Any GUI-enabled device can be controlled remotely through a connected local agent client.  

6. **Safety & Isolation**
   - Runs inside Docker sandbox.  
   - Only verified scripts executed.  
   - Encrypted communication with cloud brain.  

---

## **Architecture**

```
flowchart TB
    %%-----------------------------
    %% Local PC Body
    %%-----------------------------
    subgraph PC_Body["Local PC Body"]
        direction TB
        A[Screen Capture] -->|Frames & Regions| B[Local Agent Client]
        C[Audio Input] -->|Audio Stream| B
        B -->|Mouse / Keyboard / TTS| D[Apps / GUI]
        E[Local Storage] --> B
        F[Docker Sandbox] --> B
        G[Config.json] --> B
    end

    %%-----------------------------
    %% Cloud Brain
    %%-----------------------------
    subgraph Cloud_Brain["Cloud Brain (Google Cloud)"]
        direction TB
        H[Gemini Live API] --> I[Reasoning & Dialogue]
        J[Gemini Multimodal] --> K[UI Interpretation & Action Plan]
        I --> L[Task Planner / Autonomy Module]
        K --> L
        L --> M[High-Level Action Commands to PC]
        N[Cloud Storage / Logs] --> L
    end

    %%-----------------------------
    %% Communication
    %%-----------------------------
    B -->|Compressed frames & audio events| Cloud_Brain
    Cloud_Brain -->|"JSON commands: click, type, screenshot, TTS"| B

````

**Description of Layers:**

1. **PC Body (Local Agent)**

   * **Screen Capture:** Captures windowed or full-screen screenshots of configured apps.
   * **Audio Input:** Microphone or system audio for STT.
   * **Local Agent Client:** Receives commands, executes actions, updates local memory.
   * **Docker Sandbox:** Ensures secure, isolated execution.
   * **Config.json:** Specifies which apps to monitor, input/output preferences, and device info.

2. **Cloud Brain (Gemini + Google Cloud)**

   * **Gemini Live API:** Real-time reasoning for audio/text interactions.
   * **Gemini Multimodal:** Interprets screen images and outputs structured UI actions.
   * **Task Planner:** Maintains high-level goals, schedules tasks, updates knowledge base.
   * **Action Command Generator:** Sends structured commands to the local agent.
   * **Cloud Storage / Logs:** Optional session storage and long-term memory.

3. **Communication Layer**

   * WebSocket / Pub/Sub channel between PC client and cloud brain.
   * TLS encryption for all data transfers.
   * Compressed frame and audio streaming for efficiency.

---

## **Memory Architecture**

| Memory Type           | Purpose                                  | Implementation                                                |
| --------------------- | ---------------------------------------- | ------------------------------------------------------------- |
| **Short-Term Memory** | Immediate context for screen/audio       | Rolling buffer in RAM (1-2 min)                               |
| **Long-Term Memory**  | Knowledge base, task history, embeddings | MariaDB / Cloud SQL: `tasks`, `knowledge_base`, `action_logs` |
| **Local Cache**       | Temporary files, screenshots, task txt   | `/tmp/screenshots/`, `scheduled_tasks.txt`                    |

**MariaDB Example Schema:**

```sql
CREATE DATABASE agent_memory;

USE agent_memory;

CREATE TABLE tasks (
    id INT AUTO_INCREMENT PRIMARY KEY,
    task_name VARCHAR(255),
    status ENUM('pending','completed','in_progress'),
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP,
    updated_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP ON UPDATE CURRENT_TIMESTAMP
);

CREATE TABLE knowledge_base (
    id INT AUTO_INCREMENT PRIMARY KEY,
    topic VARCHAR(255),
    content TEXT,
    vector_embedding BLOB,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE action_logs (
    id INT AUTO_INCREMENT PRIMARY KEY,
    action_type VARCHAR(255),
    target VARCHAR(255),
    outcome TEXT,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

---

## **Configuration (`config.json`)**

Example for specifying apps and behavior:

```json
{
  "apps": [
    {
      "name": "WhatsApp",
      "type": "desktop",
      "window_title": "WhatsApp",
      "input_mode": ["text", "notifications", "audio"],
      "reply_mode": ["text", "screenshots"]
    },
    {
      "name": "Discord",
      "type": "desktop",
      "window_title": "Discord",
      "input_mode": ["text", "notifications"],
      "reply_mode": ["text"]
    }
  ],
  "screen_capture": {
    "fps": 3,
    "resolution": "720p"
  },
  "docker": {
    "sandbox": true
  }
}
```

---

## **Workflow**

```text
[User sends high-level command or message] 
        |
        v
[Local Agent detects app window / notification] 
        |
        v
[Capture screen/audio → compress → send to Cloud Brain]
        |
        v
[Gemini Live + Multimodal reasoning → generate action plan]
        |
        v
[Local Agent executes actions: type, click, capture screenshot, TTS]
        |
        v
[Optional proactive tasks / annotations]
```

* Supports real-time interruptions.
* Can reply autonomously with **text or screenshots**.
* Proactively monitors apps when idle.

---

## **Installation & Setup**

### Prerequisites

* Node.js 20+
* Docker
* MariaDB
* Gemini Live API & Gemini Multimodal access via Google ADK

### Installation

```bash
git clone https://github.com/<your-username>/The-Intern.git
cd The-Intern
docker-compose up -d
npm install
```

### Configuration

* Fill `.env` with Gemini API keys, MariaDB credentials, WebSocket port, etc.
* Update `config.json` to specify apps, input/output modes.

### Running

```bash
# Start local PC-body client
npm run start-client

# Start cloud brain
npm run start-brain
```

---

## **Folder Structure**

```text
The-Intern/
├─ client/              # Local PC agent
│  ├─ eyes/             # Screen capture modules
│  ├─ ears/             # Audio input/STT
│  ├─ hands/            # Mouse/keyboard automation
│  ├─ mouth/            # TTS output
│  └─ local_agent.js
├─ cloud/               # Cloud brain (Gemini + task planner)
│  ├─ live_agent/       # Gemini Live integration
│  ├─ multimodal/       # Gemini Multimodal UI interpretation
│  ├─ planner/          # Autonomy and task planning
│  └─ brain_server.js
├─ db/                  # MariaDB schema & migrations
├─ logs/                # Local logs and screenshots
├─ docker-compose.yml
├─ config.json
├─ package.json
└─ README.md
```

---

## **Security & Safety**

* Docker sandbox for local execution.
* Verified scripts only.
* TLS-encrypted communication.
* Optional human approval for high-risk actions.

---

## **Future Improvements**

* Multi-device orchestration.
* Vector embeddings for short-term memory and context-aware UI actions.
* Advanced UI region annotation and screenshot summarization.
* Autonomous learning for repetitive workflows.

---

## **License**

MIT License
