//! Domain logic - pure business rules independent of git operations

pub mod branch;
pub mod commit;
pub mod prerelease;
pub mod tag;
pub mod version;

pub use branch::BranchContext;
pub use commit::ParsedCommit;
pub use prerelease::{PreRelease, PreReleaseType};
pub use tag::{Tag, TagPattern};
pub use version::{Version, VersionBump};
