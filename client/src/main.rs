use runtime::Runtime;

fn main() {
    unsafe {
        std::env::set_var("RUST_BACKTRACE", "1");
    }

    let mut runtime = Runtime::default();

    runtime.start();
}
