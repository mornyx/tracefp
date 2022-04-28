fn main() {
    let pcs = func1_inlined();
    for pc in pcs {
        println!("{:#x}", pc);
        backtrace::resolve(pc as _, |s| {
            println!("    {:?}", s.name());
        });
    }
}

#[inline(always)]
fn func1_inlined() -> Vec<u64> {
    func2()
}

fn func2() -> Vec<u64> {
    let mut pcs = vec![];
    tracefp::trace(|pc| {
        pcs.push(pc);
        true
    });
    pcs
}
