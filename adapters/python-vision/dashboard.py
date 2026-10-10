"""
Handsfree Multimodal Dashboard & Control Center
Author: Handsfree Project / Pair Programming Assistant

Provides a modern, unified Graphical User Interface to individually select,
toggle, and supervise multimodal workers:
  - Eye & Gaze Tracker (MediaPipe Face Mesh + Dwell Click + Tilt Scroll)
  - Hand Gesture Controller (MediaPipe Hands + Tab Switching + Swipe)
  - Voice / Speech Assistant (Ambient SpeechRecognition + 50+ Deterministic Commands)
  - Virtual On-Screen Keyboard (Hover & Pinch Typing)

Automatically spawns and terminates worker subprocesses in the background using the
active virtual environment and monitors Rust Core connectivity in real-time.
"""

import sys
import os
import subprocess
import threading
import time
import socket
import webbrowser
import tkinter as tk
from tkinter import ttk, messagebox

# Resolve root and virtual environment paths
SCRIPT_DIR = os.path.dirname(os.path.abspath(__file__))
WORKSPACE_ROOT = os.path.dirname(os.path.dirname(SCRIPT_DIR))
VENV_PYTHON = os.path.join(SCRIPT_DIR, ".venv", "Scripts", "python.exe")
if not os.path.exists(VENV_PYTHON):
    VENV_PYTHON = sys.executable

TEST_PAGE_PATH = os.path.join(WORKSPACE_ROOT, "tests", "fixtures", "test_page.html")

# Color Palette (Dark Theme / Cyber-Slate)
BG_MAIN = "#12131C"
BG_CARD = "#1B1D2A"
BG_CARD_HOVER = "#232638"
BG_HEADER = "#161824"
BORDER_COLOR = "#2C2F45"
TEXT_PRIMARY = "#FFFFFF"
TEXT_SECONDARY = "#9DA3B4"
ACCENT_CYAN = "#00F0FF"
ACCENT_GREEN = "#00E676"
ACCENT_RED = "#FF3366"
ACCENT_YELLOW = "#FFD600"
ACCENT_PURPLE = "#7B2CBF"
BTN_DARK = "#282B3E"


class WorkerManager:
    """Manages background worker processes."""
    def __init__(self, log_callback):
        self.log_callback = log_callback
        self.processes = {}  # key -> subprocess.Popen

    def is_running(self, key):
        proc = self.processes.get(key)
        if proc is None:
            return False
        if proc.poll() is None:
            return True
        del self.processes[key]
        return False

    def start_worker(self, key, script_relpath):
        if self.is_running(key):
            self.log_callback(f"[{key}] Already running (PID: {self.processes[key].pid})")
            return True

        script_path = os.path.join(SCRIPT_DIR, script_relpath)
        if not os.path.exists(script_path):
            self.log_callback(f"[ERROR] Script not found: {script_path}")
            return False

        try:
            # Launch unprivileged worker process
            proc = subprocess.Popen(
                [VENV_PYTHON, script_path],
                cwd=SCRIPT_DIR,
                stdout=subprocess.PIPE,
                stderr=subprocess.STDOUT,
                text=True,
                bufsize=1
            )
            self.processes[key] = proc
            self.log_callback(f"[{key}] Started successfully (PID: {proc.pid})")

            # Stream output in background thread
            threading.Thread(target=self._stream_logs, args=(key, proc), daemon=True).start()
            return True
        except Exception as e:
            self.log_callback(f"[{key}] Failed to start: {e}")
            return False

    def stop_worker(self, key):
        proc = self.processes.get(key)
        if proc is not None and proc.poll() is None:
            self.log_callback(f"[{key}] Stopping (PID: {proc.pid})...")
            try:
                proc.terminate()
                try:
                    proc.wait(timeout=2.0)
                except subprocess.TimeoutExpired:
                    proc.kill()
            except Exception as e:
                self.log_callback(f"[{key}] Error stopping: {e}")
            finally:
                if key in self.processes:
                    del self.processes[key]
            self.log_callback(f"[{key}] Stopped.")
        else:
            if key in self.processes:
                del self.processes[key]

    def stop_all(self):
        for key in list(self.processes.keys()):
            self.stop_worker(key)

    def _stream_logs(self, key, proc):
        for line in proc.stdout:
            line_str = line.strip()
            if line_str:
                self.log_callback(f"[{key}] {line_str}")
        code = proc.poll()
        if code is not None and code != 0 and key in self.processes:
            self.log_callback(f"[{key}] Process exited with code {code}")


