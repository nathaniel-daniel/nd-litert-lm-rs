use anyhow::Context;

pub fn get_time() -> serde_json::Value {
    let now = jiff::ZonedDateTime::now();
    serde_json::Value::String(now.to_string())
}

pub fn read_dir(path: &str) -> anyhow::Result<serde_json::Value> {
    let mut files = Vec::new();
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let file_name = entry
            .file_name()
            .into_string()
            .ok()
            .context("File name is not unicode")?;
        files.push(file_name);
    }
    anyhow::Ok(serde_json::to_value(&files)?)
}

pub fn pwd() -> anyhow::Result<serde_json::Value> {
    let pwd = std::env::current_dir()?;
    let pwd = pwd.into_string().ok().context("Path is not unicode")?;
    Ok(serde_json::Value::String(pwd))
}

pub fn read_file(path: &str) -> anyhow::Result<serde_json::Value> {
    let value = std::fs::read_to_string(path)?;
    Ok(serde_json::Value::String(value))
}
