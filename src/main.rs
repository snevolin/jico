use std::env;

use anyhow::{Context, Result, anyhow};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use clap::{Parser, Subcommand};
use reqwest::header;
use serde_json::{Value, json};
use dotenvy::dotenv;

#[derive(Parser, Debug)]
#[command(name = "jico", version, about = "CLI helper for Jira Cloud")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create a new issue
    Create {
        /// Summary/title of the issue
        summary: String,
        /// Optional description (plain text)
        #[arg(long)]
        description: Option<String>,
        /// Project key; falls back to config
        #[arg(long)]
        project: Option<String>,
        /// Issue type name; default: Task
        #[arg(long, default_value = "Task")]
        issue_type: String,
    },
    /// List issues via JQL
    List {
        /// Optional JQL override
        #[arg(long)]
        jql: Option<String>,
        /// Limit the number of results
        #[arg(long, default_value_t = 20)]
        limit: u32,
        /// Project key to build default JQL
        #[arg(long)]
        project: Option<String>,
    },
    /// Show a single issue
    View {
        /// Issue key, e.g., PROJ-123
        key: String,
    },
    /// Transition an issue to a new status/transition
    Transition {
        /// Issue key, e.g., PROJ-123
        key: String,
        /// Target status/transition name
        #[arg(long)]
        to: String,
    },
}

#[derive(Debug, Clone)]
struct Settings {
    base_url: String,
    email: String,
    api_token: String,
    project_key: Option<String>,
    default_jql: Option<String>,
}

impl Settings {
    fn load() -> Result<Self> {
        dotenv().ok(); // load from .env in current working dir; won't override real env vars

        let base_url = required_env("JIRA_BASE_URL")?
            .trim_end_matches('/')
            .to_string();
        let email = required_env("JIRA_EMAIL")?;
        let api_token = required_env("JIRA_API_TOKEN")?;
        let project_key = env::var("JIRA_PROJECT_KEY").ok();
        let default_jql = env::var("JIRA_DEFAULT_JQL").ok();

        Ok(Self {
            base_url,
            email,
            api_token,
            project_key,
            default_jql,
        })
    }
}

fn required_env(key: &str) -> Result<String> {
    env::var(key).with_context(|| format!("Missing {key} (set in environment or .env)"))
}

struct JiraClient {
    base_url: String,
    http: reqwest::Client,
}

impl JiraClient {
    fn new(settings: &Settings) -> Result<Self> {
        let mut headers = header::HeaderMap::new();
        headers.insert(
            header::ACCEPT,
            header::HeaderValue::from_static("application/json"),
        );
        let auth = header::HeaderValue::from_str(&format!(
            "Basic {}",
            STANDARD.encode(format!("{}:{}", settings.email, settings.api_token))
        ))
        .context("Failed to encode auth header")?;
        headers.insert(header::AUTHORIZATION, auth);

        let http = reqwest::Client::builder()
            .default_headers(headers)
            .build()
            .context("Failed to build HTTP client")?;

        Ok(Self {
            base_url: settings.base_url.clone(),
            http,
        })
    }

    async fn create_issue(
        &self,
        project_key: &str,
        summary: &str,
        description: Option<String>,
        issue_type: &str,
    ) -> Result<Value> {
        let url = format!("{}/rest/api/3/issue", self.base_url);
        let description_adf = description.map(|text| {
            json!({
                "type": "doc",
                "version": 1,
                "content": [{
                    "type": "paragraph",
                    "content": [{
                        "type": "text",
                        "text": text
                    }]
                }]
            })
        });
        let body = json!({
            "fields": {
                "project": { "key": project_key },
                "summary": summary,
                "issuetype": { "name": issue_type },
                "description": description_adf.unwrap_or_else(|| json!(null)),
            }
        });

        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .context("Failed to send create issue request")?;
        let status = resp.status();
        let value: Value = resp
            .json()
            .await
            .context("Failed to parse create issue response")?;
        if !status.is_success() {
            return Err(anyhow!("Jira returned error status {}: {}", status, value));
        }
        Ok(value)
    }

