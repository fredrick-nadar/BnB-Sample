"""
Handsfree Hand Gesture & Cursor Tracking Worker
Adapted from GestureAI_Mouse (src/Gesture_Controller.py)
Author: Saket Kumar (MIT License) / Handsfree Project

Extracts 21 3D hand landmarks via MediaPipe Hands:
  - Tracks index fingertip (landmark 8) for real-time cursor movement with double-EMA smoothing
  - Pinch (thumb + index) triggers Left Click
  - Middle tap triggers Left Click
  - Two fingers closed triggers Double Click
  - 3 Fingers Up / Swipe Right triggers Next Tab (Ctrl+Tab)
  - Pinky Finger / Swipe Left triggers Previous Tab (Ctrl+Shift+Tab)
  - 4 Fingers Up triggers New Tab (Ctrl+T)
"""

import cv2
import mediapipe as mp
import math
import time
from enum import IntEnum
import sys
import os

try:
    import ctypes
    SCREEN_W = ctypes.windll.user32.GetSystemMetrics(0)
    SCREEN_H = ctypes.windll.user32.GetSystemMetrics(1)
except Exception:
    SCREEN_W, SCREEN_H = 1920, 1080

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from ipc_client import HandsfreeIpcClient


class Gest(IntEnum):
    FIST = 0
    PINKY = 1
    RING = 2
    MID = 4
    LAST3 = 7
    INDEX = 8
    FIRST2 = 12
    LAST4 = 15
    THUMB = 16
    PALM = 31
    V_GEST = 33
    TWO_FINGER_CLOSED = 34
    PINCH_MAJOR = 35
    PINCH_MINOR = 36


GEST_NAME_MAP = {
    Gest.FIST: "FIST",
    Gest.MID: "MID",
    Gest.INDEX: "INDEX",
    Gest.TWO_FINGER_CLOSED: "TWO_FINGER_CLOSED",
    Gest.V_GEST: "V_GEST",
    Gest.PINCH_MAJOR: "PINCH_MAJOR",
    Gest.PINCH_MINOR: "PINCH_MINOR",
    Gest.PALM: "PALM",
    Gest.LAST3: "TAB_NEXT",      # 3 fingers (Index, Mid, Ring) -> Next Tab (Ctrl+Tab)
    Gest.PINKY: "TAB_PREV",      # Pinky finger up -> Previous Tab (Ctrl+Shift+Tab)
    Gest.LAST4: "TAB_NEW",       # 4 fingers up -> New Tab (Ctrl+T)
}


class HandRecognizer:
    def __init__(self):
        self.finger = 0
        self.prev_gesture = Gest.PALM
        self.frame_count = 0
        self.hand_result = None

    def update(self, hand_landmarks):
        self.hand_result = hand_landmarks

    def get_signed_dist(self, p1_idx, p2_idx):
        lm = self.hand_result.landmark
        sign = 1 if lm[p1_idx].y < lm[p2_idx].y else -1
        dist = math.hypot(lm[p1_idx].x - lm[p2_idx].x, lm[p1_idx].y - lm[p2_idx].y)
        return dist * sign

    def get_dist(self, p1_idx, p2_idx):
        lm = self.hand_result.landmark
        return math.hypot(lm[p1_idx].x - lm[p2_idx].x, lm[p1_idx].y - lm[p2_idx].y)

    def set_finger_state(self):
        if not self.hand_result:
            return
        points = [[8, 5], [12, 9], [16, 13], [20, 17]]
        self.finger = 0
        for p in points:
            dist = self.get_signed_dist(p[0], p[1])
            self.finger = (self.finger << 1) | (1 if dist > 0.05 else 0)

    def get_gesture(self):
        if not self.hand_result:
            return Gest.PALM

        self.set_finger_state()
        curr = self.finger

        if self.get_dist(8, 4) < 0.05:
            curr = Gest.PINCH_MAJOR
        elif self.finger == Gest.FIRST2:
            d1 = self.get_dist(8, 12)
            d2 = self.get_dist(5, 9)
            if d1 / (d2 + 1e-6) > 1.7:
                curr = Gest.V_GEST
            else:
                curr = Gest.TWO_FINGER_CLOSED

        if curr == self.prev_gesture:
            self.frame_count += 1
        else:
            self.frame_count = 0

        self.prev_gesture = curr

        if self.frame_count > 2:
            return curr
        return Gest.PALM


