use std::env;
use std::io::{self, Write};
use std::thread;
use std::time::Duration;

pub fn run_fetch() {
    let snask_version = env!("CARGO_PKG_VERSION");
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;

    // Representação artística da logo Snask
    let s_art = [
        "       .d88888b.  ",
        "     d88P'  `88b  ",
        "     888      `Y  ",
        "     `8888b.      ",
        "        `\"Y88b.   ",
        "      db    `888  ",
        "     888.  .d88P  ",
        "      Y888888P'   ",
        "         `        ",
    ];

    print!("\x1B[2J");

    // Animação de cores (Ciano para Verde Água)
    for i in 0..15 {
        let color = 36 + (i % 2);
        print!("\x1B[1;1H");
        for line in &s_art {
            println!("\x1b[1;{}m{}\x1b[0m", color, line);
        }
        println!(
            "\n  \x1b[1;37mSnask v{} | O futuro é nosso! 🚀\x1b[0m",
            snask_version
        );
        println!("  ---------------------------------------");
        println!("  OS:      {}", os);
        println!("  Arch:    {}", arch);
        println!("  Runtime: Native (LLVM/C)");

        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(200));
    }
}
