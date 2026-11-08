use libloading::{Library, Symbol};
use std::ffi::{c_char, CStr}; 

type GetVersion =
    unsafe extern "system" fn(*mut *const c_char) -> u32;
    
fn main() -> Result<(), Box<dyn std::error::Error>> {
    unsafe {
        let lib = Library::new("phidget22.dll")?;
        
        let get_version: Symbol<GetVersion> =
            lib.get(b"Phidget_getLibraryVersion")?;
            
        let mut ptr: *const c_char = std::ptr::null();
        
        let result = get_version(&mut ptr);
        
        if result != 0 || ptr.is_null() {
            return Err("Failed to get library version".into());
        }
        
        let version = CStr::from_ptr(ptr).to_string_lossy();
        
        println!("Phodget22: {}", version);
    }
    
    Ok(())
}