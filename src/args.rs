use clap::{Parser, ValueEnum};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    /// Restart after every successful execution
    #[arg(short, long)]
    pub repeat: bool,

    #[arg(short, long, value_enum, default_value_t)]
    pub mode: DisplayMode,

    #[arg(value_enum, default_value_t)]
    pub command: Command,
}


#[derive(ValueEnum, Clone, Copy, Debug)]
pub enum DisplayMode {
    Full,
    Fast,
    Grey,
}

impl Default for DisplayMode {
    fn default() -> Self {
        Self::Full
    }
}

impl std::fmt::Display for DisplayMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let s = match self {
            Self::Full => "🐢",
            Self::Fast => "🐇",
            Self::Grey => "🌻",
        };
        write!(f, "{}", s)
    }
}


#[derive(ValueEnum, Clone, Copy, Debug)]
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
