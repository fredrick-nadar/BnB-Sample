use handsfree_core::{
    action::Action,
    events::{EventPayload, Modality, NormalizedEvent},
    DeterministicRouter, MacroPlanner, ModalityArbitrator, PolicyEngine, RiskTier,
};
use handsfree_platform::{MouseButton, VolumeControl};

#[test]
fn test_deterministic_tab_and_navigation() {
    let router = DeterministicRouter::new();

    let act = router.resolve("open new tab").expect("should resolve");
    assert_eq!(
        act,
        Action::Hotkey {
            keys: vec!["ctrl".to_string(), "t".to_string()]
        }
    );

    let act = router.resolve("close tab").expect("should resolve");
    assert_eq!(
        act,
        Action::Hotkey {
            keys: vec!["ctrl".to_string(), "w".to_string()]
        }
    );

    let act = router.resolve("go back").expect("should resolve");
    assert_eq!(
        act,
        Action::Hotkey {
            keys: vec!["alt".to_string(), "left".to_string()]
        }
    );

    let act = router.resolve("next tab").expect("should resolve");
    assert_eq!(
        act,
        Action::Hotkey {
            keys: vec!["ctrl".to_string(), "tab".to_string()]
        }
    );

    let act = router.resolve("previous tab").expect("should resolve");
    assert_eq!(
        act,
        Action::Hotkey {
            keys: vec!["ctrl".to_string(), "shift".to_string(), "tab".to_string()]
        }
    );
}

#[test]
fn test_deterministic_clipboard_and_volume() {
    let router = DeterministicRouter::new();

    let act = router.resolve("copy").expect("should resolve");
    assert_eq!(
        act,
        Action::Hotkey {
            keys: vec!["ctrl".to_string(), "c".to_string()]
        }
    );

    let act = router.resolve("volume up").expect("should resolve");
    assert_eq!(
        act,
        Action::Volume {
            control: VolumeControl::Up
        }
    );

    let act = router.resolve("mute").expect("should resolve");
    assert_eq!(
        act,
        Action::Volume {
            control: VolumeControl::Mute
        }
    );
}

#[test]
fn test_deterministic_site_and_app_map() {
    let router = DeterministicRouter::new();

    let act = router.resolve("open youtube").expect("should resolve");
    assert_eq!(
        act,
        Action::OpenUrl {
            url: "https://www.youtube.com".to_string()
        }
    );

    let act = router.resolve("open notepad").expect("should resolve");
    assert_eq!(
        act,
        Action::OpenApp {
            name: "notepad".to_string()
        }
    );

    let act = router.resolve("open brave").expect("should resolve");
    assert_eq!(
        act,
        Action::OpenApp {
            name: "brave".to_string()
        }
    );
}

#[test]
fn test_macro_planner_splitting() {
    let planner = MacroPlanner::default();
    let parts = planner.split_macro("open notepad and then type hello world then press enter");
    assert_eq!(
        parts,
        vec![
            "open notepad".to_string(),
            "type hello world".to_string(),
            "press enter".to_string()
        ]
    );
}

#[test]
fn test_policy_risk_classification() {
    let policy = PolicyEngine::new();

    let low_act = Action::Scroll { amount: 5 };
    assert_eq!(policy.classify_risk(&low_act), RiskTier::Low);
    assert!(!policy.requires_confirmation(&low_act));

    let mod_act = Action::Click {
        button: MouseButton::Left,
    };
    assert_eq!(policy.classify_risk(&mod_act), RiskTier::Moderate);
    assert!(!policy.requires_confirmation(&mod_act));

    let high_act = Action::CloseWindow;
    assert_eq!(policy.classify_risk(&high_act), RiskTier::High);
    assert!(policy.requires_confirmation(&high_act));
}

#[test]
fn test_modality_arbitration_and_emergency_stop() {
    let arbitrator = ModalityArbitrator::new(200);

    let event = NormalizedEvent::new(
        Modality::Eye,
        EventPayload::GazeMove {
            x: 100,
            y: 200,
            confidence: 0.95,
        },
        0.95,
    );

    // Inactive initially
    assert!(!arbitrator.should_process(&event));

    // Activate eye
    arbitrator.set_active(Modality::Eye, true);
    assert!(arbitrator.should_process(&event));

    // Deduplication within 200ms drops rapid duplicate
    assert!(!arbitrator.should_process(&event));

    // Emergency stop halts everything
    arbitrator.trigger_emergency_stop();
    assert!(arbitrator.is_emergency_stopped());
    assert!(!arbitrator.should_process(&event));
}
