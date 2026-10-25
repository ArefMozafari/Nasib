//! Launch at login through the desktop's autostart folder, which GNOME, KDE, Cinnamon, Xfce and MATE all
//! read.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use crate::launch::BACKGROUND_FLAG;
use crate::text::APP_NAME;

const AUTOSTART_FOLDER: &str = "autostart";
const ENTRY_FILE_NAME: &str = "nasib.desktop";
/// Characters that make an Exec argument need quotes, from the Desktop Entry spec.
const EXEC_RESERVED_CHARACTERS: &[char] = &[
    ' ', '\t', '\n', '"', '\'', '\\', '>', '<', '~', '|', '&', ';', '$', '*', '?', '#', '(', ')',
    '`',
];
/// Characters that still need a backslash inside the quotes.
const EXEC_ESCAPED_IN_QUOTES: &[char] = &['"', '`', '$', '\\'];

/// The entry inside the given configuration folder, normally `~/.config`.
pub fn autostart_file(configuration_directory: &Path) -> PathBuf {
    configuration_directory
        .join(AUTOSTART_FOLDER)
        .join(ENTRY_FILE_NAME)
}

/// The user's configuration folder: `$XDG_CONFIG_HOME`, or `~/.config` without it.
pub fn configuration_directory() -> PathBuf {
    std::env::var_os("XDG_CONFIG_HOME")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
        .unwrap_or_else(|| {
            PathBuf::from(std::env::var_os("HOME").unwrap_or_default()).join(".config")
        })
}

pub fn is_enabled(configuration_directory: &Path) -> bool {
    autostart_file(configuration_directory).exists()
}

/// Quotes the path the way the Desktop Entry spec wants when it holds a space or a reserved character.
fn exec_argument(executable: &Path) -> String {
    let path = executable.to_string_lossy();
    if !path.contains(EXEC_RESERVED_CHARACTERS) {
        return path.into_owned();
    }
    let mut quoted = String::from('"');
    for character in path.chars() {
        if EXEC_ESCAPED_IN_QUOTES.contains(&character) {
            quoted.push('\\');
        }
        quoted.push(character);
    }
    quoted.push('"');
    quoted
}

fn entry(executable: &Path) -> String {
    format!(
        "[Desktop Entry]\nType=Application\nName={APP_NAME}\nExec={} {BACKGROUND_FLAG}\nIcon={}\nX-GNOME-Autostart-enabled=true\n",
        exec_argument(executable),
        crate::APP_ICON_NAME
    )
}

/// Turns launch at login on or off. The entry names the app by its full path, since a tarball install
/// lives in a folder the session's PATH may not include.
pub fn set_enabled(
    configuration_directory: &Path,
    enabled: bool,
    executable: &Path,
) -> io::Result<()> {
    let file = autostart_file(configuration_directory);
    if enabled {
        if let Some(folder) = file.parent() {
            fs::create_dir_all(folder)?;
        }
        fs::write(file, entry(executable))
    } else {
        match fs::remove_file(file) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}
