use clap::Args;

use crate::nc::nc;

use super::{ConnectionArgs, Executor};

#[derive(Args, Debug)]
pub struct Reboot {
    #[command(flatten)]
    connection: ConnectionArgs,
}

impl Executor for Reboot {
    fn execute(&self, verbose: u8) -> anyhow::Result<()> {
        let ip = &self.connection.vita_ip;
        let port = self.connection.cmd_port;
        nc(verbose, ip, port, "reboot")?;

        Ok(())
    }
}
