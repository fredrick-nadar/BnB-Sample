# Handsfree — Repository Audit & Feature Preservation Inventory

**Date:** 2026-10-10  
**Target Environment:** Windows 10/11 x64 (Rust 1.99.0, Python 3.12.9)  
**Specification Reference:** `HANDSFREE_MASTER_IMPLEMENTATION_PROMPT_RUST_BACKEND.txt`

---

## 1. Executive Summary

Handsfree merges the multimodal vision and desktop interaction capabilities of **GestureAI_Mouse** with the modular agent-kernel orchestration principles of **AIOS** into a high-performance, memory-safe, Rust-first accessibility system.

This document records the exact state of both source repositories at the time of integration, analyzes their licenses and dependencies, identifies reusable components, highlights security vulnerabilities and technical debt in the source code, and establishes the migration matrix.

---

## 2. Upstream Repository Inventory

### 2.1 AIOS
- **Repository:** `https://github.com/agiresearch/AIOS`
- **Verified Revision:** Commit `292d98f78c835499df18cdb6de6325bba1895862` (Branch `main`)
- **License:** Apache License 2.0
- **Entry Points:**
  - Python: `runtime/launch.py`
  - Rust Scaffold: `aios-rs/src/lib.rs`
- **Architectural Findings:**
  - The Python implementation is an experimental research framework containing heavy dependencies (`litellm`, `chromadb`, `llama_index==0.10.19`, `git+https://github.com/agiresearch/Cerebrum.git`, `redis`, `qdrant-client`, `fastembed`, and optional CUDA packages). It is not suitable for a low-latency desktop accessibility tool.
  - The Rust implementation (`aios-rs`, Edition 2024, `version = "0.1.0"`) establishes a clean trait design for `Scheduler`, `LLMAdapter`, `MemoryManager`, `StorageManager`, `ContextManager`, and `ToolManager`. Currently, it contains minimal in-memory stubs (`NoopScheduler`, `InMemoryMemoryManager`, `EchoLLM`).
- **Reuse Strategy:**
  - **Adopt and expand** the clean Rust trait architecture from `aios-rs` directly into `crates/handsfree-ai` and `crates/handsfree-core`.
  - Provide asynchronous, production-ready Rust implementations of task scheduling, tool execution, context snapshotting, and memory notes.
  - Omit the heavy Python AIOS dependencies entirely to preserve system responsiveness and small binary footprint.

---

### 2.2 GestureAI_Mouse
- **Repository:** `https://github.com/Saket8538/GestureAI_Mouse`
- **Verified Revision:** Commit `f7ff775f8449698d0a2a2bfcc60bcdb825a90fc8` (Branch `main`)
- **License:** MIT License (Copyright 2026 Saket Kumar)
- **Entry Points:**
  - `src/main.py` (Tkinter GUI launcher)
  - `src/app.py` (Eel Chromium chat window)
- **Module Inventory & Evaluation:**
  - `src/eye.py`: MediaPipe Face Mesh iris landmarks (474–477), Eye Aspect Ratio (EAR) blink detection, 2-stage EMA smoothing, gaze auto-calibration, dwell clicking (1.5s in 35px radius), nose-tilt scroll. **[HIGH REUSE VALUE - Algorithms preserved in python worker]**
  - `src/Gesture_Controller.py`: MediaPipe Hands, finger state bitmask, gesture classifications (`V_GEST` cursor, `FIST` drag, `MID` click, `INDEX` right-click, `TWO_FINGER_CLOSED` double-click, `PINCH_MINOR` scroll, `PINCH_MAJOR` volume/brightness). **[HIGH REUSE VALUE - Preserved in python worker]**
  - `src/Gesture_Controller_Gloved.py`: HSV color-space segmentation alternative for users wearing gloves. **[PRESERVED AS OPTIONAL WORKER]**
  - `src/samvk.py`: OpenCV + cvzone virtual on-screen keyboard with hover and pinch typing (<30px) with 0.4s cooldown. **[HIGH REUSE VALUE - Preserved and enhanced with dwell typing]**
  - `src/Proton.py`: Comprehensive deterministic command dictionary (`APP_MAP`, `SITE_MAP`, browser hotkeys, window control, media, clipboard, weather, wiki, maps). **[HIGH REUSE VALUE - Ported to native Rust]**
  - `src/ai_brain.py`: LLM intent parser supporting Gemini, OpenAI, Azure OpenAI, Claude, and Ollama with structured JSON schema. **[HIGH REUSE VALUE - Ported to crates/handsfree-ai]**
  - `src/action_executor.py`: Action allowlist (`_ALLOWED_TYPES`, `_SAFE_KEYS`). **[PORTED TO RUST POLICY ENGINE]**
  - `src/mode_hub.py`: Active mode tracking singleton. **[PORTED TO RUST MODALITY ARBITRATOR]**
