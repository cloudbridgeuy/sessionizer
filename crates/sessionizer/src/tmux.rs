use color_eyre::eyre::{eyre, Result, WrapErr};

pub async fn kill_session(session: &str) -> Result<()> {
    match tokio::process::Command::new("tmux")
        .args(["kill-session", "-t", session])
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait()
        .await
    {
        Ok(status) => {
            if !status.success() {
                Err(eyre!("tmux failed with status: {}", status))
            } else {
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to kill tmux session {}: {}", session, err)),
    }
}

pub async fn switch_client(session: &str) -> Result<()> {
    match tokio::process::Command::new("tmux")
        .args(["switch-client", "-t", session])
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait()
        .await
    {
        Ok(status) => {
            if !status.success() {
                Err(eyre!("tmux failed with status: {}", status))
            } else {
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to switch client to session {}: {}", session, err)),
    }
}

pub async fn ls() -> Result<Vec<String>> {
    match tokio::process::Command::new("tmux")
        .args(["ls"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                Err(eyre!("tmux failed with status: {}", output.status))
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
    match tokio::process::Command::new("tmux")
        .args(["display-message", "-p", "'#S'"])
        .stdout(std::process::Stdio::piped())
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait_with_output()
        .await
    {
        Ok(output) => {
            if !output.status.success() {
                Err(eyre!("tmux failed with status: {}", output.status))
            } else {
                String::from_utf8(output.stdout).wrap_err("fail to get the output from stdout")
            }
        }
        Err(err) => Err(eyre!("fail to get the current tmux session: {}", err)),
    }
}

pub fn is_active() -> Result<bool> {
    match std::env::var("TMUX") {
        Ok(value) => Ok(!value.is_empty()),
        Err(_) => Err(eyre!("error getting TMUX environment variable")),
    }
}

pub async fn attach(session: &str) -> Result<bool> {
    match tokio::process::Command::new("tmux")
        .args(["attach", "-t", format!("={}", session).as_str()])
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait()
        .await
    {
        Ok(status) => {
            if !status.success() {
                Err(eyre!("tmux failed with status: {}", status))
            } else {
                Ok(true)
            }
        }
        Err(err) => Err(eyre!("fail attach to session {}: {}", session, err)),
    }
}

pub async fn new_session(session: &str) -> Result<()> {
    match tokio::process::Command::new("tmux")
        .args(["new-session", "-s", session, "-c", session, "-d"])
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait()
        .await
    {
        Ok(status) => {
            if !status.success() {
                Err(eyre!("tmux failed with status: {}", status))
            } else {
                Ok(())
            }
        }
        Err(err) => Err(eyre!("fail to create new session {}: {}", session, err)),
    }
}

pub async fn has_session(session: &str) -> Result<bool> {
    match tokio::process::Command::new("tmux")
        .args(["has-session", "-t", format!("={}", session).as_str()])
        .spawn()
        .wrap_err("fail to spawn tmux")?
        .wait()
        .await
    {
        Ok(status) => {
            if !status.success() {
                Err(eyre!("tmux failed with status: {}", status))
            } else {
                Ok(true)
            }
        }
        Err(err) => Err(eyre!("fail to check if tmux has the session {}: {}", session, err)),
    }
}
