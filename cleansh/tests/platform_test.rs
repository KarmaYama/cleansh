// cleansh-workspace/cleansh/tests/platform_test.rs

use cleansh::utils::platform::eof_key_combo;

#[test]
fn test_eof_key_combo_is_correct() {
    // This test will be run on the CI's Linux machine (ubuntu-latest)
    // and should assert the correct value for that platform.
    if cfg!(windows) {
        assert_eq!(eof_key_combo(), "Ctrl+Z");
    } else {
        // This is the expected path for your Ubuntu CI runner
        assert_eq!(eof_key_combo(), "Ctrl+D");
    }
}