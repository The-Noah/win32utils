use windows::{
  core::PCWSTR,
  Win32::{
    Foundation::ERROR_FILE_NOT_FOUND,
    System::Registry::{RegCloseKey, RegCreateKeyExW, RegOpenKeyExW, RegQueryValueExW, RegSetValueExW, HKEY_CURRENT_USER, KEY_READ, KEY_WRITE, REG_OPTION_NON_VOLATILE, REG_SZ},
  },
};

use crate::to_utf16;

pub enum HKEY {
  CurrentUser,
}

pub fn exists(hkey: HKEY, subkey: impl ToString, name: impl ToString) -> Result<bool, impl ToString> {
  let mut hkey = match hkey {
    HKEY::CurrentUser => HKEY_CURRENT_USER,
  };

  if unsafe { RegOpenKeyExW(HKEY_CURRENT_USER, PCWSTR::from_raw(to_utf16(subkey).as_ptr()), 0, KEY_READ, &mut hkey) }.is_err() {
    return Err("Failed to open registry key");
  }

  let result = unsafe { RegQueryValueExW(hkey, PCWSTR::from_raw(to_utf16(name).as_ptr()), None, None, None, None) };

  if result == ERROR_FILE_NOT_FOUND {
    return Ok(false);
  }

  if result.is_err() {
    return Err("Failed to query registry key");
  }

  if unsafe { RegCloseKey(hkey) }.is_err() {
    return Err("Failed to close registry key");
  }

  Ok(true)
}

pub fn write_string(hkey: HKEY, subkey: impl ToString, name: impl ToString, value: impl ToString) -> Result<(), impl ToString> {
  let mut hkey = match hkey {
    HKEY::CurrentUser => HKEY_CURRENT_USER,
  };

  if unsafe {
    RegCreateKeyExW(
      HKEY_CURRENT_USER,
      PCWSTR::from_raw(to_utf16(subkey).as_ptr()),
      0,
      None,
      REG_OPTION_NON_VOLATILE,
      KEY_WRITE,
      None,
      &mut hkey,
      None,
    )
  }
  .is_err()
  {
    return Err("Failed to create registry key");
  }

  if unsafe { RegSetValueExW(hkey, PCWSTR::from_raw(to_utf16(name).as_ptr()), 0, REG_SZ, Some(&v16_to_v8(&to_utf16(value)))) }.is_err() {
    return Err("Failed to set registry key");
  }

  if unsafe { RegCloseKey(hkey) }.is_err() {
    return Err("Failed to close registry key");
  }

  Ok(())
}

fn v16_to_v8(v: &[u16]) -> Vec<u8> {
  unsafe { std::slice::from_raw_parts(v.as_ptr() as *const u8, v.len() * 2).to_vec() }
}
