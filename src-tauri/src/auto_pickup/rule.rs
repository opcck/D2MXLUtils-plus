//! Auto Pickup rule definitions and TOML parser.
//!
//! Compatible with cleaned HackMap rule syntax:
//! `{ match_name = 1, regex = "全效活力药水", pickup = 1 },`
//! `{ quality = "unique", pickup = 2 },`
//! `{ match_name = 1, regex = "Ber|Jah|Cham|Zod", pickup = 0 },`

use regex::{Regex, RegexBuilder};
use serde::{Deserialize, Deserializer, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PickupAction {
    None = 0,
    Inventory = 1,
    Cube = 2,
    AutoBelt = 3,
}

impl From<u8> for PickupAction {
    fn from(val: u8) -> Self {
        match val {
            1 => PickupAction::Inventory,
            2 => PickupAction::Cube,
            3 => PickupAction::AutoBelt,
            _ => PickupAction::None,
        }
    }
}

/// Raw representation of a rule directly parsed from TOML
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct PickupRuleRaw {
    #[serde(rename = "id")]
    pub class_id: Option<u32>,

    #[serde(default, deserialize_with = "opt_base_code_flexible")]
    pub base_code: Option<String>,

    #[serde(default, deserialize_with = "opt_bool_flexible")]
    pub match_name: Option<bool>,

    #[serde(rename = "prop")]
    pub property: Option<String>,

    pub regex: Option<String>,

    #[serde(rename = "name_regex")]
    pub name_regex: Option<String>,

    pub quality: Option<String>,

    #[serde(default, deserialize_with = "opt_bool_flexible")]
    pub eth: Option<bool>,

    pub socks: Option<usize>,

    pub pickup: Option<u8>,
}

fn opt_base_code_flexible<'de, D>(deserializer: D) -> Result<Option<String>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Option<toml::Value> = Option::deserialize(deserializer)?;
    match v {
        Some(toml::Value::String(s)) => {
            let trimmed = s.trim().to_lowercase();
            if trimmed.is_empty() {
                Ok(None)
            } else {
                Ok(Some(trimmed))
            }
        }
        Some(toml::Value::Integer(i)) => {
            let u = i as u32;
            let bytes = u.to_le_bytes();
            let s = String::from_utf8_lossy(&bytes)
                .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                .to_lowercase();
            if !s.is_empty() && s.chars().all(|c| c.is_ascii_graphic()) {
                Ok(Some(s))
            } else {
                Ok(Some(format!("{:X}", u)))
            }
        }
        _ => Ok(None),
    }
}

fn opt_bool_flexible<'de, D>(deserializer: D) -> Result<Option<bool>, D::Error>
where
    D: Deserializer<'de>,
{
    let v: Option<toml::Value> = Option::deserialize(deserializer)?;
    match v {
        Some(toml::Value::Boolean(b)) => Ok(Some(b)),
        Some(toml::Value::Integer(i)) => Ok(Some(i != 0)),
        Some(toml::Value::String(s)) => {
            let lower = s.to_lowercase();
            if lower == "true" || lower == "1" {
                Ok(Some(true))
            } else if lower == "false" || lower == "0" {
                Ok(Some(false))
            } else {
                Ok(None)
            }
        }
        _ => Ok(None),
    }
}

/// Compiled rule with pre-compiled regular expressions for fast in-game matching
#[derive(Debug, Clone)]
pub struct CompiledPickupRule {
    pub class_id: Option<u32>,
    pub base_code: Option<String>,
    pub match_name: bool,
    pub property: Option<String>,
    pub regex: Option<Regex>,
    pub name_regex: Option<Regex>,
    pub quality: Option<u32>,
    pub eth: Option<bool>,
    pub socks: Option<usize>,
    pub pickup: PickupAction,
}

impl CompiledPickupRule {
    pub fn matches(
        &self,
        class_id: u32,
        base_code_str: Option<&str>,
        base_code_u32: Option<u32>,
        quality: u32,
        is_eth: bool,
        socks: usize,
        name: &str,
        properties: &str,
    ) -> bool {
        if let Some(cid) = self.class_id {
            if cid != class_id {
                return false;
            }
        }

        if let Some(ref bcode) = self.base_code {
            let matched = if let Some(actual_str) = base_code_str {
                actual_str.eq_ignore_ascii_case(bcode)
            } else if let Some(actual_u32) = base_code_u32 {
                let bytes = actual_u32.to_le_bytes();
                let actual_decoded = String::from_utf8_lossy(&bytes)
                    .trim_matches(|c: char| c.is_whitespace() || c == '\0')
                    .to_lowercase();
                actual_decoded == *bcode
            } else {
                false
            };
            if !matched {
                return false;
            }
        }

        if let Some(q) = self.quality {
            if q != quality {
                return false;
            }
        }

        if let Some(eth) = self.eth {
            if eth != is_eth {
                return false;
            }
        }

        if let Some(s) = self.socks {
            if s != socks {
                return false;
            }
        }

        if let Some(re) = &self.name_regex {
            if !re.is_match(name) {
                return false;
            }
        }

        let target_text = if self.match_name { name } else { properties };

        if let Some(re) = &self.regex {
            if !re.is_match(target_text) {
                return false;
            }
        } else if let Some(prop) = &self.property {
            if !prop.is_empty() && !target_text.contains(prop) {
                return false;
            }
        }

        true
    }
}

