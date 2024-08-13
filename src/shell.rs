use windows::{
  core::GUID,
  Win32::{
    Foundation::{HINSTANCE, HWND, LPARAM, LRESULT, POINT, WPARAM},
    Graphics::{
      Dwm::{DwmSetWindowAttribute, DWMWA_USE_IMMERSIVE_DARK_MODE},
      Gdi::UpdateWindow,
    },
    System::LibraryLoader::GetModuleHandleExW,
    UI::{
      Shell::{Shell_NotifyIconW, NIF_GUID, NIF_ICON, NIF_MESSAGE, NIF_TIP, NIM_ADD, NIM_DELETE, NIM_MODIFY, NOTIFYICONDATAW, NOTIFYICON_VERSION_4},
      WindowsAndMessaging::{
        AppendMenuW, CreatePopupMenu, CreateWindowExW, DefWindowProcW, DispatchMessageW, GetCursorPos, GetMessageW, GetWindowLongPtrW, LoadIconW, PostMessageW, RegisterClassExW,
        SetForegroundWindow, SetWindowLongPtrW, TrackPopupMenu, TranslateMessage, CREATESTRUCTW, CW_USEDEFAULT, GWL_USERDATA, HMENU, MF_STRING, MSG, TPM_BOTTOMALIGN,
        TPM_LEFTALIGN, WINDOW_STYLE, WM_APP, WM_COMMAND, WM_DESTROY, WM_LBUTTONUP, WM_NCCREATE, WM_QUIT, WM_RBUTTONUP, WNDCLASSEXW, WNDCLASS_STYLES, WS_EX_LAYERED,
        WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TRANSPARENT,
      },
    },
  },
};

use crate::*;

struct TrayData {
  app_name: String,
  nid: NOTIFYICONDATAW,
  menu: HMENU,
}

/// Add a tray icon to the taskbar
///
/// Note: This function will block the current thread
///
/// # Example
///
/// ```rust
/// thread::spawn(|| {
///   win32utils::shell::tray_icon("My App");
/// });
/// ```
pub fn tray_icon(app_name: impl ToString) {
  let mut tooltip = to_utf16(app_name.to_string());
  tooltip.resize(128, 0);

  let mut wnd_class: WNDCLASSEXW = unsafe { std::mem::zeroed() };
  wnd_class.cbSize = std::mem::size_of::<WNDCLASSEXW>() as u32;
  wnd_class.lpfnWndProc = Some(window_proc);
  wnd_class.hInstance = unsafe {
    let mut handle = std::mem::zeroed();
    GetModuleHandleExW(0, None, &mut handle).unwrap();
    HINSTANCE(handle.0)
  };
  wnd_class.lpszClassName = PCWSTR::from_raw(to_utf16(app_name.to_string()).as_ptr());
  wnd_class.style = WNDCLASS_STYLES::default();

  unsafe { RegisterClassExW(&wnd_class) };

  let mut nid: NOTIFYICONDATAW = unsafe { std::mem::zeroed() };

  let menu = unsafe { CreatePopupMenu() }.unwrap();
  unsafe { AppendMenuW(menu, MF_STRING, 1000, PCWSTR(to_utf16("Quit").as_ptr())) }.unwrap();

  let tray_data = TrayData {
    app_name: app_name.to_string(),
    nid,
    menu,
  };

  let hwnd = unsafe {
    CreateWindowExW(
      WS_EX_NOACTIVATE | WS_EX_TRANSPARENT | WS_EX_LAYERED | WS_EX_TOOLWINDOW,
      PCWSTR::from_raw(to_utf16(app_name.to_string()).as_ptr()),
      PCWSTR::from_raw(to_utf16(app_name.to_string()).as_ptr()),
      WINDOW_STYLE::default(),
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      CW_USEDEFAULT,
      None,
      None,
      HINSTANCE::default(),
      Some(Box::into_raw(Box::new(tray_data)) as _),
    )
  }
  .unwrap();

  unsafe {
    DwmSetWindowAttribute(
      hwnd,
      DWMWA_USE_IMMERSIVE_DARK_MODE,
      &1 as *const _ as *const std::ffi::c_void,
      std::mem::size_of::<u32>() as u32,
    )
  }
  .unwrap();

  unsafe { UpdateWindow(hwnd) }.unwrap();

  nid.cbSize = std::mem::size_of::<NOTIFYICONDATAW>() as u32;
  nid.uFlags = NIF_ICON | NIF_MESSAGE | NIF_TIP | NIF_GUID;
  nid.hIcon = unsafe {
    // Get the icon from the current executable
    let mut handle = std::mem::zeroed();
    GetModuleHandleExW(0, None, &mut handle).unwrap();
    LoadIconW(HINSTANCE(handle.0), PCWSTR::from_raw(to_utf16("ICON").as_ptr())).unwrap()
  };
  nid.hWnd = hwnd;
  nid.guidItem = GUID::from("5b8729e5-191d-48e1-942f-d35d61962894");
  nid.uCallbackMessage = WM_APP;
  nid.szTip = tooltip.try_into().unwrap();
  nid.Anonymous.uVersion = NOTIFYICON_VERSION_4;

  let mut msg = MSG::default();

  if !unsafe { Shell_NotifyIconW(NIM_ADD, &nid) }.as_bool() && !unsafe { Shell_NotifyIconW(NIM_MODIFY, &nid) }.as_bool() {
    panic!("Failed to add taskbar icon");
  }

  loop {
    unsafe {
      GetMessageW(&mut msg, nid.hWnd, 0, 0);
      TranslateMessage(&msg);
      DispatchMessageW(&msg);
    }
  }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w_param: WPARAM, l_param: LPARAM) -> LRESULT {
  if msg == WM_NCCREATE {
    let create_struct = &mut *(l_param.0 as *mut CREATESTRUCTW);
    let tray_data = create_struct.lpCreateParams as *mut TrayData;
    SetWindowLongPtrW(hwnd, GWL_USERDATA, tray_data as _);
  }

  let tray_data = GetWindowLongPtrW(hwnd, GWL_USERDATA) as *mut TrayData;
  let tray_data = &mut *tray_data;

  match msg {
    WM_DESTROY => {
      PostMessageW(hwnd, WM_QUIT, WPARAM::default(), LPARAM::default()).unwrap();
    }
    WM_QUIT => {
      println!("Quitting");

      Shell_NotifyIconW(NIM_DELETE, &tray_data.nid);

      std::process::exit(0);
    }
    WM_APP => {
      if l_param.0 as u32 == WM_LBUTTONUP {
        if let DialogResult::Yes = dialog(
          format!("Quit {}", tray_data.app_name),
          format!("Would you like to quit {}?", tray_data.app_name),
          DialogIcon::Question,
          DialogButtons::YesNo,
        ) {
          PostMessageW(hwnd, WM_QUIT, WPARAM::default(), LPARAM::default()).unwrap();
        }
      } else if l_param.0 as u32 == WM_RBUTTONUP {
        let mut cursor = POINT { x: 0, y: 0 };
        GetCursorPos(&mut cursor).unwrap();

        SetForegroundWindow(hwnd).unwrap();
        TrackPopupMenu(tray_data.menu, TPM_BOTTOMALIGN | TPM_LEFTALIGN, cursor.x, cursor.y, 0, hwnd, None).unwrap();
      }
    }
    WM_COMMAND => {
      if w_param.0 == 1000 {
        PostMessageW(hwnd, WM_QUIT, WPARAM::default(), LPARAM::default()).unwrap();
      }
    }
    _ => {}
  }

  DefWindowProcW(hwnd, msg, w_param, l_param)
}
