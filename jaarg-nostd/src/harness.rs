/* jaarg-nostd - Minimal harness to run examples in no_std on desktop
 * SPDX-FileCopyrightText: (C) 2025 Gay Pizza Specifications
 * SPDX-License-Identifier: MIT OR Apache-2.0
 */

//!! [Okay... ready for the pain?](https://media.tenor.com/cJRcMyUAiMcAAAAC/tenor.gif)

use core::alloc::{GlobalAlloc, Layout};
use core::fmt::Write;
#[allow(unused_imports)]
use core::panic::PanicInfo;

/// Unix file descriptor
pub struct FileDescriptor(core::ffi::c_int);
#[allow(unused)]
impl FileDescriptor {
  /// Standard input file descriptor
  const STDIN: Self = Self(0);
  /// Standard output file descriptor
  const STDOUT: Self = Self(1);
  /// Standard error file descriptor
  const STDERR: Self = Self(2);
}

pub struct StandardOutWriter;
impl Write for StandardOutWriter {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    unsafe {
      c::write(FileDescriptor::STDOUT.0, s.as_ptr() as *const core::ffi::c_void, s.len());
    }
    Ok(())
  }
}

pub fn print(args: core::fmt::Arguments) {
  StandardOutWriter{}.write_fmt(args).unwrap();
}

pub struct StandardErrorWriter;
impl Write for StandardErrorWriter {
  fn write_str(&mut self, s: &str) -> core::fmt::Result {
    unsafe {
      c::write(FileDescriptor::STDERR.0, s.as_ptr() as *const core::ffi::c_void, s.len());
    }
    Ok(())
  }
}

pub fn eprint(args: core::fmt::Arguments) {
  StandardErrorWriter{}.write_fmt(args).unwrap();
}

#[macro_export]
macro_rules! print {
  ($($arg:tt)*) => {{ $crate::harness::print(format_args!($($arg)*)); }};
}

#[macro_export]
macro_rules! eprint {
  ($($arg:tt)*) => {{ $crate::harness::eprint(format_args!($($arg)*)); }};
}

#[macro_export]
macro_rules! println {
  () => {{ $crate::print!("\n"); }};
  ($($arg:tt)*) => {{ $crate::print!("{}\n", format_args!($($arg)*)); }};
}

#[macro_export]
macro_rules! eprintln {
  () => {{ $crate::eprint!("\n"); }};
  ($($arg:tt)*) => {{ $crate::eprint!("{}\n", format_args!($($arg)*)); }};
}

/// Calls system abort
pub fn exit(status: i32) -> ! {
  unsafe { c::exit(status as core::ffi::c_int) }
}

/// Bare minimum malloc-based global allocator
#[derive(Default)]
pub struct MallocAlloc;
impl MallocAlloc {
  // Fundamental alignment table:
  // | Target  | 32-bit | 64-bit | Note                        |
  // |---------|--------|--------|-----------------------------|
  // | macOS   |     16 |     16 | Always 16-byte aligned      |
  // | GNU     |      8 |     16 |                             |
  // | Windows |      8 |     16 | nonstd aligned_alloc & free |
  // | OpenBSD |     16 |     16 | FIXME: Unsourced            |
  // [Darwin source](https://developer.apple.com/library/archive/documentation/Performance/Conceptual/ManagingMemory/Articles/MemoryAlloc.html)
  // [GNU glibc source](https://sourceware.org/glibc/manual/2.42/html_node/Malloc-Examples.html)
  // [Windows source](https://learn.microsoft.com/en-us/cpp/c-runtime-library/reference/malloc?view=msvc-170)

  // if ptr == 32 && !(os == "macos" || os == "openbsd")
  #[cfg(all(target_pointer_width = "32", not(any(target_os = "macos", target_os = "openbsd"))))]
  const ALIGNMENT: Option<usize> = Some(8);
  // if ptr == 64 || (ptr == 32 && (os == "macos" || os == "openbsd"))
  #[cfg(any(target_pointer_width = "64", all(target_pointer_width = "32", any(target_os = "macos", target_os = "openbsd"))))]
  const ALIGNMENT: Option<usize> = Some(16);
  // if !(ptr == 32 || ptr == 64)
  #[cfg(not(any(target_pointer_width = "32", target_pointer_width = "64")))]
  const ALIGNMENT: Option<usize> = None;

