use windows::core::PCWSTR;
use windows::Win32::UI::WindowsAndMessaging::{
  MessageBoxW, IDNO, IDOK, IDYES, MB_ICONERROR, MB_ICONINFORMATION, MB_ICONQUESTION, MB_ICONWARNING, MB_OK, MB_YESNO, MESSAGEBOX_RESULT, MESSAGEBOX_STYLE,
};

pub mod registry;

pub enum DialogIcon {
  Info = MB_ICONINFORMATION.0 as isize,
  Question = MB_ICONQUESTION.0 as isize,
  Warning = MB_ICONWARNING.0 as isize,
  Error = MB_ICONERROR.0 as isize,
}

pub enum DialogButtons {
  Ok = MB_OK.0 as isize,
  YesNo = MB_YESNO.0 as isize,
}

pub enum DialogResult {
  Ok = IDOK.0 as isize,
  Yes = IDYES.0 as isize,
  No = IDNO.0 as isize,
}

impl From<MESSAGEBOX_RESULT> for DialogResult {
  fn from(value: MESSAGEBOX_RESULT) -> Self {
    match value {
      IDOK => DialogResult::Ok,
      IDYES => DialogResult::Yes,
      IDNO => DialogResult::No,
      _ => panic!("Invalid dialog result"),
    }
  }
}

/// Display a message box dialog.
pub fn dialog(title: impl ToString, text: impl ToString, icon: DialogIcon, buttons: DialogButtons) -> DialogResult {
  let title = to_utf16(title);
  let text = to_utf16(text);

  unsafe {
    MessageBoxW(
      None,
      PCWSTR(text.as_ptr()),
      PCWSTR(title.as_ptr()),
      MESSAGEBOX_STYLE(icon as u32) | MESSAGEBOX_STYLE(buttons as u32),
    )
    .into()
  }
}

fn to_utf16(s: impl ToString) -> Vec<u16> {
  s.to_string().encode_utf16().chain(std::iter::once(0)).collect()
}
