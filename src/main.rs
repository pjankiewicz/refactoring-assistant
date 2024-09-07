use std::env;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::process::Command as ProcessCommand;

use clap::{Arg, Command};
use glob::glob;
use reqwest::Client;
use serde_json::json;
use tokio;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // Parse command-line arguments
    let matches = Command::new("Refactoring Assistant")
        .version("1.1")
        .author("Author")
        .about("Applies changes to files based on instructions using OpenAI GPT API and validates them")
        .arg(
            Arg::new("instruction")
                .short('i')
                .long("instruction")
                .value_name("INSTRUCTION")
                .help("Instruction to follow or a path to a file containing instructions")
                .required(true)
        )
        .arg(
            Arg::new("file_pattern")
                .short('p')
                .long("pattern")
                .value_name("FILE_PATTERN")
                .help("File pattern to apply the changes (e.g. *.rs for Rust files)")
                .required(true)
        )
        .arg(
            Arg::new("model")
                .short('m')
                .long("model")
                .value_name("MODEL")
                .help("OpenAI model to use for the change")
                .default_value("gpt-4")
        )
        .arg(
            Arg::new("validate_with")
                .short('v')
                .long("validate-with")
                .value_name("VALIDATION_COMMAND")
                .help("Command to validate the change (e.g. `cargo build`)")
                .required(false)
        )
        .arg(
            Arg::new("n_retries")
                .short('r')
                .long("n-retries")
                .value_name("N_RETRIES")
                .help("Number of retries for validation")
                .default_value("5")
        )
        .get_matches();

    let instruction = matches.get_one::<String>("instruction").unwrap();
    let file_pattern = matches.get_one::<String>("file_pattern").unwrap();
    let default_model = "gpt-4".to_string();
    let model = matches.get_one::<String>("model").unwrap_or(&default_model);
    let validate_command = matches.get_one::<String>("validate_with");
    let n_retries: usize = matches
        .get_one::<String>("n_retries")
        .unwrap()
        .parse()
        .expect("n_retries must be a number");

    // Load the instruction either from a string or a file
    let instruction_content = if Path::new(instruction).exists() {
        fs::read_to_string(instruction)?
    } else {
        instruction.clone()
    };

    // Load OpenAI API key from environment
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY must be set in the environment");

    // Find files matching the given pattern
    for entry in glob(file_pattern).expect("Failed to read glob pattern") {
        match entry {
            Ok(path) => {
                if let Err(e) = process_file(&path, &instruction_content, model, &api_key, validate_command, n_retries).await {
                    eprintln!("Error processing file {}: {}", path.display(), e);
                }
            }
            Err(e) => eprintln!("Error reading file pattern: {}", e),
        }
    }

    Ok(())
}

async fn process_file(
    path: &Path,
    instruction: &str,
    model: &str,
    api_key: &str,
    validate_command: Option<&String>,
    n_retries: usize,
) -> Result<(), Box<dyn Error>> {
    let original_content = fs::read_to_string(path)?;
    let mut current_content = original_content.clone();
    let client = Client::new();

    // Retry mechanism
    for attempt in 0..n_retries {
        println!("Processing file {} (attempt {}/{})", path.display(), attempt + 1, n_retries);

        // Improved system message to better reflect the task
        let messages = vec![
            json!({
                "role": "system",
                "content": "You are an expert code transformation assistant. Your task is to carefully refactor code based on the user's instruction and return only the modified file contents enclosed within <CHANGED_FILE_CONTENTS> tags. Additionally, provide your reasoning inside <REASONING> tags. Do not include any other text outside these tags."
            }),
            json!({
                "role": "user",
                "content": "<INSTRUCTION>\nReplace all variable names that start with \"old_\" to start with \"new_\".\n</INSTRUCTION>\n\n<FILECONTENTS>\nlet old_value = 10;\nlet old_name = \"example\";\nlet other_var = 5;\n</FILECONTENTS>"
            }),
            json!({
                "role": "assistant",
                "content": "<REASONING>\nThe instruction is to change all variable names that start with \"old_\" to \"new_\". This is a straightforward text transformation, so the variables old_value and old_name will be renamed to new_value and new_name, respectively. Variables that don't start with \"old_\" remain unchanged.\n</REASONING>\n\n<CHANGED_FILE_CONTENTS>\nlet new_value = 10;\nlet new_name = \"example\";\nlet other_var = 5;\n</CHANGED_FILE_CONTENTS>"
            }),
            json!({
                "role": "user",
                "content": format!(
                    "<INSTRUCTION>\n{}\n</INSTRUCTION>\n\n<FILECONTENTS>\n{}\n</FILECONTENTS>",
                    instruction, current_content
                )
            }),
        ];

        let request_body = json!({
            "model": model,
            "messages": messages
        });

        let response = client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", api_key))
            .json(&request_body)
            .send()
            .await?;

        let response_json: serde_json::Value = response.json().await?;
        let output = response_json["choices"][0]["message"]["content"]
            .as_str()
            .ok_or("Failed to parse the response")?;

        // Extract content between <CHANGED_FILE_CONTENTS> tags
        let start_tag = "<CHANGED_FILE_CONTENTS>";
        let end_tag = "</CHANGED_FILE_CONTENTS>";
        let start = output.find(start_tag).ok_or("Start tag not found")? + start_tag.len();
        let end = output.find(end_tag).ok_or("End tag not found")?;
        let transformed_content = &output[start..end].trim();

        // Write the transformed content to the file
        fs::write(path, transformed_content)?;

        // Validate if the change was successful (if validation command is provided)
        if let Some(command) = validate_command {
            if validate_change(command)? {
                println!("Changes applied and validated for {}", path.display());
                return Ok(());
            } else {
                println!("Validation failed for {}, retrying...", path.display());
                // Restore original content after final retry
                if attempt == n_retries - 1 {
                    fs::write(path, &original_content)?;
                    println!("Restored original content for {}", path.display());
                }
            }
        } else {
            // If no validation command, consider the changes successful
            println!("Changes applied successfully for {}", path.display());
            return Ok(());
        }

        // Update current content for the next retry
        current_content = fs::read_to_string(path)?;
    }

    Err("Exceeded retry limit".into())
}

fn validate_change(command: &str) -> Result<bool, Box<dyn Error>> {
    let status = ProcessCommand::new("sh")
        .arg("-c")
        .arg(command)
        .status()?;

    Ok(status.success())
}
