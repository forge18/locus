mod hook;

fn main() {
    // Harness hook contracts require exit 0 even when input or local buffering fails.
    let _ = hook::run();
}
