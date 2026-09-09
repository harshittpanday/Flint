use std::{ffi::OsStr, process::Command};

const CREATE_NO_WINDOW: u32 = 0x0800_0000;

fn creation_flags(release_build: bool, windows: bool) -> u32 {
    if release_build && windows {
        CREATE_NO_WINDOW
    } else {
        0
    }
}

pub fn std_command(program: impl AsRef<OsStr>) -> Command {
    let mut command = Command::new(program);
    #[cfg(windows)]
    {
        use std::os::windows::process::CommandExt;
        let flags = creation_flags(!cfg!(debug_assertions), true);
        if flags != 0 {
            command.creation_flags(flags);
        }
    }
    command
}

pub fn tokio_command(program: impl AsRef<OsStr>) -> tokio::process::Command {
    let mut command = tokio::process::Command::new(program);
    #[cfg(windows)]
    {
        let flags = creation_flags(!cfg!(debug_assertions), true);
        if flags != 0 {
            command.creation_flags(flags);
        }
    }
    command
}

#[cfg(test)]
mod tests {
    use super::creation_flags;

    #[test]
    fn release_windows_children_use_no_window_flag() {
        assert_eq!(creation_flags(true, true), 0x0800_0000);
    }

    #[test]
    fn debug_or_non_windows_children_keep_default_console_policy() {
        assert_eq!(creation_flags(false, true), 0);
        assert_eq!(creation_flags(true, false), 0);
    }
}
