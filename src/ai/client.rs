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

pub async fn ask(question: &str) -> Result<()> {
    println!("{}", "🤖 TerminalFlow AI".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    let api_key = std::env::var("OPENAI_API_KEY")
        .context("Please set OPENAI_API_KEY environment variable")?;

    let client = Client::new();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are TerminalFlow AI, a helpful coding assistant. Be concise and provide code examples when appropriate. Format responses nicely for terminal display.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: question.to_string(),
            },
        ],
        temperature: 0.7,
        max_tokens: 1000,
    };

    println!("{}", "Thinking...".yellow());

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .context("Failed to send request to OpenAI")?;

    let chat_response: ChatResponse = response.json().await.context("Failed to parse response")?;

    println!();
    println!("{}", "📝 Response:".green().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    for line in chat_response.choices[0].message.content.lines() {
        println!("  {}", line);
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}

pub async fn review_code(code: &str) -> Result<()> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("Please set OPENAI_API_KEY environment variable")?;

    let client = Client::new();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a senior code reviewer. Review the provided code for bugs, improvements, and best practices. Be concise but thorough. Format your response with clear sections.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Please review this code:\n\n{}", code),
            },
        ],
        temperature: 0.5,
        max_tokens: 1500,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .context("Failed to send request")?;

    let chat_response: ChatResponse = response.json().await.context("Failed to parse response")?;

    println!("{}", "🔍 Code Review".cyan().bold());
    println!("{}", "─".repeat(50).dimmed());
    println!();

    for line in chat_response.choices[0].message.content.lines() {
        println!("  {}", line);
    }

    println!();
    println!("{}", "─".repeat(50).dimmed());

    Ok(())
}

pub async fn generate_commit_message(diff: &str) -> Result<String> {
    let api_key = std::env::var("OPENAI_API_KEY")
        .context("Please set OPENAI_API_KEY environment variable")?;

    let client = Client::new();

    let request = ChatRequest {
        model: "gpt-4".to_string(),
        messages: vec![
            ChatMessage {
                role: "system".to_string(),
                content: "You are a commit message generator. Generate a concise, conventional commit message based on the provided git diff. Use the format: type(scope): description. Keep it under 72 characters. Do not include any explanation, just the commit message.".to_string(),
            },
            ChatMessage {
                role: "user".to_string(),
                content: format!("Generate a commit message for this diff:\n\n{}", diff),
            },
        ],
        temperature: 0.5,
        max_tokens: 100,
    };

    let response = client
        .post("https://api.openai.com/v1/chat/completions")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request)
        .send()
        .await
        .context("Failed to send request")?;

    let chat_response: ChatResponse = response.json().await.context("Failed to parse response")?;

    Ok(chat_response.choices[0].message.content.trim().to_string())
}
