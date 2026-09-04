// SPDX-License-Identifier: MIT OR Apache-2.0
//
// Architecture-specific memory backends. Each supported architecture owns its
// register/asm-level MMU implementation under its own submodule; the rest of
// the memory subsystem (page_allocator, mod.rs) only depends on the
// re-exported `mmu` API below and stays arch-agnostic.

#[cfg(target_arch = "aarch64")]
pub mod aarch64;

#[cfg(target_arch = "aarch64")]
pub use aarch64::mmu;
