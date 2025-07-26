PS C:\Users\alexm\Desktop\cleansh> cargo test --release --features "test-exposed clipboard" -q

running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 4 tests
Loading custom rules from: C:\Users\alexm\AppData\Local\Temp\.tmpiKeCKu\test_rules_diff.yaml
Loading custom rules from: C:\Users\alexm\AppData\Local\Temp\.tmpEwmGcU\test_rules.yaml
Loading custom rules from: C:\Users\alexm\AppData\Local\Temp\.tmpIfXf8z\test_rules_clipboard.yaml
Loading custom rules from: C:\Users\alexm\AppData\Local\Temp\.tmp6jI44J\test_rules_no_summary.yaml
Writing sanitized content to file: C:\Users\alexm\AppData\Local\Temp\.tmpiKeCKu\output_diff.txt
Generating and displaying diff.
Writing sanitized content to file: C:\Users\alexm\AppData\Local\Temp\.tmpEwmGcU\output.txt
Writing sanitized content to file: C:\Users\alexm\AppData\Local\Temp\.tmp6jI44J\output_no_summary.txt

--- Diff View ---
Displaying redaction summary.
Redaction summary display skipped per user request.
Writing sanitized content to file: C:\Users\alexm\AppData\Local\Temp\.tmpIfXf8z\output_clipboard.txt

--- Redaction Summary ---
-----------------
email (1 occurrences)
    Original Values:
        - test@example.com
    Sanitized Values:
.        - [EMAIL]
Redaction summary display skipped per user request.
Redaction summary display skipped per user request.
us_ssn (1 occurrences)
Sanitized content copied to clipboard successfully.
    Original Values:
        - 123-45-6789
    Sanitized Values:
.        - [US_SSN_REDACTED]
-------------------------

..
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s


running 6 tests
......
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.16s


running 7 tests
.......
test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s


running 18 tests
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_donation_prompt_suppression_flag
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_app_state_usage_increment
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_donation_prompt_trigger_and_cooldown
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_fail_over_not_triggered
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_json_output_to_stdout
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_debug_flag_enables_debug_logs
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_json_output_to_file
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_fail_over_not_triggered\\.tmpPjlFu2\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_app_state_usage_increment\\.tmpsJe4yl\\app_state.json"
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_fail_over_triggered
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_json_output_to_stdout\\.tmpLSp0ba\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_donation_prompt_trigger_and_cooldown\\.tmpJ1SJ1p\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_json_output_to_stdout
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_donation_prompt_suppression_flag\\.tmpAYYFTG\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_fail_over_not_triggered
[DEBUG full_stats_tests] Running test_stats_app_state_usage_increment
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_debug_flag_enables_debug_logs\\.tmpF8PDzR\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_donation_prompt_trigger_and_cooldown
[DEBUG full_stats_tests] Initial app state usage count: 0
[DEBUG full_stats_tests] Running test_stats_donation_prompt_suppression_flag
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_app_state_usage_increment\\.tmpsJe4yl\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_debug_flag_enables_debug_logs
[DEBUG full_stats_tests] Initial app state for donation prompt test: AppState { stats_only_usage_count: 4, last_prompt_timestamp: Some(1750841235), donation_prompts_disabled: false }
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_json_output_to_stdout\\.tmpLSp0ba\\app_state.json"
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_donation_prompt_trigger_and_cooldown\\.tmpJ1SJ1p\\app_state.json"
[DEBUG full_stats_tests] Initial app state for suppression test: AppState { stats_only_usage_count: 4, last_prompt_timestamp: Some(1750841235), donation_prompts_disabled: false }
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_fail_over_not_triggered\\.tmpPjlFu2\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_fail_over_triggered\\.tmp9AcNub\\app_state.json"
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_debug_flag_enables_debug_logs\\.tmpF8PDzR\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_json_output_to_file\\.tmpphxaP3\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_json_output_to_file
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_donation_prompt_suppression_flag\\.tmpAYYFTG\\app_state.json"
[DEBUG full_stats_tests] Output JSON path: "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_json_output_to_file\\.tmpphxaP3\\stats_output.json"
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_json_output_to_file\\.tmpphxaP3\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_fail_over_triggered
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_fail_over_triggered\\.tmp9AcNub\\app_state.json"
[DEBUG full_stats_tests] Stderr after first run (prompt expected): 
Reading input from stdin...
Hey! You've used Cleansh's stats feature a few times. If you find it valuable, please consider donating at least $1 to Cleansh on GitHub Sponsors to motivate us: https://github.com/sponsors/KarmaYama
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
--------------------------


