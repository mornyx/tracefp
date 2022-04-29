# tracefp

A stack backtracking library based on frame-pointer.

# Requirements

When compiling your project, set the following environment variables:

- `CFLAGS` += `-fno-omit-frame-pointer -mno-omit-leaf-frame-pointer`
- `CXXFLAGS` += `-fno-omit-frame-pointer -mno-omit-leaf-frame-pointer`
- `RUSTFLAGS` += `-Cforce-frame-pointers=yes`

And add the following parameters to `cargo build`:

```shell
rustup component add rust-src
cargo build -Z build-std --target x86_64-unknown-linux-gnu
```

Where `x86_64-unknown-linux-gnu` can be replaced with other values, and `build-std` currently only supports nightly Rust. 

> NOTE: When you're using this library on **macOS + aarch64**, you don't need to do anything, as all libraries for this platform turn on frame-pointer by default.

# Install

```toml
[dependencies]
tracefp = "0.0.1"
```

Or you want to turn off memory access checking (not recommended, this may lead to segfaults when the frame pointer does not exist):

```toml
[dependencies]
tracefp = { version = "0.0.1", default-features = false }
```

# Examples

## Stack backtrace

```rust
fn main() {
    func1_inlined();
}

#[inline(always)]
fn func1_inlined() {
    func2()
}

fn func2() {
    tracefp::trace(|pc| {
        println!("{:#x}", pc);
        backtrace::resolve(pc as _, |s| {
            println!("    {:?}", s.name());
        });
        true
    });
}
```

Sample output:

```text
0x0
0x100d7348b
    Some(hello::func2::h53002ef4ebe4d7d7)
0x100d73337
    Some(hello::func1_inlined::h30751d2ee2774466)
    Some(hello::main::h994e0b3179971102)
0x100d72ddf
    Some(core::ops::function::FnOnce::call_once::h3dec9d79421d8d27)
0x100d71723
    Some(std::sys_common::backtrace::__rust_begin_short_backtrace::h9755a7454510e50f)
0x100d716db
    Some(std::rt::lang_start::{{closure}}::ha86392d061932837)
0x100e4065f
    Some(core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once::h8eb3ac20f80eabfa)
    Some(std::panicking::try::do_call::ha6ddf2c638427188)
    Some(std::panicking::try::hda8741de507c1ad0)
    Some(std::panic::catch_unwind::h82424a01f258bd39)
    Some(std::rt::lang_start_internal::{{closure}}::h67e296ed5b030b7b)
    Some(std::panicking::try::do_call::hd3dd7e7e10f6424e)
    Some(std::panicking::try::ha0a7bd8122e3fb7c)
    Some(std::panic::catch_unwind::h809b0e1092e9475d)
    Some(std::rt::lang_start_internal::h358b6d58e23c88c7)
0x100d716a3
    Some(std::rt::lang_start::h1342399ebba7a37d)
0x100d734df
    Some("_main")
0x101059087
0xd82e7fffffffffff
```

## Stack backtrace in signal handler

```rust
use nix::sys::signal::{sigaction, SaFlags, SigAction, SigHandler, SigSet, SIGPROF};

fn main() {
    // Register perf signal handler.
    let h = SigHandler::SigAction(perf_signal_handler);
    let a = SigAction::new(h, SaFlags::SA_SIGINFO, SigSet::empty());
    unsafe {
        sigaction(SIGPROF, &a).unwrap();
    }

    // Send a SIGPROF signal to the current process.
    unsafe {
        libc::kill(libc::getpid(), libc::SIGPROF);
    }

    // Block until the signal handler finishes executing.
    loop {}
}

#[no_mangle]
pub extern "C" fn perf_signal_handler(_: libc::c_int, _: *mut libc::siginfo_t, ucontext: *mut libc::c_void) {
    tracefp::trace_from_ucontext(ucontext, |pc| {
        println!("{:#x}", pc);
        backtrace::resolve(pc as _, |s| {
            println!("    {:?}", s.name());
        });
        true
    });
    std::process::exit(0);
}
```

Sample output:

```text
0x1c32e4824
    Some("_thread_get_state")
0x100409093
    Some(core::ops::function::FnOnce::call_once::h775fb44fbbe53d95)
0x1004090eb
    Some(std::sys_common::backtrace::__rust_begin_short_backtrace::h3acd0b11747c5033)
0x100408a2b
    Some(std::rt::lang_start::{{closure}}::hf7b77a4d60d2f840)
0x1004d7faf
    Some(core::ops::function::impls::<impl core::ops::function::FnOnce<A> for &F>::call_once::h8eb3ac20f80eabfa)
    Some(std::panicking::try::do_call::ha6ddf2c638427188)
    Some(std::panicking::try::hda8741de507c1ad0)
    Some(std::panic::catch_unwind::h82424a01f258bd39)
    Some(std::rt::lang_start_internal::{{closure}}::h67e296ed5b030b7b)
    Some(std::panicking::try::do_call::hd3dd7e7e10f6424e)
    Some(std::panicking::try::ha0a7bd8122e3fb7c)
    Some(std::panic::catch_unwind::h809b0e1092e9475d)
    Some(std::rt::lang_start_internal::h358b6d58e23c88c7)
0x1004089f3
    Some(std::rt::lang_start::hd321e36029dcfdd2)
0x1004093d7
    Some("_main")
0x1007bd087
0x921a7fffffffffff
```
