mod interfaces;
use interfaces::*;

use std::{
    collections::HashMap,
    ffi::{CStr, CString, c_void},
    sync::{Arc, Mutex},
    thread,
};

use windows::{Win32::{System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED, CoCreateInstance, CLSCTX_ALL}, Foundation::HWND}, core::{Interface, Vtable}};

pub(crate) struct ComInit();

impl ComInit {
    pub fn new() -> Self {
        unsafe {
            // Notice: Only COINIT_APARTMENTTHREADED works correctly!
            //
            // Not COINIT_MULTITHREADED or CoIncrementMTAUsage, they cause a seldom crashes in threading tests.
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).unwrap();
        }
        ComInit()
    }
}

impl Drop for ComInit {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

thread_local! {
    pub(crate) static COM_INIT: ComInit = ComInit::new();
}


fn get_iservice_provider() -> IServiceProvider {
    COM_INIT.with(|_| unsafe {
        CoCreateInstance(&CLSID_ImmersiveShell, None, CLSCTX_ALL).unwrap()
    })
}

fn get_iapplication_view_collection(provider: &IServiceProvider) -> IApplicationViewCollection {
    COM_INIT.with(|_| {
        let mut obj = std::ptr::null_mut::<c_void>();
        unsafe {
            provider
                .query_service(
                    &IApplicationViewCollection::IID,
                    &IApplicationViewCollection::IID,
                    &mut obj,
                )
                .unwrap();
        }
        assert_eq!(obj.is_null(), false);

        unsafe { IApplicationViewCollection::from_raw(obj) }
    })
}

#[no_mangle]
pub extern "C" fn SetCloak(hwnd: HWND, cloak_type: u32, flags: i32) {
    COM_INIT.with(|_|{
        let provider = get_iservice_provider();
        let view_collection = get_iapplication_view_collection(&provider);
        let mut view = None;
        unsafe { view_collection.get_view_for_hwnd(hwnd, &mut view).unwrap() };
        let view = view.unwrap();

        unsafe { view.set_cloak(cloak_type, flags).unwrap() };
    })
}

#[cfg(test)]
mod test {
    use windows::{Win32::UI::WindowsAndMessaging::FindWindowW, core::PCWSTR};

    use super::*;
    #[test]
    fn test_set_cloak() {
        let notepad_hwnd = unsafe {
            let notepad = "notepad\0".encode_utf16().collect::<Vec<_>>();
            let pw = PCWSTR::from_raw(notepad.as_ptr());
            unsafe { FindWindowW(pw, PCWSTR::null()) }
        };

        // I don't know what to put here?
        SetCloak(notepad_hwnd, 1234,1234);
    }
}