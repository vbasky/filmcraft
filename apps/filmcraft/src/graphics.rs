//! Windows graphics instance configuration, applied before wgpu can initialize any driver.

use eframe::{NativeOptions, egui_wgpu::WgpuSetup, wgpu::Backends};

/// Use DirectX 12 unless WGPU_BACKEND explicitly selects another backend.
pub fn configure(options: &mut NativeOptions, backend_override: Option<Backends>) {
    if let WgpuSetup::CreateNew(create) = &mut options.wgpu_options.wgpu_setup {
        // Vulkan and OpenGL drivers can fault during instance initialization, before
        // adapter selection or Rust error handling. DX12 avoids probing those drivers.
        // An explicit override stays available for hardware without DX12 and for
        // shader-compiler compatibility (FXC can send some effects to the CPU fallback).
        create.instance_descriptor.backends = backend_override.unwrap_or(Backends::DX12);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn configured_backends(backend_override: Option<Backends>) -> Backends {
        let mut options = NativeOptions::default();
        configure(&mut options, backend_override);
        let WgpuSetup::CreateNew(create) = options.wgpu_options.wgpu_setup else {
            panic!("default native options must create a wgpu instance");
        };
        create.instance_descriptor.backends
    }

    #[test]
    fn windows_default_avoids_vulkan_and_gl() {
        // eframe's own default (PRIMARY | GL) would initialize OpenGL as well.
        let WgpuSetup::CreateNew(eframe_default) = NativeOptions::default().wgpu_options.wgpu_setup else {
            panic!("default native options must create a wgpu instance");
        };
        assert!(eframe_default.instance_descriptor.backends.contains(Backends::GL));
        let configured = configured_backends(None);
        assert_eq!(configured, Backends::DX12);
        assert!(!configured.contains(Backends::GL));
        assert!(!configured.contains(Backends::VULKAN));
    }

    #[test]
    fn explicit_backend_override_is_preserved() {
        for backends in [Backends::VULKAN, Backends::GL, Backends::DX12, Backends::VULKAN | Backends::DX12, Backends::empty()] {
            assert_eq!(configured_backends(Some(backends)), backends);
        }
    }
}
