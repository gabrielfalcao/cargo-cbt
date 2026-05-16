#[doc(hidden)]
pub mod cli;
#[doc(inline)]
pub use cli::{go, Cli};

#[doc(hidden)]
pub mod errors;
#[doc(inline)]
pub use errors::{Error, Result};

#[doc(hidden)]
pub mod sh;
#[doc(inline)]
pub use sh::shell_command;

#[doc(hidden)]
pub mod manifest;
#[doc(inline)]
pub use manifest::{ExecutableAsset, Manifest, ManifestData, Package};
