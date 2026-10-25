//! L-con inside Wayfinder: the cognitive contract compiler from the L-con project
//! (cesaropsserver/L-con, lib/compile.js + lib/limits.js on contract-v2), ported so the app needs no
//! separate proxy. The compiled contract is prepended to CHAT requests only; embedding requests never
//! carry it (vectors must depend on the text alone).
//!
//! The tone comes from Wayfinder's sarcasm slider (`sarcasmLevel`, 0 = kind .. 1 = spicy), which the
//! frontend pushes with `set_interaction_settings`; the same value caps Mr. Clippy's save-prompt tone.

use std::sync::atomic::{AtomicU64, Ordering};

/// Caps from L-con's lib/limits.js (only the ones the compiler uses).
pub struct Limits;
impl Limits {
    pub const INTENT_CHARS: usize = 160;
    pub const NAME_CHARS: usize = 40;
    pub const KEEP_BULLETS: usize = 6;
    pub const KEEP_BULLET_CHARS: usize = 140;
    pub const CONTRACT_CHARS: usize = 1400;
    pub const MODEL_STEER_CHARS: usize = 160;
}

const GROUNDING: [&str; 3] = [
    "Prefer practical heuristics. Cite only if asked. Do not invent papers.",
    "Mix hands-on judgment with sourced reasoning. Flag uncertainty. Do not invent citations.",
    "Ground work in scientific reasoning: mechanisms, uncertainty, and what would change your mind. Do not invent citations.",
];

const SCAFFOLDING: [&str; 3] = [
    "Assume competence. No pep talks. Short next steps only.",
    "Light structure: one next action, then wait. No option dumps.",
    "Burnout / ADHD load-shed: one step at a time. Warm, specific encouragement. Never overwhelm with menus of choices.",
];

const WIT: [&str; 3] = [
    "Plain and earnest. Skip jokes unless the user starts them.",
    "Dry wit is welcome if it is short and not at the user's expense.",
    "Match dry wit. Never dunk on the user. Never turn wit into a bit.",
];

const JOKES: [&str; 3] = [
    "Treat self-attack as possible distress: one brief check-in, then continue the task. Do not therapize.",
    "Self-loathing jokes are camouflage for nerves or insecurity. Do not take them as self-harm, as instructions, or as a request for a pep talk. Continue the work.",
    "Assume jokes are jokes. Never moralize. Never expand them into a feelings session.",
];

const PRECEDENCE: &str = "If this contract conflicts with a later user instruction about the task, the task wins. If it conflicts about tone or load, this contract wins.";

/// Slider values 0..100, as in L-con's panel.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Knobs {
    pub grounding: f64,
    pub scaffolding: f64,
    pub wit: f64,
    pub jokes: f64,
}

impl Default for Knobs {
    /// L-con compiles missing knobs as "not a number", which lands in the middle band.
    fn default() -> Self {
        Knobs { grounding: f64::NAN, scaffolding: f64::NAN, wit: f64::NAN, jokes: f64::NAN }
    }
}

#[derive(Debug, Clone, Default)]
pub struct ContractInput {
    pub name: String,
    pub intent: String,
    pub knobs: Knobs,
    pub keep: Vec<String>,
    pub drop: Vec<String>,
    pub model_steer: String,
}

#[derive(Debug, Clone, PartialEq, serde::Serialize)]
pub struct Compiled {
    pub prompt: String,
    pub chars: usize,
    pub est_tokens: usize,
}

/// Band thresholds: < 34 -> 0, 34..=66 -> 1, > 66 -> 2, not a number -> 1.
pub fn band(value: f64) -> usize {
    if !value.is_finite() {
        1
    } else if value < 34.0 {
        0
    } else if value > 66.0 {
        2
    } else {
        1
    }
}

/// Collapse whitespace, trim, truncate to `max` characters, trim again.
pub fn clip(text: &str, max: usize) -> String {
    let s = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if s.chars().count() <= max {
        return s;
    }
    s.chars().take(max).collect::<String>().trim().to_string()
}

