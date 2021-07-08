// pub mod render_old;
pub mod geometry;
pub mod core;
pub mod nn;
pub use nalgebra as na;
pub use nalgebra_glm as glm;


#[cfg(feature = "global_mimalloc")]
use mimalloc::MiMalloc;

#[cfg(feature = "global_mimalloc")]
#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;


pub mod util {
    pub fn round_next_pow2(x:u32)->u32 {
        let mut i = 1;
        while i < x {
            i *= 2;
        }
        i
    }
}