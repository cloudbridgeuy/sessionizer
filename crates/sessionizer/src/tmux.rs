use color_eyre::eyre::{eyre, Result, WrapErr};

pub async fn kill_session(session: &str) -> Result<()> {
    log::debug!("$ tmux kill-session -t {}", session);
    match tokio::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux kill-session failed with status: {}", output.status);
                Err(eyre!(
                    "tmux kill-session failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                ))
            } else {
                log::debug!(
                    "tmux kill-session exited successfully\n{}",
                    String::from_utf8(output.stdout)?
                );
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to kill tmux session {}: {}", session, err)),
    }
}

pub async fn switch_client(session: &str) -> Result<()> {
    let session = session.replace('.', "·");

    log::debug!("$ tmux switch-client -t {}", session);
    match tokio::process::Command::new("tmux")
        .args(["switch-client", "-t", &session])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux switch-client failed with status: {}", output.status);
                Err(eyre!(
                    "tmux switch-client failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                ))
            } else {
                log::debug!(
                    "tmux switch-client exited successfully\n{}",
                    String::from_utf8(output.stdout)?
                );
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to switch client to session {}: {}", session, err)),
    }
}

pub async fn ls() -> Result<Vec<String>> {
    log::debug!("$ tmux ls");
    match tokio::process::Command::new("tmux")
        .args(["ls"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux ls failed with status: {}", output.status);
                Err(eyre!(
                    "tmux ls failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                ))
            } else {
                let stdout = String::from_utf8(output.stdout)
                    .wrap_err("fail to get the output from stdout")?;

                let sessions: Vec<String> = stdout
                    .split('\n')
                    .filter(|s| !s.is_empty())
                    .map(|s| s.split(':').next().unwrap().to_string())
                    .collect();

                Ok(sessions)
            }
        }
        Err(err) => Err(eyre!("fail to get the current tmux session: {}", err)),
    }
}

pub async fn current_session() -> Result<String> {
    log::debug!("$ tmux display-message -p '#S'");
    match tokio::process::Command::new("tmux")
        .args(["display-message", "-p", "'#S'"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux switch-client failed with status: {}", output.status);
                Err(eyre!(
                    "tmux display-message failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                ))
            } else {
                String::from_utf8(output.stdout).wrap_err("fail to get the output from stdout")
            }
        }
        Err(err) => Err(eyre!("fail to get the current tmux session: {}", err)),
    }
}

pub async fn is_active() -> Result<bool> {
    log::debug!("$ tmux info");
    match tokio::process::Command::new("tmux")
        .args(["info"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::debug!(
                    "tmux info failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                );
                Ok(false)
            } else {
                log::debug!("tmux info exited successfully\n");
                Ok(true)
            }
        }
        Err(err) => Err(eyre!("fail to get the current tmux session: {}", err)),
    }
}

pub async fn attach(session: &str) -> Result<bool> {
    let session = session.replace('.', "·");

    log::debug!("$ tmux attach -t {}", session);
    match tokio::process::Command::new("tmux")
        .args(["attach", "-t", format!("={}", session).as_str()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux attach failed with status: {}", output.status);
                Err(eyre!(
                    "tmux attach failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                ))
            } else {
                log::debug!(
                    "tmux switch-client exited successfully\n{}",
                    String::from_utf8(output.stdout)?
                );
                Ok(true)
            }
        }
        Err(err) => Err(eyre!("fail attach to session {}: {}", session, err)),
    }
}

pub async fn new_session(session: &str) -> Result<()> {
    let name = session.replace('.', "·");

    log::debug!("$ tmux new-session -s {} -c {} -d", name, session);
    match tokio::process::Command::new("tmux")
        .args(["new-session", "-s", &name, "-c", session, "-d"])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux new-session failed with status: {}", output.status);
                Err(eyre!(
                    "tmux new-session failed with status: {}\n{}",
                    output.status,
                    String::from_utf8(output.stderr)?
                ))
            } else {
                log::debug!(
                    "tmux switch-client exited successfully\n{}",
                    String::from_utf8(output.stdout)?
                );
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to create new session {}: {}", session, err)),
    }
}

pub async fn has_session(session: &str) -> Result<bool> {
    let session = session.replace('.', "·");

    log::debug!("$ tmux has-session -t {}", session);
    match tokio::process::Command::new("tmux")
        .args(["has-session", "-t", format!("={}", session).as_str()])
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                log::error!("tmux has-session failed with status: {}", output.status);
                Ok(false)
            } else {
                log::debug!(
                    "tmux has-session exited successfully\n{}",
                    String::from_utf8(output.stdout)?
                );
                Ok(true)
            }
        }
        Err(err) => Err(eyre!("fail to check if tmux has the session {}: {}", session, err)),
    }
}

pub async fn set(session: &str) -> Result<()> {
    if !has_session(session).await? {
        new_session(session).await?
    }

    if is_active().await? {
        switch_client(session).await?;
    } else {
        attach(session).await?;
    }

    Ok(())
}
