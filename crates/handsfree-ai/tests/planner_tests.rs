use handsfree_ai::{
    AIPlanner, ContextManager, EchoLLM, InMemoryContextManager, InMemoryMemoryManager,
    MemoryManager, MemoryNote,
};
use std::sync::Arc;

#[tokio::test]
async fn test_ai_planner_offline_mode() {
    let adapter = Arc::new(EchoLLM);
    let planner = AIPlanner::new(adapter, 5);

    let plan = planner
        .plan("hello handsfree")
        .await
        .expect("planner succeeds");

    assert_eq!(plan.actions.len(), 0);
    assert!(plan.reply.contains("Echo: hello handsfree"));
}

#[test]
fn test_memory_and_context_managers() {
    let mem = InMemoryMemoryManager::new();
    let note = MemoryNote::new("note-1", "User prefers high dwell time 2.0s");
    mem.add_memory(note).expect("note added");

    let retrieved = mem.get_memory("note-1").expect("note retrieved");
    assert_eq!(retrieved.content, "User prefers high dwell time 2.0s");

    let ctx = InMemoryContextManager::new();
    ctx.record_interaction("open youtube", "Opening YouTube in browser");
    let history = ctx.get_recent_history(5);
    assert_eq!(history.len(), 1);
    assert_eq!(history[0].0, "open youtube");
}

#[test]
fn test_parse_plan_resilient_llm_formats() {
    // Exact response that previously caused deserialization failure
    let raw = r#"{"actions": [{"type": "click", "target_id": "hf-70"}, {"type": "reply", "text": "Turning on your camera."}]}"#;
    let plan = AIPlanner::parse_plan(raw).expect("should parse resiliently");

    assert_eq!(plan.reply, "Turning on your camera.");
    assert_eq!(plan.actions.len(), 1);
    assert_eq!(
        plan.actions[0],
        handsfree_core::Action::BrowserClickElement {
            target_id: "hf-70".to_string()
        }
    );
}
