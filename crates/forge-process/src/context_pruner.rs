//! Context pruner — summarizes and truncates messages to fit within a token budget.
//!
//! Pure functions, no database dependencies.

/// Utilities for pruning conversation context to fit within model token limits.
pub struct ContextPruner;

impl ContextPruner {
    /// Summarize a tool call into a one-line string.
    ///
    /// Format: `[tool_name] args_preview... -> result_preview...`
    pub fn summarize_tool_call(tool_name: &str, args: &str, result: &str) -> String {
        let args_preview = if args.len() > 50 {
            format!("{}...", &args[..50])
        } else {
            args.to_string()
        };

        let result_preview = if result.len() > 100 {
            format!("{}...", &result[..100])
        } else {
            result.to_string()
        };

        format!("[{}] {} -> {}", tool_name, args_preview, result_preview)
    }

    /// Truncate text: 80% from head, 20% from tail, marker in middle.
    ///
    /// If `text.len() <= max_chars`, returns the text unchanged.
    /// Otherwise: head (80% of max_chars) + `\n[...truncated...]\n` + tail (20% of max_chars).
    pub fn truncate_text(text: &str, max_chars: usize) -> String {
        if text.len() <= max_chars {
            return text.to_string();
        }

        let marker = "\n[...truncated...]\n";
        let available = max_chars.saturating_sub(marker.len());
        let head_len = (available * 80) / 100;
        let tail_len = available - head_len;

        let head = &text[..head_len];
        let tail = &text[text.len() - tail_len..];

        format!("{}{}{}", head, marker, tail)
    }

    /// Estimate token count using a chars / 3 heuristic.
    pub fn estimate_tokens(text: &str) -> usize {
        text.len().div_ceil(3)
    }

    /// Prune a list of messages to fit within `max_tokens`.
    ///
    /// Strategy:
    /// 1. If total tokens are already under the limit, return as-is.
    /// 2. Keep the last 50% of messages intact; summarize older ones
    ///    (summary = first N chars + "...").
    /// 3. Progressively shorten summaries until under the limit.
    /// 4. Drop oldest messages as a last resort.
    pub fn prune_messages(messages: &[String], max_tokens: usize) -> Vec<String> {
        let mut result: Vec<String> = messages.to_vec();

        let total: usize = result.iter().map(|m| Self::estimate_tokens(m)).sum();
        if total <= max_tokens {
            return result;
        }

        // Keep the last 50% intact, summarize older ones
        let keep_count = result.len() / 2;
        let summarize_count = result.len() - keep_count;

        if summarize_count == 0 {
            return result;
        }

        // Progressively shorten the target length for older messages
        let mut target_len = 80usize;
        loop {
            let suffix = "...";
            for msg in result.iter_mut().take(summarize_count) {
                if msg.len() > target_len + suffix.len() {
                    *msg = format!("{}{}", &msg[..target_len], suffix);
                }
            }

            let current: usize = result.iter().map(|m| Self::estimate_tokens(m)).sum();
            if current <= max_tokens {
                return result;
            }

            if target_len <= 3 {
                // Cannot shorten further — drop oldest messages
                break;
            }
            target_len /= 2;
        }

        // Last resort: drop oldest messages until under the limit
        while result.len() > 1 {
            result.remove(0);
            let current: usize = result.iter().map(|m| Self::estimate_tokens(m)).sum();
            if current <= max_tokens {
                return result;
            }
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn truncate_short_text_unchanged() {
        let text = "Hello, world!";
        let result = ContextPruner::truncate_text(text, 100);
        assert_eq!(result, text);
    }

    #[test]
    fn truncate_long_text_splits_80_20() {
        // Create a 200-char string
        let text: String = (0..200).map(|i| (b'A' + (i % 26) as u8) as char).collect();
        let max = 100;
        let result = ContextPruner::truncate_text(&text, max);

        assert!(result.contains("[...truncated...]"));

        // The result should contain head + marker + tail
        let marker = "\n[...truncated...]\n";
        let parts: Vec<&str> = result.splitn(2, marker).collect();
        assert_eq!(parts.len(), 2);

        let head = parts[0];
        let tail = parts[1];

        // Available = 100 - 19 (marker len) = 81
        // Head = 81 * 80 / 100 = 64
        // Tail = 81 - 64 = 17
        assert_eq!(head.len(), 64);
        assert_eq!(tail.len(), 17);

        // Head should be the start of the text
        assert_eq!(head, &text[..64]);
        // Tail should be the end of the text
        assert_eq!(tail, &text[text.len() - 17..]);
    }

    #[test]
    fn estimate_tokens_approximation() {
        // "hello world" = 11 chars, (11 + 2) / 3 = 4 tokens (ceiling)
        let tokens = ContextPruner::estimate_tokens("hello world");
        assert_eq!(tokens, 4);

        // Empty string = 0 tokens... wait, (0 + 2) / 3 = 0 with integer division
        let empty = ContextPruner::estimate_tokens("");
        assert_eq!(empty, 0);

        // 3 chars = (3 + 2) / 3 = 1
        let three = ContextPruner::estimate_tokens("abc");
        assert_eq!(three, 1);
    }

    #[test]
    fn prune_reduces_below_limit() {
        // Create 10 long messages (~100 chars each, ~33 tokens each = ~330 total tokens)
        let messages: Vec<String> = (0..10)
            .map(|i| format!("Message {} with a lot of content to pad it out: {}", i, "x".repeat(90)))
            .collect();

        let total_before: usize = messages
            .iter()
            .map(|m| ContextPruner::estimate_tokens(m))
            .sum();
        assert!(total_before > 100, "precondition: total tokens ({}) should exceed 100", total_before);

        let pruned = ContextPruner::prune_messages(&messages, 100);
        let total_after: usize = pruned
            .iter()
            .map(|m| ContextPruner::estimate_tokens(m))
            .sum();

        assert!(
            total_after <= 100,
            "pruned total ({}) should be <= 100",
            total_after
        );
        // Some messages may be dropped as a last resort, but total must fit
        assert!(pruned.len() <= messages.len());
        assert!(!pruned.is_empty(), "at least one message should remain");
    }

    #[test]
    fn summarize_tool_call_format() {
        let result = ContextPruner::summarize_tool_call(
            "read_file",
            "/path/to/file.rs",
            "fn main() { println!(\"hello\"); }",
        );
        assert!(result.starts_with("[read_file]"));
        assert!(result.contains("/path/to/file.rs"));
        assert!(result.contains("-> "));
        assert!(result.contains("fn main()"));

        // Test truncation of long args
        let long_args = "a".repeat(100);
        let long_result = "b".repeat(200);
        let summary = ContextPruner::summarize_tool_call("tool", &long_args, &long_result);
        // Args should be truncated to 50 + "..."
        assert!(summary.contains(&format!("{}...", &long_args[..50])));
        // Result should be truncated to 100 + "..."
        assert!(summary.contains(&format!("{}...", &long_result[..100])));
    }
}
