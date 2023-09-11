use anyhow::Context;
use clap::Args;

use crate::{meta::parse_crate_metadata, nc::nc};

use super::{ConnectionArgs, Executor, TitleId};

#[derive(Args, Debug)]
pub struct Run {
    /// An alphanumeric string of 9 characters.
    #[arg(long, short = 'i', value_parser = clap::value_parser!(TitleId))]
    pub title_id: Option<TitleId>,
    #[command(flatten)]
    pub connection: ConnectionArgs,
}

impl Executor for Run {
    fn execute(&self, verbose: u8) -> anyhow::Result<()> {
        let title_id = match &self.title_id {
            Some(title_id) => title_id.clone(),
            None => parse_crate_metadata(None)?.0.title_id
            .context("Title id must either be provided by a flag or set in the `package.metadata.vita.title_id` field of your Cargo.toml")?,
        };

        let ip = &self.connection.vita_ip;
        let port = self.connection.cmd_port;

        nc(verbose, ip, port, "destroy")?;
        nc(verbose, ip, port, &format!("launch {title_id}"))?;

        Ok(())
    }
}
