use runtime::Runtime;
#[global_allocator]
static ALLOC: dhat::Alloc = dhat::Alloc;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    // let _profiler = dhat::Profiler::new_heap();
    
    let mut runtime = Runtime::default();
    
    runtime.start();
}