  /// If target alignment % requested_align == 0 then malloc is good enough.
  #[inline(always)]
  const fn layout_can_use_malloc(requested_layout: &Layout) -> bool {
    let align = requested_layout.align();
    align != 0 && matches!(Self::ALIGNMENT,
      Some(sys_align) if (sys_align & (align - 1)) == 0)
  }
}
unsafe impl GlobalAlloc for MallocAlloc {
  unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
    if Self::layout_can_use_malloc(&layout) {
      c::malloc(layout.size()).cast::<u8>()
    } else {
      #[cfg(target_family = "windows")]
      return c::aligned_alloc(layout.size(), layout.align()).cast::<u8>();
      #[cfg(not(target_family = "windows"))]
      return c::aligned_alloc(layout.align(), layout.size()).cast::<u8>();
    }
  }
  #[allow(unused_variables)]
  unsafe fn dealloc(&self, ptr: *mut u8, layout: Layout) {
    #[cfg(target_family = "windows")]
    if !Self::layout_can_use_malloc(&layout) {
      c::aligned_free(ptr as *mut core::ffi::c_void);
      return;
    }
    c::free(ptr as *mut core::ffi::c_void);
  }
}

#[global_allocator]
static GLOBAL_ALLOCATOR: MallocAlloc = MallocAlloc;

/// Hurt me plenty (intellidumb will think this a lang duplicate pls ignore)
#[cfg(not(test))]
#[panic_handler]
unsafe fn panic(info: &PanicInfo) -> ! {
  eprintln!("panic abort: {}", info.message());
  c::abort()
}

/// Ultra-Violence
#[cfg(not(test))]
#[no_mangle]
pub extern "C" fn rust_eh_personality() {}

/// Nightmare!
#[cfg(not(test))]
#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn _Unwind_Resume() {}

extern "C" {
  #[allow(improper_ctypes)]
  pub fn safe_main(args: &[&str]) -> ExitCode;
}

/// Exit code to be passed to entry point wrapper.
#[allow(non_camel_case_types)]
#[repr(i32)]
pub enum ExitCode {
  SUCCESS = 0,
  FAILURE = 1,
}

/// C main entry point, collects argc/argv and calls `safe_main`.
#[cfg(not(test))]
#[no_mangle]
pub unsafe extern "C" fn main(argc: core::ffi::c_int, argv: *const *const core::ffi::c_char) -> core::ffi::c_int {
  let mut args = alloc::vec::Vec::<&str>::with_capacity(argc as usize);
  for i in 0..argc as usize {
    args.push(core::ffi::CStr::from_ptr(*argv.wrapping_add(i)).to_str().unwrap());
  }
  safe_main(&args) as core::ffi::c_int
}

mod c {
  use core::ffi::{c_int, c_void};

  /// Until size_t is stabilised
  #[allow(non_camel_case_types)]
  type c_size_t = usize;

  #[allow(dead_code)]
  extern "C" {
    pub(crate) fn atexit(function: extern "C" fn()) -> c_int;
    pub(crate) fn abort() -> !;
    pub(crate) fn exit(status: c_int) -> !;
    #[cfg(not(target_family = "windows"))]
    pub(crate) fn write(fd: c_int, buf: *const c_void, bytes: c_size_t) -> c_int;
    #[cfg(target_family = "windows")]
    #[link_name = "_write"]
    pub(crate) fn write(fd: c_int, buf: *const c_void, bytes: c_size_t) -> c_int;
    pub(crate) fn malloc(size: c_size_t) -> *mut c_void;
    pub(crate) fn calloc(count: c_size_t, size: c_size_t) -> *mut c_void;
    #[cfg(not(target_family = "windows"))]
    pub(crate) fn aligned_alloc(alignment: c_size_t, size: c_size_t) -> *mut c_void;
    #[cfg(target_family = "windows")]
    #[link_name = "_aligned_malloc"]
    pub(crate) fn aligned_alloc(size: c_size_t, alignment: c_size_t) -> *mut c_void;
    #[cfg(target_family = "windows")]
    #[link_name = "_aligned_free"]
    pub(crate) fn aligned_free(memblock: *mut c_void);
    pub(crate) fn free(ptr: *mut c_void);
  }
}
