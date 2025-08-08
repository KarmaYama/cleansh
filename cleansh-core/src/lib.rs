//! # CleanSH Core Library
//!
//! `cleansh-core` provides the fundamental, platform-independent logic for data sanitization
//! and redaction. It defines the core data structures for redaction rules, provides mechanisms
//! for compiling these rules, and implements a pluggable `SanitizationEngine` trait for
//! applying redaction logic.
//!
//! The library is designed to be pure and stateless, focusing solely on the transformation
//! of input data based on defined rules, without concerns for I/O or application-specific
//! state management.
//!
//! ## Modules
//!
//! * `config`: Defines `RedactionRule`s and `RedactionConfig` for specifying sensitive patterns.
//! * `sanitizer`: Implements the core logic for compiling rules and applying them.
//! * `validators`: Provides programmatic validation for specific data types.
//! * `redaction_match`: Defines data structures for detailed reporting of redaction events.
//! * `engine`: Defines the `SanitizationEngine` trait, enabling a modular design.
//!
//! ## Public API
//!
//! The public API provides a cohesive set of types and functions for configuring and running
//! a sanitization engine. Key components are organized by functionality:
//!
//! **Configuration & Rules**
//!
//! * [`RedactionConfig`]: Manages collections of `RedactionRule`s, including loading, merging, and filtering.
//! * [`RedactionRule`]: Defines a single rule for identifying and replacing sensitive patterns.
//! * [`merge_rules`]: Merges default and user-defined configurations.
//! * [`RedactionConfig::load_from_file`]: Loads rules from a YAML file.
//! * [`RedactionConfig::load_default_rules`]: Loads the built-in set of default rules.
//!
//! **Sanitization Engine**
//!
//! * [`SanitizationEngine`]: A trait for pluggable sanitization methods. `RegexEngine` is the default implementation.
//! * [`RegexEngine`]: The concrete implementation of `SanitizationEngine` that uses regular expressions.
//!
//! **Redaction Reporting**
//!
//! * [`RedactionMatch`]: A detailed record of a single matched and redacted item, including its location.
//! * [`RedactionSummaryItem`]: A summary of all matches for a specific rule.
//!
//! ## Usage Example
//!
//! ```rust
//! use cleansh_core::{RedactionConfig, SanitizationEngine, RegexEngine, RedactionSummaryItem};
//! use anyhow::Result;
//!
//! fn main() -> Result<()> {
//!     // 1. Load default redaction rules and prepare the engine configuration.
//!     let default_config = RedactionConfig::load_default_rules()?;
//!
//!     // 2. Create the sanitization engine (using the RegexEngine implementation).
//!     //    This compiles all non-opt-in default rules.
//!     let engine: Box<dyn SanitizationEngine> = Box::new(RegexEngine::new(default_config)?);
//!     println!("Successfully initialized the sanitization engine with {} rules.", engine.get_rules().rules.len());
//!
//!     // 3. Prepare some content to sanitize.
//!     let input = "My email is test@example.com and my SSN is 123-45-6789. Another email: user@domain.org.";
//!     println!("\nOriginal Input:\n{}", input);
//!
//!     // 4. Sanitize the content using the engine's `sanitize` method.
//!     let (sanitized_output, summary) = engine.sanitize(input)?;
//!     println!("\nSanitized Output:\n{}", sanitized_output);
//!
//!     // 5. Print the collected redaction summary.
//!     println!("\n--- Redaction Summary ---");
//!     for item in summary {
//!         println!(" Rule: '{}', Occurrences: {}", item.rule_name, item.occurrences);
//!         println!("   - Original (unique): {:?}", item.original_texts);
//!         println!("   - Sanitized (unique): {:?}", item.sanitized_texts);
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! ## Error Handling
//!
//! The library uses `anyhow::Error` for fallible operations and defines specific error
//! types like `RuleConfigNotFoundError` for clearer error reporting.
//!
//! ## Design Principles
//!
//! * **Pluggable Architecture:** The `SanitizationEngine` trait allows for different
//!    sanitization methods (e.g., regex, entropy) to be swapped out seamlessly.
//! * **Stateless:** The core library does not maintain application state.
//! * **Testable:** Logic is easily unit-testable in isolation.
//! * **Extensible:** The design supports adding new rule types or engines with minimal
//!    changes to the core application logic.
//!
//! ---
//! License: BUSL-1.1

pub mod config;
pub mod sanitizer;
pub mod validators;
pub mod redaction_match;
pub mod engine;

// Re-export key types and functions from the config module
pub use config::{
    merge_rules,
    RedactionConfig,
    RedactionRule,
    RedactionSummaryItem,
    RuleConfigNotFoundError,
    MAX_PATTERN_LENGTH,
};

// Re-export key types and functions from the sanitizer module
pub use sanitizer::{
    compile_rules,
    CompiledRule,
    CompiledRules,
};

// Re-export key types from the redaction_match module
pub use redaction_match::{
    RedactionMatch,
    redact_sensitive,
};

// Re-export key types from the engine module
pub use engine::{
    SanitizationEngine,
    RegexEngine, // Re-export the concrete implementation once.
};