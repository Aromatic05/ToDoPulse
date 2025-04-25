use log::{error, info};
use reqwest;
use serde::{Deserialize, Serialize};
use std::env;
use tauri::State;

use crate::data::Event;
use crate::ipc::get_tags;
use crate::storage::StorageState;

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
#[allow(dead_code)]
struct RawResponse {
    #[serde(flatten)]
    extra: std::collections::HashMap<String, serde_json::Value>,
}

pub async fn gen_tag(
    state: State<'_, StorageState>,
    evnet: & mut Event,
) -> Result<(), Box<dyn std::error::Error>> {
    let url = "https://api.siliconflow.cn/v1/chat/completions";
    let client = reqwest::Client::new();
    let tags = get_tags(state)
        .await?
        .iter()
        .map(|tag| tag.name.clone())
        .collect::<Vec<String>>()
        .join(",");
    let prompt = format!("你的任务是帮助用户为以下文本打上标签{}。
    标签可以不止一个，但是只能从给出的词语中进行选择：{}。
    你的答案只包含你选择的标签的内容，并且标签之间用英文逗号分隔。"
    ,evnet.content.clone(),tags);

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
                    evnet.metadata.tag = Some(generated_tag
                        .split(',')
                        .map(|s| s.to_string())
                        .collect());

                    Ok(())
                } else {
                    info!("No tag choices returned, using default");
                    Ok(())
                }
            }
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
