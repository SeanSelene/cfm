use crate::config::{RepoConfig, UserConfig};

pub fn execute() -> Result<(), String> {
    let user_config = UserConfig::load()?;
    let repo_config = RepoConfig::from_user_cfg(&user_config)?;
    repo_config.print(&user_config.repo_path);
    Ok(())
}
