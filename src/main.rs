extern crate wustc;

use std::process::{Command, Stdio, exit};
use std::io::{stderr, stdout};
use std::{thread, env};


fn main() {
    let code = code_main();
    exit(code);
}

fn code_main() -> i32 {
    let mut rustc =
        Command::new("rustc").args(env::args_os().skip(1)).stdout(Stdio::piped()).stderr(Stdio::piped()).spawn().expect("Spawning rustc process failed");

    let stdout_thread = thread::spawn({
        let input = rustc.stdout.take().expect("stdout is captured");
        move || wustc::owo_copy(input, stdout())
    });
    let stderr_thread = thread::spawn({
        let input = rustc.stderr.take().expect("stderr is captured");
        move || wustc::owo_copy(input, stderr())
    });

    let status = rustc.wait().expect("rustc not running");
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    status.code().unwrap_or(0)
}