    async fn list_issues(&self, jql: &str, limit: u32) -> Result<Value> {
        // Atlassian migrated search to /search/jql; body still uses "jql".
        let url = format!("{}/rest/api/3/search/jql", self.base_url);
        let body = json!({
            "jql": jql,
            "maxResults": limit,
        });
        let resp = self
            .http
            .post(url)
            .json(&body)
            .send()
            .await
            .context("Failed to send search request")?;
        let status = resp.status();
        let value: Value = resp
            .json()
            .await
            .context("Failed to parse search response")?;
        if !status.is_success() {
            return Err(anyhow!("Jira returned error status {}: {}", status, value));
        }
        Ok(value)
    }

    async fn get_issue(&self, key: &str) -> Result<Value> {
        let url = format!("{}/rest/api/3/issue/{}", self.base_url, key);
        let resp = self
            .http
            .get(url)
            .send()
            .await
            .context("Failed to send get issue request")?;
        let status = resp.status();
        let value: Value = resp
            .json()
            .await
            .context("Failed to parse get issue response")?;
        if !status.is_success() {
            return Err(anyhow!("Jira returned error status {}: {}", status, value));
        }
        Ok(value)
    }

    async fn transition_issue(&self, key: &str, target: &str) -> Result<Value> {
        let transitions_url = format!("{}/rest/api/3/issue/{}/transitions", self.base_url, key);
        let resp = self
            .http
            .get(&transitions_url)
            .send()
            .await
            .context("Failed to fetch transitions")?;
        let status = resp.status();
        let payload: Value = resp
            .json()
            .await
            .context("Failed to parse transitions response")?;
        if !status.is_success() {
            return Err(anyhow!(
                "Jira returned error status {} when fetching transitions: {}",
                status,
                payload
            ));
        }
        let transitions = payload
            .get("transitions")
            .and_then(|t| t.as_array())
            .ok_or_else(|| anyhow!("No transitions found in response"))?;
        let target_transition = transitions.iter().find(|t| {
            t.get("name")
                .and_then(|n| n.as_str())
                .map(|n| n.eq_ignore_ascii_case(target))
                .unwrap_or(false)
        });
        let transition_id = target_transition
            .and_then(|t| t.get("id"))
            .and_then(|id| id.as_str())
            .ok_or_else(|| anyhow!("Transition '{}' not available for {}", target, key))?;

        let resp = self
            .http
            .post(&transitions_url)
            .json(&json!({"transition": { "id": transition_id }}))
            .send()
            .await
            .context("Failed to send transition request")?;
        let status = resp.status();
        let value: Value = resp
            .json()
            .await
            .context("Failed to parse transition response")?;
        if !status.is_success() {
            return Err(anyhow!("Jira returned error status {}: {}", status, value));
        }
        Ok(value)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let settings = Settings::load()?;
    let client = JiraClient::new(&settings)?;

    match cli.command {
        Commands::Create {
            summary,
            description,
            project,
            issue_type,
        } => {
            let project_key = resolve_project(&settings, project)?;
            let created = client
                .create_issue(&project_key, &summary, description, &issue_type)
                .await?;
            print_json(&created);
        }
        Commands::List {
            jql,
            limit,
            project,
        } => {
            let jql = jql
                .or_else(|| settings.default_jql.clone())
                .or_else(|| {
                    resolve_project(&settings, project)
                        .ok()
                        .map(|key| format!("project = {} ORDER BY created DESC", key))
                })
                .ok_or_else(|| anyhow!("Provide --jql or configure a project key"))?;
            let results = client.list_issues(&jql, limit).await?;
            print_json(&results);
        }
        Commands::View { key } => {
            let issue = client.get_issue(&key).await?;
            print_json(&issue);
        }
        Commands::Transition { key, to } => {
            let result = client.transition_issue(&key, &to).await?;
            print_json(&result);
        }
    }

    Ok(())
}

fn resolve_project(settings: &Settings, override_key: Option<String>) -> Result<String> {
    override_key
        .or_else(|| settings.project_key.clone())
        .ok_or_else(|| anyhow!("Project key is required (pass --project or set JIRA_PROJECT_KEY)"))
}

fn print_json(value: &Value) {
    match serde_json::to_string_pretty(value) {
        Ok(s) => println!("{s}"),
        Err(_) => println!("{}", value),
    }
}
