use clap::Args;

use crate::nc::nc;

use super::{parse_crate_metadata, ConnectionArgs, TitleId};

#[derive(Args, Debug)]
pub struct Run {
    /// An alphanumeric string of 9 characters.
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    title_id: Option<TitleId>,
    #[command(flatten)]
    connection: ConnectionArgs,
}

impl Run {
    pub fn execute(&self, verbose: u8) {
        let ip = self.connection.get_vita_ip();
        let title_id = self.title_id.clone().or_else(|| parse_crate_metadata().title_id)
            .expect("Title id must either be provided by a flag or set in the `package.metadata.vita.title_id` field of your Cargo.toml");

        nc(verbose, &ip, self.connection.cmd_port, "destroy");
        nc(
            verbose,
            &ip,
            self.connection.cmd_port,
            &format!("launch {}", &title_id.0),
        );
    }
}
