use regex::Regex;
use std::sync::LazyLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Severity {
    Critical,
    High,
    Medium,
    Low,
}

impl Severity {
    fn rank(&self) -> u8 {
        match self {
            Severity::Critical => 0,
            Severity::High => 1,
            Severity::Medium => 2,
            Severity::Low => 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct SecurityFinding {
    pub pattern: String,
    pub severity: Severity,
    pub line: usize,
    pub snippet: String,
    pub description: String,
}

struct ScanPattern {
    name: &'static str,
    severity: Severity,
    regex: Regex,
    description: &'static str,
}

/// Security scan patterns compiled once via `LazyLock`.
/// All regex patterns are known at compile time and guaranteed to be valid,
/// so `expect` is appropriate here (a failure would indicate a programming bug).
static SCAN_PATTERNS: LazyLock<Vec<ScanPattern>> = LazyLock::new(|| {
    vec![
        ScanPattern {
            name: "command_injection",
            severity: Severity::Critical,
            regex: Regex::new(r#"os\.system\(|subprocess\.(call|run|Popen)\(.*f"|exec\(.*shell"#)
                .expect("command_injection regex is valid"),
            description: "Potential command injection via os.system/subprocess/exec",
        },
        ScanPattern {
            name: "xss_dangerous_html",
            severity: Severity::High,
            regex: Regex::new(r#"\.innerHTML\s*=|dangerouslySetInnerHTML|v-html"#)
                .expect("xss_dangerous_html regex is valid"),
            description: "Dangerous HTML assignment may allow XSS",
        },
        ScanPattern {
            name: "eval_injection",
            severity: Severity::Critical,
            regex: Regex::new(r#"eval\(|exec\(|new Function\("#)
                .expect("eval_injection regex is valid"),
            description: "Dynamic code execution via eval/exec/new Function",
        },
        ScanPattern {
            name: "sql_injection",
            severity: Severity::Critical,
            regex: Regex::new(r#"f".*(?i:SELECT|INSERT|UPDATE|DELETE)|"(?i:SELECT).*"\s*\+|\.format\(.*(?i:SELECT)"#)
                .expect("sql_injection regex is valid"),
            description: "Potential SQL injection via string interpolation",
        },
        ScanPattern {
            name: "path_traversal",
            severity: Severity::High,
            regex: Regex::new(r#"\.\./.*open|open\(.*\.\./|os\.path\.join\(.*request"#)
                .expect("path_traversal regex is valid"),
            description: "Path traversal via ../ in file operations",
        },
        ScanPattern {
            name: "pickle_deserialization",
            severity: Severity::High,
            regex: Regex::new(r#"pickle\.loads?\(|yaml\.load\("#)
                .expect("pickle_deserialization regex is valid"),
            description: "Unsafe deserialization via pickle/yaml.load",
        },
        ScanPattern {
            name: "hardcoded_secrets",
            severity: Severity::Medium,
            regex: Regex::new(r#"(?i)(api_key|password|secret|token)\s*=\s*["'][^"'\s$\{][^"']*["']"#)
                .expect("hardcoded_secrets regex is valid"),
            description: "Hardcoded secret or credential in source code",
        },
        ScanPattern {
            name: "insecure_random",
            severity: Severity::Low,
            regex: Regex::new(r#"Math\.random\(\)|random\.random\(\)"#)
                .expect("insecure_random regex is valid"),
            description: "Insecure random number generator used (not suitable for crypto/auth)",
        },
        ScanPattern {
            name: "open_redirect",
            severity: Severity::Medium,
            regex: Regex::new(r#"redirect\(request\.(GET|POST|query)"#)
                .expect("open_redirect regex is valid"),
            description: "Open redirect using unvalidated user input",
        },
    ]
});

pub struct SecurityScanner;

impl SecurityScanner {
    pub fn new() -> Self {
        // Force initialization of the static patterns on first use.
        LazyLock::force(&SCAN_PATTERNS);
        Self
    }

    pub fn scan(&self, code: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in code.lines().enumerate() {
            for pat in SCAN_PATTERNS.iter() {
                if pat.regex.is_match(line) {
                    findings.push(SecurityFinding {
                        pattern: pat.name.to_string(),
                        severity: pat.severity.clone(),
                        line: line_idx + 1,
                        snippet: line.trim().to_string(),
                        description: pat.description.to_string(),
                    });
                }
            }
        }

        findings.sort_by_key(|f| f.severity.rank());
        findings
    }
}

impl Default for SecurityScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_command_injection() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"os.system(f"rm -rf {user_input}")"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "command_injection");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn detects_xss_dangerous_html() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"element.innerHTML = userContent;"#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "xss_dangerous_html");
    }

    #[test]
    fn detects_eval_injection() {
        let s = SecurityScanner::new();
        let findings = s.scan("eval(user_input)");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "eval_injection");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn detects_sql_injection() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"query = f"SELECT * FROM users WHERE id = {user_id}""#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "sql_injection");
        assert_eq!(findings[0].severity, Severity::Critical);
    }

    #[test]
    fn detects_path_traversal() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"open("../../etc/passwd")"#);
        assert!(findings.iter().any(|f| f.pattern == "path_traversal"));
    }

    #[test]
    fn detects_pickle_deserialization() {
        let s = SecurityScanner::new();
        let findings = s.scan("data = pickle.load(file)");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "pickle_deserialization");
        assert_eq!(findings[0].severity, Severity::High);
    }

    #[test]
    fn detects_hardcoded_secrets() {
        let s = SecurityScanner::new();
        let findings = s.scan(r#"api_key = "sk-1234567890abcdef""#);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "hardcoded_secrets");
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn detects_insecure_random() {
        let s = SecurityScanner::new();
        let findings = s.scan("let token = Math.random().toString(36)");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "insecure_random");
        assert_eq!(findings[0].severity, Severity::Low);
    }

    #[test]
    fn detects_open_redirect() {
        let s = SecurityScanner::new();
        let findings = s.scan("return redirect(request.GET['next'])");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].pattern, "open_redirect");
        assert_eq!(findings[0].severity, Severity::Medium);
    }

    #[test]
    fn clean_code_passes() {
        let s = SecurityScanner::new();
        let findings = s.scan("let x = 1 + 2;\nfn hello() { println!(\"hi\"); }");
        assert!(findings.is_empty());
    }

    #[test]
    fn multiple_findings_returns_all() {
        let s = SecurityScanner::new();
        let code = "eval(user_input)\nelement.innerHTML = data\nos.system(cmd)";
        let findings = s.scan(code);
        assert!(findings.len() >= 3);
    }

    #[test]
    fn handles_empty_input() {
        let s = SecurityScanner::new();
        assert!(s.scan("").is_empty());
    }
}
