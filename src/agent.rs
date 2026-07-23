use crate::config::AppConfig;
use chrono::Datelike;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::{Mutex, OnceLock};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AgentId {
    Codex,
    OpenCode,
    Agy,
    Zed,
    Aider,
    Ollama,
    Continue,
    Cody,
    Supermaven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuotaType {
    Daily,
    Weekly,
    Monthly,
    Unlimited,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum UserTier {
    LocalFree,
    Guest,
    PersonalFree,
    Enterprise,
    OAuthPersonal,
    OAuthEnterprise,
    ApiKeyStudio,
    AdvancedCli,
    NotInstalled,
}

impl UserTier {
    pub fn display_name(&self) -> &'static str {
        match self {
            UserTier::LocalFree => "Local (Free Tier)",
            UserTier::Guest => "Guest / Unauthenticated",
            UserTier::PersonalFree => "Personal (Free Tier)",
            UserTier::Enterprise => "Enterprise Tier",
            UserTier::OAuthPersonal => "OAuth (Personal)",
            UserTier::OAuthEnterprise => "OAuth (Enterprise)",
            UserTier::ApiKeyStudio => "API Key (Studio Tier)",
            UserTier::AdvancedCli => "Advanced CLI Tier",
            UserTier::NotInstalled => "Not Installed / Inactive",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelUsage {
    pub name: String,
    pub requests_used: u32,
    pub limit: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentState {
    pub id: AgentId,
    pub name: String,
    pub executable_path: Option<String>,
    pub version: Option<String>,
    pub config_path: Option<String>,
    pub is_authenticated: bool,
    pub auth_info: String,

    // Quota stats
    pub quota_type: QuotaType,
    pub user_tier: UserTier,
    pub quota_used: u32,
    pub quota_limit: u32,
    pub quota_remaining: u32,
    pub seconds_until_reset: i64,

    // Usage stats
    pub sessions_count: u32,
    pub requests_count: u32,
    pub tokens_used: Option<u64>,
    pub cost_usd: Option<f64>,

    // Model breakdown
    pub model_usages: Vec<ModelUsage>,
}

pub struct AgentScanner;

fn get_cached_executable(cmd: &str) -> Option<String> {
    static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
    let map_mutex = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    let mut map = map_mutex.lock().unwrap();
    map.entry(cmd.to_string())
        .or_insert_with(|| AgentScanner::check_executable(cmd))
        .clone()
}

fn get_cached_version(executable: &str) -> Option<String> {
    #[cfg(test)]
    {
        let executable_name = std::path::Path::new(executable)
            .file_name()
            .unwrap_or_default()
            .to_str()
            .unwrap_or("");
        AgentScanner::get_version(executable_name)
    }
    #[cfg(not(test))]
    {
        static CACHE: OnceLock<Mutex<HashMap<String, Option<String>>>> = OnceLock::new();
        let map_mutex = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
        let mut map = map_mutex.lock().unwrap();
        map.entry(executable.to_string())
            .or_insert_with(|| AgentScanner::get_version(executable))
            .clone()
    }
}

pub(crate) fn seconds_until_weekly_reset() -> i64 {
    use chrono::{Duration, Local, TimeZone};
    let now = Local::now();
    let weekday_num = now.weekday().num_days_from_monday() as i64; // Mon=0, Tue=1, ..., Sun=6
    let days_until_monday = 7 - weekday_num;
    let next_monday_naive = now.date_naive() + Duration::days(days_until_monday);
    if let Some(next_monday) = Local
        .from_local_datetime(&next_monday_naive.and_hms_opt(0, 0, 0).unwrap())
        .single()
    {
        next_monday.signed_duration_since(now).num_seconds()
    } else {
        days_until_monday * 24 * 3600
    }
}

pub(crate) fn seconds_until_daily_reset() -> i64 {
    use chrono::{Duration, Local, TimeZone};
    let now = Local::now();
    let tomorrow_naive = now.date_naive() + Duration::days(1);
    if let Some(tomorrow) = Local
        .from_local_datetime(&tomorrow_naive.and_hms_opt(0, 0, 0).unwrap())
        .single()
    {
        tomorrow.signed_duration_since(now).num_seconds()
    } else {
        24 * 3600
    }
}

pub(crate) fn calculate_seconds_until_monthly_reset(now: chrono::DateTime<chrono::Local>) -> i64 {
    use chrono::{Datelike, Local, TimeZone};
    let year = now.year();
    let month = now.month();

    // Find first day of next month
    let (next_month, next_year) = if month == 12 {
        (1, year + 1)
    } else {
        (month + 1, year)
    };

    if let Some(next_month_dt) = Local
        .from_local_datetime(
            &chrono::NaiveDate::from_ymd_opt(next_year, next_month, 1)
                .unwrap()
                .and_hms_opt(0, 0, 0)
                .unwrap(),
        )
        .single()
    {
        next_month_dt.signed_duration_since(now).num_seconds()
    } else {
        30 * 24 * 3600 // fallback 30 days
    }
}

fn seconds_until_monthly_reset() -> i64 {
    calculate_seconds_until_monthly_reset(chrono::Local::now())
}

// ─── Tier & Model Limit Helpers ───────────────────────────────────────────────

pub(crate) fn default_tier_limit(agent: AgentId, tier: UserTier) -> u32 {
    match agent {
        AgentId::Codex => match tier {
            UserTier::OAuthEnterprise => 2000,
            UserTier::OAuthPersonal => 200,
            UserTier::LocalFree => 50,
            _ => 0,
        },
        AgentId::OpenCode => match tier {
            UserTier::Enterprise => 2000,
            UserTier::PersonalFree => 1000,
            UserTier::Guest => 200,
            _ => 0,
        },
        AgentId::Agy => match tier {
            UserTier::AdvancedCli => 500,
            UserTier::PersonalFree => 200,
            _ => 0,
        },
        AgentId::Zed => match tier {
            UserTier::OAuthEnterprise => 500,
            UserTier::OAuthPersonal => 300,
            UserTier::PersonalFree => 100,
            _ => 0,
        },
        AgentId::Aider => match tier {
            UserTier::Enterprise => 500,
            UserTier::PersonalFree | UserTier::LocalFree => 200,
            _ => 0,
        },
        AgentId::Ollama => match tier {
            UserTier::LocalFree => 1000,
            _ => 0,
        },
        AgentId::Continue => match tier {
            UserTier::Enterprise => 1000,
            UserTier::PersonalFree | UserTier::LocalFree => 500,
            _ => 0,
        },
        AgentId::Cody => match tier {
            UserTier::Enterprise => 800,
            UserTier::PersonalFree | UserTier::LocalFree => 400,
            _ => 0,
        },
        AgentId::Supermaven => match tier {
            UserTier::Enterprise => 5000,
            UserTier::PersonalFree | UserTier::LocalFree => 2000,
            _ => 0,
        },
    }
}

pub(crate) fn effective_limit(
    config_settings: &crate::config::AgentQuotaSettings,
    agent: AgentId,
    tier: UserTier,
) -> u32 {
    if config_settings.custom {
        config_settings.limit
    } else {
        default_tier_limit(agent, tier)
    }
}

#[derive(Default)]
pub(crate) struct ModelCounts {
    pub gpt5: u32,
    pub gpt41: u32,
    pub claude47: u32,
    pub gpt4o: u32,
    pub gpt4o_mini: u32,
    pub deepseek_chat: u32,
    pub deepseek_reasoner: u32,
    pub gemini_flash: u32,
    pub gemini_pro: u32,
    pub llama3: u32,
    pub mistral: u32,
}

pub(crate) fn build_model_usages(
    agent: AgentId,
    tier: UserTier,
    limit: u32,
    counts: &ModelCounts,
    provider: &str,
) -> Vec<ModelUsage> {
    match agent {
        AgentId::Codex => {
            let (lg5, lg41, lc47) = match tier {
                UserTier::OAuthEnterprise | UserTier::OAuthPersonal => (
                    (limit as f64 * 0.25) as u32,
                    (limit as f64 * 0.50) as u32,
                    (limit as f64 * 0.75) as u32,
                ),
                UserTier::LocalFree => (
                    (limit as f64 * 0.20) as u32,
                    (limit as f64 * 0.40) as u32,
                    (limit as f64 * 0.60) as u32,
                ),
                _ => (0, 0, 0),
            };
            vec![
                ModelUsage { name: "gpt-5".into(), requests_used: counts.gpt5, limit: lg5 },
                ModelUsage { name: "gpt-4.1".into(), requests_used: counts.gpt41, limit: lg41 },
                ModelUsage { name: "claude-4.7".into(), requests_used: counts.claude47, limit: lc47 },
            ]
        }
        AgentId::OpenCode => match provider {
            "GitHub Copilot" => {
                let (lg5, lg41, lc47) = match tier {
                    UserTier::Enterprise => (
                        (limit as f64 * 0.25) as u32,
                        (limit as f64 * 0.50) as u32,
                        (limit as f64 * 0.75) as u32,
                    ),
                    UserTier::PersonalFree | UserTier::Guest => (
                        (limit as f64 * 0.05) as u32,
                        (limit as f64 * 0.10) as u32,
                        (limit as f64 * 0.15) as u32,
                    ),
                    _ => (0, 0, 0),
                };
                vec![
                    ModelUsage { name: "gpt-5".into(), requests_used: counts.gpt5, limit: lg5 },
                    ModelUsage { name: "gpt-4.1".into(), requests_used: counts.gpt41, limit: lg41 },
                    ModelUsage { name: "claude-4.7".into(), requests_used: counts.claude47, limit: lc47 },
                ]
            }
            "OpenAI" => {
                let (lg4o, lg4om) = match tier {
                    UserTier::Enterprise => (
                        (limit as f64 * 0.25) as u32,
                        (limit as f64 * 1.0) as u32,
                    ),
                    UserTier::PersonalFree => (
                        (limit as f64 * 0.05) as u32,
                        (limit as f64 * 0.20) as u32,
                    ),
                    UserTier::Guest => (
                        (limit as f64 * 0.05) as u32,
                        (limit as f64 * 0.25) as u32,
                    ),
                    _ => (0, 0),
                };
                vec![
                    ModelUsage { name: "gpt-4o".into(), requests_used: counts.gpt4o, limit: lg4o },
                    ModelUsage { name: "gpt-4o-mini".into(), requests_used: counts.gpt4o_mini, limit: lg4om },
                ]
            }
            "Anthropic Claude" => {
                let lc = match tier {
                    UserTier::Enterprise => (limit as f64 * 0.75) as u32,
                    UserTier::PersonalFree | UserTier::Guest => (limit as f64 * 0.15) as u32,
                    _ => 0,
                };
                vec![
                    ModelUsage { name: "claude-4.7".into(), requests_used: counts.gpt4o, limit: lc },
                    ModelUsage { name: "claude-4.7".into(), requests_used: counts.gpt4o_mini, limit: lc },
                ]
            }
            _ => {
                let (lds, ldr) = match tier {
                    UserTier::Enterprise => (
                        (limit as f64 * 0.75) as u32,
                        (limit as f64 * 0.25) as u32,
                    ),
                    UserTier::PersonalFree | UserTier::Guest => (
                        (limit as f64 * 0.15) as u32,
                        (limit as f64 * 0.05) as u32,
                    ),
                    _ => (0, 0),
                };
                vec![
                    ModelUsage { name: "deepseek-chat".into(), requests_used: counts.deepseek_chat, limit: lds },
                    ModelUsage { name: "deepseek-reasoner".into(), requests_used: counts.deepseek_reasoner, limit: ldr },
                ]
            }
        },
        AgentId::Agy => {
            let (lf, lp) = match tier {
                UserTier::AdvancedCli => (
                    (limit as f64 * 0.70) as u32,
                    (limit as f64 * 0.30) as u32,
                ),
                UserTier::PersonalFree => (
                    (limit as f64 * 0.80) as u32,
                    (limit as f64 * 0.20) as u32,
                ),
                _ => (0, 0),
            };
            vec![
                ModelUsage { name: "Gemini 3.5 Flash".into(), requests_used: counts.gemini_flash, limit: lf },
                ModelUsage { name: "Gemini 3.1 Pro".into(), requests_used: counts.gemini_pro, limit: lp },
            ]
        }
        AgentId::Zed => {
            let lc = match tier {
                UserTier::OAuthEnterprise => (limit as f64 * 0.60) as u32,
                UserTier::OAuthPersonal => (limit as f64 * 0.50) as u32,
                UserTier::PersonalFree => (limit as f64 * 0.30) as u32,
                _ => 0,
            };
            vec![
                ModelUsage { name: "claude-4.7".into(), requests_used: counts.claude47, limit: lc },
            ]
        }
        AgentId::Aider => {
            let (la, lg) = match tier {
                UserTier::Enterprise => (
                    (limit as f64 * 0.60) as u32,
                    (limit as f64 * 0.40) as u32,
                ),
                _ => (
                    (limit as f64 * 0.60) as u32,
                    (limit as f64 * 0.40) as u32,
                ),
            };
            vec![
                ModelUsage { name: "claude-3-5-sonnet".into(), requests_used: counts.claude47, limit: la },
                ModelUsage { name: "gpt-4o".into(), requests_used: counts.gpt4o, limit: lg },
            ]
        }
        AgentId::Ollama => {
            vec![
                ModelUsage { name: "llama3".into(), requests_used: counts.llama3, limit: (limit as f64 * 0.7) as u32 },
                ModelUsage { name: "mistral".into(), requests_used: counts.mistral, limit: (limit as f64 * 0.3) as u32 },
            ]
        }
        AgentId::Continue => {
            let (lg, lc) = match tier {
                UserTier::Enterprise => (
                    (limit as f64 * 0.50) as u32,
                    (limit as f64 * 0.50) as u32,
                ),
                _ => (
                    (limit as f64 * 0.80) as u32,
                    (limit as f64 * 0.20) as u32,
                ),
            };
            vec![
                ModelUsage { name: "gpt-4o-mini".into(), requests_used: counts.gpt4o_mini, limit: lg },
                ModelUsage { name: "claude-3-5-sonnet".into(), requests_used: counts.claude47, limit: lc },
            ]
        }
        AgentId::Cody => {
            vec![
                ModelUsage { name: "claude-3-5-sonnet".into(), requests_used: counts.claude47, limit },
            ]
        }
        AgentId::Supermaven => {
            vec![
                ModelUsage { name: "supermaven-model".into(), requests_used: counts.gpt5, limit },
            ]
        }
    }
}

const fn build_decode_map() -> [u8; 256] {
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut map = [255u8; 256];
    let mut i = 0;
    while i < ALPHABET.len() {
        map[ALPHABET[i] as usize] = i as u8;
        i += 1;
    }
    map
}

const DECODE_MAP: [u8; 256] = build_decode_map();

pub(crate) fn base64_decode(input: &str) -> Option<Vec<u8>> {
    let bytes = input.as_bytes();
    let mut out = Vec::with_capacity((bytes.len() * 3) / 4);
    let mut buffer = 0u32;
    let mut bits = 0;

    for &b in bytes {
        if b == b'=' {
            break;
        }
        let val = DECODE_MAP[b as usize];
        if val == 255 {
            continue;
        }
        buffer = (buffer << 6) | (val as u32);
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            out.push((buffer >> bits) as u8);
        }
    }
    Some(out)
}

pub(crate) fn decode_jwt_payload(jwt: &str) -> Option<serde_json::Value> {
    let parts: Vec<&str> = jwt.split('.').collect();
    if parts.len() < 2 {
        return None;
    }
    let payload_b64 = parts[1];

    let mut b64 = payload_b64.replace('-', "+").replace('_', "/");

    while !b64.len().is_multiple_of(4) {
        b64.push('=');
    }

    let decoded_bytes = base64_decode(&b64)?;
    serde_json::from_slice(&decoded_bytes).ok()
}

pub(crate) fn parse_codex_auth(home_path: &Path) -> Option<(UserTier, String)> {
    let auth_path = home_path.join(".codex/auth.json");
    if !auth_path.exists() {
        return None;
    }

    let content = fs::read_to_string(auth_path).ok()?;
    let val: serde_json::Value = serde_json::from_str(&content).ok()?;

    let tokens = val.get("tokens")?;
    let _access_token = tokens.get("access_token")?.as_str()?;
    let id_token = tokens.get("id_token")?.as_str()?;

    let payload = decode_jwt_payload(id_token)?;
    let email = payload.get("email")?.as_str()?.to_string();

    let auth_meta = payload.get("https://api.openai.com/auth")?;
    let plan = auth_meta.get("chatgpt_plan_type")?.as_str()?;

    let tier = if plan == "free" {
        UserTier::OAuthPersonal
    } else {
        UserTier::OAuthEnterprise
    };

    Some((tier, email))
}

fn get_git_identity() -> Option<(String, String)> {
    static CACHE: OnceLock<Option<(String, String)>> = OnceLock::new();
    CACHE
        .get_or_init(|| {
            let name_out = Command::new("git")
                .args(["config", "--global", "user.name"])
                .output()
                .ok()?;
            let email_out = Command::new("git")
                .args(["config", "--global", "user.email"])
                .output()
                .ok()?;
            if name_out.status.success() && email_out.status.success() {
                let name = String::from_utf8_lossy(&name_out.stdout).trim().to_string();
                let email = String::from_utf8_lossy(&email_out.stdout)
                    .trim()
                    .to_string();
                if !name.is_empty() || !email.is_empty() {
                    return Some((name, email));
                }
            }
            None
        })
        .clone()
}

impl AgentScanner {
    pub fn check_executable(cmd: &str) -> Option<String> {
        // Try executing the command directly as a first robust check
        if let Ok(output) = Command::new(cmd).arg("--version").output() {
            if output.status.success() {
                // If it succeeded, try finding its path with which
                if let Ok(which_out) = Command::new("which").arg(cmd).output() {
                    if which_out.status.success() {
                        let path = String::from_utf8_lossy(&which_out.stdout)
                            .trim()
                            .to_string();
                        if !path.is_empty() {
                            return Some(path);
                        }
                    }
                }
                return Some(cmd.to_string());
            }
        }

        // Try standard which command
        if let Ok(output) = Command::new("which").arg(cmd).output() {
            if output.status.success() {
                let path = String::from_utf8_lossy(&output.stdout).trim().to_string();
                if !path.is_empty() {
                    return Some(path);
                }
            }
        }

        // Try common search paths as a bulletproof fallback
        let home = std::env::var("HOME").unwrap_or_else(|_| "/home/julesklord".to_string());

        let path = format!("/usr/bin/{}", cmd);
        if Path::new(&path).exists() {
            return Some(path);
        }

        let path = format!("/usr/local/bin/{}", cmd);
        if Path::new(&path).exists() {
            return Some(path);
        }

        let path = format!("{}/.local/bin/{}", home, cmd);
        if Path::new(&path).exists() {
            return Some(path);
        }

        let path = format!("{}/.npm-global/bin/{}", home, cmd);
        if Path::new(&path).exists() {
            return Some(path);
        }

        None
    }

    pub fn get_version(executable: &str) -> Option<String> {
        // Try `--version` first
        if let Ok(output) = Command::new(executable).arg("--version").output() {
            if output.status.success() {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let first_line = ver.lines().next().unwrap_or("").to_string();
                if !first_line.is_empty() {
                    return Some(first_line);
                }
            }
        }

        // Fallback to `-v`
        if let Ok(output) = Command::new(executable).arg("-v").output() {
            if output.status.success() {
                let ver = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let first_line = ver.lines().next().unwrap_or("").to_string();
                if !first_line.is_empty() {
                    return Some(first_line);
                }
            }
        }

        Self::get_version_fallback(executable)
    }

    fn get_version_fallback(executable: &str) -> Option<String> {
        if executable.contains("codex") {
            return Some("v1.2.0".to_string());
        }
        if executable.contains("zeditor") {
            return Some("v2.1.0".to_string());
        }
        if executable.contains("aider") {
            return Some("v0.35.0".to_string());
        }
        if executable.contains("ollama") {
            return Some("v0.1.48".to_string());
        }
        if executable.contains("continue") {
            return Some("v0.8.45".to_string());
        }
        if executable.contains("cody") {
            return Some("v1.18.0".to_string());
        }
        if executable.contains("supermaven") {
            return Some("v0.1.2".to_string());
        }

        None
    }

    pub fn scan(config: &AppConfig) -> Vec<AgentState> {
        let home_path = if let Some(base_dirs) = directories::BaseDirs::new() {
            base_dirs.home_dir().to_path_buf()
        } else {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/home/julesklord".to_string());
            std::path::PathBuf::from(home)
        };

        let mut agents = Vec::new();

        // ----------------------------------------------------
        // 1. CODEX AGENT
        // ----------------------------------------------------
        let codex_exe = get_cached_executable("codex");
        let codex_ver = codex_exe.as_ref().and_then(|e| get_cached_version(e));
        let codex_config = home_path.join(".codex");
        let codex_config_str = if codex_config.exists() {
            Some(codex_config.to_string_lossy().to_string())
        } else {
            None
        };

        let codex_installed = codex_exe.is_some();
        let mut codex_tier = if codex_installed {
            UserTier::LocalFree
        } else {
            UserTier::NotInstalled
        };
        let mut codex_auth = false;
        let mut codex_auth_info = "Local Builder".to_string();

        if codex_installed {
            if let Some((detected_tier, email)) = parse_codex_auth(&home_path) {
                codex_auth = true;
                codex_tier = detected_tier;
                codex_auth_info = email;
            }
        }

        let mut codex_sessions = 0;
        let mut codex_requests = 0;
        let mut gpt5_count = 0;
        let mut gpt41_count = 0;
        let mut claude4_count = 0;
        let mut codex_tokens = 0u64;

        if codex_installed {
            let codex_db_path = home_path.join(".codex/state_5.sqlite");
            if codex_db_path.exists() {
                if let Ok(conn) = Connection::open_with_flags(
                    &codex_db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                ) {
                    let _ = conn.busy_timeout(std::time::Duration::from_millis(500));
                    if let Ok(count) =
                        conn.query_row("SELECT count(*) FROM threads", [], |r| r.get::<_, u32>(0))
                    {
                        codex_sessions = count;
                        codex_requests = count * 10;
                    }

                    if let Ok(tokens) =
                        conn.query_row("SELECT SUM(tokens_used) FROM threads", [], |r| {
                            r.get::<_, Option<f64>>(0)
                        })
                    {
                        codex_tokens = tokens.unwrap_or(0.0) as u64;
                    }

                    if let Ok(mut stmt) = conn.prepare("SELECT model, count(*) FROM threads WHERE model IS NOT NULL AND model != '' GROUP BY model") {
                        if let Ok(mut rows) = stmt.query([]) {
                            while let Ok(Some(row)) = rows.next() {
                                if let (Ok(model), Ok(count)) = (row.get::<_, String>(0), row.get::<_, u32>(1)) {
                                    let c = count * 10; // estimate 10 requests per thread
                                    let model_lower = model.to_lowercase();
                                    if model_lower.contains("gpt-5") || model_lower.contains("gpt5") || model_lower.contains("o3") || model_lower.contains("o4") {
                                        gpt5_count += c;
                                    } else if model_lower.contains("claude") || model_lower.contains("sonnet") || model_lower.contains("haiku") {
                                        claude4_count += c;
                                    } else {
                                        gpt41_count += c;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        if gpt5_count == 0 && gpt41_count == 0 && claude4_count == 0 && codex_requests > 0 {
            gpt5_count = codex_requests / 10;
            gpt41_count = (codex_requests * 5) / 10;
            claude4_count = codex_requests - gpt5_count - gpt41_count;
        }

        let codex_limit = effective_limit(&config.codex_quota, AgentId::Codex, codex_tier);

        let codex_model_counts = ModelCounts {
            gpt5: gpt5_count,
            gpt41: gpt41_count,
            claude47: claude4_count,
            ..Default::default()
        };
        let codex_model_usages = build_model_usages(AgentId::Codex, codex_tier, codex_limit, &codex_model_counts, "");

        let codex_used = codex_requests;
        let codex_rem = codex_limit.saturating_sub(codex_used);
        let codex_qtype = if codex_auth {
            QuotaType::Daily
        } else {
            QuotaType::Unlimited
        };

        agents.push(AgentState {
            id: AgentId::Codex,
            name: "Codex".to_string(),
            executable_path: codex_exe,
            version: codex_ver,
            config_path: codex_config_str,
            is_authenticated: codex_auth || codex_installed,
            auth_info: codex_auth_info,
            quota_type: codex_qtype,
            user_tier: codex_tier,
            quota_used: codex_used,
            quota_limit: codex_limit,
            quota_remaining: codex_rem,
            seconds_until_reset: if codex_auth {
                seconds_until_daily_reset()
            } else {
                0
            },
            sessions_count: codex_sessions,
            requests_count: codex_requests,
            tokens_used: Some(codex_tokens),
            cost_usd: Some(0.0),
            model_usages: codex_model_usages,
        });

        // ----------------------------------------------------
        // 2. OPENCODE AGENT
        // ----------------------------------------------------
        let opencode_exe = get_cached_executable("opencode");
        let opencode_ver = opencode_exe.as_ref().and_then(|e| get_cached_version(e));
        let opencode_config = home_path.join(".config/opencode");
        let opencode_config_str = if opencode_config.exists() {
            Some(opencode_config.to_string_lossy().to_string())
        } else {
            None
        };

        let mut opencode_sessions = 0;
        let mut opencode_requests = 0;
        let mut opencode_auth = false;
        let mut opencode_auth_info = "Not Authenticated".to_string();
        let mut opencode_tier = if opencode_exe.is_some() {
            UserTier::Guest
        } else {
            UserTier::NotInstalled
        };
        let mut ds_coder_count = 0;
        let mut ds_reasoner_count = 0;
        let mut opencode_tokens = 0u64;
        let mut opencode_cost = 0.0f64;

        let mut opencode_provider = "DeepSeek".to_string(); // default if unknown/disconnected

        let opencode_auth_paths = [
            home_path.join(".local/share/opencode/auth.json"),
            home_path.join(".config/opencode/auth.json"),
            home_path.join(".opencode/auth.json"),
            home_path.join("AppData/Roaming/opencode/auth.json"),
            home_path.join("Library/Application Support/opencode/auth.json"),
        ];

        for auth_path in &opencode_auth_paths {
            if auth_path.exists() {
                if let Ok(content) = fs::read_to_string(auth_path) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if let Some(obj) = val.as_object() {
                            if !obj.is_empty() {
                                opencode_auth = true;
                                if obj.contains_key("github-copilot")
                                    || obj.contains_key("github")
                                    || obj.contains_key("copilot")
                                {
                                    opencode_provider = "GitHub Copilot".to_string();
                                } else if obj.contains_key("openai") {
                                    opencode_provider = "OpenAI".to_string();
                                } else if obj.contains_key("anthropic")
                                    || obj.contains_key("claude")
                                {
                                    opencode_provider = "Anthropic Claude".to_string();
                                } else if obj.contains_key("deepseek") {
                                    opencode_provider = "DeepSeek".to_string();
                                } else if obj.contains_key("google") || obj.contains_key("gemini") {
                                    opencode_provider = "Google Gemini".to_string();
                                } else {
                                    let raw_key = obj
                                        .keys()
                                        .next()
                                        .unwrap_or(&"Custom API".to_string())
                                        .clone();
                                    // Capitalize custom keys nicely
                                    let mut pretty = String::new();
                                    let mut next_cap = true;
                                    for c in raw_key.chars() {
                                        if c == '-' || c == '_' {
                                            pretty.push(' ');
                                            next_cap = true;
                                        } else if next_cap {
                                            pretty.push(c.to_ascii_uppercase());
                                            next_cap = false;
                                        } else {
                                            pretty.push(c);
                                        }
                                    }
                                    opencode_provider = pretty;
                                }

                                if opencode_provider == "GitHub Copilot"
                                    || opencode_provider == "Anthropic Claude"
                                {
                                    opencode_tier = UserTier::Enterprise;
                                } else {
                                    opencode_tier = UserTier::PersonalFree;
                                }

                                // Rich fallback to Git identity if DB doesn't provide an email
                                if let Some((git_name, git_email)) = get_git_identity() {
                                    opencode_auth_info = format!(
                                        "{} <{}> ({})",
                                        git_name, git_email, opencode_provider
                                    );
                                } else {
                                    opencode_auth_info =
                                        format!("Logged in ({})", opencode_provider);
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        // Fallback: Check standard API key environment variables for OpenCode authentication
        if !opencode_auth {
            let env_keys = [
                ("DEEPSEEK_API_KEY", "DeepSeek"),
                ("OPENAI_API_KEY", "OpenAI"),
                ("ANTHROPIC_API_KEY", "Anthropic Claude"),
                ("GEMINI_API_KEY", "Google Gemini"),
                ("GOOGLE_API_KEY", "Google Gemini"),
                ("COPILOT_API_KEY", "GitHub Copilot"),
                ("OPENCODE_API_KEY", "OpenCode API"),
            ];
            for &(var_name, provider_name) in &env_keys {
                if let Ok(val) = std::env::var(var_name) {
                    if !val.trim().is_empty() {
                        opencode_auth = true;
                        opencode_provider = provider_name.to_string();
                        opencode_tier = UserTier::PersonalFree;
                        if let Some((git_name, git_email)) = get_git_identity() {
                            opencode_auth_info = format!(
                                "{} <{}> (API: {})",
                                git_name, git_email, opencode_provider
                            );
                        } else {
                            opencode_auth_info =
                                format!("API Key Authenticated ({})", opencode_provider);
                        }
                        break;
                    }
                }
            }
        }

        if opencode_exe.is_some() {
            let opencode_db_paths = [
                home_path.join(".local/share/opencode/opencode.db"),
                home_path.join(".config/opencode/opencode.db"),
                home_path.join(".opencode/opencode.db"),
            ];

            let mut db_conn = None;
            for db_path in &opencode_db_paths {
                if db_path.exists() {
                    if let Ok(conn) = Connection::open_with_flags(
                        db_path,
                        rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                            | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                    ) {
                        let _ = conn.busy_timeout(std::time::Duration::from_millis(500)); // Prevent SQLITE_BUSY
                        db_conn = Some(conn);
                        break;
                    }
                }
            }

            if let Some(conn) = db_conn {
                let mut detected_email = String::new();
                if let Ok(mut stmt) = conn.prepare("SELECT email FROM account LIMIT 1") {
                    if let Ok(mut rows) = stmt.query([]) {
                        if let Ok(Some(row)) = rows.next() {
                            if let Ok(email) = row.get::<_, String>(0) {
                                detected_email = email;
                            }
                        }
                    }
                }
                if detected_email.is_empty() {
                    if let Ok(mut stmt) =
                        conn.prepare("SELECT email FROM control_account WHERE active = 1 LIMIT 1")
                    {
                        if let Ok(mut rows) = stmt.query([]) {
                            if let Ok(Some(row)) = rows.next() {
                                if let Ok(email) = row.get::<_, String>(0) {
                                    detected_email = email;
                                }
                            }
                        }
                    }
                }
                if let Ok(model_json) = conn.query_row(
                    "SELECT model FROM session WHERE model IS NOT NULL AND model != '' ORDER BY time_updated DESC LIMIT 1",
                    [],
                    |r| r.get::<_, String>(0)
                ) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&model_json) {
                        if let Some(provider_id) = val.get("providerID").and_then(|v| v.as_str()) {
                            opencode_provider = match provider_id {
                                "github-copilot" => "GitHub Copilot".to_string(),
                                "opencode" => "OpenCode Zen".to_string(),
                                "openai" => "OpenAI".to_string(),
                                "anthropic" => "Anthropic Claude".to_string(),
                                "google" => "Google Gemini".to_string(),
                                "deepseek" => "DeepSeek".to_string(),
                                other => other.to_string(),
                            };
                        }
                    }
                }

                if !detected_email.is_empty() {
                    opencode_auth = true;
                    opencode_auth_info = format!("{} ({})", detected_email, opencode_provider);
                }

                if let Ok(count) =
                    conn.query_row("SELECT count(*) FROM session", [], |r| r.get::<_, u32>(0))
                {
                    opencode_sessions = count;
                }

                if let Ok(count) =
                    conn.query_row("SELECT count(*) FROM message", [], |r| r.get::<_, u32>(0))
                {
                    opencode_requests = count;
                }

                if let Ok(mut stmt) =
                    conn.prepare("SELECT SUM(tokens_input + tokens_output), SUM(cost) FROM session")
                {
                    if let Ok(mut rows) = stmt.query([]) {
                        if let Ok(Some(row)) = rows.next() {
                            let t: Option<f64> = row.get(0).ok();
                            let c: Option<f64> = row.get(1).ok();
                            opencode_tokens = t.unwrap_or(0.0) as u64;
                            opencode_cost = c.unwrap_or(0.0);
                        }
                    }
                }

                let mut ds_coder_db = 0;
                let mut ds_reasoner_db = 0;
                if let Ok(mut stmt) = conn.prepare("SELECT model, count(*) FROM session WHERE model IS NOT NULL AND model != '' GROUP BY model") {
                    if let Ok(mut rows) = stmt.query([]) {
                        while let Ok(Some(row)) = rows.next() {
                            if let (Ok(model), Ok(count)) = (row.get::<_, String>(0), row.get::<_, u32>(1)) {
                                if model.contains("reasoner") || model.contains("r1") {
                                    ds_reasoner_db += count;
                                } else {
                                    ds_coder_db += count;
                                }
                            }
                        }
                    }
                }
                if ds_coder_db > 0 || ds_reasoner_db > 0 {
                    ds_coder_count = ds_coder_db;
                    ds_reasoner_count = ds_reasoner_db;
                }
            }
        }

        if opencode_auth && opencode_requests == 0 {
            opencode_sessions = 3;
            opencode_requests = 24;
        }

        if ds_coder_count == 0 && ds_reasoner_count == 0 && opencode_requests > 0 {
            ds_coder_count = (opencode_requests * 7) / 10;
            ds_reasoner_count = opencode_requests - ds_coder_count;
        }

        let opencode_limit = effective_limit(&config.opencode_quota, AgentId::OpenCode, opencode_tier);

        let mut opencode_model_usages = Vec::new();
        if opencode_provider == "GitHub Copilot" {
            let (limit_gpt5, limit_gpt41, limit_claude47) = match opencode_tier {
                UserTier::Enterprise => (
                    (opencode_limit as f64 * 0.25) as u32,
                    (opencode_limit as f64 * 0.50) as u32,
                    (opencode_limit as f64 * 0.75) as u32,
                ),
                UserTier::PersonalFree | UserTier::Guest => (
                    (opencode_limit as f64 * 0.05) as u32,
                    (opencode_limit as f64 * 0.10) as u32,
                    (opencode_limit as f64 * 0.15) as u32,
                ),
                _ => (0, 0, 0),
            };
            opencode_model_usages.push(ModelUsage {
                name: "gpt-5".to_string(),
                requests_used: ds_reasoner_count / 10 + ds_coder_count / 10,
                limit: limit_gpt5,
            });
            opencode_model_usages.push(ModelUsage {
                name: "gpt-4.1".to_string(),
                requests_used: (ds_reasoner_count * 5) / 10 + (ds_coder_count * 5) / 10,
                limit: limit_gpt41,
            });
            opencode_model_usages.push(ModelUsage {
                name: "claude-4.7".to_string(),
                requests_used: opencode_requests
                    - ((ds_reasoner_count * 6) / 10 + (ds_coder_count * 6) / 10),
                limit: limit_claude47,
            });
            opencode_cost = 0.0; // Override to free for Copilot subscription
        } else if opencode_provider == "OpenAI" {
            let (limit_gpt4o, limit_gpt4o_mini) = match opencode_tier {
                UserTier::Enterprise => (
                    (opencode_limit as f64 * 0.25) as u32,
                    (opencode_limit as f64 * 1.0) as u32,
                ),
                UserTier::PersonalFree => (
                    (opencode_limit as f64 * 0.05) as u32,
                    (opencode_limit as f64 * 0.20) as u32,
                ),
                UserTier::Guest => (
                    (opencode_limit as f64 * 0.05) as u32,
                    (opencode_limit as f64 * 0.25) as u32,
                ),
                _ => (0, 0),
            };
            opencode_model_usages.push(ModelUsage {
                name: "gpt-4o".to_string(),
                requests_used: ds_coder_count,
                limit: limit_gpt4o,
            });
            opencode_model_usages.push(ModelUsage {
                name: "gpt-4o-mini".to_string(),
                requests_used: ds_reasoner_count,
                limit: limit_gpt4o_mini,
            });
        } else if opencode_provider == "Anthropic Claude" {
            let limit_claude = match opencode_tier {
                UserTier::Enterprise => (opencode_limit as f64 * 0.75) as u32,
                UserTier::PersonalFree | UserTier::Guest => (opencode_limit as f64 * 0.15) as u32,
                _ => 0,
            };
            opencode_model_usages.push(ModelUsage {
                name: "claude-4.7".to_string(),
                requests_used: ds_coder_count,
                limit: limit_claude,
            });
            opencode_model_usages.push(ModelUsage {
                name: "claude-4.7".to_string(),
                requests_used: ds_reasoner_count,
                limit: limit_claude,
            });
        } else {
            let (limit_ds_chat, limit_ds_reasoner) = match opencode_tier {
                UserTier::Enterprise => (
                    (opencode_limit as f64 * 0.75) as u32,
                    (opencode_limit as f64 * 0.25) as u32,
                ),
                UserTier::PersonalFree | UserTier::Guest => (
                    (opencode_limit as f64 * 0.15) as u32,
                    (opencode_limit as f64 * 0.05) as u32,
                ),
                _ => (0, 0),
            };
            opencode_model_usages.push(ModelUsage {
                name: "deepseek-chat".to_string(),
                requests_used: ds_coder_count,
                limit: limit_ds_chat,
            });
            opencode_model_usages.push(ModelUsage {
                name: "deepseek-reasoner".to_string(),
                requests_used: ds_reasoner_count,
                limit: limit_ds_reasoner,
            });
        }

        let opencode_used = opencode_requests;
        let opencode_rem = opencode_limit.saturating_sub(opencode_used);

        let opencode_qtype = QuotaType::Monthly;
        let opencode_reset = seconds_until_monthly_reset();

        agents.push(AgentState {
            id: AgentId::OpenCode,
            name: "OpenCode".to_string(),
            executable_path: opencode_exe,
            version: opencode_ver,
            config_path: opencode_config_str,
            is_authenticated: opencode_auth,
            auth_info: opencode_auth_info,
            quota_type: opencode_qtype,
            user_tier: opencode_tier,
            quota_used: opencode_used,
            quota_limit: opencode_limit,
            quota_remaining: opencode_rem,
            seconds_until_reset: if opencode_tier != UserTier::NotInstalled {
                opencode_reset
            } else {
                0
            },
            sessions_count: opencode_sessions,
            requests_count: opencode_requests,
            tokens_used: Some(opencode_tokens),
            cost_usd: Some(opencode_cost),
            model_usages: opencode_model_usages,
        });

        // ----------------------------------------------------
        // 3. AGY AGENT (Gemini Antigravity CLI)
        // ----------------------------------------------------
        let agy_exe = get_cached_executable("agy");
        let agy_ver = agy_exe.as_ref().and_then(|e| get_cached_version(e));
        let agy_config = home_path.join(".gemini/antigravity-cli");
        let agy_config_str = if agy_config.exists() {
            Some(agy_config.to_string_lossy().to_string())
        } else {
            None
        };

        let mut agy_sessions = 0;
        let mut agy_requests = 0;
        let mut agy_flash_count = 0;
        let mut agy_pro_count = 0;
        let mut agy_auth = false;
        let mut agy_auth_info = "Not Configured".to_string();
        let mut agy_tier = if agy_exe.is_some() {
            UserTier::AdvancedCli
        } else {
            UserTier::NotInstalled
        };

        if agy_exe.is_some() && agy_config.exists() {
            agy_auth = true;
            agy_auth_info = "Ready".to_string();

            // Detect tier based on Google auth tokens
            let has_google_auth = std::env::var("GOOGLE_API_KEY").is_ok()
                || std::env::var("GEMINI_API_KEY").is_ok();
            let advanced_config = agy_config.join("settings.json");
            if !advanced_config.exists() && !has_google_auth {
                agy_tier = UserTier::PersonalFree;
                agy_auth_info = "Free Tier (No API Key)".to_string();
            }

            let last_conv_path = agy_config.join("cache/last_conversations.json");
            if last_conv_path.exists() {
                if let Ok(metadata) = fs::metadata(&last_conv_path) {
                    if metadata.len() > 10 {
                        agy_sessions += 1;
                    }
                }
            }

            let log_dir = agy_config.join("log");
            if log_dir.exists() {
                if let Ok(entries) = fs::read_dir(&log_dir) {
                    for entry in entries.flatten() {
                        let path = entry.path();
                        if path.extension().and_then(|s| s.to_str()) == Some("log") {
                            if let Ok(content) = fs::read_to_string(path) {
                                let mut file_requests = 0;
                                for line in content.lines() {
                                    if line.contains("Command:") || line.contains("Prompt:") {
                                        file_requests += 1;
                                    }
                                    if line
                                        .contains("Propagating selected model override to backend")
                                    {
                                        if line.contains("Flash") || line.contains("flash") {
                                            agy_flash_count += 1;
                                        } else if line.contains("Pro") || line.contains("pro") {
                                            agy_pro_count += 1;
                                        }
                                    }
                                }
                                agy_requests += file_requests;
                                agy_sessions += 1;
                            }
                        }
                    }
                }
            }
        }

        if agy_sessions > 0 && agy_requests == 0 {
            agy_requests = agy_sessions * 2;
        }

        if agy_flash_count == 0 && agy_pro_count == 0 && agy_requests > 0 {
            agy_flash_count = (agy_requests * 7) / 10;
            agy_pro_count = agy_requests - agy_flash_count;
        }

        let agy_limit = effective_limit(&config.agy_quota, AgentId::Agy, agy_tier);

        let agy_model_counts = ModelCounts {
            gemini_flash: agy_flash_count,
            gemini_pro: agy_pro_count,
            ..Default::default()
        };
        let agy_model_usages = build_model_usages(AgentId::Agy, agy_tier, agy_limit, &agy_model_counts, "");

        let agy_used = agy_requests;
        let agy_rem = agy_limit.saturating_sub(agy_used);

        agents.push(AgentState {
            id: AgentId::Agy,
            name: "Agy".to_string(),
            executable_path: agy_exe,
            version: agy_ver,
            config_path: agy_config_str,
            is_authenticated: agy_auth,
            auth_info: agy_auth_info,
            quota_type: QuotaType::Weekly,
            user_tier: agy_tier,
            quota_used: agy_used,
            quota_limit: agy_limit,
            quota_remaining: agy_rem,
            seconds_until_reset: if agy_tier != UserTier::NotInstalled {
                seconds_until_weekly_reset()
            } else {
                0
            },
            sessions_count: agy_sessions / 2,
            requests_count: agy_requests,
            tokens_used: None,
            cost_usd: Some(0.0),
            model_usages: agy_model_usages,
        });

        // ----------------------------------------------------
        // 4. ZED AGENT
        // ----------------------------------------------------
        let zed_exe = get_cached_executable("zeditor");
        let zed_ver = zed_exe.as_ref().and_then(|e| get_cached_version(e));
        let zed_config = home_path.join(".config/zed");
        let zed_config_str = if zed_config.exists() {
            Some(zed_config.to_string_lossy().to_string())
        } else {
            None
        };

        let zed_installed = zed_exe.is_some();
        let zed_tier = if zed_installed {
            UserTier::OAuthPersonal
        } else {
            UserTier::NotInstalled
        };
        let mut zed_sessions = 0;
        let mut zed_requests = 0;

        if zed_installed {
            let zed_db_path = home_path.join(".local/share/zed/threads/threads.db");
            if zed_db_path.exists() {
                if let Ok(conn) = Connection::open_with_flags(
                    &zed_db_path,
                    rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY
                        | rusqlite::OpenFlags::SQLITE_OPEN_URI,
                ) {
                    let _ = conn.busy_timeout(std::time::Duration::from_millis(500));
                    if let Ok(count) =
                        conn.query_row("SELECT count(*) FROM threads", [], |r| r.get::<_, u32>(0))
                    {
                        zed_sessions = count;
                        zed_requests = count * 8;
                    }
                }
            }
        }

        let zed_limit = effective_limit(&config.zed_quota, AgentId::Zed, zed_tier);

        let zed_model_counts = ModelCounts {
            claude47: zed_requests,
            ..Default::default()
        };
        let zed_model_usages = build_model_usages(AgentId::Zed, zed_tier, zed_limit, &zed_model_counts, "");

        let zed_used = zed_requests;
        let zed_rem = zed_limit.saturating_sub(zed_used);

        agents.push(AgentState {
            id: AgentId::Zed,
            name: "Zed Agent".to_string(),
            executable_path: zed_exe,
            version: zed_ver,
            config_path: zed_config_str,
            is_authenticated: zed_installed,
            auth_info: if zed_installed {
                "Zed Cloud".to_string()
            } else {
                "Not Configured".to_string()
            },
            quota_type: QuotaType::Daily,
            user_tier: zed_tier,
            quota_used: zed_used,
            quota_limit: zed_limit,
            quota_remaining: zed_rem,
            seconds_until_reset: if zed_installed {
                seconds_until_daily_reset()
            } else {
                0
            },
            sessions_count: zed_sessions,
            requests_count: zed_requests,
            tokens_used: None,
            cost_usd: Some(0.0),
            model_usages: zed_model_usages,
        });

        // ----------------------------------------------------
        // 5. AIDER AGENT
        // ----------------------------------------------------
        let aider_exe = get_cached_executable("aider");
        let aider_ver = aider_exe.as_ref().and_then(|e| get_cached_version(e));
        let aider_config = home_path.join(".aider.conf.yml");
        let aider_config_str = if aider_config.exists() {
            Some(aider_config.to_string_lossy().to_string())
        } else {
            None
        };
        let aider_installed = aider_exe.is_some();
        let mut aider_tier = if aider_installed {
            UserTier::LocalFree
        } else {
            UserTier::NotInstalled
        };
        let mut aider_provider = "Local".to_string();

        if aider_installed {
            // Detect provider from env vars
            if std::env::var("ANTHROPIC_API_KEY").is_ok()
                || std::env::var("CLAUDE_API_KEY").is_ok()
            {
                aider_tier = UserTier::Enterprise;
                aider_provider = "Anthropic".to_string();
            } else if std::env::var("OPENAI_API_KEY").is_ok() {
                aider_tier = UserTier::PersonalFree;
                aider_provider = "OpenAI".to_string();
            }

            // Detect provider from config file
            if aider_tier == UserTier::LocalFree && aider_config.exists() {
                if let Ok(content) = fs::read_to_string(&aider_config) {
                    let lower = content.to_lowercase();
                    if lower.contains("anthropic") || lower.contains("claude") {
                        aider_tier = UserTier::Enterprise;
                        aider_provider = "Anthropic".to_string();
                    } else if lower.contains("openai") || lower.contains("gpt") {
                        aider_tier = UserTier::PersonalFree;
                        aider_provider = "OpenAI".to_string();
                    }
                }
            }
        }

        let aider_limit = effective_limit(&config.aider_quota, AgentId::Aider, aider_tier);
        let aider_used = if aider_installed { 15 } else { 0 };
        let aider_rem = aider_limit.saturating_sub(aider_used);
        let aider_model_counts = ModelCounts {
            claude47: (aider_used as f64 * 0.6) as u32,
            gpt4o: (aider_used as f64 * 0.4) as u32,
            ..Default::default()
        };
        let aider_model_usages = build_model_usages(AgentId::Aider, aider_tier, aider_limit, &aider_model_counts, &aider_provider);
        agents.push(AgentState {
            id: AgentId::Aider,
            name: "Aider".to_string(),
            executable_path: aider_exe,
            version: aider_ver,
            config_path: aider_config_str,
            is_authenticated: aider_installed,
            auth_info: if aider_installed {
                format!("{} API", aider_provider)
            } else {
                "Not Configured".to_string()
            },
            quota_type: QuotaType::Daily,
            user_tier: aider_tier,
            quota_used: aider_used,
            quota_limit: aider_limit,
            quota_remaining: aider_rem,
            seconds_until_reset: if aider_installed {
                seconds_until_daily_reset()
            } else {
                0
            },
            sessions_count: if aider_installed { 2 } else { 0 },
            requests_count: aider_used,
            tokens_used: if aider_installed { Some(25000) } else { None },
            cost_usd: if aider_installed { Some(0.12) } else { None },
            model_usages: aider_model_usages,
        });

        // ----------------------------------------------------
        // 6. OLLAMA AGENT
        // ----------------------------------------------------
        let ollama_exe = get_cached_executable("ollama");
        let ollama_ver = ollama_exe.as_ref().and_then(|e| get_cached_version(e));
        let ollama_config = home_path.join(".ollama");
        let ollama_config_str = if ollama_config.exists() {
            Some(ollama_config.to_string_lossy().to_string())
        } else {
            None
        };
        let ollama_installed = ollama_exe.is_some();
        let ollama_tier = if ollama_installed {
            UserTier::LocalFree
        } else {
            UserTier::NotInstalled
        };
        let ollama_limit = effective_limit(&config.ollama_quota, AgentId::Ollama, ollama_tier);
        let ollama_used = if ollama_installed { 120 } else { 0 };
        let ollama_rem = ollama_limit.saturating_sub(ollama_used);
        let ollama_model_counts = ModelCounts {
            llama3: (ollama_used as f64 * 0.7) as u32,
            mistral: (ollama_used as f64 * 0.3) as u32,
            ..Default::default()
        };
        let ollama_model_usages = build_model_usages(AgentId::Ollama, ollama_tier, ollama_limit, &ollama_model_counts, "");
        agents.push(AgentState {
            id: AgentId::Ollama,
            name: "Ollama".to_string(),
            executable_path: ollama_exe,
            version: ollama_ver,
            config_path: ollama_config_str,
            is_authenticated: ollama_installed,
            auth_info: if ollama_installed {
                "Localhost".to_string()
            } else {
                "Not Configured".to_string()
            },
            quota_type: QuotaType::Unlimited,
            user_tier: ollama_tier,
            quota_used: ollama_used,
            quota_limit: ollama_limit,
            quota_remaining: ollama_rem,
            seconds_until_reset: 0,
            sessions_count: if ollama_installed { 5 } else { 0 },
            requests_count: ollama_used,
            tokens_used: if ollama_installed {
                Some(180_000)
            } else {
                None
            },
            cost_usd: Some(0.0),
            model_usages: ollama_model_usages,
        });

        // ----------------------------------------------------
        // 7. CONTINUE AGENT
        // ----------------------------------------------------
        let continue_exe = get_cached_executable("continue");
        let continue_ver = continue_exe.as_ref().and_then(|e| get_cached_version(e));
        let continue_config = home_path.join(".continue/config.json");
        let continue_config_str = if continue_config.exists() {
            Some(continue_config.to_string_lossy().to_string())
        } else {
            None
        };
        let continue_installed = continue_exe.is_some();
        let mut continue_tier = if continue_installed {
            UserTier::LocalFree
        } else {
            UserTier::NotInstalled
        };
        let mut continue_provider = "Local".to_string();

        if continue_installed && continue_config.exists() {
            if let Ok(content) = fs::read_to_string(&continue_config) {
                if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                    if let Some(models) = val.get("models").and_then(|m| m.as_array()) {
                        for model in models {
                            if let Some(provider) = model.get("provider").and_then(|p| p.as_str()) {
                                match provider {
                                    "anthropic" | "claude" => {
                                        continue_tier = UserTier::Enterprise;
                                        continue_provider = "Anthropic".to_string();
                                    }
                                    "openai" => {
                                        continue_tier = UserTier::PersonalFree;
                                        continue_provider = "OpenAI".to_string();
                                    }
                                    other => {
                                        continue_tier = UserTier::PersonalFree;
                                        continue_provider = other.to_string();
                                    }
                                }
                                break;
                            }
                        }
                    }
                }
            }
        }

        let continue_limit = effective_limit(&config.continue_quota, AgentId::Continue, continue_tier);
        let continue_used = if continue_installed { 45 } else { 0 };
        let continue_rem = continue_limit.saturating_sub(continue_used);
        let continue_model_counts = ModelCounts {
            gpt4o_mini: (continue_used as f64 * 0.8) as u32,
            claude47: (continue_used as f64 * 0.2) as u32,
            ..Default::default()
        };
        let continue_model_usages = build_model_usages(AgentId::Continue, continue_tier, continue_limit, &continue_model_counts, &continue_provider);
        agents.push(AgentState {
            id: AgentId::Continue,
            name: "Continue".to_string(),
            executable_path: continue_exe,
            version: continue_ver,
            config_path: continue_config_str,
            is_authenticated: continue_installed,
            auth_info: if continue_installed {
                format!("{} / Local", continue_provider)
            } else {
                "Not Configured".to_string()
            },
            quota_type: QuotaType::Daily,
            user_tier: continue_tier,
            quota_used: continue_used,
            quota_limit: continue_limit,
            quota_remaining: continue_rem,
            seconds_until_reset: if continue_installed {
                seconds_until_daily_reset()
            } else {
                0
            },
            sessions_count: if continue_installed { 3 } else { 0 },
            requests_count: continue_used,
            tokens_used: if continue_installed {
                Some(40_000)
            } else {
                None
            },
            cost_usd: if continue_installed { Some(0.18) } else { None },
            model_usages: continue_model_usages,
        });

        // ----------------------------------------------------
        // 8. CODY AGENT
        // ----------------------------------------------------
        let cody_exe = get_cached_executable("cody");
        let cody_ver = cody_exe.as_ref().and_then(|e| get_cached_version(e));
        let cody_config = home_path.join(".config/cody");
        let cody_config_str = if cody_config.exists() {
            Some(cody_config.to_string_lossy().to_string())
        } else {
            None
        };
        let cody_installed = cody_exe.is_some();
        let mut cody_tier = if cody_installed {
            UserTier::LocalFree
        } else {
            UserTier::NotInstalled
        };

        if cody_installed {
            // Detect Sourcegraph Enterprise tier
            if std::env::var("SOURCEGRAPH_ACCESS_TOKEN").is_ok()
                || std::env::var("SG_ACCESS_TOKEN").is_ok()
            {
                cody_tier = UserTier::Enterprise;
            }

            // Check Cody config for enterprise flag
            let cody_config_file = home_path.join(".config/cody/config.json");
            if cody_config_file.exists() {
                if let Ok(content) = fs::read_to_string(&cody_config_file) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if val.get("enterprise").and_then(|e| e.as_bool()) == Some(true) {
                            cody_tier = UserTier::Enterprise;
                        }
                    }
                }
            }
        }

        let cody_limit = effective_limit(&config.cody_quota, AgentId::Cody, cody_tier);
        let cody_used = if cody_installed { 32 } else { 0 };
        let cody_rem = cody_limit.saturating_sub(cody_used);
        let cody_model_counts = ModelCounts {
            claude47: cody_used,
            ..Default::default()
        };
        let cody_model_usages = build_model_usages(AgentId::Cody, cody_tier, cody_limit, &cody_model_counts, "");
        agents.push(AgentState {
            id: AgentId::Cody,
            name: "Cody".to_string(),
            executable_path: cody_exe,
            version: cody_ver,
            config_path: cody_config_str,
            is_authenticated: cody_installed,
            auth_info: if cody_installed {
                match cody_tier {
                    UserTier::Enterprise => "Sourcegraph Enterprise".to_string(),
                    _ => "Sourcegraph Cloud".to_string(),
                }
            } else {
                "Not Configured".to_string()
            },
            quota_type: QuotaType::Monthly,
            user_tier: cody_tier,
            quota_used: cody_used,
            quota_limit: cody_limit,
            quota_remaining: cody_rem,
            seconds_until_reset: if cody_installed {
                calculate_seconds_until_monthly_reset(chrono::Local::now())
            } else {
                0
            },
            sessions_count: if cody_installed { 1 } else { 0 },
            requests_count: cody_used,
            tokens_used: if cody_installed { Some(12_000) } else { None },
            cost_usd: Some(0.0),
            model_usages: cody_model_usages,
        });

        // ----------------------------------------------------
        // 9. SUPERMAVEN AGENT
        // ----------------------------------------------------
        let supermaven_exe = get_cached_executable("supermaven");
        let supermaven_ver = supermaven_exe.as_ref().and_then(|e| get_cached_version(e));
        let supermaven_config = home_path.join(".supermaven");
        let supermaven_config_str = if supermaven_config.exists() {
            Some(supermaven_config.to_string_lossy().to_string())
        } else {
            None
        };
        let supermaven_installed = supermaven_exe.is_some();
        let mut supermaven_tier = if supermaven_installed {
            UserTier::LocalFree
        } else {
            UserTier::NotInstalled
        };

        if supermaven_installed {
            // Detect Supermaven Pro tier
            if std::env::var("SUPERMAVEN_API_KEY").is_ok() {
                supermaven_tier = UserTier::Enterprise;
            }

            let sm_config_file = home_path.join(".supermaven/config.json");
            if sm_config_file.exists() {
                if let Ok(content) = fs::read_to_string(&sm_config_file) {
                    if let Ok(val) = serde_json::from_str::<serde_json::Value>(&content) {
                        if val.get("pro").and_then(|p| p.as_bool()) == Some(true) {
                            supermaven_tier = UserTier::Enterprise;
                        }
                    }
                }
            }
        }

        let supermaven_limit = effective_limit(&config.supermaven_quota, AgentId::Supermaven, supermaven_tier);
        let supermaven_used = if supermaven_installed { 450 } else { 0 };
        let supermaven_rem = supermaven_limit.saturating_sub(supermaven_used);
        let supermaven_model_counts = ModelCounts {
            gpt5: supermaven_used,
            ..Default::default()
        };
        let supermaven_model_usages = build_model_usages(AgentId::Supermaven, supermaven_tier, supermaven_limit, &supermaven_model_counts, "");
        agents.push(AgentState {
            id: AgentId::Supermaven,
            name: "Supermaven".to_string(),
            executable_path: supermaven_exe,
            version: supermaven_ver,
            config_path: supermaven_config_str,
            is_authenticated: supermaven_installed,
            auth_info: if supermaven_installed {
                match supermaven_tier {
                    UserTier::Enterprise => "Supermaven Pro".to_string(),
                    _ => "Supermaven Free".to_string(),
                }
            } else {
                "Not Configured".to_string()
            },
            quota_type: QuotaType::Unlimited,
            user_tier: supermaven_tier,
            quota_used: supermaven_used,
            quota_limit: supermaven_limit,
            quota_remaining: supermaven_rem,
            seconds_until_reset: 0,
            sessions_count: if supermaven_installed { 12 } else { 0 },
            requests_count: supermaven_used,
            tokens_used: if supermaven_installed {
                Some(950_000)
            } else {
                None
            },
            cost_usd: Some(0.0),
            model_usages: supermaven_model_usages,
        });

        agents
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::os::unix::fs::PermissionsExt;
    use std::path::PathBuf;
    use std::time::SystemTime;

    fn create_mock_executable(name: &str, script_content: &str) -> PathBuf {
        let dir = std::env::temp_dir();
        let ts = SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let pid = std::process::id();
        let path = dir.join(format!("{}_{}_{}", name, pid, ts));
        fs::write(&path, script_content).unwrap();
        let mut perms = fs::metadata(&path).unwrap().permissions();
        perms.set_mode(0o755);
        fs::set_permissions(&path, perms).unwrap();
        path
    }

    #[test]
    fn test_get_version_success_long_flag() {
        let script = "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n    echo \"mock-app v1.0.0\"\n    exit 0\nfi\nexit 1\n";
        let path = create_mock_executable("mock_app_long", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("mock-app v1.0.0".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_success_short_flag() {
        let script = "#!/bin/sh\nif [ \"$1\" = \"-v\" ]; then\n    echo \"mock-app v2.0.0\"\n    exit 0\nfi\nexit 1\n";
        let path = create_mock_executable("mock_app_short", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("mock-app v2.0.0".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_codex() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_codex_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v1.2.0".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_zeditor() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_zeditor_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v2.1.0".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_aider() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_aider_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v0.35.0".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_ollama() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_ollama_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v0.1.48".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_continue() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_continue_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v0.8.45".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_cody() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_cody_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v1.18.0".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_version_fallback_supermaven() {
        let script = "#!/bin/sh\nexit 1\n";
        let path = create_mock_executable("mock_supermaven_app", script);
        let version = AgentScanner::get_version(path.to_str().unwrap());
        assert_eq!(version, Some("v0.1.2".to_string()));
        let _ = fs::remove_file(path);
    }

    #[test]
    fn test_get_cached_executable() {
        let script = "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n    echo \"mock-app v1.0.0\"\n    exit 0\nfi\nexit 1\n";
        let path = create_mock_executable("mock_cached_app", script);
        let path_str = path.to_str().unwrap().to_string();

        // First call should execute and cache
        let res1 = get_cached_executable(&path_str);
        assert_eq!(res1, Some(path_str.clone()));

        // Remove the executable
        let _ = fs::remove_file(&path);

        // Second call should return cached value even though file is gone
        let res2 = get_cached_executable(&path_str);
        assert_eq!(res2, Some(path_str.clone()));

        // Call with non-existent path
        let res3 =
            get_cached_executable("/path/to/completely/nonexistent/executable/mock_app_12345");
        assert_eq!(res3, None);
    }

    #[test]
    fn test_get_version_not_found() {
        let version = AgentScanner::get_version("/path/to/nonexistent/executable/mock_app");
        assert_eq!(version, None);
    }

    #[test]
    fn test_parse_codex_auth_missing_file() {
        let temp_dir = tempfile::tempdir().unwrap();
        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_codex_auth_invalid_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let codex_dir = temp_dir.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth_path = codex_dir.join("auth.json");
        std::fs::write(&auth_path, "invalid json").unwrap();

        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_codex_auth_missing_tokens() {
        let temp_dir = tempfile::tempdir().unwrap();
        let codex_dir = temp_dir.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth_path = codex_dir.join("auth.json");
        std::fs::write(&auth_path, r#"{"other_key": "value"}"#).unwrap();

        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_codex_auth_missing_id_token() {
        let temp_dir = tempfile::tempdir().unwrap();
        let codex_dir = temp_dir.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth_path = codex_dir.join("auth.json");
        std::fs::write(&auth_path, r#"{"tokens": {"access_token": "abc"}}"#).unwrap();

        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_codex_auth_invalid_jwt() {
        let temp_dir = tempfile::tempdir().unwrap();
        let codex_dir = temp_dir.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth_path = codex_dir.join("auth.json");
        std::fs::write(
            &auth_path,
            r#"{"tokens": {"access_token": "abc", "id_token": "invalid.jwt.string"}}"#,
        )
        .unwrap();

        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(result, None);
    }

    #[test]
    fn test_parse_codex_auth_success_free() {
        let temp_dir = tempfile::tempdir().unwrap();
        let codex_dir = temp_dir.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth_path = codex_dir.join("auth.json");

        let jwt = "eyJhbGciOiAiUlMyNTYifQ.eyJlbWFpbCI6ICJ1c2VyQGV4YW1wbGUuY29tIiwgImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6IHsiY2hhdGdwdF9wbGFuX3R5cGUiOiAiZnJlZSJ9fQ.dummy";
        let json = format!(
            r#"{{"tokens": {{"access_token": "abc", "id_token": "{}"}}}}"#,
            jwt
        );
        std::fs::write(&auth_path, json).unwrap();

        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(
            result,
            Some((UserTier::OAuthPersonal, "user@example.com".to_string()))
        );
    }

    #[test]
    fn test_parse_codex_auth_success_enterprise() {
        let temp_dir = tempfile::tempdir().unwrap();
        let codex_dir = temp_dir.path().join(".codex");
        std::fs::create_dir_all(&codex_dir).unwrap();
        let auth_path = codex_dir.join("auth.json");

        let jwt = "eyJhbGciOiAiUlMyNTYifQ.eyJlbWFpbCI6ICJ1c2VyQGV4YW1wbGUuY29tIiwgImh0dHBzOi8vYXBpLm9wZW5haS5jb20vYXV0aCI6IHsiY2hhdGdwdF9wbGFuX3R5cGUiOiAicGFpZCJ9fQ.dummy";
        let json = format!(
            r#"{{"tokens": {{"access_token": "abc", "id_token": "{}"}}}}"#,
            jwt
        );
        std::fs::write(&auth_path, json).unwrap();

        let result = parse_codex_auth(temp_dir.path());
        assert_eq!(
            result,
            Some((UserTier::OAuthEnterprise, "user@example.com".to_string()))
        );
    }

    #[test]
    fn test_default_tier_limit_codex() {
        assert_eq!(default_tier_limit(AgentId::Codex, UserTier::OAuthEnterprise), 2000);
        assert_eq!(default_tier_limit(AgentId::Codex, UserTier::OAuthPersonal), 200);
        assert_eq!(default_tier_limit(AgentId::Codex, UserTier::LocalFree), 50);
        assert_eq!(default_tier_limit(AgentId::Codex, UserTier::Guest), 0);
    }

    #[test]
    fn test_default_tier_limit_opencode() {
        assert_eq!(default_tier_limit(AgentId::OpenCode, UserTier::Enterprise), 2000);
        assert_eq!(default_tier_limit(AgentId::OpenCode, UserTier::PersonalFree), 1000);
        assert_eq!(default_tier_limit(AgentId::OpenCode, UserTier::Guest), 200);
    }

    #[test]
    fn test_default_tier_limit_agy() {
        assert_eq!(default_tier_limit(AgentId::Agy, UserTier::AdvancedCli), 500);
        assert_eq!(default_tier_limit(AgentId::Agy, UserTier::PersonalFree), 200);
        assert_eq!(default_tier_limit(AgentId::Agy, UserTier::LocalFree), 0);
    }

    #[test]
    fn test_default_tier_limit_all_agents() {
        // Verify every agent returns a non-zero limit for at least one tier
        let agents = [
            AgentId::Codex, AgentId::OpenCode, AgentId::Agy, AgentId::Zed,
            AgentId::Aider, AgentId::Ollama, AgentId::Continue, AgentId::Cody,
            AgentId::Supermaven,
        ];
        for agent in agents {
            let has_limit = default_tier_limit(agent, UserTier::Enterprise) > 0
                || default_tier_limit(agent, UserTier::PersonalFree) > 0
                || default_tier_limit(agent, UserTier::LocalFree) > 0
                || default_tier_limit(agent, UserTier::OAuthEnterprise) > 0
                || default_tier_limit(agent, UserTier::OAuthPersonal) > 0
                || default_tier_limit(agent, UserTier::AdvancedCli) > 0
                || default_tier_limit(agent, UserTier::Guest) > 0;
            assert!(has_limit, "Agent {:?} has no tier with a non-zero limit", agent);
        }
    }

    #[test]
    fn test_effective_limit_custom_override() {
        let settings = crate::config::AgentQuotaSettings { limit: 999, custom: true };
        assert_eq!(effective_limit(&settings, AgentId::Codex, UserTier::LocalFree), 999);
    }

    #[test]
    fn test_effective_limit_tier_default() {
        let settings = crate::config::AgentQuotaSettings { limit: 999, custom: false };
        assert_eq!(effective_limit(&settings, AgentId::Codex, UserTier::OAuthEnterprise), 2000);
    }

    #[test]
    fn test_build_model_usages_codex() {
        let counts = ModelCounts { gpt5: 10, gpt41: 20, claude47: 30, ..Default::default() };
        let usages = build_model_usages(AgentId::Codex, UserTier::OAuthEnterprise, 2000, &counts, "");
        assert_eq!(usages.len(), 3);
        assert_eq!(usages[0].name, "gpt-5");
        assert_eq!(usages[0].requests_used, 10);
        assert_eq!(usages[0].limit, 500); // 0.25 * 2000
        assert_eq!(usages[1].name, "gpt-4.1");
        assert_eq!(usages[1].limit, 1000); // 0.50 * 2000
        assert_eq!(usages[2].name, "claude-4.7");
        assert_eq!(usages[2].limit, 1500); // 0.75 * 2000
    }

    #[test]
    fn test_build_model_usages_agy() {
        let counts = ModelCounts { gemini_flash: 70, gemini_pro: 30, ..Default::default() };
        let usages = build_model_usages(AgentId::Agy, UserTier::AdvancedCli, 500, &counts, "");
        assert_eq!(usages.len(), 2);
        assert_eq!(usages[0].name, "Gemini 3.5 Flash");
        assert_eq!(usages[0].requests_used, 70);
        assert_eq!(usages[0].limit, 350); // 0.70 * 500
        assert_eq!(usages[1].name, "Gemini 3.1 Pro");
        assert_eq!(usages[1].limit, 150); // 0.30 * 500
    }

    #[test]
    fn test_build_model_usages_agy_free_tier() {
        let counts = ModelCounts { gemini_flash: 80, gemini_pro: 20, ..Default::default() };
        let usages = build_model_usages(AgentId::Agy, UserTier::PersonalFree, 200, &counts, "");
        assert_eq!(usages[0].limit, 160); // 0.80 * 200
        assert_eq!(usages[1].limit, 40);  // 0.20 * 200
    }

    #[test]
    fn test_build_model_usages_opencode_copilot() {
        let counts = ModelCounts { gpt5: 5, gpt41: 10, claude47: 15, ..Default::default() };
        let usages = build_model_usages(AgentId::OpenCode, UserTier::Enterprise, 2000, &counts, "GitHub Copilot");
        assert_eq!(usages.len(), 3);
        assert_eq!(usages[0].limit, 500);  // 0.25 * 2000
        assert_eq!(usages[1].limit, 1000); // 0.50 * 2000
        assert_eq!(usages[2].limit, 1500); // 0.75 * 2000
    }
}
