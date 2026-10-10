# Handsfree — Technical Architecture & Subsystem Specification

## 1. System Overview

Handsfree is an accessibility-first, multimodal desktop and browser control system built with a **Rust-first backend core** and **unprivileged Python sensor adapters**.

### Core Architecture Diagram

```
+-----------------------------------------------------------------------------------+
|                            Input Modalities (Sensors)                             |
|  [MediaPipe FaceMesh/Gaze]   [MediaPipe Hands/Gestures]   [SpeechRecognizer]     |
+--------------------------+----------------------------+---------------------------+
                           |                            |
                           +------------+---------------+
                                        |
                            JSON-RPC over Loopback TCP
                                        |
+---------------------------------------v-------------------------------------------+
|                           HANDSFREE RUST-FIRST CORE                               |
|                                                                                   |
|  +-----------------------------------------------------------------------------+  |
|  |  Modality Arbitrator                                                        |  |
|  |  - Priority: Emergency Stop > Gaze > Speech > Gestures                      |  |
|  |  - Deduplication window: 250ms                                              |  |
|  |  - Camera device mutual exclusion & state tracking                          |  |
|  +-------------------------------------+---------------------------------------+  |
|                                        |                                          |
|  +-------------------------------------v---------------------------------------+  |
|  |  Intent Interpretation Pipeline                                             |  |
|  |  - Layer 1: Deterministic Engine (<1ms resolution, no network, no LLM)       |  |
|  |  - Layer 2: Macro Planner (splits "open notepad and then type hello")       |  |
|  |  - Layer 3: AI Planner (structured JSON schemas with bounded steps)         |  |
|  +-------------------------------------+---------------------------------------+  |
|                                        |                                          |
|  +-------------------------------------v---------------------------------------+  |
|  |  Policy Engine & Risk Model                                                 |  |
|  |  - Low: Read / Scroll / Tab Nav (Auto-execute)                               |  |
|  |  - Moderate: Click / Type / App Launch (Notified)                           |  |
|  |  - High: Close Window / Lock Screen / Submit (Explicit Nonverbal Confirm)    |  |
|  +------------------+----------------------------------+-----------------------+  |
|                     |                                  |                          |
|  +------------------v---------------+  +---------------v-----------------------+  |
|  |  Platform Adapter                |  |  Browser Local Bridge Server          |  |
|  |  - Win32 SendInput / mouse_event |  |  - Loopback WebSocket (127.0.0.1:8765)  |  |
|  |  - Win32 ShellExecuteW           |  |  - Ephemeral Auth Token Handshake     |  |
|  |  - MockPlatform for Headless CI  |  |  - DOM Accessibility Tree Snapshots   |  |
|  +----------------------------------+  +---------------+-----------------------+  |
+--------------------------------------------------------|--------------------------+
                                                         |
                                            WebSocket Frame Protocol
                                                         |
                                         +---------------v---------------+
                                         |  Chromium Browser Extension   |
                                         |  (Manifest V3 Content Script) |
                                         |  - Semantic DOM Extractor     |
                                         |  - High-Contrast Element Ring |
                                         |  - Semantic Click & Fill      |
                                         +-------------------------------+
```

---

## 2. Subsystem Descriptions

### 2.1 `crates/handsfree-core`
- **Events (`events.rs`):** Strongly-typed `NormalizedEvent` records containing timestamps, unique event UUIDs, modality tags, and payloads (`GazeMove`, `DwellTrigger`, `BlinkTrigger`, `GestureDetected`, `KeyboardInput`, `VoicePhrase`, `Scroll`, `EmergencyHalt`).
- **Arbitration (`arbitration.rs`):** Thread-safe coordinator that enforces strict modality priority, suppresses duplicate triggers within a 250ms window, and guarantees immediate nonverbal emergency halts.
- **Deterministic Router (`router/deterministic.rs`):** Evaluates voice and text phrases against an extensive dictionary of 50+ desktop and web shortcuts. Executes immediately without LLM calls or network latency.
- **Action Engine (`action.rs`):** Central allowlist of safe OS and browser operations with typed arguments.
- **Policy Engine (`policy.rs`):** Classifies actions into Risk Tiers (Low, Moderate, High), manages pending target-specific confirmations, and writes redacted structured audit entries.

### 2.2 `crates/handsfree-platform`
- **PlatformAdapter Trait (`traits.rs`):** Decouples OS input injection from higher-level logic.
- **WindowsPlatform (`windows.rs`):** Native Win32 implementation utilizing `mouse_event`, `keybd_event`, `ShellExecuteW`, and `LockWorkStation`. Replaces Python's brittle `pyautogui` and `pycaw`.
- **MockPlatform (`mock.rs`):** Memory-backed recording platform enabling 100% automated headless integration tests without moving the physical cursor or disturbing user screens.

### 2.3 `crates/handsfree-bridge`
- **BrowserBridgeServer (`browser_server.rs`):** Loopback-only WebSocket listener enforcing ephemeral token authentication.
- **Protocol (`protocol.rs`):** Versioned JSON envelope definition exchanging `PageSnapshot`, `SemanticAction`, and `ActionResult`.
- **WorkerIpcServer (`worker_ipc.rs`):** Listens on `127.0.0.1:8766` for line-delimited JSON events emitted by the Python vision and speech workers.

### 2.4 `crates/handsfree-ai`
- **Architectural Port from AIOS (`aios-rs`):**
  - `Scheduler`: Async queue and worker coordinator.
  - `MemoryManager`: User-inspectable notes (`MemoryNote`) with key-value tag associations.
  - `ContextManager`: Session snapshot and recovery.
- **Multi-Provider LLM Adapters (`llm.rs`):**
  - `OpenAICompatibleProvider`: Standard implementation supporting OpenAI, Groq Cloud, and Ollama.
  - `GeminiProvider`: Native Google Gemini generateContent API integration.
  - `EchoLLM`: Offline local test adapter.
- **AIPlanner (`planner.rs`):** Context-aware multi-step planner. Ingests live environmental snapshots (`PageSnapshot` URLs, titles, and interactive DOM elements) and plans autonomous action sequences (`screenshot`, `browser_click_element`, `browser_fill_field`, `hotkey`, `search_youtube`, etc.) executed end to end via local Ollama (`qwen2.5:latest` on NVIDIA CUDA GPU) or cloud models.

### 2.5 `adapters/python-vision/`
- **Unprivileged Workers:**
  - `eye_worker.py`: Iris centroid tracking, EAR blink calculation, double-EMA smoothing, and dwell detection (requires `mediapipe==0.10.14` for `mp.solutions.face_mesh`).
  - `gesture_worker.py`: 21 3D hand landmarks, real-time index fingertip cursor tracking with double-EMA smoothing, pinch-to-click, and tab switching gestures (`TAB_NEXT`, `TAB_PREV`, `TAB_NEW`).
  - `keyboard_worker.py`: OpenCV on-screen keyboard with hover and pinch typing.
  - `speech_worker.py`: Speech recognition and interruptible TTS.
- All workers communicate strictly through `ipc_client.py` without privileged OS access.

### 2.6 `browser-extension/`
- **Manifest V3 Extension:**
  - `semantic_dom.js`: Traverses visible DOM elements, extracts accessible names, and tags them with short-lived `data-hf-id` dataset attributes. Ignores password/credit card fields by design.
  - `action_handler.js`: Scrolls targets into center view, applies a high-contrast focus ring, and dispatches native events.
  - `service_worker.js`: Establishes loopback WebSocket connection, handles reconnects, and exchanges snapshots.
