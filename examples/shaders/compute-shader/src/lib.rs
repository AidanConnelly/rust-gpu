#![cfg_attr(
    target_arch = "spirv",
    feature(register_attr),
    register_attr(spirv),
    no_std
)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

extern crate spirv_std;

use glam::UVec3;
use spirv_std::glam;
#[cfg(not(target_arch = "spirv"))]
use spirv_std::macros::spirv;

// Adapted from the wgpu hello-compute example

pub fn collatz(n: u32) -> Option<u32> {
    if n == 1{
        return Some(n)
    }
    else if n % 2 == 0 {
            collatz(n / 2)
        }
    else {
            // Overflow? (i.e. 3*n + 1 > 0xffff_ffff)
            if n >= 0x5555_5555 {
                return None;
            }
            // TODO: Use this instead when/if checked add/mul can work: n.checked_mul(3)?.checked_add(1)?
            collatz(3 * n + 1)
    }
}

// LocalSize/numthreads of (x = 64, y = 1, z = 1)
#[spirv(compute(threads(64)))]
pub fn main_cs(
    #[spirv(global_invocation_id)] id: UVec3,
    #[spirv(storage_buffer, descriptor_set = 0, binding = 0)] prime_indices: &mut [u32],
) {
    let index = id.x as usize;
    prime_indices[index] = unwrap_or_max(collatz(prime_indices[index]));
}

// Work around https://github.com/EmbarkStudios/rust-gpu/issues/677
fn unwrap_or_max(option: Option<u32>) -> u32 {
    match option {
        Some(inner) => inner,
        None => u32::MAX,
    }
}
