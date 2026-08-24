mod hook;

fn main() {
    // Harness hook contracts require exit 0 even when input or local buffering fails.
    if let Ok(Some(output)) = hook::run() {
        let _ = serde_json::to_writer(std::io::stdout(), &output);
        println!();
    }
}
