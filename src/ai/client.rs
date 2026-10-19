use anyhow::{Context, Result};
use colored::*;
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Debug, Deserialize)]
struct Choice {
    message: MessageResponse,
}

#[derive(Debug, Deserialize)]
struct MessageResponse {
    content: String,
}

const SYSTEM_ASSISTANT: &str = "You are TerminalFlow AI, a helpful coding assistant. Be concise and provide code examples when appropriate. Format responses nicely for terminal display.";
const SYSTEM_REVIEWER: &str = "You are a senior code reviewer. Review the provided code for bugs, improvements, and best practices. Be concise but thorough. Format your response with clear sections.";
const SYSTEM_COMMITTER: &str = "You are a commit message generator. Generate a concise, conventional commit message based on the provided git diff. Use the format: type(scope): description. Keep it under 72 characters. Do not include any explanation, just the commit message.";

/// True when an OpenAI-compatible API key is configured.
pub fn has_api_key() -> bool {
    api_key().is_some()
}

fn api_key() -> Option<String> {
    ["OPENAI_API_KEY", "TF_OPENAI_API_KEY"]
        .iter()
        .find_map(|name| std::env::var(name).ok())
        .map(|value| value.trim().to_string())
        .filter(|value| !value.is_empty())
}

fn model() -> String {
    std::env::var("OPENAI_MODEL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .unwrap_or_else(|| "gpt-4o-mini".to_string())
}

fn base_url() -> String {
    std::env::var("OPENAI_BASE_URL")
        .ok()
        .filter(|value| !value.trim().is_empty())
        .map(|value| value.trim_end_matches('/').to_string())
        .unwrap_or_else(|| "https://api.openai.com".to_string())
}

/// Instructions shown when no API key is configured.
pub fn print_setup_help() {
    println!(
        "  {} AI features need an API key (they are optional — everything else works offline)",
        "🔑".yellow()
    );
    println!();
    println!("  {} PowerShell", "Set it:".dimmed());
    println!("    {}", "$env:OPENAI_API_KEY = \"sk-...\"".cyan());
    println!("  {} bash / zsh", "Set it:".dimmed());
    println!("    {}", "export OPENAI_API_KEY=sk-...".cyan());
    println!();
    println!(
        "  {} optional: {} / {} / {}",
        "⚙️".dimmed(),
        "OPENAI_MODEL".cyan(),
        "OPENAI_BASE_URL".cyan(),
        "TF_OPENAI_API_KEY".cyan()
    );
}

async fn chat(system: &str, user: &str, temperature: f32, max_tokens: u32) -> Result<String> {
    let api_key = api_key().context("OPENAI_API_KEY is not set")?;
    let client = Client::new();

    let request = ChatRequest {
        model: model(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: system.to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: user.to_string(),
            },
        ],
        temperature,
        max_tokens,
    };

    let response = client
        .post(format!("{}/v1/chat/completions", base_url()))
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .context("Could not reach the AI API (network error)")?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("AI API returned {}: {}", status, truncate(&body, 200));
    }

    let chat_response: ChatResponse = response
        .json()
        .await
        .context("Failed to parse the AI response")?;

    let choice = chat_response
        .choices
        .first()
        .context("AI response contained no choices")?;

    Ok(choice.message.content.trim().to_string())
}

fn print_block(title: &str, body: &str) {
    println!();
    println!("{}", title.green().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();
    for line in body.lines() {
        println!("  {}", line);
    }
    println!();
    println!("{}", "─".repeat(60).dimmed());
}

pub async fn ask(question: &str) -> Result<()> {
    println!("{}", "🤖 TerminalFlow AI".cyan().bold());
    println!("{}", "─".repeat(60).dimmed());
    println!();

    if question.trim().is_empty() {
        println!("  {} Ask a question, e.g. tf ai \"how do I reverse a linked list?\"", "💡".yellow());
        return Ok(());
    }

    if !has_api_key() {
        println!("  {} {}", "❓".dimmed(), question);
        println!();
        print_setup_help();
        return Ok(());
    }

    println!("  {} {}", "❓".dimmed(), question);
    println!();
    println!("{}", "  ⏳ Thinking…".yellow());

    match chat(SYSTEM_ASSISTANT, question, 0.7, 1000).await {
        Ok(answer) => print_block("✨ Response:", &answer),
        Err(err) => {
            println!("  {} {}", "❌".red(), err.to_string().red());
            return Ok(());
        }
    }

    Ok(())
}

/// AI review of a diff. Falls back to `None` if no key is configured.
pub async fn review_code(code: &str) -> Result<String> {
    chat(
        SYSTEM_REVIEWER,
        &format!("Please review this code:\n\n{}", code),
        0.5,
        1500,
    )
    .await
}

/// AI commit message. Falls back to `None` if no key is configured.
pub async fn generate_commit_message(diff: &str) -> Result<String> {
    chat(
        SYSTEM_COMMITTER,
        &format!("Generate a commit message for this diff:\n\n{}", diff),
        0.5,
        100,
    )
    .await
}

fn truncate(value: &str, max: usize) -> String {
    if value.chars().count() <= max {
        value.to_string()
    } else {
        let mut out: String = value.chars().take(max.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
