pub enum DebuggerCommand {
    Quit,
    Cont,
    Back,
    Break(String),
    Info(String),
    Run(Vec<String>),
}

impl DebuggerCommand {
    pub fn from_tokens(tokens: &Vec<&str>) -> Option<DebuggerCommand> {
        match tokens[0] {
            "q" | "quit" => Some(DebuggerCommand::Quit),
            "r" | "run" => {
                let args = tokens[1..].to_vec();
                Some(DebuggerCommand::Run(
                    args.iter().map(|s| s.to_string()).collect(),
                ))
            }
            "c" | "cont" | "continue" => Some(DebuggerCommand::Cont),

            "bt" | "back" | "backtrace" => Some(DebuggerCommand::Back),

            "b" | "break" => {
                let arg = tokens[1].to_string();
                Some(DebuggerCommand::Break(arg))
            }

            "info" => {
                let arg = tokens[1].to_string();
                Some(DebuggerCommand::Info(arg))
            }
            // Default case:
            _ => None,
        }
    }
}
