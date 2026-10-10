"""
Handsfree Speech Recognition & TTS Worker
Adapted from GestureAI_Mouse (src/Proton.py)
Author: Saket Kumar (MIT License) / Handsfree Project

Listens to ambient microphone input, transcribes speech, and emits
normalized VoicePhrase events to the Rust Core.
"""

import speech_recognition as sr
import pyttsx3
import time
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
from ipc_client import HandsfreeIpcClient


def run_speech_worker():
    client = HandsfreeIpcClient()
    r = sr.Recognizer()
    engine = pyttsx3.init()
    engine.setProperty('rate', 160)

    print("[SpeechWorker] Microphone listener active. Calibrating ambient noise...")
    with sr.Microphone() as source:
        r.adjust_for_ambient_noise(source, duration=0.8)

    print("[SpeechWorker] Ready. Listening for voice phrases...")

    while True:
        try:
            with sr.Microphone() as source:
                audio = r.listen(source, phrase_time_limit=12)
            phrase = r.recognize_google(audio)
            if phrase:
                print(f"[SpeechWorker] Heard: {phrase}")
                client.emit_voice(phrase)
        except sr.UnknownValueError:
            pass
        except sr.RequestError as e:
            print(f"[SpeechWorker] Network recognition error: {e}")
            time.sleep(1)
        except Exception as e:
            print(f"[SpeechWorker] Listener error: {e}")
            time.sleep(0.5)


if __name__ == "__main__":
    run_speech_worker()
