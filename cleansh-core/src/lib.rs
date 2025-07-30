// cleansh-workspace/cleansh-core/src/lib.rs
//! # Cleansh Core Library
//!
//! `cleansh-core` provides the fundamental, platform-independent logic for data sanitization
//! and redaction. It defines the core data structures for redaction rules, provides mechanisms
//! for compiling these rules, and implements the `sanitize_content` function which is
//! the heart of the redaction engine.
//!
//! This library is designed to be pure and stateless, focusing solely on the transformation
//! of input data based on defined rules, without concerning itself with I/O, user interfaces,
//! or application-specific state management.
//!
//! ## Modules
//!
//! * `config`: Defines `RedactionRule`s, `RedactionConfig` structures, and `RuleSet`s for
//!     specifying what and how data should be redacted.
//! * `sanitizer`: Contains the core logic for compiling `RedactionRule`s into efficient
//!     regular expressions and applying them to input content to perform redaction.
//! * `validators`: Provides programmatic validation functions for specific sensitive
//!     data types (e.g., SSN, UK NINO) to reduce false positives.
//! * `redaction_match`: Defines the `RedactionMatch` struct for detailed reporting
//!     of sanitization operations and includes utility functions for sensitive data logging.
//!
//! ## Public API
//!
//! The primary components you'll interact with in `cleansh-core` are:
//!
//! ### Configuration & Rules
//! * [`RedactionRule`]: Defines a single rule for identifying and replacing sensitive patterns.
//! * [`RedactionConfig`]: Manages collections of `RedactionRule`s, allowing loading from
//!     files or default configurations, and rule merging.
//! * `MAX_PATTERN_LENGTH`: A constant defining the maximum allowed length for a regex pattern.
//! * [`RuleConfigNotFoundError`]: An error type specific to configuration loading issues.
//!
//! ### Sanitization Engine
//! * [`CompiledRule`]: Represents a single compiled redaction rule, used internally by the `sanitizer`.
//! * [`CompiledRules`]: A collection of all compiled rules, ready for content sanitization.
//!
//! ### Redaction Reporting
//! * [`RedactionMatch`]: Reports details about detected and redacted items.
//!
//! ### Key Functions:
//! * [`RedactionConfig::load_from_file`]: Load redaction rules from a specified YAML file.
//! * [`RedactionConfig::load_default_rules`]: Load the built-in set of default redaction rules.
//! * [`RedactionConfig::set_active_rules_config`]: Filter the loaded rules based on a named configuration (e.g., "default", "strict").
//! * [`merge_rules`]: Combine a default `RedactionConfig` with user-defined overrides.
//! * [`compile_rules`]: Compiles a list of `RedactionRule`s into `CompiledRules`.
//! * [`sanitize_content`]: The core function to sanitize input content using `CompiledRules`.
//!
//! For detailed API usage, refer to the documentation of individual modules and functions.
//!
//! ## Usage Example
//!
//! ```rust
//! use cleansh_core::{RedactionConfig, compile_rules, sanitize_content};
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     // 1. Load default redaction rules
//!     let mut config = RedactionConfig::load_default_rules()?;
//!     println!("Loaded {} default rules.", config.rules.len());
//!
//!     // 2. Set active configuration (e.g., "default" to include only non-opt-in rules)
//!     config.set_active_rules_config("default")?;
//!     let active_rules = config.rules; // Get the rules active for the "default" config
//!     println!("Active rules count after setting 'default' config: {}", active_rules.len());
//!
//!     // 3. Compile the active rules
//!     let compiled_rules = compile_rules(active_rules, &[], &[])?; // No explicit enables/disables here for simplicity
//!     println!("Successfully compiled {} rules.", compiled_rules.rules.len());
//!
//!     // 4. Prepare some content to sanitize
//!     let input = "My email is test@example.com and my SSN is 123-45-6789. Another email: user@domain.org.";
//!     println!("\nOriginal Input:\n{}", input);
//!
//!     // 5. Sanitize the content
//!     let (sanitized_output, matches) = sanitize_content(input, &compiled_rules);
//!     println!("\nSanitized Output:\n{}", sanitized_output);
//!
//!     // 6. Print collected matches
//!     println!("\n--- Redaction Matches ---");
//!     for m in matches {
//!         println!("  Rule: '{}', Original: '{}', Sanitized: '{}'",
//!                  m.rule_name, m.original_string, m.sanitized_string);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The library uses `anyhow::Error` for general fallible operations and defines
//! specific error types like `RuleConfigNotFoundError` where more granularity is beneficial.
//!
//! ## Design Principles
//!
//! * **Pure Functions:** Core sanitization functions are designed to be pure;
//!     given an input and rules, they produce a predictable output.
//! * **Stateless:** The core library does not maintain application state.
//! * **Testable:** Logic is easily unit-testable in isolation.
//! * **Extensible:** Rule definitions and the sanitization process are designed
//!     to be extensible for future rule types or transformation logic.
//!
//! ---
//! License: BUSL-1.1

pub mod config;
pub mod sanitizer;
pub mod validators;
pub mod redaction_match;

// Re-export key types and functions from the config module for convenience
pub use config::{
    MAX_PATTERN_LENGTH,
    RedactionConfig,
    RedactionRule,
    RedactionSummaryItem,
    RuleConfigNotFoundError,
    merge_rules,
};

// Re-export key types and functions from the sanitizer module
pub use sanitizer::{
    CompiledRule,
    CompiledRules,
    compile_rules,
    sanitize_content,
};

// Re-export key types from the redaction_match module (functions are generally internal/logging)
pub use redaction_match::{
    RedactionMatch,
    redact_sensitive, // This is explicitly public for utility/testing if needed
};

// Validators module functions are generally used internally by `sanitizer`, so no public re-export needed for them
// unless they were intended for direct external consumption.