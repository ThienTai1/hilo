mod client;
mod config;
mod error;
mod models;
mod tools;

use client::OllamaClient;
use config::AppConfig;
use models::ChatMessage;
use std::process;
use tools::{available_tools, execute_tool};

#[tokio::main]
async fn main() {
    let config = AppConfig::from_env();
    let ollama = match OllamaClient::new(&config) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Lỗi khởi tạo: {e}");
            process::exit(1);
        }
    };

    let tools = available_tools();
    let mut messages = vec![ChatMessage {
        role: "user".to_string(),
        content: Some("Máy chủ đang chạy OS và kiến trúc CPU nào? Nhớ dùng tool.".to_string()),
        tool_calls: None,
    }];

    let max_turns = 5;
    for _ in 0..max_turns {
        let res = match ollama.chat(messages.clone(), available_tools()).await {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Lỗi gọi chat: {e}");
                process::exit(1);
            }
        };

        let assistant_msg = res.message;
        messages.push(assistant_msg.clone());

        match assistant_msg.tool_calls {
            Some(calls) if !calls.is_empty() => {
                for call in calls {
                    let tool_name = &call.function.name;
                    let tool_args = &call.function.arguments;
                    println!("[Tool Call] {tool_name} với args: {tool_args}");

                    let output = execute_tool(tool_name, tool_args);
                    messages.push(ChatMessage {
                        role: "tool".to_string(),
                        content: Some(output),
                        tool_calls: None,
                    });
                }
            }
            _ => {
                println!(
                    "\nPhản hồi cuối:\n{}",
                    assistant_msg.content.unwrap_or_default()
                );
                return;
            }
        }
    }

    eprintln!("Đã đạt giới hạn {max_turns} lượt gọi tool.");
    let _ = tools;
}
