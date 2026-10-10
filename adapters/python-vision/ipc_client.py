"""
Handsfree Python Worker IPC Client
Connects to the Rust core over loopback TCP (127.0.0.1:8766) and emits
normalized event envelopes in line-delimited JSON.
"""

import json
import socket
import time
import uuid
from datetime import datetime, timezone
import logging

logging.basicConfig(level=logging.INFO, format="%(asctime)s [%(levelname)s] %(message)s")
log = logging.getLogger("HandsfreeIpc")


class HandsfreeIpcClient:
    def __init__(self, host: str = "127.0.0.1", port: int = 8766):
        self.host = host
        self.port = port
        self.sock = None
        self.connect()

    def connect(self):
        try:
            self.sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)
            self.sock.connect((self.host, self.port))
            log.info(f"Connected to Handsfree Rust Core at {self.host}:{self.port}")
        except Exception as e:
            log.warning(f"Could not connect to Rust core ({e}). Will retry on emit.")
            self.sock = None

    def _send_payload(self, modality: str, payload_type: str, payload_content: dict, confidence: float = 1.0):
        envelope = {
            "id": str(uuid.uuid4()),
            "timestamp": datetime.now(timezone.utc).isoformat(),
            "modality": modality,
            "confidence": confidence,
            "payload": {
                "type": payload_type,
                "payload": payload_content,
            },
        }
        line = json.dumps(envelope) + "\n"

        for attempt in range(2):
            if self.sock is None:
                self.connect()
            if self.sock:
                try:
                    self.sock.sendall(line.encode("utf-8"))
                    return True
                except Exception as e:
                    log.warning(f"IPC send error: {e}. Reconnecting...")
                    self.sock = None
        return False

    def emit_gaze(self, x: int, y: int, confidence: float = 1.0, modality: str = "Eye"):
        return self._send_payload(modality, "GazeMove", {"x": x, "y": y, "confidence": confidence}, confidence)

    def emit_dwell(self, x: int, y: int):
        return self._send_payload("Eye", "DwellTrigger", {"x": x, "y": y})

    def emit_blink(self, eye: str, duration_sec: float):
        return self._send_payload("Eye", "BlinkTrigger", {"eye": eye, "duration_sec": duration_sec})

    def emit_scroll(self, delta_y: int):
        return self._send_payload("Eye", "Scroll", {"delta_y": delta_y})

    def emit_gesture(self, gesture: str, confidence: float = 1.0):
        return self._send_payload("Gesture", "GestureDetected", {"gesture": gesture, "confidence": confidence}, confidence)

    def emit_key(self, key: str):
        return self._send_payload("VirtualKeyboard", "KeyboardInput", {"key": key})

    def emit_voice(self, text: str, confidence: float = 1.0):
        return self._send_payload("Voice", "VoicePhrase", {"text": text, "confidence": confidence}, confidence)

    def emit_emergency_halt(self):
        return self._send_payload("EmergencyStop", "EmergencyHalt", {})
