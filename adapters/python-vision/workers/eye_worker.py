"""
Handsfree Eye & Gaze Tracking Worker
Adapted from GestureAI_Mouse (src/eye.py)
Author: Saket Kumar (MIT License) / Handsfree Project

Extracts iris landmarks (474-477), computes Eye Aspect Ratio (EAR),
applies double-EMA smoothing and deadzone filtering, and emits
normalized GazeMove, BlinkTrigger, DwellTrigger, and Scroll events to the Rust core.
"""

import cv2
import mediapipe as mp
import time
import math
import collections
import sys
import os

# Ensure parent directory is in sys.path
sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from ipc_client import HandsfreeIpcClient

try:
    import ctypes
    SYS_SCREEN_W = ctypes.windll.user32.GetSystemMetrics(0)
    SYS_SCREEN_H = ctypes.windll.user32.GetSystemMetrics(1)
except Exception:
    SYS_SCREEN_W, SYS_SCREEN_H = 1920, 1080


def run_eye_worker(camera_idx: int = 0, screen_w: int = None, screen_h: int = None):
    if screen_w is None:
        screen_w = SYS_SCREEN_W
    if screen_h is None:
        screen_h = SYS_SCREEN_H

    client = HandsfreeIpcClient()
    face_mesh = mp.solutions.face_mesh.FaceMesh(refine_landmarks=True)
    cam = cv2.VideoCapture(camera_idx)

    # Configuration constants
    DWELL_THRESHOLD = 1.5
    DWELL_RADIUS = 35
    EMA_ALPHA = 0.08
    GAZE_CALIB_FRAMES = 120
    BLINK_CONFIRM_FRAMES = 3
    SHORT_BLINK_MAX = 0.4
    MEDIAN_WINDOW = 5
    NOSE_TILT_THRESH = 0.015

    LEFT_EYE_IDX = [33, 160, 158, 133, 153, 144]
    RIGHT_EYE_IDX = [362, 385, 387, 263, 373, 380]

    def _ear(landmarks, eye_idx):
        p = [landmarks[i] for i in eye_idx]
        v1 = math.hypot(p[1].x - p[5].x, p[1].y - p[5].y)
        v2 = math.hypot(p[2].x - p[4].x, p[2].y - p[4].y)
        h = math.hypot(p[0].x - p[3].x, p[0].y - p[3].y)
        return (v1 + v2) / (2.0 * h + 1e-6)

    smooth_x, smooth_y = screen_w / 2, screen_h / 2
    smooth2_x, smooth2_y = screen_w / 2, screen_h / 2
    blink_start = None
    dwell_start = None
    dwell_anchor_x, dwell_anchor_y = None, None
    frame_count = 0

    iris_x_buf = collections.deque(maxlen=MEDIAN_WINDOW)
    iris_y_buf = collections.deque(maxlen=MEDIAN_WINDOW)

    gaze_samples = []
    iris_x_min, iris_x_max = 0.30, 0.70
    iris_y_min, iris_y_max = 0.30, 0.70
    gaze_calib_done = False
    blink_threshold = 0.21

    avg_closed_frames = 0
    nose_y_baseline = None
    nose_calib_samples = []

    print("[EyeWorker] Camera loop active. Emitting events to Rust core...")

    cv2.namedWindow("Handsfree - Eye & Gaze Tracker", cv2.WINDOW_AUTOSIZE)

    try:
        while True:
            ret, frame = cam.read()
            if not ret:
                time.sleep(0.01)
                continue

            frame = cv2.flip(frame, 1)
            fh, fw, _ = frame.shape
            rgb_frame = cv2.cvtColor(frame, cv2.COLOR_BGR2RGB)
            output = face_mesh.process(rgb_frame)
            now = time.time()
            frame_count += 1

            status_tilt = "NEUTRAL"
            dwell_prog = 0.0

            if output.multi_face_landmarks:
                landmarks = output.multi_face_landmarks[0].landmark

                # 1. Nose-tilt scroll detection
                nose_y = landmarks[1].y
                if nose_y_baseline is None:
                    nose_calib_samples.append(nose_y)
                    if len(nose_calib_samples) >= 30:
                        nose_y_baseline = sum(nose_calib_samples) / len(nose_calib_samples)
                elif abs(nose_y - nose_y_baseline) > NOSE_TILT_THRESH:
                    if nose_y < nose_y_baseline:
                        client.emit_scroll(3)  # tilted up
                        status_tilt = "SCROLL UP"
                    else:
                        client.emit_scroll(-3)  # tilted down
                        status_tilt = "SCROLL DOWN"

                # 2. Iris centroid (landmarks 474-477 for right eye, 469-472 for left eye)
                iris_x = sum(landmarks[i].x for i in range(474, 478)) / 4.0
                iris_y = sum(landmarks[i].y for i in range(474, 478)) / 4.0

                # Draw iris markers on frame
                for i in [468, 473]:  # iris center points
                    px = int(landmarks[i].x * fw)
                    py = int(landmarks[i].y * fh)
                    cv2.circle(frame, (px, py), 4, (0, 255, 255), -1)
                    cv2.circle(frame, (px, py), 9, (255, 120, 0), 1)

                if not gaze_calib_done:
                    gaze_samples.append((iris_x, iris_y))
                    if len(gaze_samples) >= GAZE_CALIB_FRAMES:
                        xs = [s[0] for s in gaze_samples]
                        ys = [s[1] for s in gaze_samples]
                        iris_x_min = max(0.0, min(xs) - 0.05)
                        iris_x_max = min(1.0, max(xs) + 0.05)
                        iris_y_min = max(0.0, min(ys) - 0.05)
                        iris_y_max = min(1.0, max(ys) + 0.05)
                        gaze_calib_done = True

                iris_x_buf.append(iris_x)
                iris_y_buf.append(iris_y)
                med_x = sorted(iris_x_buf)[len(iris_x_buf) // 2]
                med_y = sorted(iris_y_buf)[len(iris_y_buf) // 2]

                span_x = max(1e-4, iris_x_max - iris_x_min)
                span_y = max(1e-4, iris_y_max - iris_y_min)
                norm_x = (med_x - iris_x_min) / span_x
                norm_y = (med_y - iris_y_min) / span_y
                norm_x = max(0.0, min(1.0, norm_x))
                norm_y = max(0.0, min(1.0, norm_y))

                target_x = norm_x * screen_w
                target_y = norm_y * screen_h

                smooth_x += EMA_ALPHA * (target_x - smooth_x)
                smooth_y += EMA_ALPHA * (target_y - smooth_y)
                smooth2_x += EMA_ALPHA * (smooth_x - smooth2_x)
                smooth2_y += EMA_ALPHA * (smooth_y - smooth2_y)

                cur_x, cur_y = int(smooth2_x), int(smooth2_y)
                client.emit_gaze(cur_x, cur_y)

                # 3. Dwell click calculation
                if dwell_anchor_x is None:
                    dwell_anchor_x, dwell_anchor_y = cur_x, cur_y
                    dwell_start = now
                else:
                    dist = math.hypot(cur_x - dwell_anchor_x, cur_y - dwell_anchor_y)
                    if dist > DWELL_RADIUS:
                        dwell_anchor_x, dwell_anchor_y = cur_x, cur_y
                        dwell_start = now
                    else:
                        elapsed = now - dwell_start
                        dwell_prog = min(1.0, elapsed / DWELL_THRESHOLD)
                        if elapsed >= DWELL_THRESHOLD:
                            client.emit_dwell(cur_x, cur_y)
                            dwell_start = now + 1.0  # cooldown
                            cv2.putText(frame, "CLICK TRIGGERED!", (fw // 2 - 100, fh // 2),
                                        cv2.FONT_HERSHEY_SIMPLEX, 0.9, (0, 0, 255), 2)

                # 4. Blink detection via EAR
                ear_l = _ear(landmarks, LEFT_EYE_IDX)
                ear_r = _ear(landmarks, RIGHT_EYE_IDX)
                ear_avg = (ear_l + ear_r) / 2.0

                if ear_avg < blink_threshold:
                    avg_closed_frames += 1
                    if avg_closed_frames >= BLINK_CONFIRM_FRAMES and blink_start is None:
                        blink_start = now
                else:
                    if blink_start is not None:
                        duration = now - blink_start
                        client.emit_blink("both", duration)
                        blink_start = None
                    avg_closed_frames = 0

            # Render HUD Overlay
            # Top Banner
            cv2.rectangle(frame, (0, 0), (fw, 64), (20, 20, 26), -1)
            cv2.putText(frame, "HANDSFREE EYE & GAZE TRACKER", (14, 24),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.65, (0, 240, 255), 2)

            calib_str = "CALIBRATED" if gaze_calib_done else f"CALIBRATING ({len(gaze_samples)}/{GAZE_CALIB_FRAMES})"
            cv2.putText(frame, f"STATUS: {calib_str} | TILT: {status_tilt}", (14, 48),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.45, (200, 200, 200), 1)

            # Gaze cursor & dwell timer indicator on preview
            preview_cx = int((smooth2_x / screen_w) * fw)
            preview_cy = int((smooth2_y / screen_h) * fh)
            cv2.circle(frame, (preview_cx, preview_cy), 8, (0, 255, 0), 2)
            if dwell_prog > 0.05:
                dwell_r = int(12 + dwell_prog * 18)
                cv2.circle(frame, (preview_cx, preview_cy), dwell_r, (0, 240, 255), 2)
                cv2.putText(frame, f"{int(dwell_prog * 100)}%", (preview_cx + 14, preview_cy - 10),
                            cv2.FONT_HERSHEY_SIMPLEX, 0.4, (0, 240, 255), 1)

            # Footer
            cv2.rectangle(frame, (0, fh - 26), (fw, fh), (15, 15, 20), -1)
            cv2.putText(frame, "Press 'q' in this window to stop Eye Tracker", (14, fh - 8),
                        cv2.FONT_HERSHEY_SIMPLEX, 0.42, (150, 150, 160), 1)

            cv2.imshow("Handsfree - Eye & Gaze Tracker", frame)
            if cv2.waitKey(1) & 0xFF == ord('q'):
                break

    finally:
        cam.release()
        cv2.destroyAllWindows()


if __name__ == "__main__":
    run_eye_worker()

