use handsfree_ai::{AIPlanner, EchoLLM, GeminiProvider, LLMAdapter, OpenAICompatibleProvider};
use handsfree_bridge::{BridgeAuthenticator, BrowserBridgeServer, WorkerIpcServer};
use handsfree_core::{
    action::{Action, ActionExecutor},
    arbitration::ModalityArbitrator,
    events::{EventPayload, NormalizedEvent},
    policy::PolicyEngine,
    router::{DeterministicRouter, MacroPlanner},
};
use handsfree_platform::{create_default_platform, PlatformAdapter};
use std::env;
use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{error, info, warn};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // 1. Initialize structured logging & load .env
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenvy::dotenv().ok();

    info!("Starting Handsfree Accessibility Agent (Rust-first core)...");

    // 2. Platform initialization
    let platform: Arc<dyn PlatformAdapter> = Arc::from(create_default_platform());
    let action_executor = Arc::new(ActionExecutor::new(platform.clone()));
    let policy_engine = Arc::new(PolicyEngine::new());
    let arbitrator = Arc::new(ModalityArbitrator::new(250));
    arbitrator.set_active(handsfree_core::events::Modality::Eye, true);
    arbitrator.set_active(handsfree_core::events::Modality::Gesture, true);
    arbitrator.set_active(handsfree_core::events::Modality::Voice, true);
    arbitrator.set_active(handsfree_core::events::Modality::VirtualKeyboard, true);
    let deterministic_router = Arc::new(DeterministicRouter::new());
    let macro_planner = Arc::new(MacroPlanner::default());

    // 3. AI Provider setup (falls back gracefully to offline mode if none configured)
    let ai_provider_name = env::var("HANDSFREE_AI_PROVIDER").unwrap_or_else(|_| "none".to_string());
    let llm_adapter: Arc<dyn LLMAdapter> = match ai_provider_name.to_lowercase().as_str() {
        "gemini" => {
            let key = env::var("GEMINI_API_KEY").unwrap_or_default();
            let model = env::var("GEMINI_MODEL").unwrap_or_else(|_| "gemini-1.5-flash".to_string());
            Arc::new(GeminiProvider::new(&key, &model))
        }
        "openai" => {
            let key = env::var("OPENAI_API_KEY").unwrap_or_default();
            let base_url = env::var("OPENAI_BASE_URL")
                .unwrap_or_else(|_| "https://api.openai.com/v1".to_string());
            let model = env::var("OPENAI_MODEL").unwrap_or_else(|_| "gpt-4o-mini".to_string());
            Arc::new(OpenAICompatibleProvider::new(&base_url, &key, &model))
        }
        "groq" => {
            let key = env::var("GROQ_API_KEY").unwrap_or_default();
            let base_url = "https://api.groq.com/openai/v1";
            let model =
                env::var("GROQ_MODEL").unwrap_or_else(|_| "llama-3.3-70b-versatile".to_string());
            Arc::new(OpenAICompatibleProvider::new(base_url, &key, &model))
        }
        "ollama" => {
            let base_url = env::var("OLLAMA_BASE_URL")
                .unwrap_or_else(|_| "http://localhost:11434/v1".to_string());
            let model = env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3.2".to_string());
            Arc::new(OpenAICompatibleProvider::new(&base_url, "", &model))
        }
        _ => {
            info!("AI Provider disabled or set to offline mode (deterministic shortcuts active)");
            Arc::new(EchoLLM)
        }
    };
    let ai_planner = Arc::new(AIPlanner::new(llm_adapter, 5));

    // 4. Local Bridge & IPC initialization
    let authenticator = BridgeAuthenticator::new();
    info!("Bridge security: Loopback 127.0.0.1 (tokenless auto-pairing active)");

    let bridge_server = Arc::new(BrowserBridgeServer::new(authenticator.clone()));
    let bridge_port: u16 = env::var("HANDSFREE_BRIDGE_PORT")
        .unwrap_or_else(|_| "8765".to_string())
        .parse()
        .unwrap_or(8765);
    bridge_server.start("127.0.0.1", bridge_port).await?;

    // 5. Worker Event Channel
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<NormalizedEvent>();
    let worker_ipc_server = WorkerIpcServer::new(event_tx.clone());
    let ipc_port: u16 = env::var("HANDSFREE_IPC_PORT")
        .unwrap_or_else(|_| "8766".to_string())
        .parse()
        .unwrap_or(8766);
    worker_ipc_server.start("127.0.0.1", ipc_port).await?;

    info!("Handsfree core active and listening. Ready for multimodal input.");

    // 6. Central Event Loop & Arbitration Pipeline
    let executor_clone = action_executor.clone();
    let policy_clone = policy_engine.clone();
    let arbitrator_clone = arbitrator.clone();
    let router_clone = deterministic_router.clone();
    let planner_clone = ai_planner.clone();
    let macro_clone = macro_planner.clone();
    let bridge_server_clone = bridge_server.clone();

    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            if !arbitrator_clone.should_process(&event) {
                continue;
            }

            match event.payload {
                EventPayload::GazeMove { x, y, .. } => {
                    let _ = executor_clone.execute(&Action::MoveCursor { x, y });
                }
                EventPayload::DwellTrigger { .. } => {
                    let act = Action::Click {
                        button: handsfree_platform::MouseButton::Left,
                    };
                    let _ = executor_clone.execute(&act);
                }
                EventPayload::BlinkTrigger {
                    eye: _eye,
                    duration_sec,
                } => {
                    if duration_sec < 0.4 {
                        let _ = executor_clone.execute(&Action::Click {
                            button: handsfree_platform::MouseButton::Left,
                        });
                    } else if duration_sec < 1.5 {
                        let _ = executor_clone.execute(&Action::Click {
                            button: handsfree_platform::MouseButton::Right,
                        });
                    }
                }
                EventPayload::Scroll { delta_y } => {
                    let _ = executor_clone.execute(&Action::Scroll { amount: delta_y });
                }
                EventPayload::KeyboardInput { key } => {
                    let _ = executor_clone.execute(&Action::Press { key });
                }
                EventPayload::GestureDetected { gesture, .. } => match gesture.as_str() {
                    "PINCH_MAJOR" | "MID" => {
                        let _ = executor_clone.execute(&Action::Click {
                            button: handsfree_platform::MouseButton::Left,
                        });
                    }
                    "INDEX" => {
                        let _ = executor_clone.execute(&Action::Click {
                            button: handsfree_platform::MouseButton::Right,
                        });
                    }
                    "TWO_FINGER_CLOSED" => {
                        let _ = executor_clone.execute(&Action::DoubleClick);
                    }
                    "TAB_NEXT" => {
                        let _ = executor_clone.execute(&Action::Hotkey {
                            keys: vec!["ctrl".to_string(), "tab".to_string()],
                        });
                    }
                    "TAB_PREV" => {
                        let _ = executor_clone.execute(&Action::Hotkey {
                            keys: vec!["ctrl".to_string(), "shift".to_string(), "tab".to_string()],
                        });
                    }
                    "TAB_NEW" => {
                        let _ = executor_clone.execute(&Action::Hotkey {
                            keys: vec!["ctrl".to_string(), "t".to_string()],
                        });
                    }
                    "TAB_CLOSE" => {
                        let _ = executor_clone.execute(&Action::Hotkey {
                            keys: vec!["ctrl".to_string(), "w".to_string()],
                        });
                    }
                    _ => {}
                },
                EventPayload::VoicePhrase { text, .. } => {
                    // Check if confirmation pending
                    if policy_clone.has_pending_confirmation() {
                        let lower = text.to_lowercase();
                        if lower.contains("confirm")
                            || lower.contains("yes")
                            || lower.contains("proceed")
                        {
                            if let Some(act) = policy_clone.resolve_confirmation(true) {
                                info!("Executing confirmed action: {:?}", act);
                                let _ = executor_clone.execute(&act);
                            }
                            continue;
                        } else if lower.contains("cancel")
                            || lower.contains("no")
                            || lower.contains("stop")
                        {
                            policy_clone.resolve_confirmation(false);
                            info!("Action cancelled by user");
                            continue;
                        }
                    }

                    // Multi-command macro splitting
                    let sub_commands = macro_clone.split_macro(&text);
                    for sub_cmd in sub_commands {
                        // 1. Deterministic resolution first (<1ms)
                        if let Some(act) = router_clone.resolve(&sub_cmd) {
                            if policy_clone.requires_confirmation(&act) {
                                let conf = policy_clone.request_confirmation(
                                    act.clone(),
                                    &format!("Confirm dangerous action for '{}'?", sub_cmd),
                                    &sub_cmd,
                                    30,
                                );
                                info!("High-risk action requires confirmation: {:?}", conf);
                            } else {
                                info!("Executing deterministic action: {:?}", act);
                                let _ = executor_clone.execute(&act);
                            }
                        } else {
                            // 2. In-page semantic browser actions (e.g. "click subscribe", "click search")
                            let lower = sub_cmd.to_lowercase();
                            let mut handled_in_page = false;

                            if lower.starts_with("click ")
                                || lower.starts_with("press ")
                                || lower.starts_with("tap ")
                                || lower.starts_with("select ")
                            {
                                let target = lower
                                    .trim_start_matches("click ")
                                    .trim_start_matches("press ")
                                    .trim_start_matches("tap ")
                                    .trim_start_matches("select ")
                                    .trim();

                                if let Some(snapshot) = bridge_server_clone.get_latest_snapshot() {
                                    if let Some(el) = snapshot.elements.iter().find(|e| {
                                        e.is_clickable && e.name.to_lowercase().contains(target)
                                    }) {
                                        info!(
                                            "In-page semantic click matched target '{}' -> element '{}' (id: {})",
                                            target, el.name, el.id
                                        );
                                        let cmd = handsfree_bridge::protocol::BridgeCommand::SemanticAction {
                                            id: uuid::Uuid::new_v4(),
                                            target_id: el.id.clone(),
                                            action_type: "click".to_string(),
                                            value: None,
                                        };
                                        if let Err(e) = bridge_server_clone.send_command(cmd) {
                                            warn!("Failed sending semantic action to browser extension: {}", e);
                                        } else {
                                            handled_in_page = true;
                                        }
                                    }
                                }
                            }

                            if handled_in_page {
                                continue;
                            }

                            if ai_provider_name != "none" {
                                // 3. Context-aware AI Planning (powered by local Ollama or configured provider)
                                let context = if let Some(snapshot) = bridge_server_clone.get_latest_snapshot() {
                                    let mut elements_summary = String::new();
                                    for el in snapshot.elements.iter().take(30) {
                                        elements_summary.push_str(&format!(
                                            "- [ID: {}, Role: {}, Name: \"{}\"]\n",
                                            el.id, el.role, el.name
                                        ));
                                    }
                                    Some(format!(
                                        "Active Webpage URL: {}\nTitle: {}\nVisible Interactive Elements:\n{}",
                                        snapshot.url, snapshot.title, elements_summary
                                    ))
                                } else {
                                    None
                                };

                                match planner_clone.plan_with_context(&sub_cmd, context.as_deref()).await {
                                    Ok(plan) => {
                                        info!("AI Planner reply: {}", plan.reply);
                                        for act in plan.actions {
                                            match &act {
                                                Action::BrowserClickElement { target_id } => {
                                                    let cmd = handsfree_bridge::protocol::BridgeCommand::SemanticAction {
                                                        id: uuid::Uuid::new_v4(),
                                                        target_id: target_id.clone(),
                                                        action_type: "click".to_string(),
                                                        value: None,
                                                    };
                                                    let _ = bridge_server_clone.send_command(cmd);
                                                }
                                                Action::BrowserFillField { target_id, text } => {
                                                    let cmd = handsfree_bridge::protocol::BridgeCommand::SemanticAction {
                                                        id: uuid::Uuid::new_v4(),
                                                        target_id: target_id.clone(),
                                                        action_type: "fill".to_string(),
                                                        value: Some(text.clone()),
                                                    };
                                                    let _ = bridge_server_clone.send_command(cmd);
                                                }
                                                _ => {
                                                    if policy_clone.requires_confirmation(&act) {
                                                        policy_clone.request_confirmation(
                                                            act.clone(),
                                                            &format!("Confirm action: {:?}", act),
                                                            &sub_cmd,
                                                            30,
                                                        );
                                                    } else {
                                                        let _ = executor_clone.execute(&act);
                                                    }
                                                }
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        error!("AI Planning error: {}", e);
                                    }
                                }
                            } else {
                                warn!("Unrecognized command in offline mode: {}", sub_cmd);
                            }
                        }
                    }
                }
                EventPayload::EmergencyHalt => {
                    arbitrator_clone.trigger_emergency_stop();
                    info!("Emergency halt triggered! All input streams halted.");
                }
            }
        }
    });

    // Wait for shutdown signal (Ctrl+C)
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received. Handsfree shutting down gracefully.");

    Ok(())
}
