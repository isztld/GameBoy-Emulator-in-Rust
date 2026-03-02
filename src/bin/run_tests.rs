/// Binary to run the Gameboy CPU test suite
///
/// This binary runs the official Gameboy CPU test suite from the
/// GameboyCPUTests directory and reports pass/fail results.

use gb_emu::run_all_tests;

fn main() {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").unwrap_or_else(|_| ".".to_string());
    let test_dir = format!("{}/GameboyCPUTests", manifest_dir);

    println!("Loading tests from: {}", test_dir);
    let start_time = std::time::Instant::now();

    let (passed, failed, failures) = run_all_tests(&test_dir);

    let duration = start_time.elapsed();
    println!("\n=== Test Results ===");
    println!("Total: {}", passed + failed);
    println!("Passed: {}", passed);
    println!("Failed: {}", failed);
    println!("Time: {:.2?}", duration);

    if failed > 0 {
        println!("\n=== Failures ===");
        for failure in failures.iter().take(20) {
            println!("{}", failure);
        }
        if failures.len() > 20 {
            println!("... and {} more failures", failures.len() - 20);
        }
        std::process::exit(1);
    } else {
        println!("\nAll tests passed!");
    }
}
