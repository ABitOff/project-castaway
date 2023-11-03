use ash::{vk, Entry};
use std::ffi::CStr;

// use project_castaway::hello_world;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Load the Vulkan library linked at compile time
    let entry = Entry::linked();

    let app_info = vk::ApplicationInfo {
        api_version: vk::API_VERSION_1_3,
        ..Default::default()
    };

    let create_info = vk::InstanceCreateInfo {
        p_application_info: &app_info,
        ..Default::default()
    };

    // Create a Vulkan instance
    let instance = unsafe { entry.create_instance(&create_info, None)? };

    // Enumerate physical devices
    let physical_devices = unsafe { instance.enumerate_physical_devices()? };

    // Iterate over the physical devices and print their names
    for &device in physical_devices.iter() {
        let device_properties = unsafe { instance.get_physical_device_properties(device) };

        // The device name is a cstr, convert it to a Rust string
        let device_name =
            unsafe { CStr::from_ptr(device_properties.device_name.as_ptr()).to_string_lossy() };

        println!("Device name: {}", device_name);
    }

    // Clean up
    unsafe {
        instance.destroy_instance(None);
    }

    Ok(())
}
