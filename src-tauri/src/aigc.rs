use reqwest;
use serde::{Deserialize, Serialize};
use tokio;
use crate::storage::Event;
use std::env;
use log::{error, info};

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Serialize, Deserialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct Choice {
    message: Message,
    index: u32,
    // Adding finish_reason which is often part of these APIs
    #[serde(default)]
    finish_reason: String,
}

#[derive(Deserialize)]
#[allow(dead_code)]
struct ChatResponse {
    id: String,
    choices: Vec<Choice>,
}

// For debugging response structure
#[derive(Deserialize)]
struct RawResponse {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

#[tokio::main]
async fn gen_tag() -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.siliconflow.cn/v1/chat/completions";
    let client = reqwest::Client::new();
    
    // Create a prompt for the tag generation based on event
    let prompt = "一加一等于几？".to_string();

    let request_body = ChatRequest {
        model: "deepseek-ai/DeepSeek-V3".to_string(),
        messages: vec![Message {
            role: "user".to_string(),
            content: prompt,
        }],
        stream: false,
        temperature: 0.7,
        max_tokens: 2048,
    };
        
    // Get API key from environment or config
    let api_key = match env::var("DEEPSEEK_TOKEN") {
        Ok(key) if !key.is_empty() => key,
        _ => {
            error!("No DEEPSEEK_TOKEN environment variable set or it's empty");
            return Err("API key is required but not provided".into());
        }
    };
    
    // Send request
    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_key))
        .json(&request_body)
        .send()
        .await?;
    
    let status = response.status();
    info!("AI service response status: {}", status);
    
    if status.is_success() {
        // Try to parse as our expected type
        match response.json::<ChatResponse>().await {
            Ok(chat_response) => {
                if let Some(choice) = chat_response.choices.first() {
                    let generated_tag = choice.message.content.trim().to_string();
                    println!("Response: {}", generated_tag);
                    Ok(())
                } else {
                    info!("No tag choices returned, using default");
                    println!("Response: general");
                    Ok(())
                }
            },
            Err(e) => {
                error!("Failed to parse AI response: {}", e);
                // Try to get raw response to see structure
                let raw_text = e.to_string();
                error!("Raw error: {}", raw_text);
                println!("Response: general");
                Ok(())
            }
        }
    } else {
        let error_text = response.text().await?;
        error!("AI service request failed: {}", error_text);
        Err(format!("API request failed with status {}: {}", status, error_text).into())
    }
}
