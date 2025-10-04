use bevy::prelude::*;

#[derive(Component)]
pub struct UrlButton {
    pub url: String,
}

impl UrlButton {
    pub fn new(url: String) -> Self {
        Self { url }
    }
}

pub struct UrlPlugin;

impl Plugin for UrlPlugin {
    fn build(&self, app: &mut App) {
        // 这里可以添加系统，如果需要的话
    }
}

// 将 fn 改为 pub fn
pub fn open_url(url: &str) -> Result<(), String> {
    #[cfg(target_os = "windows")]
    {
        std::process::Command::new("cmd")
            .args(["/C", "start", url])
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    #[cfg(target_os = "macos")]
    {
        std::process::Command::new("open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    #[cfg(target_os = "linux")]
    {
        std::process::Command::new("xdg-open")
            .arg(url)
            .spawn()
            .map_err(|e| format!("Failed to open URL: {}", e))?;
    }

    Ok(())
}