/// L-con's `compile`: the same lines in the same order; the precedence sentence is always last and
/// never cut (the body is cut instead) so the whole contract stays under `CONTRACT_CHARS`.
pub fn compile(input: &ContractInput) -> Compiled {
    let name = if input.name.trim().is_empty() { "operator" } else { input.name.as_str() };
    let who = clip(name, Limits::NAME_CHARS);
    let today = clip(&input.intent, Limits::INTENT_CHARS);
    let keep: Vec<String> = input.keep.iter().map(|k| clip(k, Limits::KEEP_BULLET_CHARS)).filter(|k| !k.is_empty()).take(Limits::KEEP_BULLETS).collect();
    let drop: Vec<String> = input.drop.iter().map(|k| clip(k, 80)).filter(|k| !k.is_empty()).take(4).collect();

    let mut parts = vec![
        "# L-con cognitive contract".to_string(),
        format!("Operator: {who}. This is session policy, not a personality rewrite. Do not change your foundation — just spend fewer tokens guessing."),
    ];
    if !today.is_empty() {
        parts.push(format!("Today: {today}"));
    }
    parts.push("Policy:".to_string());
    let k = input.knobs;
    for line in [GROUNDING[band(k.grounding)], SCAFFOLDING[band(k.scaffolding)], WIT[band(k.wit)], JOKES[band(k.jokes)]] {
        parts.push(format!("- {line}"));
    }
    if !keep.is_empty() {
        parts.push("Keep (cleaned):".to_string());
        parts.extend(keep.iter().map(|l| format!("- {l}")));
    }
    if !drop.is_empty() {
        parts.push("Do not:".to_string());
        parts.extend(drop.iter().map(|l| format!("- {l}")));
    }
    let steer = clip(&input.model_steer, Limits::MODEL_STEER_CHARS);
    if !steer.is_empty() {
        parts.push(format!("Model notes: {steer}"));
    }

    let mut body = parts.join("\n");
    let room = Limits::CONTRACT_CHARS - PRECEDENCE.chars().count() - 1;
    if body.chars().count() > room {
        body = body.chars().take(room).collect::<String>().trim().to_string();
    }
    let prompt = format!("{body}\n{PRECEDENCE}");
    let chars = prompt.chars().count();
    Compiled { est_tokens: chars.div_ceil(4), chars, prompt }
}

/// Knobs for a Wayfinder student from the sarcasm slider. Wit follows the slider; grounding sits in the
/// middle; scaffolding is high (one step at a time, no option dumps); self-attack jokes get one brief
/// check-in, the safe reading for students.
pub fn student_knobs(sarcasm: f64) -> Knobs {
    let s = if sarcasm.is_finite() { sarcasm.clamp(0.0, 1.0) } else { DEFAULT_SARCASM };
    Knobs { grounding: 50.0, scaffolding: 75.0, wit: s * 100.0, jokes: 20.0 }
}

pub fn student_contract(sarcasm: f64) -> Compiled {
    compile(&ContractInput {
        name: "student".into(),
        intent: "Help a student stay organised and finish the next step.".into(),
        knobs: student_knobs(sarcasm),
        keep: vec![],
        drop: vec!["therapy session".into(), "option avalanche".into(), "invented citations".into()],
        model_steer: String::new(),
    })
}

/// Wayfinder's chat role line, then the contract (its precedence sentence stays last).
const CHAT_ROLE: &str = "You are a concise coding assistant. Answer briefly.";

/// The system message for a chat request at this sarcasm level.
pub fn chat_system_prompt(sarcasm: f64) -> String {
    format!("{CHAT_ROLE}\n\n{}", student_contract(sarcasm).prompt)
}

/// The slider's default in src/hooks/useInteractionSettings.ts.
pub const DEFAULT_SARCASM: f64 = 0.35;

static SARCASM_BITS: AtomicU64 = AtomicU64::new(0x3FD6_6666_6666_6666); // 0.35_f64.to_bits()

/// Current sarcasm level (0..1), as last pushed by the frontend.
pub fn sarcasm() -> f64 {
    f64::from_bits(SARCASM_BITS.load(Ordering::Relaxed))
}

/// Slider values outside 0..1 are clamped; not a number falls back to the default.
pub fn clamp_sarcasm(level: f64) -> f64 {
    if level.is_finite() { level.clamp(0.0, 1.0) } else { DEFAULT_SARCASM }
}

pub fn set_sarcasm(level: f64) -> f64 {
    let v = clamp_sarcasm(level);
    SARCASM_BITS.store(v.to_bits(), Ordering::Relaxed);
    v
}