- **Critical Architectural Flaws to Remediate:**
  1. *Direct OS Injection inside CV Loops:* Computer vision threads call `pyautogui.click()` and `pycaw` directly without rate limiting, policy validation, or cancellation.
  2. *Command Injection Vulnerability:* `_open_anything()` executes `os.system(f'start {name}')`.
  3. *Unauthenticated WebSockets:* Eel opens unauthenticated local websocket ports.
  4. *Zero Automated Tests:* All verification was previously manual.

---

## 3. Feature Preservation Matrix

| ID | Feature | Original Source | Handsfree Target Architecture | Status |
|---|---|---|---|---|
| **F-01** | Iris Gaze & Cursor Smoothing | `src/eye.py` | `adapters/python-vision/workers/eye_worker.py` -> `crates/handsfree-core` | Ported to unprivileged worker emitting `NormalizedEvent::GazeMove` |
| **F-02** | Blink & Dwell Interaction | `src/eye.py` | `adapters/python-vision/workers/eye_worker.py` -> `crates/handsfree-core` | Algorithmic EAR preserved; dwell state arbitrated in Rust |
| **F-03** | Head Tilt Scrolling | `src/eye.py` | `adapters/python-vision/workers/eye_worker.py` | Nose-tip landmark tracking emitting `NormalizedEvent::Scroll` |
| **F-04** | Hand Gesture Recognition | `src/Gesture_Controller.py` | `adapters/python-vision/workers/gesture_worker.py` | 21-landmark finger bitmask preserved; emits `GestureDetected` |
| **F-05** | Gloved Hand Gesture Support | `src/Gesture_Controller_Gloved.py` | `adapters/python-vision/workers/gesture_gloved_worker.py` | HSV color-segmentation fallback retained |
| **F-06** | Virtual On-Screen Keyboard | `src/samvk.py` | `adapters/python-vision/workers/keyboard_worker.py` + Rust Overlay | Key layout retained; keystrokes routed through Rust Action Engine |
| **F-07** | Deterministic Shortcuts | `src/Proton.py` | `crates/handsfree-core/src/router/deterministic.rs` | Ported to native Rust; executes in <1ms without LLM |
| **F-08** | Multi-Command Chaining | `src/Proton.py` | `crates/handsfree-core/src/pipeline/macro_planner.rs` | Multi-step macro splitting ported to Rust |
| **F-09** | LLM Provider Integration | `src/ai_brain.py` | `crates/handsfree-ai/src/providers/` | Ported to Rust with strict JSON Schema enforcement |
| **F-10** | Sandboxed Action Allowlist | `src/action_executor.py` | `crates/handsfree-core/src/action/` | Typed Rust `enum Action` with compile-time safety |
| **F-11** | Mode & Conflict Arbitration | `src/mode_hub.py` | `crates/handsfree-core/src/arbitration/` | Centralized modality arbitration with emergency stop |
| **F-12** | Speech Recognition & TTS | `src/Proton.py` | `adapters/python-vision/workers/speech_worker.py` | Connects over versioned IPC with interruptible TTS |
| **F-13** | **NEW:** Semantic Browser Extension | Master Prompt Requirement | `browser-extension/` + `crates/handsfree-bridge` | Manifest V3 DOM accessibility snapshot & semantic click/fill |
| **F-14** | **NEW:** AIOS-Style Orchestration | `aios-rs` traits | `crates/handsfree-ai` & `crates/handsfree-core` | Asynchronous task scheduling, memory notes, and context snapshots |

---

## 4. Dependencies & Risk Review

| Dependency | Original Use | Handsfree Treatment | Rationale |
|---|---|---|---|
| `mediapipe` | Iris & Hand Tracking | Preserved in Python vision worker | Highest accuracy lightweight models available; isolated to unprivileged sub-process |
| `opencv-python` | Camera capture & drawing | Preserved in Python vision worker | Reliable cross-platform video capture backend |
| `pyautogui` | OS input injection | **REMOVED**; replaced with Rust `enigo` & Win32 | Memory-safe, non-blocking native injection without Python GIL locks |
| `pycaw` / `comtypes` | Windows volume control | **REMOVED**; replaced with native Rust Win32 bindings | Native Windows API via `windows` crate eliminates runtime COM crashes |
| `screen-brightness-control` | Display brightness | **REMOVED**; replaced with Rust Win32 Wmi/DXVA2 | Safer, zero Python subprocess spawn overhead |
| `eel` | Local webview | **REMOVED**; replaced with Manifest V3 Extension & Rust Desktop UI | Eliminates unauthenticated websocket security vulnerability |
| `litellm` / `chromadb` | AIOS Agent dependencies | **REMOVED**; replaced with native Rust `reqwest` & local memory | Eliminates multi-gigabyte dependency footprint and CUDA requirements |
