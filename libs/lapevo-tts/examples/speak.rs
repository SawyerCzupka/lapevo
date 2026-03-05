/// Quick voice-line tester for lapevo-tts (sherpa-rs Kokoro backend).
///
/// Usage:
///   cargo run -p lapevo-tts --example speak
///   cargo run -p lapevo-tts --example speak -- "Your brake point is too late."
///
/// Without arguments it cycles through a set of sample racing coach lines.
/// On first run, downloads the Kokoro-en-v0_19 model (~340MB).
use lapevo_tts::Tts;

const SAMPLE_LINES: &[&str] = &[
    "Brake a little earlier into turn one.",
    "Good lap — you gained three tenths in the final sector.",
    "You're carrying too much speed into the hairpin. Try trail-braking.",
    "Smooth inputs through the chicane. Nice work.",
    "Watch your tire temperatures — the rears are starting to overheat.",
    "Pit window opens in four laps.",
];

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let tts = Tts::new().await.expect("Failed to initialize TTS");

    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.is_empty() {
        println!("No text provided — playing {} sample lines.\n", SAMPLE_LINES.len());
        for line in SAMPLE_LINES {
            println!(">> {line}");
            tts.speak(line).await.expect("Failed to speak");
            // Small pause between lines so they don't blur together.
            tokio::time::sleep(std::time::Duration::from_millis(500)).await;
        }
    } else {
        let text = args.join(" ");
        println!(">> {text}");
        tts.speak(&text).await.expect("Failed to speak");
    }
}
