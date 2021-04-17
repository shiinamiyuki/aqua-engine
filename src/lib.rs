pub mod render;
pub mod geometry;
pub mod core;
pub use nalgebra as na;
pub use nalgebra_glm as glm;


#[cfg(feature = "global_mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "global_mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;