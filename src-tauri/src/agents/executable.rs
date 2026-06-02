use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

pub fn command_exists(binary: &str) -> bool {
    find_binary(binary).is_some()
}

pub fn find_binary(binary: &str) -> Option<PathBuf> {
    if let Some(path) = which(binary) {
        return Some(path);
    }

    candidate_dirs()
        .into_iter()
        .find_map(|dir| candidate_paths(&dir, binary).into_iter().find(|path| path.is_file()))
}

fn which(binary: &str) -> Option<PathBuf> {
    #[cfg(windows)]
    {
        return std::process::Command::new("where.exe")
            .arg(binary)
            .output()
            .ok()
            .filter(|output| output.status.success())
            .and_then(|output| String::from_utf8(output.stdout).ok())
            .and_then(|stdout| stdout.lines().map(str::trim).find(|line| !line.is_empty()).map(PathBuf::from));
    }

    #[cfg(not(windows))]
    {
    std::process::Command::new("which")
        .arg(binary)
        .output()
        .ok()
        .filter(|output| output.status.success())
        .and_then(|output| String::from_utf8(output.stdout).ok())
        .map(|path| path.trim().to_string())
        .filter(|path| !path.is_empty())
        .map(PathBuf::from)
    }
}

fn candidate_dirs() -> Vec<PathBuf> {
    let mut dirs = BTreeSet::new();

    if let Some(path) = std::env::var_os("PATH") {
        dirs.extend(std::env::split_paths(&path));
    }

    dirs.extend([
        PathBuf::from("/opt/homebrew/bin"),
        PathBuf::from("/usr/local/bin"),
        PathBuf::from("/usr/bin"),
        PathBuf::from("/bin"),
        PathBuf::from("/usr/sbin"),
        PathBuf::from("/sbin"),
    ]);

    if let Some(home) = dirs::home_dir() {
        dirs.extend([
            home.join(".local").join("bin"),
            home.join(".npm-global").join("bin"),
            home.join(".bun").join("bin"),
            home.join(".cargo").join("bin"),
        ]);
        dirs.extend(nvm_node_bins(&home));
    }

    dirs.into_iter().collect()
}

fn candidate_paths(dir: &Path, binary: &str) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        let path = Path::new(binary);
        if path.extension().is_some() {
            return vec![dir.join(binary)];
        }

        let mut candidates = vec![dir.join(binary)];
        let pathext = std::env::var_os("PATHEXT")
            .and_then(|value| value.into_string().ok())
            .unwrap_or_else(|| ".COM;.EXE;.BAT;.CMD".to_string());
        for ext in pathext.split(';').filter(|ext| !ext.trim().is_empty()) {
            let normalized = if ext.starts_with('.') {
                ext.to_string()
            } else {
                format!(".{ext}")
            };
            candidates.push(dir.join(format!("{binary}{normalized}")));
        }
        return candidates;
    }

    #[cfg(not(windows))]
    {
        vec![dir.join(binary)]
    }
}

fn nvm_node_bins(home: &Path) -> Vec<PathBuf> {
    let versions = home.join(".nvm").join("versions").join("node");
    let Ok(entries) = std::fs::read_dir(versions) else {
        return Vec::new();
    };

    entries
        .filter_map(Result::ok)
        .map(|entry| entry.path().join("bin"))
        .filter(|path| path.is_dir())
        .collect()
}
