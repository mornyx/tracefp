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
