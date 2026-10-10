@echo off
title Handsfree Multimodal Dashboard
echo ========================================================
echo   Starting Handsfree Multimodal Control Center
echo ========================================================

cd /d "%~dp0adapters\python-vision"
if exist ".venv\Scripts\python.exe" (
    start "" ".venv\Scripts\python.exe" "dashboard.py"
) else (
    start "" python "dashboard.py"
)
exit
