# Handsfree — Accessibility Test Plan & Verification Matrix

## 1. Scope & Objective
This test plan provides comprehensive validation procedures across all supported input modalities to ensure a person can operate the computer and web pages using whichever modalities are accessible to them, including users who cannot speak or cannot use hands.

---

## 2. Test Cases & Verification Procedures

### Mode 1: Voice-Only Navigation
| Test ID | Procedure | Expected Outcome | Verification |
|---|---|---|---|
| **A11Y-V1** | Speak `"open new tab"`, `"open youtube"`, `"close tab"` | Browser opens a new tab, navigates to YouTube, and closes the tab deterministically in <1ms without LLM. | Automated in `router_tests.rs` + Manual Speech |
| **A11Y-V2** | Speak `"search google for accessibility guidelines"` | Opens Google search with the exact transcribed query. | Verified |
| **A11Y-V3** | Speak `"volume up"`, `"mute"`, `"unmute"` | System volume adjusts or mutes immediately. | Verified |
| **A11Y-V4** | Speak `"close window"` | Agent halts and asks for target-specific confirmation: *"Confirm dangerous action for 'close window'?"* Does not close window until user says `"confirm"`. | Verified in `PolicyEngine` |

---

### Mode 2: Eye-Only & Gaze Navigation (Non-Speaking, No Hands)
| Test ID | Procedure | Expected Outcome | Verification |
|---|---|---|---|
| **A11Y-E1** | Look across screen quadrants | Cursor smoothly tracks iris centroid without jitter using double-EMA filtering. | Hardware Camera Check |
| **A11Y-E2** | Hold gaze still on a target for 1.5 seconds | Dwell trigger fires, executing a left-click at the anchor position. | Algorithmic test in `eye_worker.py` |
| **A11Y-E3** | Perform deliberate short blink (<0.4s) | Fires left-click without accidental double-clicks. | EAR frame test in `eye_worker.py` |
| **A11Y-E4** | Tilt head upward or glance at top 10% screen | Triggers vertical scroll up. | Verified |
| **A11Y-E5** | Look at the Emergency Stop control and dwell | Halts all input processing and clears active vision modes. | Verified in `arbitration.rs` |

---

### Mode 3: Hand Gesture Navigation (Non-Speaking)
| Test ID | Procedure | Expected Outcome | Verification |
|---|---|---|---|
| **A11Y-G1** | Show open palm with index and middle finger up (`V_GEST`) | Cursor moves smoothly following hand centroid. | Verified |
| **A11Y-G2** | Close middle finger (`MID`) while pointing | Triggers left click. | Verified |
| **A11Y-G3** | Close index finger (`INDEX`) while pointing | Triggers right click. | Verified |
| **A11Y-G4** | Close index and middle fingers (`TWO_FINGER_CLOSED`) | Triggers double click. | Verified |
| **A11Y-G5** | Pinch thumb and index finger (`PINCH_MAJOR`) | Adjusts volume or brightness proportionally to pinch movement. | Verified |

---

### Mode 4: Virtual Keyboard & Nonverbal Text Entry
| Test ID | Procedure | Expected Outcome | Verification |
|---|---|---|---|
| **A11Y-K1** | Hover index finger over a key button | Key button highlights with high-contrast color. | OpenCV Visual Verification |
| **A11Y-K2** | Pinch fingers together (<35px) over hovered key | Types the corresponding letter and applies a 0.4s debounce cooldown. | Verified |
| **A11Y-K3** | Type `"H"`, `"E"`, `"L"`, `"L"`, `"O"` | Characters appear in active focus without duplicate misfires. | Verified |

---

### Mode 5: Semantic Browser Extension Navigation
| Test ID | Procedure | Expected Outcome | Verification |
|---|---|---|---|
| **A11Y-B1** | Load `tests/fixtures/test_page.html` | Content script extracts all interactive buttons and inputs, assigning short-lived `hf-id` tags. | Verified in `bridge_integration_tests.rs` |
| **A11Y-B2** | Command: Click `"Search Catalog"` | Extension scrolls button into center view, applies accessible blue focus outline, and triggers click. | Verified |
| **A11Y-B3** | Command: Fill `"Search Keyword"` with `"screen reader"` | Extension locates input by accessible label and sets text. | Verified |
| **A11Y-B4** | Inspect password field on login form | Extension ignores password input completely; field value is never read into memory. | Verified in `semantic_dom.js` |

---

## 3. Hardware-Dependent vs. Automated Status

- **Automated Tests (Passing 100% in CI/Rust):**
  - Deterministic shortcuts, tab navigation, clipboard, and volume commands (`router_tests.rs`).
  - Macro command splitting (`router_tests.rs`).
  - Risk tier policy and target confirmation flow (`router_tests.rs`).
  - Modality priority, 250ms deduplication window, and emergency stop (`router_tests.rs`).
  - Browser bridge WebSocket handshake, token authentication, and snapshot exchange (`bridge_integration_tests.rs`).
  - AI Planner offline graceful degradation (`planner_tests.rs`).
  - Memory notes and session context management (`planner_tests.rs`).

- **Hardware-Dependent Tests (Requires Connected Camera/Mic):**
  - Physical camera FPS under low-light ambient conditions.
  - Microphone distance and noise-suppression in loud environments.
  - Eye tracking calibration accuracy on users with thick eyeglasses.
