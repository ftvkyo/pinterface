use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// If set, the program will restart after every successful execution
    #[arg(short, long)]
    pub repeat: bool,

    #[arg(value_enum, default_value_t)]
    pub mode: Mode,
}


#[derive(ValueEnum, Clone, Debug)]
pub enum Mode {
    Clear,
    Network,
    Calendar,
}

impl Default for Mode {
    fn default() -> Self {
        Self::Clear
    }
}