class HandsfreeDashboard(tk.Tk):
    def __init__(self):
        super().__init__()
        self.title("Handsfree Multimodal Dashboard")
        self.geometry("860x720")
        self.minsize(800, 680)
        self.configure(bg=BG_MAIN)

        self.worker_mgr = WorkerManager(self.append_log)
        self.card_widgets = {}

        self._build_ui()
        self.protocol("WM_DELETE_WINDOW", self._on_close)

        # Start periodic health monitor
        self.after(1000, self._periodic_health_check)

    def _build_ui(self):
        # 1. Header Frame
        header = tk.Frame(self, bg=BG_HEADER, bd=0, relief="flat", padx=20, pady=16)
        header.pack(fill="x", side="top")

        title_box = tk.Frame(header, bg=BG_HEADER)
        title_box.pack(side="left")

        title_lbl = tk.Label(
            title_box,
            text="HANDSFREE CONTROL CENTER",
            font=("Segoe UI", 16, "bold"),
            fg=ACCENT_CYAN,
            bg=BG_HEADER
        )
        title_lbl.pack(anchor="w")

        sub_lbl = tk.Label(
            title_box,
            text="Multimodal Accessibility Agent · Select & Launch Desired Modalities",
            font=("Segoe UI", 9),
            fg=TEXT_SECONDARY,
            bg=BG_HEADER
        )
        sub_lbl.pack(anchor="w", pady=(2, 0))

        # Core status badges in header
        status_box = tk.Frame(header, bg=BG_HEADER)
        status_box.pack(side="right", anchor="e")

        self.core_status_lbl = tk.Label(
            status_box,
            text="● CORE: CHECKING...",
            font=("Segoe UI", 8, "bold"),
            fg=ACCENT_YELLOW,
            bg="#212433",
            padx=10,
            pady=4,
            relief="flat"
        )
        self.core_status_lbl.pack(side="right", padx=(6, 0))

        self.bridge_status_lbl = tk.Label(
            status_box,
            text="● BROWSER BRIDGE: CHECKING...",
            font=("Segoe UI", 8, "bold"),
            fg=ACCENT_YELLOW,
            bg="#212433",
            padx=10,
            pady=4,
            relief="flat"
        )
        self.bridge_status_lbl.pack(side="right")

        # 2. Main Content Cards (Grid)
        content_frame = tk.Frame(self, bg=BG_MAIN, padx=20, pady=14)
        content_frame.pack(fill="both", expand=True)

        cards_grid = tk.Frame(content_frame, bg=BG_MAIN)
        cards_grid.pack(fill="x", pady=(0, 10))
        cards_grid.columnconfigure(0, weight=1)
        cards_grid.columnconfigure(1, weight=1)

        # Modality configurations
        modalities = [
            (
                "EYE",
                "👁️ Eye & Gaze Tracking",
                "Iris centroid tracking, double-EMA cursor smoothing, 1.5s dwell-clicking, and head-tilt vertical scrolling.",
                "workers/eye_worker.py",
                0, 0
            ),
            (
                "GESTURE",
                "✋ Hand Gesture Controller",
                "21 3D hand landmarks, 3-finger / swipe tab switching, index cursor pointing, and pinch-to-click.",
                "workers/gesture_worker.py",
                0, 1
            ),
            (
                "VOICE",
                "🎙️ Voice & Speech Assistant",
                "Ambient voice commands: 'open brave', 'next tab', 'scroll down', 'type ...', and 'emergency stop'.",
                "workers/speech_worker.py",
                1, 0
            ),
            (
                "KEYBOARD",
                "⌨️ Virtual On-Screen Keyboard",
                "OpenCV on-screen virtual keyboard for handsfree text input and pinch/dwell typing.",
                "workers/keyboard_worker.py",
                1, 1
            ),
        ]

        for key, name, desc, script, r, c in modalities:
            self._create_modality_card(cards_grid, key, name, desc, script, r, c)

        # 3. Global Control Bar
        global_bar = tk.Frame(content_frame, bg=BG_CARD, padx=16, pady=12, highlightthickness=1, highlightbackground=BORDER_COLOR)
        global_bar.pack(fill="x", pady=(0, 10))

        btn_start_all = tk.Button(
            global_bar,
            text="🚀 START ALL MODALITIES",
            font=("Segoe UI", 9, "bold"),
            bg=ACCENT_PURPLE,
            fg="#FFFFFF",
            activebackground="#9D4EDD",
            activeforeground="#FFFFFF",
            relief="flat",
            padx=14,
            pady=6,
            cursor="hand2",
            command=self._start_all
        )
        btn_start_all.pack(side="left", padx=(0, 8))

        btn_stop_all = tk.Button(
            global_bar,
            text="🛑 EMERGENCY STOP ALL",
            font=("Segoe UI", 9, "bold"),
            bg="#D90429",
            fg="#FFFFFF",
            activebackground="#EF233C",
            activeforeground="#FFFFFF",
            relief="flat",
            padx=14,
            pady=6,
            cursor="hand2",
            command=self._stop_all
        )
        btn_stop_all.pack(side="left", padx=(0, 8))

        btn_open_test = tk.Button(
            global_bar,
            text="🌐 OPEN TEST WEBPAGE",
            font=("Segoe UI", 9, "bold"),
            bg=BTN_DARK,
            fg=ACCENT_CYAN,
            activebackground="#33374D",
            activeforeground=ACCENT_CYAN,
            relief="flat",
            padx=14,
            pady=6,
            cursor="hand2",
            command=self._open_test_page
        )
        btn_open_test.pack(side="right")

        # 4. Console Logs Section
        log_header = tk.Frame(content_frame, bg=BG_MAIN)
        log_header.pack(fill="x", pady=(4, 2))

        log_lbl = tk.Label(
            log_header,
            text="ACTIVITY & SENSOR TELEMETRY LOG",
            font=("Segoe UI", 8, "bold"),
            fg=TEXT_SECONDARY,
            bg=BG_MAIN
        )
        log_lbl.pack(side="left")

        clear_btn = tk.Button(
            log_header,
            text="Clear",
            font=("Segoe UI", 8),
            fg=TEXT_SECONDARY,
            bg=BG_MAIN,
            relief="flat",
            command=self._clear_logs,
            cursor="hand2"
        )
        clear_btn.pack(side="right")

        self.log_text = tk.Text(
            content_frame,
            height=7,
            bg="#0D0E15",
            fg="#00E676",
            insertbackground="#00E676",
            font=("Consolas", 8),
            relief="flat",
            bd=0,
            padx=10,
            pady=8
        )
        self.log_text.pack(fill="both", expand=True)

    def _create_modality_card(self, parent, key, name, desc, script, row, col):
        card = tk.Frame(
            parent,
            bg=BG_CARD,
            padx=14,
            pady=12,
            highlightthickness=1,
            highlightbackground=BORDER_COLOR
        )
        card.grid(row=row, column=col, sticky="nsew", padx=6, pady=6)

        # Top row: Title and Status Badge
        top_row = tk.Frame(card, bg=BG_CARD)
        top_row.pack(fill="x")

        title_lbl = tk.Label(
            top_row,
            text=name,
            font=("Segoe UI", 11, "bold"),
            fg=TEXT_PRIMARY,
            bg=BG_CARD
        )
        title_lbl.pack(side="left")

        status_badge = tk.Label(
            top_row,
            text="OFFLINE",
            font=("Segoe UI", 8, "bold"),
            fg="#6C728A",
            bg="#141520",
            padx=8,
            pady=2
        )
        status_badge.pack(side="right")

        # Description
        desc_lbl = tk.Label(
            card,
            text=desc,
            font=("Segoe UI", 8),
            fg=TEXT_SECONDARY,
            bg=BG_CARD,
            wraplength=340,
            justify="left"
        )
        desc_lbl.pack(fill="x", pady=(6, 12))

        # Bottom row: Action Toggle Button
        btn_action = tk.Button(
            card,
            text="▶ START TOOL",
            font=("Segoe UI", 9, "bold"),
            bg=BTN_DARK,
            fg=ACCENT_CYAN,
            activebackground="#33374D",
            activeforeground=ACCENT_CYAN,
            relief="flat",
            padx=12,
            pady=6,
            cursor="hand2",
            command=lambda k=key, s=script: self._toggle_worker(k, s)
        )
        btn_action.pack(anchor="e")

        self.card_widgets[key] = {
            "status": status_badge,
            "btn": btn_action,
            "script": script
        }

    def _toggle_worker(self, key, script):
        if self.worker_mgr.is_running(key):
            self.worker_mgr.stop_worker(key)
        else:
            self.worker_mgr.start_worker(key, script)
        self._update_card_ui(key)

    def _update_card_ui(self, key):
        widgets = self.card_widgets.get(key)
        if not widgets:
            return
        running = self.worker_mgr.is_running(key)
        if running:
            pid = self.worker_mgr.processes[key].pid
            widgets["status"].config(text=f"ACTIVE (PID {pid})", fg=ACCENT_GREEN, bg="#0E2E1E")
            widgets["btn"].config(text="■ STOP TOOL", fg=ACCENT_RED, bg="#2E141E")
        else:
            widgets["status"].config(text="OFFLINE", fg="#6C728A", bg="#141520")
            widgets["btn"].config(text="▶ START TOOL", fg=ACCENT_CYAN, bg=BTN_DARK)

    def _start_all(self):
        for key, w in self.card_widgets.items():
            if not self.worker_mgr.is_running(key):
                self.worker_mgr.start_worker(key, w["script"])
                self._update_card_ui(key)

    def _stop_all(self):
        self.worker_mgr.stop_all()
        for key in self.card_widgets.keys():
            self._update_card_ui(key)

    def _open_test_page(self):
        if os.path.exists(TEST_PAGE_PATH):
            webbrowser.open(f"file:///{os.path.abspath(TEST_PAGE_PATH)}")
            self.append_log("[DASHBOARD] Opened accessibility test page in browser.")
        else:
            messagebox.showwarning("Not Found", f"Test page not found at:\n{TEST_PAGE_PATH}")

    def append_log(self, text):
        def _insert():
            timestamp = time.strftime("%H:%M:%S")
            self.log_text.insert("end", f"[{timestamp}] {text}\n")
            self.log_text.see("end")
        self.after(0, _insert)

    def _clear_logs(self):
        self.log_text.delete("1.0", "end")

    def _check_port(self, host, port):
        try:
            with socket.socket(socket.AF_INET, socket.SOCK_STREAM) as s:
                s.settimeout(0.2)
                return s.connect_ex((host, port)) == 0
        except Exception:
            return False

    def _periodic_health_check(self):
        # 1. Check Rust Worker IPC Port (8766)
        core_up = self._check_port("127.0.0.1", 8766)
        if core_up:
            self.core_status_lbl.config(text="● CORE IPC: ONLINE (8766)", fg=ACCENT_GREEN, bg="#0E2E1E")
        else:
            self.core_status_lbl.config(text="● CORE IPC: OFFLINE", fg=ACCENT_RED, bg="#2E141E")

        # 2. Check Browser Bridge Port (8765)
        bridge_up = self._check_port("127.0.0.1", 8765)
        if bridge_up:
            self.bridge_status_lbl.config(text="● BRIDGE WS: ONLINE (8765)", fg=ACCENT_GREEN, bg="#0E2E1E")
        else:
            self.bridge_status_lbl.config(text="● BRIDGE WS: OFFLINE", fg=ACCENT_RED, bg="#2E141E")

        # 3. Synchronize worker cards
        for key in self.card_widgets.keys():
            self._update_card_ui(key)

        self.after(1500, self._periodic_health_check)

    def _on_close(self):
        self.worker_mgr.stop_all()
        self.destroy()


if __name__ == "__main__":
    app = HandsfreeDashboard()
    app.mainloop()
