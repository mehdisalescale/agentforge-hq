use crate::model::Persona;
use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use uuid::Uuid;

/// Parsed persona before assigning stable IDs and timestamps.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParsedPersona {
    pub division_slug: String,
    pub slug: String,
    pub name: String,
    pub short_description: String,
    pub personality: Option<String>,
    pub deliverables: Option<String>,
    pub success_metrics: Option<String>,
    pub workflow: Option<String>,
    pub tags: Vec<String>,
    pub source_file: String,
}

#[derive(Debug)]
pub enum PersonaParseError {
    Io {
        path: PathBuf,
        source: std::io::Error,
    },
    Utf8 { path: PathBuf },
}

/// Wave 1 division whitelist, matching the expansion plan.
const DIVISION_WHITELIST: &[&str] = &[
    "engineering",
    "design",
    "marketing",
    "paid-media",
    "product",
    "project-management",
    "testing",
    "support",
    "spatial-computing",
    "specialized",
    "game-development",
];

pub struct PersonaParser {
    root: PathBuf,
}

impl PersonaParser {
    pub fn new(root: impl Into<PathBuf>) -> Self {
        Self { root: root.into() }
    }

    pub fn parse_all(&self) -> Result<Vec<ParsedPersona>, PersonaParseError> {
        let mut out = Vec::new();
        self.walk_dir(&self.root, &mut out)?;
        Ok(out)
    }

    fn walk_dir(&self, dir: &Path, out: &mut Vec<ParsedPersona>) -> Result<(), PersonaParseError> {
        for entry in fs::read_dir(dir).map_err(|source| PersonaParseError::Io {
            path: dir.to_path_buf(),
            source,
        })? {
            let entry = entry.map_err(|source| PersonaParseError::Io {
                path: dir.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            if path.is_dir() {
                self.walk_dir(&path, out)?;
            } else if path.extension().and_then(|e| e.to_str()) == Some("md") {
                if let Some(parsed) = self.parse_file(&path)? {
                    out.push(parsed);
                }
            }
        }
        Ok(())
    }

    /// Returns Ok(None) when the file is outside the division whitelist.
    fn parse_file(&self, path: &Path) -> Result<Option<ParsedPersona>, PersonaParseError> {
        let rel = path
            .strip_prefix(&self.root)
            .unwrap_or(path)
            .to_path_buf();
        let source_file = rel.to_string_lossy().to_string();

        let mut components = rel.components();
        let division_slug = match components.next() {
            Some(c) => c.as_os_str().to_string_lossy().to_string(),
            None => return Ok(None),
        };
        if !DIVISION_WHITELIST.contains(&division_slug.as_str()) {
            return Ok(None);
        }

        let file_name = path
            .file_stem()
            .and_then(|s| s.to_str())
            .unwrap_or("persona");
        let slug = file_name.replace('_', "-");

        let content = fs::read_to_string(path).map_err(|source| PersonaParseError::Io {
            path: path.to_path_buf(),
            source,
        })?;

        let (name, short_description, sections) = parse_markdown_sections(&content);

        let personality = sections.get("personality").cloned();
        let deliverables = sections
            .get("deliverables")
            .cloned()
            .or_else(|| sections.get("responsibilities").cloned());
        let success_metrics = sections
            .get("success metrics")
            .cloned()
            .or_else(|| sections.get("success-metrics").cloned());
        let workflow = sections
            .get("workflow")
            .cloned()
            .or_else(|| sections.get("workflow steps").cloned());
        let tags = parse_tags(sections.get("tags"));

        Ok(Some(ParsedPersona {
            division_slug,
            slug,
            name,
            short_description,
            personality,
            deliverables,
            success_metrics,
            workflow,
            tags,
            source_file,
        }))
    }
}

fn parse_markdown_sections(content: &str) -> (String, String, HashMap<String, String>) {
    let mut lines = content.lines().peekable();

    let mut name = String::from("Persona");
    for line in lines.by_ref() {
        let trimmed = line.trim();
        if trimmed.starts_with("# ") {
            name = trimmed.trim_start_matches("# ").trim().to_string();
            break;
        }
    }

    let mut collecting = false;
    let mut para_lines = Vec::new();
    let mut sections: HashMap<String, String> = HashMap::new();
    let mut current_section: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in lines {
        let trimmed = line.trim_end();
        if trimmed.starts_with("## ") {
            if let Some(sec) = current_section.take() {
                let text = current_lines.join("\n").trim().to_string();
                if !text.is_empty() {
                    sections.insert(sec, text);
                }
            }
            current_lines.clear();
            let title = trimmed.trim_start_matches("## ").trim().to_lowercase();
            current_section = Some(title);
            collecting = false;
            continue;
        }

        if current_section.is_some() {
            current_lines.push(trimmed.to_string());
        } else if !collecting {
            if !trimmed.is_empty() && !trimmed.starts_with('#') {
                collecting = true;
                para_lines.push(trimmed.to_string());
            }
        } else if trimmed.is_empty() {
            break;
        } else {
            para_lines.push(trimmed.to_string());
        }
    }

    if let Some(sec) = current_section {
        let text = current_lines.join("\n").trim().to_string();
        if !text.is_empty() {
            sections.insert(sec, text);
        }
    }

    let first_para = if para_lines.is_empty() {
        name.clone()
    } else {
        para_lines.join(" ")
    };

    (name, first_para, sections)
}

fn parse_tags(section: Option<&String>) -> Vec<String> {
    let Some(sec) = section else {
        return Vec::new();
    };
    let mut tags = Vec::new();
    for line in sec.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("- ") {
            tags.push(rest.trim().to_string());
        } else if let Some(rest) = trimmed.strip_prefix("* ") {
            tags.push(rest.trim().to_string());
        } else {
            for part in trimmed.split(',') {
                let t = part.trim();
                if !t.is_empty() {
                    tags.push(t.to_string());
                }
            }
            break;
        }
    }
    tags
}

impl From<ParsedPersona> for Persona {
    fn from(p: ParsedPersona) -> Self {
        let now = Utc::now();
        Persona {
            id: crate::model::PersonaId(Uuid::new_v4()),
            division_slug: p.division_slug,
            slug: p.slug,
            name: p.name,
            short_description: p.short_description,
            personality: p.personality,
            deliverables: p.deliverables,
            success_metrics: p.success_metrics,
            workflow: p.workflow,
            tags: p.tags,
            source_file: p.source_file,
            created_at: now,
            updated_at: now,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_basic_markdown() {
        let md = r#"
# Marketing Growth Hacker

Drives rapid experimentation to grow key metrics.

## Personality

- Curious
- Data-driven

## Deliverables

- Weekly growth experiments

## Success Metrics

- Activation rate

## Tags

- marketing
- growth
        "#;

        let (_name, short, sections) = parse_markdown_sections(md);
        assert!(short.to_lowercase().contains("drives rapid"));
        assert!(sections.contains_key("personality"));
        let tags = parse_tags(sections.get("tags"));
        assert_eq!(tags.len(), 2);
    }
}