def run_gesture_worker(camera_idx: int = 0):
    client = HandsfreeIpcClient()
    mp_hands = mp.solutions.hands.Hands(
        max_num_hands=1,
        min_detection_confidence=0.7,
        min_tracking_confidence=0.6
    )
    mp_drawing = mp.solutions.drawing_utils
    mp_drawing_styles = mp.solutions.drawing_styles
    cam = cv2.VideoCapture(camera_idx)
    recognizer = HandRecognizer()

    screen_w, screen_h = SCREEN_W, SCREEN_H
    EMA_ALPHA = 0.22  # Responsive fingertip tracking
    smooth_x, smooth_y = screen_w / 2, screen_h / 2
    smooth2_x, smooth2_y = screen_w / 2, screen_h / 2

    last_emitted = None
    last_emit_time = 0.0
    last_click_time = 0.0
    prev_wrist_x = None
    last_swipe_time = 0.0

    print(f"[GestureWorker] Screen resolution detected: {screen_w}x{screen_h}")
    print("[GestureWorker] Camera loop active. Emitting gesture & cursor movement events...")
    print("  - Index Finger Point: Real-time Cursor Movement")
    print("  - Pinch (Thumb + Index): Left Click")
    print("  - 3 Fingers Open: Next Tab (Ctrl+Tab)")
    print("  - Pinky Open: Previous Tab (Ctrl+Shift+Tab)")
    print("  - 4 Fingers Open: New Tab (Ctrl+T)")
    print("  - Swipe Right: Next Tab | Swipe Left: Previous Tab")

    cv2.namedWindow("Handsfree - Hand Gesture Controller", cv2.WINDOW_AUTOSIZE)

    try:
        while True:
            ret, frame = cam.read()
            if not ret:
                time.sleep(0.01)
                continue

            frame = cv2.flip(frame, 1)
            fh, fw, _ = frame.shape
            rgb = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            results = mp_hands.process(rgb)
            now = time.time()
            active_gesture_display = "IDLE"
            cursor_active = False

            if results.multi_hand_landmarks:
                for hand_landmarks in results.multi_hand_landmarks:
                    mp_drawing.draw_landmarks(
                        frame,
                        hand_landmarks,
                        mp.solutions.hands.HAND_CONNECTIONS,
                        mp_drawing_styles.get_default_hand_landmarks_style(),
                        mp_drawing_styles.get_default_hand_connections_style(),
                    )

                hand_landmarks = results.multi_hand_landmarks[0]
                recognizer.update(hand_landmarks)

                # 1. Real-time Cursor Movement via Index Fingertip (Landmark 8)
                lm8 = hand_landmarks.landmark[8]
                tip_px = int(lm8.x * fw)
                tip_py = int(lm8.y * fh)

                # Active interaction margins (15% padding so user easily reaches screen borders)
                norm_x = max(0.0, min(1.0, (lm8.x - 0.15) / 0.70))
                norm_y = max(0.0, min(1.0, (lm8.y - 0.15) / 0.70))

                target_x = norm_x * screen_w
                target_y = norm_y * screen_h

                smooth_x += EMA_ALPHA * (target_x - smooth_x)
                smooth_y += EMA_ALPHA * (target_y - smooth_y)
                smooth2_x += EMA_ALPHA * (smooth_x - smooth2_x)
                smooth2_y += EMA_ALPHA * (smooth_y - smooth2_y)

                cur_x, cur_y = int(smooth2_x), int(smooth2_y)

                # 2. Gesture classification
                gesture = recognizer.get_gesture()
                name = GEST_NAME_MAP.get(gesture, "UNKNOWN")

                # If pointing or navigating with fingers, stream cursor position!
                if gesture in [Gest.INDEX, Gest.V_GEST, Gest.MID, Gest.PALM] or (recognizer.finger & 0b1000):
                    client.emit_gaze(cur_x, cur_y, modality="Gesture")
                    cursor_active = True
                    cv2.circle(frame, (tip_px, tip_py), 14, (0, 255, 0), 2)
                    cv2.circle(frame, (tip_px, tip_py), 4, (0, 255, 255), -1)

                # 3. Pinch Left-Click Trigger
                pinch_dist = recognizer.get_dist(8, 4)
                if pinch_dist < 0.055:
                    active_gesture_display = "PINCH -> LEFT CLICK"
                    cv2.circle(frame, (tip_px, tip_py), 22, (0, 0, 255), 3)
                    cv2.putText(frame, "CLICK!", (tip_px + 16, tip_py),
                                cv2.FONT_HERSHEY_SIMPLEX, 0.6, (0, 0, 255), 2)
                    if now - last_click_time > 0.40:
                        client.emit_gesture("PINCH_MAJOR")
                        last_click_time = now

                # 4. Horizontal swipe motion detection for tab switching
                wrist_x = hand_landmarks.landmark[0].x
                if prev_wrist_x is not None:
                    dx = wrist_x - prev_wrist_x
                    if dx > 0.12 and (now - last_swipe_time > 0.6):
                        client.emit_gesture("TAB_NEXT")
                        last_swipe_time = now
                        active_gesture_display = "SWIPE RIGHT -> NEXT TAB"
                    elif dx < -0.12 and (now - last_swipe_time > 0.6):
                        client.emit_gesture("TAB_PREV")
                        last_swipe_time = now
                        active_gesture_display = "SWIPE LEFT -> PREV TAB"
                prev_wrist_x = wrist_x

                # 5. Discrete static poses (Tab controls, clicks)
                if name in ["TAB_NEXT", "TAB_PREV", "TAB_NEW", "TWO_FINGER_CLOSED", "MID"]:
                    active_gesture_display = f"{name}"
                    cooldown = 0.55 if name.startswith("TAB_") else 0.40
                    if name != last_emitted or (now - last_emit_time > cooldown):
                        client.emit_gesture(name)
                        last_emitted = name
                        last_emit_time = now
            else:
                last_emitted = None
                prev_wrist_x = None

            # Render HUD Overlay
            # Top Banner
            cv2.rectangle(frame, (0, 0), (fw, 64), (20, 20, 26), -1)
            cv2.putText(frame, "HANDSFREE GESTURE & CURSOR CONTROLLER", (14, 24),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.62, (0, 240, 255), 2)

            cursor_str = f"CURSOR: ({cur_x}, {cur_y})" if cursor_active else "CURSOR: IDLE"
            cv2.putText(frame, f"GESTURE: {active_gesture_display} | {cursor_str}", (14, 48),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.48, (0, 255, 0) if cursor_active else (180, 180, 180), 2)

            # Footer
            cv2.rectangle(frame, (0, fh - 28), (fw, fh), (15, 15, 20), -1)
            cv2.putText(frame, "Point: Move Cursor | Pinch: Click | 3-Fingers: Next Tab | Pinky: Prev Tab | 'q': Exit",
                        (10, fh - 8), cv2.FONT_HERSHEY_SIMPLEX, 0.38, (150, 150, 160), 1)

            cv2.imshow("Handsfree - Hand Gesture Controller", frame)
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cam.release()
        cv2.destroyAllWindows()


if __name__ == "__main__":
    run_gesture_worker()
