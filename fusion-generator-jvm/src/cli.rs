use clap::{AppSettings, Clap};

#[derive(Clap)]
#[clap(version = "0.1", author = "Vaadin Ltd.")]
#[clap(setting = AppSettings::ColoredHelp)]
pub struct CLI {
  input: String,
  #[clap(short, long, default_value = "./frontend/generated")]
  output: String,
  #[clap(short, long, default_value = "deps")]
  deps: String,
}
