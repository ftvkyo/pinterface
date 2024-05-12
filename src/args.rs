use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Use fast display
    #[arg(short, long)]
    pub fast: bool,

    /// Use grayscale
    #[arg(short, long)]
    pub grayscale: bool,

    /// If set, the program will restart after every successful execution
    #[arg(short, long)]
    pub repeat: bool,

    #[arg(value_enum, default_value_t)]
    pub command: Command,
}


#[derive(ValueEnum, Clone, Debug)]
pub enum Command {
    Clear,
    Debug,
    Tasks,
    Network,
    Calendar,
}

impl Default for Command {
    fn default() -> Self {
        Self::Clear
    }
}
