use std::path::{Path, PathBuf};
use windows_sys::Win32::System::Registry::{
    RegGetValueW, HKEY, HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, RRF_RT_REG_SZ, RRF_SUBKEY_WOW6464KEY,
};

pub const REGISTRY_KEY: &str = r"Software\Edoli\Edolview";
pub const RESULT_FILE: &str = "update-result.txt";

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InstallScope {
    User,
    Machine,
}

/// Match the actual executable directory so copied portable builds are not
/// mistaken for registered installations.
pub fn install_scope(exe: &Path) -> Option<InstallScope> {
    let directory = exe.parent()?.canonicalize().ok()?;
    for (root, scope) in [
        (HKEY_CURRENT_USER, InstallScope::User),
        (HKEY_LOCAL_MACHINE, InstallScope::Machine),
    ] {
        if registered_directory(root)
            .and_then(|path| path.canonicalize().ok())
            .is_some_and(|path| same_path(&path, &directory))
        {
            return Some(scope);
        }
    }
    None
}

fn same_path(left: &Path, right: &Path) -> bool {
    left.as_os_str()
        .to_string_lossy()
        .eq_ignore_ascii_case(&right.as_os_str().to_string_lossy())
}

fn registered_directory(root: HKEY) -> Option<PathBuf> {
    let wide = |text: &str| text.encode_utf16().chain(Some(0)).collect::<Vec<_>>();
    let mut buffer = [0u16; 32768];
    let mut size = std::mem::size_of_val(&buffer) as u32;
    // SAFETY: Live fixed-size buffer, NUL-terminated names, and a predefined registry root.
    let status = unsafe {
        RegGetValueW(
            root,
            wide(REGISTRY_KEY).as_ptr(),
            wide("InstallLocation").as_ptr(),
            RRF_RT_REG_SZ | RRF_SUBKEY_WOW6464KEY,
            std::ptr::null_mut(),
            buffer.as_mut_ptr().cast(),
            &mut size,
        )
    };
    if status != 0 || size < 2 {
        return None;
    }
    let length = (size as usize / 2).min(buffer.len());
    let end = buffer[..length].iter().position(|&c| c == 0)?;
    Some(PathBuf::from(String::from_utf16(&buffer[..end]).ok()?))
}
