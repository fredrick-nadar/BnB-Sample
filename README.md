# Handsfree — Production Multimodal Accessibility Agent

[![Rust](https://img.shields.io/badge/rust-1.80%2B-blue.svg)](https://www.rust-lang.org)
[![Python](https://img.shields.io/badge/python-3.10%2B-brightgreen.svg)](https://www.python.org)
[![Manifest](https://img.shields.io/badge/Chrome-Manifest%20V3-yellow.svg)](https://developer.chrome.com/docs/extensions/develop)
[![License: MIT](https://img.shields.io/badge/License-MIT-green.svg)](LICENSE)

Handsfree is a production-oriented, multimodal accessibility agent featuring a **high-performance Rust-first backend core** and **unprivileged Python computer-vision workers**. It combines the multimodal interaction capabilities of [GestureAI_Mouse](https://github.com/Saket8538/GestureAI_Mouse) with the modular agent-kernel orchestration principles of [AIOS](https://github.com/agiresearch/AIOS).

Handsfree enables a person to operate their computer and web pages using whichever modalities are accessible to them:
- **Voice & Speech:** 50+ deterministic shortcuts (<1ms execution) and natural language planning.
- **Eye Gaze & Blink:** Iris tracking (MediaPipe Face Mesh), double-EMA smoothing, blink detection, and dwell clicking.
- **Hand Gestures:** 21-landmark 3D hand tracking, finger state bitmasks, pointing, dragging, clicking, and scrolling.
- **Virtual Keyboard:** On-screen OpenCV keyboard with hover and pinch typing.
- **Semantic Browser Extension:** Manifest V3 extension extracting accessibility-tree page snapshots to operate web pages semantically by accessible name rather than brittle screen coordinates.
- **Shared Action Engine & Policy:** Centralized risk classification, nonverbal confirmations for dangerous actions, and an immediate emergency stop.

---

## 1. Architecture Overview

```
+--------------------------+    +--------------------------+    +--------------------------+
|  Eye/Gaze Vision Worker  |    |  Gesture Vision Worker   |    |  Virtual Keyboard Worker |
|    (MediaPipe Face)      |    |    (MediaPipe Hands)     |    |    (OpenCV / Pinch)      |
+------------+-------------+    +------------+-------------+    +------------+-------------+
             |                               |                               |
             +-------------------------------+-------------------------------+
                                             |
                                 Normalized JSON-RPC (127.0.0.1:8766)
                                             |
+--------------------------------------------v---------------------------------------------+
|                                HANDSFREE RUST CORE                                       |
|                                                                                          |
|  - Modality Arbitrator: Priority (Emergency Stop > Gaze > Speech > Gesture)              |
|  - Deduplication Engine: Suppresses duplicate multi-modal events within 250ms             |
|  - Deterministic Router: Instant local execution of common commands without an LLM      |
|  - Macro Planner: Splits multi-step commands ("open notepad and then type hello")        |
|  - Policy & Risk Engine: Low / Moderate / High tiers with target-specific confirmations   |
|  - AIOS-Inspired Agent Subsystems: Async Task Scheduler, Memory Notes, Context Snapshots  |
|  - Native Platform Adapters: Windows Win32 API injection (replaces pyautogui)             |
|  - Local Browser Bridge: Authenticated WebSocket server (127.0.0.1:8765)                  |
+--------------------------------------------+---------------------------------------------+
                                             |
                                 Authenticated WebSocket
                                             |
                               +-------------v---------------+
                               |  Chromium Browser Extension |
                               |  - Accessibility Tree DOM   |
                               |  - High-Contrast Focus Ring |
                               |  - Semantic Click & Fill    |
                               +-----------------------------+
```

---

## 2. Prerequisites & Environment

- **Operating System:** Windows 10/11 x64 (macOS / Linux supported via Mock/Linux Platform)
- **Rust Toolchain:** Stable Rust (1.80+ / 1.99+ tested) with Cargo
- **Python:** Python 3.10 to 3.12 (Python 3.12.9 tested)
- **Browser:** Google Chrome, Microsoft Edge, Brave, or any Chromium-compatible browser

---

## 3. Quick Start & Setup

### 3.1 Clone & Build the Rust Backend

```powershell
# Clone or navigate to the workspace
cd d:\Programming\BnB-Sample

# Run compiler checks and test suite
cargo test --workspace

# Start the Handsfree desktop backend
cargo run -p handsfree-desktop
```

On startup, Handsfree displays its security token in the terminal:
```text
[INFO] Starting Handsfree Accessibility Agent (Rust-first core)...
[INFO] Bridge security token: a1b2c3d4-e5f6-7890-abcd-ef0123456789
[INFO] Browser bridge WebSocket server listening on 127.0.0.1:8765
[INFO] Worker IPC server listening on 127.0.0.1:8766
[INFO] Handsfree core active and listening. Ready for multimodal input.
```

---

### 3.2 Multimodal Control Center (Dashboard) & Sensor Workers

Instead of running each worker in a separate console, use the **Handsfree Multimodal Dashboard** to visually select, toggle, and manage tools on demand:

```powershell
# Launch the Graphical Control Center (from workspace root)
.\run_dashboard.bat

# Or directly via Python
cd adapters\python-vision
.\.venv\Scripts\python.exe dashboard.py
```

The Dashboard GUI allows you to:
- 👁️ **Eye & Gaze Tracking:** Click to start/stop gaze tracking; immediately displays camera HUD with iris landmarks, dwell progress circle, and head-tilt scrolling.
- ✋ **Hand Gestures:** Click to start/stop hand tracking with 21-landmark skeleton rendering, 3-finger / swipe tab switching, and click detection.
- 🎙️ **Voice Assistant:** Click to start/stop ambient microphone transcription for 50+ deterministic desktop & browser commands.
- ⌨️ **Virtual Keyboard:** Click to toggle OpenCV on-screen keyboard.
- 🚀 **Start All / Emergency Stop:** Unified controls with live process health monitoring and Rust core IPC status badges.

---

### 3.3 Install the Chromium Browser Extension

1. Open Chrome, Edge, or Brave and navigate to `chrome://extensions/`.
2. Enable **Developer mode** (toggle in top right).
3. Click **Load unpacked** and select the folder:
   `d:\Programming\BnB-Sample\browser-extension`
4. Click the **Handsfree** extension icon in your browser toolbar.
5. Paste the **Pairing Security Token** displayed in your Rust terminal and click **Connect & Save**.
6. The badge will change to **Connected** (Green).

To test semantic interactions, open the test fixture in your browser:
```text
file:///d:/Programming/BnB-Sample/tests/fixtures/test_page.html
```

---

## 4. Deterministic Commands (Works 100% Offline Without an LLM)

Common desktop and browser commands execute locally in **<1ms** with **zero network traffic**:

| Spoken / Typed Command | Executed Action |
|---|---|
| `new tab` / `open tab` | Opens new browser tab (`Ctrl+T`) |
| `close tab` / `shut tab` | Closes active browser tab (`Ctrl+W`) |
| `next tab` / `switch tab` | Switches to the next browser tab (`Ctrl+Tab`) |
| `previous tab` / `prev tab` | Switches to the previous browser tab (`Ctrl+Shift+Tab`) |
| `go back` / `previous page` | Browser back navigation (`Alt+Left`) |
| `go forward` / `next page` | Browser forward navigation (`Alt+Right`) |
| `zoom in` / `zoom out` | Page zoom (`Ctrl++` / `Ctrl+-`) |
| `copy` / `paste` / `cut` | Clipboard operations (`Ctrl+C`, `Ctrl+V`, `Ctrl+X`) |
| `select all` / `undo` / `redo` | Selection and history (`Ctrl+A`, `Ctrl+Z`, `Ctrl+Y`) |
| `save` / `save file` | Save active document (`Ctrl+S`) |
| `volume up` / `volume down` / `mute` | System volume controls via native Win32 |
| `scroll up` / `scroll down` | Mouse wheel vertical scrolling |
| `click` / `right click` / `double click` | Mouse button actions |
| `type <text>` | Types text into active input focus |
| `open brave` / `open brave browser` | Launches Brave Browser executable on Windows |
| `open chrome` / `open edge` | Launches Chrome or Edge browser |
| `open youtube` / `open google docs` / `open github` | Opens known website from `SITE_MAP` |
| `open notepad` / `open calculator` / `open cmd` | Launches registered application from `APP_MAP` |
| `search for <query>` / `search youtube for <query>` | Performs web search |
| `close window` / `lock screen` | High-risk actions (requests explicit confirmation) |
| `start eye control` / `stop eye control` | Toggles iris gaze tracking mode |
| `start gesture control` / `stop gesture control` | Toggles hand gesture mode |
| `start virtual keyboard` / `stop virtual keyboard` | Toggles on-screen keyboard |

### Hand Gesture Shortcuts:
- **3 Fingers Open (`Index, Mid, Ring`):** Switch to Next Tab (`Ctrl+Tab`)
- **Pinky Finger Open:** Switch to Previous Tab (`Ctrl+Shift+Tab`)
- **4 Fingers Open:** Open New Tab (`Ctrl+T`)
- **Swipe Hand Right:** Switch to Next Tab (`Ctrl+Tab`)
- **Swipe Hand Left:** Switch to Previous Tab (`Ctrl+Shift+Tab`)
- **Middle Finger Tap:** Left Click
- **Index Finger Tap:** Right Click
- **Two Fingers Closed:** Double Click
- **Pinch Thumb + Index (`PINCH_MAJOR`):** Adjust volume or brightness proportionally

---

## 5. Multi-Provider AI Configuration (.env)

For contextual, semantic, and ambiguous commands, configure your preferred LLM provider in `.env`:

```ini
# Copy .env.example to .env
cp .env.example .env

# Set provider: "none", "gemini", "openai", "claude", "groq", or "ollama"
HANDSFREE_AI_PROVIDER=gemini
GEMINI_API_KEY=your_gemini_api_key_here
```

### Supported Providers:
- **Google Gemini:** `HANDSFREE_AI_PROVIDER=gemini`, `GEMINI_API_KEY=...`
- **OpenAI:** `HANDSFREE_AI_PROVIDER=openai`, `OPENAI_API_KEY=...`, `OPENAI_MODEL=gpt-4o-mini`
- **Anthropic Claude:** `HANDSFREE_AI_PROVIDER=claude`, `ANTHROPIC_API_KEY=...`
- **Groq Cloud:** `HANDSFREE_AI_PROVIDER=groq`, `GROQ_API_KEY=...`, `GROQ_MODEL=llama-3.3-70b-versatile`
- **Ollama (Free / Local / Offline):** `HANDSFREE_AI_PROVIDER=ollama`, `OLLAMA_BASE_URL=http://localhost:11434/v1`

---

## 6. Safety, Confirmations & Emergency Stop

1. **Risk Tiers:**
   - **Low (Auto-executable):** Page reading, scrolling, opening known websites.
   - **Moderate (Notified):** Clicking, typing, opening applications.
   - **High (Mandatory Confirmation):** Closing windows, locking screen, submitting payments/forms.
2. **Target-Specific Confirmation:** Prompts state the exact target (e.g., *"Confirm dangerous action for 'close window'?"*). The user can confirm or cancel via voice (`"confirm"`, `"cancel"`), dwelling on confirmation buttons, or gesture.
3. **Emergency Stop:** Pressing **Escape**, the spacebar, or dwelling on the emergency halt control immediately disables all active vision tracking modes and cancels executing plans.

---

## 7. Testing & Quality Verification

Handsfree includes an automated test suite verifying deterministic routing, macro splitting, policy risk classification, bridge WebSocket authentication, and AI plan generation:

```powershell
# Run the complete test suite
cargo test --workspace
```

Test Results:
- `router_tests.rs`: **6 passed** (deterministic commands, clipboard, volume, site map, macro planner, policy risk classification).
- `bridge_integration_tests.rs`: **1 passed** (WebSocket bridge auth handshake, token pairing, snapshot exchange).
- `planner_tests.rs`: **2 passed** (AI planner offline graceful degradation, memory notes, context history).
- **Total: 9 passed, 0 failed, 0 warnings.**

---

## 8. License & Attribution

- **Handsfree:** Licensed under the [MIT License](LICENSE).
- **Third-Party Attribution:**
  - [GestureAI_Mouse](https://github.com/Saket8538/GestureAI_Mouse) — MIT License (Copyright 2026 Saket Kumar).
  - [AIOS](https://github.com/agiresearch/AIOS) — Apache License 2.0 (Copyright 2024-2026 AGI Research).
  - See [THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) for full license texts.
