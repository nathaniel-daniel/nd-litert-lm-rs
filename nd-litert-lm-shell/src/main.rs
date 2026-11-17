use anyhow::Context;
use anyhow::bail;
use etcetera::AppStrategy;
use etcetera::AppStrategyArgs;
use nd_litert_lm::Conversation;
use nd_litert_lm::ConversationConfig;
use nd_litert_lm::ConversationMessage;
use nd_litert_lm::ConversationMessageContent;
use nd_litert_lm::ConversationMessageRole;
use nd_litert_lm::ConversationOptionalArgs;
use nd_litert_lm::Engine;
use nd_litert_lm::EngineSettings;
use nd_litert_lm::EngineSettingsBackend;
use nd_litert_lm::LogSeverity;
use nd_litert_lm::ThinkingConfig;
use nd_litert_lm::get_api_version;
use once_cell::sync::OnceCell;
use std::collections::HashMap;
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
            app_name: env!("CARGO_PKG_NAME").to_string(),
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

fn run_attach_command(
    command_parser: &mut CommandParser,
    image_attachment: &mut Option<Vec<u8>>,
    audio_attachment: &mut Option<Vec<u8>>,
) -> anyhow::Result<()> {
    if image_attachment.is_some() {
        bail!("An image is already attached");
    }

    let path = command_parser.next_arg()?.context(".attach needs a path")?;
    let extension = path
        .rsplit_once('.')
        .map(|(_head, tail)| tail)
        .with_context(|| format!("Path \"{path}\" is missing an extension"))?
        .to_lowercase();

    match extension.as_str() {
        "jpg" | "jpeg" | "png" => {
            let data = std::fs::read(path).with_context(|| format!("Failed to read \"{path}\""))?;
            *image_attachment = Some(data);

            println!("Attached \"{path}\" as an image.");
        }
        // TODO: Add support for OGG by converting to WAV.
        "mp3" | "wav" => {
            let data = std::fs::read(path).with_context(|| format!("Failed to read \"{path}\""))?;
            *audio_attachment = Some(data);

            println!("Attached \"{path}\" as audio.");
        }
        extension => {
            bail!("Unknown extension \"{extension}\"");
        }
    }

    Ok(())
}

fn run_help_command() {
    let command_info = [
        (
            ".attach <path>",
            "Attach an image or audio to the next chat.",
        ),
        (".bye", "Exit the program."),
        (".disable-thinking", "Disable thinking."),
        (".enable-thinking", "Enable thinking."),
        (".exit", "Exit the program."),
        (".help", "Get help about various commands."),
        (".hide-thinking", "Hide thought process from the user."),
        (".show-thinking", "Show thought process to the user."),
        (".quit", "Exit the program."),
    ];

    println!("Help");
    for (name, desc) in command_info {
        println!("{name}: {desc}");
    }
}

#[derive(Debug)]
struct CommandParser<'a> {
    text: &'a str,
}

impl<'a> CommandParser<'a> {
    fn new(text: &'a str) -> Self {
        Self { text }
    }

    fn advance(&mut self, n: usize) {
        self.text = &self.text[n..];
    }

    fn read_word(&mut self) -> Option<&'a str> {
        if self.text.is_empty() {
            return None;
        }

        let optional_arg = self.text.split_once(' ');
        let arg = match optional_arg {
            Some((head, tail)) => {
                self.text = tail;
                head
            }
            None => std::mem::take(&mut self.text),
        };

