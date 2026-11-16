use anyhow::Context;
use anyhow::bail;
use etcetera::AppStrategy;
use etcetera::AppStrategyArgs;
use nd_litert_lm::Conversation;
use nd_litert_lm::ConversationConfig;
use nd_litert_lm::ConversationMessage;
use nd_litert_lm::ConversationMessageContent;
use nd_litert_lm::ConversationMessageContentType;
use nd_litert_lm::ConversationMessageRole;
use nd_litert_lm::Engine;
use nd_litert_lm::EngineSettings;
use nd_litert_lm::EngineSettingsBackend;
use nd_litert_lm::LogSeverity;
use nd_litert_lm::get_api_version;
use once_cell::sync::OnceCell;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use url::Url;

// const MODEL_URL: &str = "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/main/gemma-4-E2B-it-gpu.litertlm?download=true";
const MODEL_URL: &str = "https://huggingface.co/litert-community/gemma-4-E2B-it-litert-lm/resolve/main/gemma-4-E2B-it.litertlm?download=true";

#[derive(Debug)]
struct AppPaths {
    cache_dir: PathBuf,
}

fn get_app_paths() -> anyhow::Result<&'static AppPaths> {
    static APP_PATHS: OnceCell<AppPaths> = OnceCell::new();
    APP_PATHS.get_or_try_init(|| {
        let strategy = etcetera::choose_app_strategy(AppStrategyArgs {
            top_level_domain: "".to_string(),
            author: "".to_string(),
            app_name: env!("CARGO_CRATE_NAME").to_string(),
        })
        .context("Failed to locate app directories")?;

        let cache_dir = strategy.cache_dir();
        std::fs::create_dir_all(&cache_dir).context("Failed to create cache dir")?;

        anyhow::Ok(AppPaths { cache_dir })
    })
}

fn read_input() -> anyhow::Result<String> {
    let mut input = String::new();

    std::io::stdin()
        .read_line(&mut input)
        .context("Failed to read stdin")?;
    if input.ends_with("\r\n") {
        input.pop();
        input.pop();
    }
    if input.ends_with("\n") {
        input.pop();
    }

    Ok(input)
}

fn read_input_yn() -> anyhow::Result<bool> {
    let input = read_input()?;
    match input.to_lowercase().as_str() {
        "yes" | "y" => Ok(true),
        "no" | "n" => Ok(false),
        _ => bail!("\"{input}\" is not recognized"),
    }
}

fn download_to_path_cli(url: &str, path: impl AsRef<Path>) -> anyhow::Result<()> {
    let path = path.as_ref();

    let response = ureq::get(url).call()?;
    let body = response.into_body();
    let content_length = body
        .content_length()
        .context("Missing content length header")?;

    let progress_bar = indicatif::ProgressBar::new(content_length);
    let progress_bar_style_template = "[Time = {elapsed_precise} | ETA = {eta_precise} | Speed = {bytes_per_sec}] {wide_bar} {bytes}/{total_bytes}";
    let progress_bar_style = indicatif::ProgressStyle::default_bar()
        .template(progress_bar_style_template)
        .expect("Invalid progress bar style template");
    progress_bar.set_style(progress_bar_style);

    let reader = body.into_reader();
    let temp_path = path.with_added_extension("temp");
    let mut out_file = File::create(&temp_path)?;
    std::io::copy(&mut progress_bar.wrap_read(reader), &mut out_file)?;
    out_file.flush()?;
    out_file.sync_all()?;
    drop(out_file);
    std::fs::rename(temp_path, path)?;

    Ok(())
}

fn get_file_name_from_url(url: &str) -> anyhow::Result<String> {
    let url = Url::parse(url).context("Failed to parse url")?;
    let path = url
        .path_segments()
        .context("Url missing path")?
        .next_back()
        .context("Missing file name")?;
    Ok(path.to_string())
}

fn main() -> anyhow::Result<()> {
    let api_version = get_api_version().context("Failed to get api version")?;
    println!("Lite-RT-LM Version: {api_version}");
    nd_litert_lm::set_min_log_level(LogSeverity::Silent)?;

    let app_paths = get_app_paths()?;
    let model_file_name = get_file_name_from_url(MODEL_URL)?;
    let model_path = app_paths.cache_dir.join(&model_file_name);
    if !model_path.try_exists()? {
        println!("Model \"{model_file_name}\" has not been downloaded yet.");
        println!("Download? [Y/N]");
        let yes = read_input_yn()?;
        if !yes {
            println!("Exiting...");
            return Ok(());
        }

        download_to_path_cli(MODEL_URL, &model_path).context("Failed to download model")?;
    }

    let engine_settings = EngineSettings::new(&model_path, EngineSettingsBackend::Cpu)
        .context("Failed to create engine settings")?;
    let engine = Engine::new(&engine_settings).context("Failed to create engine")?;

    let conversation_config =
        ConversationConfig::new().context("Failed to create conversation config")?;
    let conversation = Conversation::new(&engine, &conversation_config)
        .context("Failed to create conversation")?;

    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let input = read_input()?;
        if input.to_lowercase() == ".exit" {
            break;
        }

        let response = conversation
            .send_message(&ConversationMessage {
                role: ConversationMessageRole::User,
                content: vec![ConversationMessageContent {
                    type_: ConversationMessageContentType::Text,
                    text: input,
                }],
            })
            .context("Failed to send message")?;

        let parsed = response.get_string_parsed()?;
        for content in parsed.content.iter() {
            println!("< {}", content.text);
        }
    }
    println!("Exiting...");

    Ok(())
}
