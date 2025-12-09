use clap::Parser;

#[derive(Debug, Parser)]
pub struct Args {
    /// Name to greet
    #[arg(conflicts_with = "uppercase", value_name = "NAME")]
    pub name: Option<String>,

    /// Sets name to greet to uppercase
    #[arg(short, long, value_name = "UPPERCASE_NAME")]
    pub uppercase: Option<String>,
}
