"""
Handsfree Virtual On-Screen Keyboard Worker
Adapted from GestureAI_Mouse (src/samvk.py)
Author: Saket Kumar (MIT License) / Handsfree Project

Draws on-screen keyboard layout over OpenCV frame, tracks index fingertip hover,
detects index+middle pinch typing (<30px), and emits KeyboardInput events to Rust.
"""

import cv2
import mediapipe as mp
import math
import time
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from ipc_client import HandsfreeIpcClient


def run_keyboard_worker(camera_idx: int = 0):
    client = HandsfreeIpcClient()
    mp_hands = mp.solutions.hands.Hands(max_num_hands=1, min_detection_confidence=0.8)
    cam = cv2.VideoCapture(camera_idx)

    cam.set(3, 1280)
    cam.set(4, 720)

    keys = [
        ["1", "2", "3", "4", "5", "6", "7", "8", "9", "0"],
        ["Q", "W", "E", "R", "T", "Y", "U", "I", "O", "P"],
        ["A", "S", "D", "F", "G", "H", "J", "K", "L", ";"],
        ["Z", "X", "C", "V", "B", "N", "M", ",", ".", "/"],
    ]

    last_press_time = {}
    KEY_COOLDOWN = 0.40

    print("[KeyboardWorker] Camera loop active. Emitting virtual keystrokes...")

    try:
        while True:
            ret, frame = cam.read()
            if not ret:
                time.sleep(0.01)
                continue

            frame = cv2.flip(frame, 1)
            h, w, _ = frame.shape
            rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            results = mp_hands.process(rgb)

            btn_w, btn_h = 80, 80
            start_x, start_y = 100, 100

            # Draw keys on frame
            for r_idx, row in enumerate(keys):
                for c_idx, key_char in enumerate(row):
                    bx = start_x + c_idx * (btn_w + 10)
                    by = start_y + r_idx * (btn_h + 10)
                    cv2.rectangle(frame, (bx, by), (bx + btn_w, by + btn_h), (255, 0, 255), cv2.FILLED)
                    cv2.putText(frame, key_char, (bx + 25, by + 55), cv2.FONT_HERSHEY_PLAIN, 3, (255, 255, 255), 3)

            if results.multi_hand_landmarks:
                landmarks = results.multi_hand_landmarks[0].landmark
                # Index tip = 8, Middle tip = 12
                ix, iy = int(landmarks[8].x * w), int(landmarks[8].y * h)
                mx, my = int(landmarks[12].x * w), int(landmarks[12].y * h)
                pinch_dist = math.hypot(ix - mx, iy - my)

                # Check if hovered over a button
                for r_idx, row in enumerate(keys):
                    for c_idx, key_char in enumerate(row):
                        bx = start_x + c_idx * (btn_w + 10)
                        by = start_y + r_idx * (btn_h + 10)

                        if bx < ix < bx + btn_w and by < iy < by + btn_h:
                            cv2.rectangle(frame, (bx, by), (bx + btn_w, by + btn_h), (0, 255, 0), cv2.FILLED)
                            cv2.putText(frame, key_char, (bx + 25, by + 55), cv2.FONT_HERSHEY_PLAIN, 3, (255, 255, 255), 3)

                            # Pinch detected to type
                            if pinch_dist < 35:
                                now = time.time()
                                if key_char not in last_press_time or (now - last_press_time[key_char] > KEY_COOLDOWN):
                                    client.emit_key(key_char.lower())
                                    last_press_time[key_char] = now

            cv2.imshow("Handsfree Virtual Keyboard", frame)
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break
    finally:
        cam.release()
        cv2.destroyAllWindows()


if __name__ == "__main__":
    run_keyboard_worker()