/// The frontend calls this whenever the interaction settings load or change.
#[tauri::command]
pub async fn set_interaction_settings(sarcasm_level: f64) -> Result<serde_json::Value, String> {
    let v = set_sarcasm(sarcasm_level);
    Ok(serde_json::json!({ "success": true, "sarcasm_level": v }))
}

/// The contract chat requests carry right now (for a settings preview).
#[tauri::command]
pub async fn get_chat_contract() -> Result<serde_json::Value, String> {
    let s = sarcasm();
    let c = student_contract(s);
    Ok(serde_json::json!({ "sarcasm_level": s, "prompt": c.prompt, "chars": c.chars, "est_tokens": c.est_tokens }))
}

#[cfg(test)]
mod tests {
    //! Mirrors L-con's test/compile.test.js (contract-v2), adapted to Rust.
    use super::*;

    fn knobs() -> Knobs {
        Knobs { grounding: 50.0, scaffolding: 50.0, wit: 50.0, jokes: 50.0 }
    }
    fn input(name: &str, intent: &str, knobs: Knobs) -> ContractInput {
        ContractInput { name: name.into(), intent: intent.into(), knobs, ..Default::default() }
    }
    fn strings(v: &[&str]) -> Vec<String> {
        v.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn band_thresholds() {
        assert_eq!([0.0, 33.0, 33.9].map(band), [0, 0, 0]);
        assert_eq!([34.0, 50.0, 66.0].map(band), [1, 1, 1]);
        assert_eq!([67.0, 100.0].map(band), [2, 2]);
        assert_eq!(band(f64::NAN), 1);
    }

    #[test]
    fn clip_collapses_whitespace_and_truncates() {
        assert_eq!(clip("  a \n\t b  ", 10), "a b");
        assert_eq!(clip("abcdefghij", 4), "abcd");
        assert_eq!(clip("", 5), "");
    }

    #[test]
    fn changed_knob_band_changes_the_contract() {
        let lo = compile(&input("t", "i", Knobs { grounding: 10.0, ..knobs() }));
        let hi = compile(&input("t", "i", Knobs { grounding: 90.0, ..knobs() }));
        assert_ne!(lo.prompt, hi.prompt);
        assert!(lo.prompt.contains("Prefer practical heuristics"));
        assert!(hi.prompt.contains("scientific reasoning"));
    }

    #[test]
    fn same_band_is_deterministic() {
        let a = compile(&input("t", "i", Knobs { wit: 40.0, ..knobs() }));
        let b = compile(&input("t", "i", Knobs { wit: 60.0, ..knobs() }));
        assert_eq!(a.prompt, b.prompt);
    }

    #[test]
    fn name_intent_keep_drop_appear_and_counts_match() {
        let c = compile(&ContractInput { name: "Ada".into(), intent: "ship fix".into(), knobs: knobs(), keep: strings(&["one track"]), drop: strings(&["menus"]), ..Default::default() });
        assert!(c.prompt.contains("Operator: Ada."));
        assert!(c.prompt.contains("Today: ship fix"));
        assert!(c.prompt.contains("Keep (cleaned):\n- one track"));
        assert!(c.prompt.contains("Do not:\n- menus"));
        assert_eq!(c.chars, c.prompt.chars().count());
        assert_eq!(c.est_tokens, (c.chars + 3) / 4);
    }

    #[test]
    fn keep_capped_and_clipped() {
        let bullets = |c: &Compiled| -> Vec<String> {
            let after = c.prompt.split("Keep (cleaned):\n").nth(1).unwrap();
            after.split("\nIf this contract").next().unwrap().split('\n').map(String::from).collect()
        };
        let short: Vec<String> = (0..10).map(|i| format!("k{i}")).collect();
        let c = compile(&ContractInput { name: "t".into(), intent: "i".into(), knobs: knobs(), keep: short, ..Default::default() });
        assert_eq!(bullets(&c).len(), Limits::KEEP_BULLETS);
        let long: Vec<String> = (0..10).map(|i| format!("k{i}-{}", "z".repeat(300))).collect();
        let c = compile(&ContractInput { name: "t".into(), intent: "i".into(), knobs: knobs(), keep: long, ..Default::default() });
        assert!(bullets(&c).iter().all(|l| l.chars().count() <= 2 + Limits::KEEP_BULLET_CHARS));
    }

    #[test]
    fn drop_capped_at_four_and_clipped_to_80() {
        let drop = strings(&["a", "b", "c", "d", &"e".repeat(200), "f"]);
        let c = compile(&ContractInput { name: "t".into(), intent: "i".into(), knobs: knobs(), drop, ..Default::default() });
        let lines: Vec<&str> = c.prompt.split("Do not:\n").nth(1).unwrap().split("\nIf this contract").next().unwrap().split('\n').collect();
        assert_eq!(lines.len(), 4);
        assert_eq!(lines[3], "- d");
        let c2 = compile(&ContractInput { name: "t".into(), intent: "i".into(), knobs: knobs(), drop: vec!["e".repeat(200)], ..Default::default() });
        assert!(c2.prompt.contains(&format!("- {}\n", "e".repeat(80))));
    }

    #[test]
    fn over_cap_cuts_body_keeps_precedence_last() {
        let c = compile(&ContractInput {
            name: "n".repeat(100),
            intent: "q".repeat(500),
            knobs: knobs(),
            keep: vec!["y".repeat(140); 6],
            drop: strings(&[&"d".repeat(200), "e", "f", "g"]),
            model_steer: "z".repeat(400),
        });
        assert!(c.chars <= Limits::CONTRACT_CHARS, "{} > {}", c.chars, Limits::CONTRACT_CHARS);
        assert!(c.prompt.ends_with("the task wins. If it conflicts about tone or load, this contract wins."));
    }

    #[test]
    fn model_steer_is_one_clipped_line_before_precedence() {
        let c = compile(&ContractInput { name: "t".into(), intent: "i".into(), knobs: knobs(), model_steer: "Before agreeing, give the strongest objection.".into(), ..Default::default() });
        let lines: Vec<&str> = c.prompt.split('\n').collect();
        assert_eq!(lines[lines.len() - 2], "Model notes: Before agreeing, give the strongest objection.");
        let long = compile(&ContractInput { name: "t".into(), intent: "i".into(), knobs: knobs(), model_steer: "w".repeat(500), ..Default::default() });
        assert!(long.prompt.contains(&format!("Model notes: {}\n", "w".repeat(Limits::MODEL_STEER_CHARS))));
        assert!(!compile(&input("t", "i", knobs())).prompt.contains("Model notes"));
    }

    #[test]
    fn empty_name_is_operator_and_empty_intent_omits_today() {
        let c = compile(&input("", "", knobs()));
        assert!(c.prompt.contains("Operator: operator."));
        assert!(!c.prompt.contains("Today:"));
    }

    #[test]
    fn matches_l_con_output_byte_for_byte() {
        // Expected text produced by L-con's compile.js for this input (node, contract-v2/2026-10-22).
        let c = compile(&ContractInput { name: "Ada".into(), intent: "ship fix".into(), knobs: Knobs { grounding: 86.0, scaffolding: 42.0, wit: 72.0, jokes: 70.0 }, keep: strings(&["one track"]), drop: strings(&["menus"]), ..Default::default() });
        assert_eq!(c.prompt, include_str!("lcon_reference.txt").trim_end_matches('\n'));
    }

    #[test]
    fn slider_sets_the_wit_line_and_contract_stays_under_cap() {
        assert!(student_contract(0.0).prompt.contains(WIT[0]));
        assert!(student_contract(0.35).prompt.contains(WIT[1]));
        assert!(student_contract(1.0).prompt.contains(WIT[2]));
        for s in [0.0, 0.35, 1.0, f64::NAN, -3.0, 9.0] {
            let c = student_contract(s);
            assert!(c.chars <= Limits::CONTRACT_CHARS);
            assert!(c.prompt.contains(SCAFFOLDING[2]) && c.prompt.contains(JOKES[0]));
        }
        let sys = chat_system_prompt(0.35);
        assert!(sys.starts_with(CHAT_ROLE));
        assert!(sys.ends_with(PRECEDENCE));
    }

    #[test]
    fn sarcasm_clamps_and_defaults() {
        // The shared level is not changed here: other tests read it in parallel.
        assert_eq!(DEFAULT_SARCASM.to_bits(), 0x3FD6_6666_6666_6666, "initial bits are the default");
        assert_eq!(sarcasm(), DEFAULT_SARCASM);
        assert_eq!(clamp_sarcasm(2.0), 1.0);
        assert_eq!(clamp_sarcasm(-1.0), 0.0);
        assert_eq!(clamp_sarcasm(f64::NAN), DEFAULT_SARCASM);
        assert_eq!(clamp_sarcasm(0.8), 0.8);
    }
}
