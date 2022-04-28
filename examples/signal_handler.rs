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
