use std::env;
use std::thread;
use std::time::Duration;
use std::io::{self, Write};

pub fn run_fetch() {
    let os = std::env::consts::OS;
    let arch = std::env::consts::ARCH;
    let snask_version = env!("CARGO_PKG_VERSION");

    let s_art = r#"
        ██████████
      ███        ███
     ███          ██
     █████          
       ███████      
              ███   
     ██        ███  
      ███      ███  
        █████████   
    "#;

    // Efeito de pulsação e brilho simples
    for i in 0..8 {
        print!("\x1B[2J\x1B[1;1H");
        let intensity = if i % 2 == 0 { "\x1b[36m" } else { "\x1b[34m" };
        println!("{}{}", intensity, s_art);
        println!("\n  \x1b[1;37mSnask O futuro é nosso! 🚀\x1b[0m");
        println!("  --------------------------");
        println!("  Version: v{}", snask_version);
        println!("  OS:      {}", os);
        println!("  Arch:    {}", arch);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(300));
    }
}
