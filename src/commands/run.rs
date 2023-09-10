use clap::Args;

use crate::{meta::parse_crate_metadata, nc::nc};

use super::{ConnectionArgs, Executor, TitleId};

#[derive(Args, Debug)]
pub struct Run {
    /// An alphanumeric string of 9 characters.
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    title_id: Option<TitleId>,
    #[command(flatten)]
    connection: ConnectionArgs,
}

impl Executor for Run {
    fn execute(&self, verbose: u8) {
        let title_id = self.title_id.clone().or_else(|| parse_crate_metadata().title_id)
            .expect("Title id must either be provided by a flag or set in the `package.metadata.vita.title_id` field of your Cargo.toml");

        let ip = &self.connection.vita_ip;
        let port = self.connection.cmd_port;

        nc(verbose, ip, port, "destroy");
        nc(verbose, ip, port, &format!("launch {}", &title_id.0));
    }
}
