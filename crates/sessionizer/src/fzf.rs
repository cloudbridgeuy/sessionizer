use color_eyre::eyre::{eyre, OptionExt, Result, WrapErr};
use tokio::io::AsyncWriteExt;

pub async fn sessions(sessions: Vec<String>) -> Result<String> {
    let mut fzf = tokio::process::Command::new("fzf")
        .args([
            "--header",
            "Press CTRL-X to delete a session.",
            "--bind",
            "ctrl-x:execute-silent(sessionizer sessions remove {+})+reload(sessionizer sessions list)"
        ])
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .wrap_err("Failed to spawn fzf")?;

    let mut stdin = fzf.stdin.take().ok_or_eyre("fail to take stdin")?;
    tokio::spawn(async move {
        stdin.write_all(sessions.join("\n").as_bytes()).await.expect("fail to write to stdin");
        drop(stdin);
    });

    // wait for the process to complete
    let fzf = fzf.wait_with_output().await?;

    // Bail if the status of fzf was an error
    if !fzf.status.success() {
        Err(eyre!("fzf error"))
    } else {
        Ok(String::from_utf8(fzf.stdout)?)
    }
}

pub async fn directories() -> Result<String> {
    let dirs = crate::directories::evaluate()?;

    let mut fzf = tokio::process::Command::new("fzf")
        .args(["--header", "Select a directory from the list to start a new session"])
        .stdout(std::process::Stdio::piped())
        .stdin(std::process::Stdio::piped())
        .spawn()
        .wrap_err("Failed to spawn fzf")?;

    let mut stdin = fzf.stdin.take().ok_or_eyre("fail to take stdin")?;
    tokio::spawn(async move {
        stdin.write_all(dirs.join("\n").as_bytes()).await.expect("fail to write to stdin");
        drop(stdin);
    });

    // wait for the process to complete
    let fzf = fzf.wait_with_output().await?;

    // Bail if the status of fzf was an error
    if !fzf.status.success() {
        Err(eyre!("fzf error"))
    } else {
        Ok(String::from_utf8(fzf.stdout)?)
    }
}
