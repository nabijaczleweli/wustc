use std::io::{Result as IoResult, Write, Read, stderr, stdout};
use std::process::{Command, Stdio, exit};
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
        move || output_child(input, stdout())
    });
    let stderr_thread = thread::spawn({
        let input = rustc.stderr.take().expect("stderr is captured");
        move || output_child(input, stderr())
    });

    let status = rustc.wait().expect("rustc not running");
    let _ = stdout_thread.join();
    let _ = stderr_thread.join();

    status.code().unwrap_or(0)
}



static FACES: &[&str] = &["(・`ω´・)", "OwO", "owo", "oωo", "òωó", "°ω°", "UwU", ">w<", "^w^"];

#[derive(Debug, Copy, Clone, Hash, PartialOrd, Ord, PartialEq, Eq)]
enum State {
    None,

    HaveSmallN,
    HaveBigN,

    HaveOveO,
    HaveOveV,

    HaveExclamationMarks,
}

/// Adapted from https://github.com/EvilDeaaaadd/owoify/blob/fee2bdb05013b48d39c2e5f6ed76ade769bec2f8/src/lib.rs
fn output_child<R: Read, W: Write>(from: R, mut to: W) -> IoResult<()> {
    let face_step = (&from as *const R as usize) % FACES.len();

    let mut cur_face_idx = (&to as *const W as usize) % FACES.len();
    let mut cur_state = State::None;

    for b in from.bytes() {
        handle_byte(b?, &mut to, face_step, &mut cur_face_idx, &mut cur_state)?;
    }

    Ok(())
}

fn handle_byte<W: Write>(b: u8, to: &mut W, face_step: usize, cur_face_idx: &mut usize, cur_state: &mut State) -> IoResult<()> {
    match (*cur_state, b) {
        (State::None, c) if c == b'r' || c == b'l' => to.write_all(b"w")?,
        (State::None, c) if c == b'R' || c == b'L' => to.write_all(b"W")?,

        (State::None, b'n') => *cur_state = State::HaveSmallN,
        (State::HaveSmallN, c) if c == b'a' || c == b'e' || c == b'i' || c == b'o' || c == b'u' => to.write_all(b"ny").and_then(|_| to.write_all(&[c]))?,
        (State::HaveSmallN, c) => {
            *cur_state = State::None;
            to.write_all(&[b'n']).and_then(|_| handle_byte(c, to, face_step, cur_face_idx, cur_state))?
        }

        (State::None, b'N') => *cur_state = State::HaveBigN,
        (State::HaveBigN, c) if c == b'a' || c == b'e' || c == b'i' || c == b'o' || c == b'u' => to.write_all(b"Ny").and_then(|_| to.write_all(&[c]))?,
        (State::HaveBigN, c) if c == b'A' || c == b'E' || c == b'I' || c == b'O' || c == b'U' => to.write_all(b"NY").and_then(|_| to.write_all(&[c]))?,
        (State::HaveBigN, c) => {
            *cur_state = State::None;
            to.write_all(b"N").and_then(|_| handle_byte(c, to, face_step, cur_face_idx, cur_state))?
        }

        (State::None, b'o') => *cur_state = State::HaveOveO,
        (State::HaveOveO, b'v') => *cur_state = State::HaveOveV,
        (State::HaveOveO, c) => {
            *cur_state = State::None;
            to.write_all(b"o").and_then(|_| handle_byte(c, to, face_step, cur_face_idx, cur_state))?
        }
        (State::HaveOveV, b'e') => {
            *cur_state = State::None;
            to.write_all(b"uv")?
        }
        (State::HaveOveV, c) => {
            *cur_state = State::None;
            to.write_all(b"ov").and_then(|_| handle_byte(c, to, face_step, cur_face_idx, cur_state))?
        }

        (s, b'!') if s == State::None || s == State::HaveExclamationMarks => *cur_state = State::HaveExclamationMarks,
        (State::HaveExclamationMarks, c) => {
            *cur_state = State::None;
            to.write_all(b" ")
                .and_then(|_| to.write_all(FACES[*cur_face_idx].as_bytes()))
                .and_then(|_| to.write_all(b" "))
                .and_then(|_| handle_byte(c, to, face_step, cur_face_idx, cur_state))?;
            *cur_face_idx = (*cur_face_idx + face_step) % FACES.len();
        }

        (State::None, c) => to.write_all(&[c])?,
    }

    Ok(())
}
