use regex::Regex;

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
    name: String,
    severity: Severity,
    regex: Regex,
    description: String,
}

pub struct SecurityScanner {
    patterns: Vec<ScanPattern>,
}

impl SecurityScanner {
    pub fn new() -> Self {
        let patterns = vec![
            ScanPattern {
                name: "command_injection".into(),
                severity: Severity::Critical,
                regex: Regex::new(r#"os\.system\(|subprocess\.(call|run|Popen)\(.*f"|exec\(.*shell"#).unwrap(),
                description: "Potential command injection via os.system/subprocess/exec".into(),
            },
            ScanPattern {
                name: "xss_dangerous_html".into(),
                severity: Severity::High,
                regex: Regex::new(r#"\.innerHTML\s*=|dangerouslySetInnerHTML|v-html"#).unwrap(),
                description: "Dangerous HTML assignment may allow XSS".into(),
            },
            ScanPattern {
                name: "eval_injection".into(),
                severity: Severity::Critical,
                regex: Regex::new(r#"eval\(|exec\(|new Function\("#).unwrap(),
                description: "Dynamic code execution via eval/exec/new Function".into(),
            },
            ScanPattern {
                name: "sql_injection".into(),
                severity: Severity::Critical,
                regex: Regex::new(r#"f".*(?i:SELECT|INSERT|UPDATE|DELETE)|"(?i:SELECT).*"\s*\+|\.format\(.*(?i:SELECT)"#).unwrap(),
                description: "Potential SQL injection via string interpolation".into(),
            },
            ScanPattern {
                name: "path_traversal".into(),
                severity: Severity::High,
                regex: Regex::new(r#"\.\./.*open|open\(.*\.\./|os\.path\.join\(.*request"#).unwrap(),
                description: "Path traversal via ../ in file operations".into(),
            },
            ScanPattern {
                name: "pickle_deserialization".into(),
                severity: Severity::High,
                regex: Regex::new(r#"pickle\.loads?\(|yaml\.load\("#).unwrap(),
                description: "Unsafe deserialization via pickle/yaml.load".into(),
            },
            ScanPattern {
                name: "hardcoded_secrets".into(),
                severity: Severity::Medium,
                regex: Regex::new(r#"(?i)(api_key|password|secret|token)\s*=\s*["'][^"'\s$\{][^"']*["']"#).unwrap(),
                description: "Hardcoded secret or credential in source code".into(),
            },
            ScanPattern {
                name: "insecure_random".into(),
                severity: Severity::Low,
                regex: Regex::new(r#"Math\.random\(\)|random\.random\(\)"#).unwrap(),
                description: "Insecure random number generator used (not suitable for crypto/auth)".into(),
            },
            ScanPattern {
                name: "open_redirect".into(),
                severity: Severity::Medium,
                regex: Regex::new(r#"redirect\(request\.(GET|POST|query)"#).unwrap(),
                description: "Open redirect using unvalidated user input".into(),
            },
        ];

        Self { patterns }
    }

    pub fn scan(&self, code: &str) -> Vec<SecurityFinding> {
        let mut findings = Vec::new();

        for (line_idx, line) in code.lines().enumerate() {
            for pat in &self.patterns {
                if pat.regex.is_match(line) {
                    findings.push(SecurityFinding {
                        pattern: pat.name.clone(),
                        severity: pat.severity.clone(),
                        line: line_idx + 1,
                        snippet: line.trim().to_string(),
                        description: pat.description.clone(),
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