[DEBUG full_stats_tests] App state after first run: AppState { stats_only_usage_count: 5, last_prompt_timestamp: Some(1753519635), donation_prompts_disabled: false }
[DEBUG full_stats_tests] App state usage count after run 1: 1
[DEBUG full_stats_tests] Stdout JSON for json_output_to_stdout:
{
  "redaction_summary": {
    "IPv4Address": {
      "count": 1
    },
    "EmailAddress": {
      "count": 1
    }
  }
}

[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_app_state_usage_increment\\.tmpsJe4yl\\app_state.json"
[DEBUG full_stats_tests] Stderr for json_output_to_stdout:
Reading input from stdin...

.[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_donation_prompt_trigger_and_cooldown\\.tmpJ1SJ1p\\app_state.json"
[DEBUG full_stats_tests] Stderr for fail_over_not_triggered:
Reading input from stdin...
Total secrets (3) are below the fail-over threshold (3).
Redaction Statistics Summary:

--- Redaction Statistics ---
AWSAccessKey: 1 match
EmailAddress: 1 match
IPv4Address: 1 match
--------------------------


[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_no_debug_flag_disables_debug_logs
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_no_debug_flag_disables_debug_logs\\.tmpS0Kunn\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_no_debug_flag_disables_debug_logs
[DEBUG full_stats_tests] Stderr for debug_flag_enables_debug_logs:
[INFO cleansh] cleansh started. Version: 0.1.5
Reading input from stdin...
[INFO cleansh::commands::stats] Starting cleansh --stats-only operation.
[DEBUG cleansh::commands::stats] [stats.rs] Starting stats-only operation.
[DEBUG cleansh::commands::stats] [stats.rs] Received enable_rules: []
[DEBUG cleansh::commands::stats] [stats.rs] Received disable_rules: []
[DEBUG cleansh::utils::app_state] Donation prompt threshold not met. Current count: 1
[DEBUG cleansh::config] [config.rs] Loading default rules from embedded string...
[DEBUG cleansh::config] [config.rs] Loaded 24 default rules.
[DEBUG cleansh::config] [config.rs] Default Rule - Name: email, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ipv4_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ipv6_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: jwt_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: github_pat, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: github_pat_fine_grained, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: stripe_secret, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: aws_access_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: aws_secret_key, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: gcp_api_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: google_oauth_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ssh_private_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_hex_secret_32, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_hex_secret_64, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_token, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: credit_card, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: us_ssn, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: uk_nino, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: sa_id, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: absolute_linux_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: absolute_macos_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: windows_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: slack_webhook_url, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: http_basic_auth, Opt_in: false
[DEBUG cleansh::commands::stats] [stats.rs] Loaded 24 default rules.
[DEBUG cleansh::commands::stats] [stats.rs] No custom config path provided.
[DEBUG cleansh::config] [config.rs] merge_rules called. Initial default rules count: 24
[DEBUG cleansh::config] No user configuration provided. Using 24 default rules.
[DEBUG cleansh::config] [config.rs] No user configuration to merge. Final rules count: 24
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: email, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ipv4_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ipv6_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: jwt_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: github_pat, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: github_pat_fine_grained, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: stripe_secret, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: aws_access_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: aws_secret_key, .Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: gcp_api_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: google_oauth_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ssh_private_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_hex_secret_32, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_hex_secret_64, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_token, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: credit_card, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: us_ssn, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: uk_nino, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: sa_id, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: absolute_linux_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: absolute_macos_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: windows_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: slack_webhook_url, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: http_basic_auth, Opt_in: false
[DEBUG cleansh::commands::stats] [stats.rs] Merged config contains 24 rules before compilation.
[DEBUG cleansh::config] [config.rs] Setting active rules configuration to: 'default'
[DEBUG cleansh::config] [config.rs] 'default' config applied. All rules loaded from config will be passed to compilation.
[DEBUG cleansh::commands::stats] [stats.rs] Active rules config set to: default
[DEBUG cleansh::commands::stats] Compiling rules for stats mode...
[DEBUG cleansh::tools::sanitize_shell] compile_rules called with 24 rules.
[DEBUG cleansh::tools::sanitize_shell] enable_set: {}
[DEBUG cleansh::tools::sanitize_shell] disable_set: {}
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'email', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ipv4_address', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ipv6_address', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'jwt_token', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'github_pat', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'github_pat_fine_grained', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'stripe_secret', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'aws_access_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'aws_secret_key', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'aws_secret_key' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'gcp_api_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'google_oauth_token', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ssh_private_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_hex_secret_32', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_hex_secret_32' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_hex_secret_64', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_hex_secret_64' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_token', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_token' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'credit_card', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'us_ssn', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'uk_nino', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'uk_nino' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'sa_id', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'absolute_linux_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'absolute_macos_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'windows_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'slack_webhook_url', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'http_basic_auth', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Finished compiling rules. Total compiled: 19
[DEBUG cleansh::commands::stats] [stats.rs] Compiled 19 rules successfully.
[DEBUG cleansh::commands::stats] [stats.rs] Names of compiled rules available for stats processing:
[DEBUG cleansh::commands::stats] [stats.rs] - email
[DEBUG cleansh::commands::stats] [stats.rs] - ipv4_address
[DEBUG cleansh::commands::stats] [stats.rs] - ipv6_address
[DEBUG cleansh::commands::stats] [stats.rs] - jwt_token
[DEBUG cleansh::commands::stats] [stats.rs] - github_pat
[DEBUG cleansh::commands::stats] [stats.rs] - github_pat_fine_grained
[DEBUG cleansh::commands::stats] [stats.rs] - stripe_secret
[DEBUG cleansh::commands::stats] [stats.rs] - aws_access_key
[DEBUG cleansh::commands::stats] [stats.rs] - gcp_api_key
[DEBUG cleansh::commands::stats] [stats.rs] - google_oauth_token
[DEBUG cleansh::commands::stats] [stats.rs] - ssh_private_key
[DEBUG cleansh::commands::stats] [stats.rs] - credit_card
[DEBUG cleansh::commands::stats] [stats.rs] - us_ssn
[DEBUG cleansh::commands::stats] [stats.rs] - sa_id
[DEBUG cleansh::commands::stats] [stats.rs] - absolute_linux_path
[DEBUG cleansh::commands::stats] [stats.rs] - absolute_macos_path
[DEBUG cleansh::commands::stats] [stats.rs] - windows_path
[DEBUG cleansh::commands::stats] [stats.rs] - slack_webhook_url
[DEBUG cleansh::commands::stats] [stats.rs] - http_basic_auth
[DEBUG cleansh::tools::sanitize_shell] sanitize_content called. Num compiled rules: 19
[DEBUG cleansh::tools::sanitize_shell] Sanitize called. Input content length: 23
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'email'
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' captured match (original): [REDACTED: 16 chars]
[DEBUG cleansh::tools::sanitize_shell] Redacting '[REDACTED: 16 chars]' with '[REDACTED: 16 chars]' for rule 'email'
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ipv4_address'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ipv6_address'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'jwt_token'
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'github_pat'
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'github_pat_fine_grained'
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'stripe_secret'
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'aws_access_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'gcp_api_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'google_oauth_token'
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ssh_private_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'credit_card'
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'us_ssn'
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'sa_id'
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'absolute_linux_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'absolute_macos_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'windows_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'slack_webhook_url'
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'http_basic_auth'
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' compiled.
[DEBUG cleansh::tools::sanitize_shell] Sanitization complete. Total individual matches found: 1
[DEBUG cleansh::commands::stats] [stats.rs] Analysis completed. Total individual matches (including those not programmatically validated for redaction): 1      
[DEBUG cleansh::commands::stats] [stats.rs] Total matches found (including those failing programmatic validation): 1
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
--------------------------

[INFO cleansh::commands::stats] Cleansh --stats-only operation completed.
[DEBUG cleansh::commands::stats] [stats.rs] Cleansh stats-only operation completed.
[INFO cleansh] cleansh finished successfully.

[DEBUG full_stats_tests] Stderr for fail_over_triggered:
Reading input from stdin...
ERROR: Fail-over triggered: Total secrets (3) exceeded threshold (2).

[DEBUG full_stats_tests] Stderr for donation_prompt_suppression_flag:
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
--------------------------


[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_no_debug_flag_disables_debug_logs\\.tmpS0Kunn\\app_state.json"
[DEBUG full_stats_tests] App state after suppression run: AppState { stats_only_usage_count: 5, last_prompt_timestamp: Some(1750841235), donation_prompts_disabled: true }
..[DEBUG full_stats_tests] Content of stats_output.json:
{
  "redaction_summary": {
    "GenericToken": {
      "count": 1
    }
  }
}
[DEBUG full_stats_tests] Parsed JSON stats: Object {"redaction_summary": Object {"GenericToken": Object {"count": Number(1)}}}
.[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_only_programmatic_validation_valid_match
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_only_with_sample_matches
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_only_no_matches
.[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_only_programmatic_validation_regex_match_only
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_no_matches\\.tmpmZ4pHl\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_programmatic_validation_valid_match\\.tmpduU0cR\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_only_no_matches
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_with_sample_matches\\.tmpBXJGHQ\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_only_programmatic_validation_valid_match
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_programmatic_validation_regex_match_only\\.tmpaz3hlD\\app_state.json"
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_no_matches\\.tmpmZ4pHl\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_only_with_sample_matches
[DEBUG full_stats_tests] App state usage count after run 2: 2
[DEBUG full_stats_tests] Running test_stats_only_programmatic_validation_regex_match_only
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_programmatic_validation_valid_match\\.tmpduU0cR\\app_state.json"
.[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_only_with_simple_matches
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_with_sample_matches\\.tmpBXJGHQ\\app_state.json"
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_programmatic_validation_regex_match_only\\.tmpaz3hlD\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_with_simple_matches\\.tmpJgtXcg\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_only_with_simple_matches
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_pii_debug_env_var
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_only_with_simple_matches\\.tmpJgtXcg\\app_state.json"
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_pii_debug_env_var\\.tmpCvRLj7\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_pii_debug_env_var
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_pii_debug_env_var\\.tmpCvRLj7\\app_state.json"
[DEBUG full_stats_tests] Stderr after second run (no prompt expected):
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
--------------------------


[DEBUG full_stats_tests] App state after second run: AppState { stats_only_usage_count: 6, last_prompt_timestamp: Some(1753519635), donation_prompts_disabled: false }
.[DEBUG full_stats_tests] Stderr for no_debug_flag_disables_debug_logs: 
[INFO cleansh] cleansh started. Version: 0.1.5
Reading input from stdin...
[INFO cleansh::commands::stats] Starting cleansh --stats-only operation.
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
--------------------------

[INFO cleansh::commands::stats] Cleansh --stats-only operation completed.
[INFO cleansh] cleansh finished successfully.

.[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_pii_debug_env_var_not_set
[DEBUG full_stats_tests] Stderr for no_matches:
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---

No redaction matches found.

--------------------------


.[DEBUG full_stats_tests] Stderr for programmatic_validation_regex_match_only: 
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---

No redaction matches found.

--------------------------


[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_pii_debug_env_var_not_set\\.tmp9KAaKU\\app_state.json"
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_quiet_flag_suppresses_info
.[DEBUG full_stats_tests] Running test_stats_pii_debug_env_var_not_set
[DEBUG full_stats_tests] Test setup: Initializing test paths for test_stats_rule_enable_and_disable
[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_quiet_flag_suppresses_info\\.tmpsXIgtk\\app_state.json"
[DEBUG full_stats_tests] Stderr for sample_matches:
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 3 matches
    Sample Matches:
        - example@domain.com
        - test@example.com
        ... (1 more unique samples)
--------------------------


[DEBUG full_stats_tests] Stderr for programmatic_validation_valid_match:
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---
us_ssn: 1 match
--------------------------


[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_pii_debug_env_var_not_set\\.tmp9KAaKU\\app_state.json"
[DEBUG full_stats_tests] Running test_stats_quiet_flag_suppresses_info
[DEBUG full_stats_tests] Stderr for simple_matches:
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
IPv4Address: 1 match
--------------------------


[DEBUG full_stats_tests] Test setup: App state file created at "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_rule_enable_and_disable\\.tmp50NQhR\\app_state.json"
[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_quiet_flag_suppresses_info\\.tmpsXIgtk\\app_state.json"
.[DEBUG full_stats_tests] Running test_stats_rule_enable_and_disable
.[DEBUG full_stats_tests] Command setup: CLEANSH_STATE_FILE_OVERRIDE_FOR_TESTS set to "C:\\Users\\alexm\\Desktop\\cleansh\\target\\tmp\\cleansh_full_stats_tests_data\\test_stats_rule_enable_and_disable\\.tmp50NQhR\\app_state.json"
.[DEBUG full_stats_tests] Stderr for pii_debug_env_var: 
[INFO cleansh] cleansh started. Version: 0.1.5
Reading input from stdin...
[INFO cleansh::commands::stats] Starting cleansh --stats-only operation.
[DEBUG cleansh::commands::stats] [stats.rs] Starting stats-only operation.
[DEBUG cleansh::commands::stats] [stats.rs] Received enable_rules: []
[DEBUG cleansh::commands::stats] [stats.rs] Received disable_rules: []
[DEBUG cleansh::utils::app_state] Donation prompt threshold not met. Current count: 1
[DEBUG cleansh::config] [config.rs] Loading default rules from embedded string...
[DEBUG cleansh::config] [config.rs] Loaded 24 default rules.
[DEBUG cleansh::config] [config.rs] Default Rule - Name: email, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ipv4_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ipv6_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: jwt_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: github_pat, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: github_pat_fine_grained, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: stripe_secret, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: aws_access_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: aws_secret_key, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: gcp_api_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: google_oauth_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ssh_private_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_hex_secret_32, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_hex_secret_64, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_token, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: credit_card, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: us_ssn, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: uk_nino, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: sa_id, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: absolute_linux_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: absolute_macos_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: windows_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: slack_webhook_url, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: http_basic_auth, Opt_in: false
[DEBUG cleansh::commands::stats] [stats.rs] Loaded 24 default rules.
[DEBUG cleansh::commands::stats] [stats.rs] No custom config path provided.
[DEBUG cleansh::config] [config.rs] merge_rules called. Initial default rules count: 24
[DEBUG cleansh::config] No user configuration provided. Using 24 default rules.
[DEBUG cleansh::config] [config.rs] No user configuration to merge. Final rules count: 24
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: email, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ipv4_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ipv6_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: jwt_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: github_pat, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: github_pat_fine_grained, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: stripe_secret, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: aws_access_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: aws_secret_key, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: gcp_api_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: google_oauth_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ssh_private_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_hex_secret_32, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_hex_secret_64, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_token, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: credit_card, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: us_ssn, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: uk_nino, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: sa_id, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: absolute_linux_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: absolute_macos_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: windows_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: slack_webhook_url, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: http_basic_auth, Opt_in: false
[DEBUG cleansh::commands::stats] [stats.rs] Merged config contains 24 rules before compilation.
[DEBUG cleansh::config] [config.rs] Setting active rules configuration to: 'default'
[DEBUG cleansh::config] [config.rs] 'default' config applied. All rules loaded from config will be passed to compilation.
[DEBUG cleansh::commands::stats] [stats.rs] Active rules config set to: default
[DEBUG cleansh::commands::stats] Compiling rules for stats mode...
[DEBUG cleansh::tools::sanitize_shell] compile_rules called with 24 rules.
[DEBUG cleansh::tools::sanitize_shell] enable_set: {}
[DEBUG cleansh::tools::sanitize_shell] disable_set: {}
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'email', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ipv4_address', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ipv6_address', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'jwt_token', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'github_pat', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'github_pat_fine_grained', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'stripe_secret', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'aws_access_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'aws_secret_key', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'aws_secret_key' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'gcp_api_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'google_oauth_token', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ssh_private_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_hex_secret_32', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_hex_secret_32' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_hex_secret_64', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_hex_secret_64' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_token', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_token' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'credit_card', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'us_ssn', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'uk_nino', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'uk_nino' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'sa_id', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'absolute_linux_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'absolute_macos_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'windows_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'slack_webhook_url', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'http_basic_auth', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Finished compiling rules. Total compiled: 19
[DEBUG cleansh::commands::stats] [stats.rs] Compiled 19 rules successfully.
[DEBUG cleansh::commands::stats] [stats.rs] Names of compiled rules available for stats processing:
[DEBUG cleansh::commands::stats] [stats.rs] - email
[DEBUG cleansh::commands::stats] [stats.rs] - ipv4_address
[DEBUG cleansh::commands::stats] [stats.rs] - ipv6_address
[DEBUG cleansh::commands::stats] [stats.rs] - jwt_token
[DEBUG cleansh::commands::stats] [stats.rs] - github_pat
[DEBUG cleansh::commands::stats] [stats.rs] - github_pat_fine_grained
[DEBUG cleansh::commands::stats] [stats.rs] - stripe_secret
[DEBUG cleansh::commands::stats] [stats.rs] - aws_access_key
[DEBUG cleansh::commands::stats] [stats.rs] - gcp_api_key
[DEBUG cleansh::commands::stats] [stats.rs] - google_oauth_token
[DEBUG cleansh::commands::stats] [stats.rs] - ssh_private_key
[DEBUG cleansh::commands::stats] [stats.rs] - credit_card
[DEBUG cleansh::commands::stats] [stats.rs] - us_ssn
[DEBUG cleansh::commands::stats] [stats.rs] - sa_id
[DEBUG cleansh::commands::stats] [stats.rs] - absolute_linux_path
[DEBUG cleansh::commands::stats] [stats.rs] - absolute_macos_path
[DEBUG cleansh::commands::stats] [stats.rs] - windows_path
[DEBUG cleansh::commands::stats] [stats.rs] - slack_webhook_url
[DEBUG cleansh::commands::stats] [stats.rs] - http_basic_auth
[DEBUG cleansh::tools::sanitize_shell] sanitize_content called. Num compiled rules: 19
[DEBUG cleansh::tools::sanitize_shell] Sanitize called. Input content length: 52
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'email'
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' does pattern match input? true
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' captured match (original): test@example.com
[DEBUG cleansh::tools::sanitize_shell] Added RedactionMatch for rule 'email'. Current total matches: 1
[DEBUG cleansh::tools::sanitize_shell] Redacting '[REDACTED: 16 chars]' with '[REDACTED: 16 chars]' for rule 'email'
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ipv4_address'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ipv6_address'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'jwt_token'
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'github_pat'
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'github_pat_fine_grained'
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'stripe_secret'
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'aws_access_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'gcp_api_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'google_oauth_token'
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ssh_private_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'credit_card'
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'us_ssn'
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' does pattern match input? true
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' captured match (original): 123-45-6789
[DEBUG cleansh::tools::sanitize_shell] Added RedactionMatch for rule 'us_ssn'. Current total matches: 2
[DEBUG cleansh::tools::sanitize_shell] Redacting '[REDACTED: 11 chars]' with '[REDACTED: 17 chars]' for rule 'us_ssn'
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'sa_id'
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'absolute_linux_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'absolute_macos_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'windows_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'slack_webhook_url'
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'http_basic_auth'
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' does pattern match input? false
[DEBUG cleansh::tools::sanitize_shell] Sanitization complete. Total individual matches found: 2
[DEBUG cleansh::commands::stats] [stats.rs] Analysis completed. Total individual matches (including those not programmatically validated for redaction): 2      
[DEBUG cleansh::commands::stats] [stats.rs] Found RedactionMatch: Rule='email', Original='test@example.com', Sanitized='[EMAIL_REDACTED]'
[DEBUG cleansh::commands::stats] [stats.rs] Found RedactionMatch: Rule='us_ssn', Original='123-45-6789', Sanitized='[US_SSN_REDACTED]'
[DEBUG cleansh::commands::stats] [stats.rs] Total matches found (including those failing programmatic validation): 2
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
us_ssn: 1 match
--------------------------

[INFO cleansh::commands::stats] Cleansh --stats-only operation completed.
[DEBUG cleansh::commands::stats] [stats.rs] Cleansh stats-only operation completed.
[INFO cleansh] cleansh finished successfully.

.[DEBUG full_stats_tests] Stderr for pii_debug_env_var_not_set:
[INFO cleansh] cleansh started. Version: 0.1.5
Reading input from stdin...
[INFO cleansh::commands::stats] Starting cleansh --stats-only operation.
[DEBUG cleansh::commands::stats] [stats.rs] Starting stats-only operation.
[DEBUG cleansh::commands::stats] [stats.rs] Received enable_rules: []
[DEBUG cleansh::commands::stats] [stats.rs] Received disable_rules: []
[DEBUG cleansh::utils::app_state] Donation prompt threshold not met. Current count: 1
[DEBUG cleansh::config] [config.rs] Loading default rules from embedded string...
[DEBUG cleansh::config] [config.rs] Loaded 24 default rules.
[DEBUG cleansh::config] [config.rs] Default Rule - Name: email, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ipv4_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ipv6_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: jwt_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: github_pat, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: github_pat_fine_grained, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: stripe_secret, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: aws_access_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: aws_secret_key, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: gcp_api_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: google_oauth_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: ssh_private_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_hex_secret_32, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_hex_secret_64, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: generic_token, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: credit_card, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: us_ssn, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: uk_nino, Opt_in: true
[DEBUG cleansh::config] [config.rs] Default Rule - Name: sa_id, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: absolute_linux_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: absolute_macos_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: windows_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: slack_webhook_url, Opt_in: false
[DEBUG cleansh::config] [config.rs] Default Rule - Name: http_basic_auth, Opt_in: false
[DEBUG cleansh::commands::stats] [stats.rs] Loaded 24 default rules.
[DEBUG cleansh::commands::stats] [stats.rs] No custom config path provided.
[DEBUG cleansh::config] [config.rs] merge_rules called. Initial default rules count: 24
[DEBUG cleansh::config] No user configuration provided. Using 24 default rules.
[DEBUG cleansh::config] [config.rs] No user configuration to merge. Final rules count: 24
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: email, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ipv4_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ipv6_address, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: jwt_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: github_pat, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: github_pat_fine_grained, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: stripe_secret, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: aws_access_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: aws_secret_key, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: gcp_api_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: google_oauth_token, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: ssh_private_key, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_hex_secret_32, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_hex_secret_64, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: generic_token, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: credit_card, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: us_ssn, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: uk_nino, Opt_in: true
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: sa_id, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: absolute_linux_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: absolute_macos_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: windows_path, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: slack_webhook_url, Opt_in: false
[DEBUG cleansh::config] [config.rs] Final Merged Rule (no user config) - Name: http_basic_auth, Opt_in: false
[DEBUG cleansh::commands::stats] [stats.rs] Merged config contains 24 rules before compilation.
[DEBUG cleansh::config] [config.rs] Setting active rules configuration to: 'default'
[DEBUG cleansh::config] [config.rs] 'default' config applied. All rules loaded from config will be passed to compilation.
[DEBUG cleansh::commands::stats] [stats.rs] Active rules config set to: default
[DEBUG cleansh::commands::stats] Compiling rules for stats mode...
[DEBUG cleansh::tools::sanitize_shell] compile_rules called with 24 rules.
[DEBUG cleansh::tools::sanitize_shell] enable_set: {}
[DEBUG cleansh::tools::sanitize_shell] disable_set: {}
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'email', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ipv4_address', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ipv6_address', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'jwt_token', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'github_pat', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'github_pat_fine_grained', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'stripe_secret', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'aws_access_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'aws_secret_key', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'aws_secret_key' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'gcp_api_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'google_oauth_token', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'ssh_private_key', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_hex_secret_32', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_hex_secret_32' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_hex_secret_64', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_hex_secret_64' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'generic_token', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'generic_token' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'credit_card', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'us_ssn', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'uk_nino', opt_in: true
[DEBUG cleansh::tools::sanitize_shell] Opt-in rule 'uk_nino' not explicitly enabled, skipping compilation.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'sa_id', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'absolute_linux_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'absolute_macos_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'windows_path', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'slack_webhook_url', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Processing rule: 'http_basic_auth', opt_in: false
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' compiled successfully.
[DEBUG cleansh::tools::sanitize_shell] Finished compiling rules. Total compiled: 19
[DEBUG cleansh::commands::stats] [stats.rs] Compiled 19 rules successfully.
[DEBUG cleansh::commands::stats] [stats.rs] Names of compiled rules available for stats processing:
[DEBUG cleansh::commands::stats] [stats.rs] - email
[DEBUG cleansh::commands::stats] [stats.rs] - ipv4_address
[DEBUG cleansh::commands::stats] [stats.rs] - ipv6_address
[DEBUG cleansh::commands::stats] [stats.rs] - jwt_token
[DEBUG cleansh::commands::stats] [stats.rs] - github_pat
[DEBUG cleansh::commands::stats] [stats.rs] - github_pat_fine_grained
[DEBUG cleansh::commands::stats] [stats.rs] - stripe_secret
[DEBUG cleansh::commands::stats] [stats.rs] - aws_access_key
[DEBUG cleansh::commands::stats] [stats.rs] - gcp_api_key
[DEBUG cleansh::commands::stats] [stats.rs] - google_oauth_token
[DEBUG cleansh::commands::stats] [stats.rs] - ssh_private_key
[DEBUG cleansh::commands::stats] [stats.rs] - credit_card
[DEBUG cleansh::commands::stats] [stats.rs] - us_ssn
[DEBUG cleansh::commands::stats] [stats.rs] - sa_id
[DEBUG cleansh::commands::stats] [stats.rs] - absolute_linux_path
[DEBUG cleansh::commands::stats] [stats.rs] - absolute_macos_path
[DEBUG cleansh::commands::stats] [stats.rs] - windows_path
[DEBUG cleansh::commands::stats] [stats.rs] - slack_webhook_url
[DEBUG cleansh::commands::stats] [stats.rs] - http_basic_auth
[DEBUG cleansh::tools::sanitize_shell] sanitize_content called. Num compiled rules: 19
[DEBUG cleansh::tools::sanitize_shell] Sanitize called. Input content length: 52
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'email'
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'email' captured match (original): [REDACTED: 16 chars]
[DEBUG cleansh::tools::sanitize_shell] Redacting '[REDACTED: 16 chars]' with '[REDACTED: 16 chars]' for rule 'email'
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ipv4_address'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv4_address' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ipv6_address'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ipv6_address' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'jwt_token'
[DEBUG cleansh::tools::sanitize_shell] Rule 'jwt_token' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'github_pat'
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'github_pat_fine_grained'
[DEBUG cleansh::tools::sanitize_shell] Rule 'github_pat_fine_grained' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'stripe_secret'
[DEBUG cleansh::tools::sanitize_shell] Rule 'stripe_secret' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'aws_access_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'aws_access_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'gcp_api_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'gcp_api_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'google_oauth_token'
[DEBUG cleansh::tools::sanitize_shell] Rule 'google_oauth_token' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'ssh_private_key'
[DEBUG cleansh::tools::sanitize_shell] Rule 'ssh_private_key' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'credit_card'
[DEBUG cleansh::tools::sanitize_shell] Rule 'credit_card' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'us_ssn'
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' compiled.
[DEBUG cleansh::tools::sanitize_shell] Rule 'us_ssn' captured match (original): [REDACTED: 11 chars]
[DEBUG cleansh::tools::sanitize_shell] Redacting '[REDACTED: 11 chars]' with '[REDACTED: 17 chars]' for rule 'us_ssn'
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'sa_id'
[DEBUG cleansh::tools::sanitize_shell] Rule 'sa_id' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'absolute_linux_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_linux_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'absolute_macos_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'absolute_macos_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'windows_path'
[DEBUG cleansh::tools::sanitize_shell] Rule 'windows_path' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'slack_webhook_url'
[DEBUG cleansh::tools::sanitize_shell] Rule 'slack_webhook_url' compiled.
[DEBUG cleansh::tools::sanitize_shell] Applying rule: 'http_basic_auth'
[DEBUG cleansh::tools::sanitize_shell] Rule 'http_basic_auth' compiled.
[DEBUG cleansh::tools::sanitize_shell] Sanitization complete. Total individual matches found: 2
[DEBUG cleansh::commands::stats] [stats.rs] Analysis completed. Total individual matches (including those not programmatically validated for redaction): 2      
[DEBUG cleansh::commands::stats] [stats.rs] Total matches found (including those failing programmatic validation): 2
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
us_ssn: 1 match
--------------------------

[INFO cleansh::commands::stats] Cleansh --stats-only operation completed.
[DEBUG cleansh::commands::stats] [stats.rs] Cleansh stats-only operation completed.
[INFO cleansh] cleansh finished successfully.

[DEBUG full_stats_tests] Stderr for quiet_flag_suppresses_info:
Redaction Statistics Summary:

--- Redaction Statistics ---
EmailAddress: 1 match
--------------------------


..[DEBUG full_stats_tests] Stderr for rule_enable_and_disable:
Reading input from stdin...
Redaction Statistics Summary:

--- Redaction Statistics ---
AWSSecretKey: 1 match
--------------------------


.
test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s


running 21 tests
.....................
test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s


running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

PS C:\Users\alexm\Desktop\cleansh> 