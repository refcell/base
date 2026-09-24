//! Temporary migration wrapper for the Base load tester CLI.

fn main() {
    base_cli_utils::run_cli_main!(base_load_tests_cli::LoadTestCli);
}
