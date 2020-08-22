use std::{
    env::args,
    io::{stdout, Write},
};

mod modules;

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    if args().count() <= 1 {
        let _ = write!(stdout, "{}", modules::Ps1);
    } else {
        let _ = write!(stdout, "{}", modules::Ps2);
    }
}
