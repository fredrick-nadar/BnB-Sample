pub mod mock;
pub mod traits;

#[cfg(windows)]
pub mod windows;

pub use mock::MockPlatform;
pub use traits::*;

#[cfg(windows)]
pub use windows::WindowsPlatform;

/// Create the default platform adapter for the current runtime.
pub fn create_default_platform() -> Box<dyn PlatformAdapter> {
    #[cfg(windows)]
    {
        Box::new(WindowsPlatform::new())
    }
    #[cfg(not(windows))]
    {
        Box::new(MockPlatform::new())
    }
}