fn parse_quality(q: &str) -> Option<u32> {
    match q.trim().to_lowercase().as_str() {
        "inferior" | "low" => Some(1),
        "normal" => Some(2),
        "superior" => Some(3),
        "magic" => Some(4),
        "set" => Some(5),
        "rare" => Some(6),
        "unique" => Some(7),
        "craft" | "crafted" => Some(8),
        _ => None,
    }
}

#[derive(Deserialize)]
struct RulesContainer {
    #[serde(alias = "item_colors")]
    rules: Vec<PickupRuleRaw>,
}

/// Parses raw TOML text into a list of compiled pickup rules
pub fn parse_pickup_rules(text: &str) -> Result<Vec<CompiledPickupRule>, String> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(Vec::new());
    }

    let toml_string = if trimmed.starts_with("rules =") || trimmed.starts_with("item_colors =") {
        trimmed.to_string()
    } else {
        format!("rules = [\n{}\n]", trimmed)
    };

    let container: RulesContainer =
        toml::from_str(&toml_string).map_err(|e| format!("Failed to parse pickup rules: {}", e))?;

    let mut compiled = Vec::with_capacity(container.rules.len());
    for (i, raw) in container.rules.into_iter().enumerate() {
        let regex = match raw.regex {
            Some(ref pat) => {
                let re = RegexBuilder::new(pat)
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| format!("Rule #{}: invalid regex '{}': {}", i + 1, pat, e))?;
                Some(re)
            }
            None => None,
        };

        let name_regex = match raw.name_regex {
            Some(ref pat) => {
                let re = RegexBuilder::new(pat)
                    .case_insensitive(true)
                    .build()
                    .map_err(|e| format!("Rule #{}: invalid name_regex '{}': {}", i + 1, pat, e))?;
                Some(re)
            }
            None => None,
        };

        let quality = raw.quality.as_deref().and_then(parse_quality);

        compiled.push(CompiledPickupRule {
            class_id: raw.class_id,
            base_code: raw.base_code,
            match_name: raw.match_name.unwrap_or(false),
            property: raw.property,
            regex,
            name_regex,
            quality,
            eth: raw.eth,
            socks: raw.socks,
            pickup: PickupAction::from(raw.pickup.unwrap_or(0)),
        });
    }

    Ok(compiled)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rules_with_string_and_hex_base_code() {
        let text = r#"
            { base_code = "box", pickup = 2 },
            { base_code = "bxa", pickup = 2 },
            { base_code = 0x20786F62, pickup = 1 },
            { id = 1204, pickup = 2 },
        "#;
        let rules = parse_pickup_rules(text).expect("should parse rules");
        assert_eq!(rules.len(), 4);
        assert_eq!(rules[0].base_code.as_deref(), Some("box"));
        assert_eq!(rules[0].pickup, PickupAction::Cube);
        assert_eq!(rules[1].base_code.as_deref(), Some("bxa"));
        assert_eq!(rules[1].pickup, PickupAction::Cube);
        assert_eq!(rules[2].base_code.as_deref(), Some("box"));
        assert_eq!(rules[2].pickup, PickupAction::Inventory);
        assert_eq!(rules[3].class_id, Some(1204));
        assert_eq!(rules[3].pickup, PickupAction::Cube);
    }

    #[test]
    fn test_matches_base_code_string() {
        let rule = CompiledPickupRule {
            class_id: None,
            base_code: Some("box".to_string()),
            match_name: false,
            property: None,
            regex: None,
            name_regex: None,
            quality: None,
            eth: None,
            socks: None,
            pickup: PickupAction::Cube,
        };

        // Match with base_code_str Some("box")
        assert!(rule.matches(1204, Some("box"), None, 7, false, 0, "Horadric Cube", ""));
        // Match case-insensitively
        assert!(rule.matches(1204, Some("BOX"), None, 7, false, 0, "Horadric Cube", ""));
        // Match with base_code_u32 0x20786F62 (b"box ")
        assert!(rule.matches(
            1204,
            None,
            Some(0x20786F62),
            7,
            false,
            0,
            "Horadric Cube",
            ""
        ));
        // Mismatch
        assert!(!rule.matches(1204, Some("bxa"), None, 7, false, 0, "Horadric Cube", ""));
    }
}
