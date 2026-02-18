use std::io::{self, Write};

/// REPL interativo para Snask
pub struct Repl {
    interpreter: crate::interpreter::Interpreter,
    history: Vec<String>,
}

impl Repl {
    pub fn new() -> Self {
        let mut interpreter = crate::interpreter::Interpreter::new();
        crate::stdlib::register_stdlib(interpreter.get_globals_mut());
        
        Repl {
            interpreter,
            history: Vec::new(),
        }
    }

    pub fn run(&mut self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║          Snask REPL v0.2.0 - Interactive Shell           ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
        println!();
        println!("Type 'exit' or 'quit' to leave");
        println!("Type 'help' to see available commands");
        println!("Type 'clear' to clear history");
        println!();

        loop {
            print!("snask> ");
            io::stdout().flush().unwrap();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                eprintln!("Failed to read input");
                continue;
            }

            let input = input.trim();

            if input.is_empty() {
                continue;
            }

            // Comandos especiais
            match input {
                "exit" | "quit" => {
                    println!("Bye!");
                    break;
                }
                "help" => {
                    self.show_help();
                    continue;
                }
                "clear" => {
                    self.history.clear();
                    println!("History cleared");
                    continue;
                }
                "history" => {
                    self.show_history();
                    continue;
                }
                _ => {}
            }

            // Adicionar ao histórico
            self.history.push(input.to_string());

            // Tentar executar
            self.execute(input);
        }
    }

    fn execute(&mut self, input: &str) {
        // Adicionar ponto e vírgula se não tiver
        let input_with_semicolon = if !input.ends_with(';') && !input.ends_with('}') {
            format!("{};", input)
        } else {
            input.to_string()
        };

        // Parse e execute
        match crate::parser::parse_program(&input_with_semicolon) {
            Ok(program) => {
                match self.interpreter.interpret(program) {
                    crate::interpreter::InterpretResult::Ok => {
                        // Sucesso silencioso
                    }
                    crate::interpreter::InterpretResult::RuntimeError(msg) => {
                        eprintln!("❌ Runtime error: {}", msg);
                    }
                }
            }
            Err(e) => {
                eprintln!("❌ Syntax error: {}", e);
            }
        }
    }

    fn show_help(&self) {
        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║                    Comandos do REPL                       ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  exit, quit    - Sair do REPL                             ║");
        println!("║  help          - Mostrar esta ajuda                       ║");
        println!("║  clear         - Limpar histórico                         ║");
        println!("║  history       - Mostrar histórico de comandos            ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║                        Code Examples                      ║");
        println!("╠═══════════════════════════════════════════════════════════╣");
        println!("║  let x = 10                                               ║");
        println!("║  print(\"Hello, Snask!\")                                   ║");
        println!("║  fun add(a, b) {{ return a + b; }}                        ║");
        println!("║  sqrt(16)                                                 ║");
        println!("║  range(10)                                                ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
    }

    fn show_history(&self) {
        if self.history.is_empty() {
            println!("(empty)");
            return;
        }

        println!("╔═══════════════════════════════════════════════════════════╗");
        println!("║                       Command History                     ║");
        println!("╚═══════════════════════════════════════════════════════════╝");
        
        for (i, cmd) in self.history.iter().enumerate() {
            println!("{:3}. {}", i + 1, cmd);
        }
    }
}
