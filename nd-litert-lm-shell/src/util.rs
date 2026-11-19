use anyhow::Context;
use anyhow::bail;
use url::Url;

pub fn read_input() -> anyhow::Result<String> {
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

pub fn read_input_yn() -> anyhow::Result<bool> {
    let input = read_input()?;
    match input.to_lowercase().as_str() {
        "yes" | "y" => Ok(true),
        "no" | "n" => Ok(false),
        _ => bail!("\"{input}\" is not recognized"),
    }
}

pub fn get_file_name_from_url(url: &str) -> anyhow::Result<String> {
    let url = Url::parse(url).context("Failed to parse url")?;
    let path = url
        .path_segments()
        .context("Url missing path")?
        .next_back()
        .context("Missing file name")?;
    Ok(path.to_string())
}