        Some(arg)
    }

    fn next_command(&mut self) -> Option<&'a str> {
        self.text = self.text.trim();
        self.read_word()
    }

    fn next_arg(&mut self) -> anyhow::Result<Option<&'a str>> {
        loop {
            self.text = self.text.trim();

            if self.text.is_empty() {
                return Ok(None);
            }

            if self.text.starts_with('"') {
                let mut end_index = None;

                for (index, ch) in self.text.char_indices().skip(1) {
                    if ch == '"' {
                        // Index should be non-inclusive for indexing.
                        end_index = Some(index + 1);
                        break;
                    }
                }
                let end_index = end_index.context("Unpaired \" in argument")?;
                let arg = &self.text[..end_index];
                self.advance(arg.len());

                return Ok(Some(&arg[1..end_index - 1]));
            }

            let arg = match self.read_word() {
                Some(arg) => arg,
                None => return Ok(None),
            };
            if !arg.is_empty() {
                return Ok(Some(arg));
            }
        }
    }
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

    println!("Enter system message (Press Enter to skip.):");
    let system_message = read_input()?;

    let engine_settings = EngineSettings::new(
        &model_path,
        EngineSettingsBackend::Cpu,
        Some(EngineSettingsBackend::Cpu),
        Some(EngineSettingsBackend::Cpu),
    )
    .context("Failed to create engine settings")?;
    let engine = Engine::new(&engine_settings).context("Failed to create engine")?;

    let conversation_config =
        ConversationConfig::new().context("Failed to create conversation config")?;
    if !system_message.trim().is_empty() {
        conversation_config.set_system_message(system_message.as_str())?;
        println!("System prompt set.");
    }
    let conversation = Conversation::new(&engine, &conversation_config)
        .context("Failed to create conversation")?;

    let thinking_config = ThinkingConfig::new().context("Failed to create thinking config")?;
    thinking_config.enable_thinking(true)?;

    let conversation_optional_args = ConversationOptionalArgs::new()?;
    conversation_optional_args.set_thinking_config(&thinking_config)?;

    let mut show_thinking = false;
    let mut image_attachment = None;
    let mut audio_attachment = None;
    run_help_command();
    loop {
        print!("> ");
        std::io::stdout().flush()?;
        let input = read_input()?;
        let mut command_parser = CommandParser::new(input.trim());
        let command_name = command_parser.next_command().unwrap_or("").to_lowercase();
        match command_name.as_str() {
            ".attach" => {
                match run_attach_command(
                    &mut command_parser,
                    &mut image_attachment,
                    &mut audio_attachment,
                ) {
                    Ok(()) => {}
                    Err(error) => {
                        println!("Error: {error:?}");
                    }
                }

                continue;
            }
            ".bye" | ".exit" | ".quit" => {
                break;
            }
            ".enable-thinking" => {
                thinking_config.enable_thinking(true)?;
                conversation_optional_args.set_thinking_config(&thinking_config)?;

                println!("Thinking enabled.");
                continue;
            }
            ".disable-thinking" => {
                thinking_config.enable_thinking(false)?;
                conversation_optional_args.set_thinking_config(&thinking_config)?;

                println!("Thinking disabled.");
                continue;
            }
            ".help" => {
                run_help_command();

                continue;
            }
            ".hide-thinking" => {
                show_thinking = false;

                println!("Thinking will be hidden.");
                continue;
            }
            ".show-thinking" => {
                show_thinking = true;

                println!("Thinking will be shown.");
                continue;
            }
            _ => {}
        }

        let mut content = Vec::with_capacity(2);
        if let Some(image) = image_attachment.take() {
            content.push(ConversationMessageContent::Image {
                blob: Some(image),
                path: None,
            });
        }
        if let Some(audio) = audio_attachment.take() {
            content.push(ConversationMessageContent::Audio {
                blob: Some(audio),
                path: None,
            });
        }
        content.push(ConversationMessageContent::Text { text: input });

        let response = conversation
            .send_message(
                &ConversationMessage {
                    role: ConversationMessageRole::User,
                    content,
                    channels: HashMap::new(),
                },
                Some(&conversation_optional_args),
            )
            .context("Failed to send message")?;

        let parsed = response.get_string_parsed()?;
        dbg!(&parsed);
        if let Some(channel) = parsed.channels.get("thought")
            && show_thinking
        {
            println!("<< {channel}");
        }
        for content in parsed.content.iter() {
            match content {
                ConversationMessageContent::Text { text } => {
                    println!("< {text}");
                }
                ConversationMessageContent::Image { .. } => {
                    bail!("Model unexpectedly generated an image.");
                }
                ConversationMessageContent::Audio { .. } => {
                    bail!("Model unexpectedly generated audio.");
                }
            }
        }
    }
    println!("Exiting...");

    Ok(())
}
