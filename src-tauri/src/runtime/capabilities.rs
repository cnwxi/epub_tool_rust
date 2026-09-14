use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PlatformCapabilities {
    pub platform: &'static str,
    pub runtime: &'static str,
    pub supports_directory_picker: bool,
    pub supports_directory_scan: bool,
    pub supports_open_path: bool,
    pub requires_output_export: bool,
    pub supports_file_associations: bool,
    pub supports_font_ocr: bool,
}

impl PlatformCapabilities {
    pub fn current() -> Self {
        Self::for_platform(std::env::consts::OS)
    }

    fn for_platform(platform: &'static str) -> Self {
        let desktop = matches!(platform, "macos" | "windows" | "linux");
        Self {
            platform,
            runtime: "inProcess",
            supports_directory_picker: desktop,
            supports_directory_scan: desktop,
            supports_open_path: desktop,
            requires_output_export: platform == "android",
            supports_file_associations: desktop,
            supports_font_ocr: desktop,
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn android_capabilities_require_export_without_font_or_desktop_paths() {
        let capabilities = PlatformCapabilities::for_platform("android");
        assert_eq!(capabilities.platform, "android");
        assert_eq!(capabilities.runtime, "inProcess");
        assert!(capabilities.requires_output_export);
        assert!(!capabilities.supports_directory_picker);
        assert!(!capabilities.supports_directory_scan);
        assert!(!capabilities.supports_open_path);
        assert!(!capabilities.supports_font_ocr);
    }

    use super::PlatformCapabilities;

    #[test]
    fn desktop_platforms_preserve_path_and_font_capabilities() {
        for platform in ["macos", "windows", "linux"] {
            let capabilities = PlatformCapabilities::for_platform(platform);
            assert_eq!(capabilities.platform, platform);
            assert_eq!(capabilities.runtime, "inProcess");
            assert!(capabilities.supports_directory_picker);
            assert!(capabilities.supports_directory_scan);
            assert!(capabilities.supports_open_path);
            assert!(capabilities.supports_file_associations);
            assert!(!capabilities.requires_output_export);
            assert!(capabilities.supports_font_ocr);
        }
    }

    #[test]
    fn current_capabilities_report_the_compiled_target() {
        assert_eq!(
            PlatformCapabilities::current().platform,
            std::env::consts::OS
        );
    }
}
