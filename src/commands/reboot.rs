use clap::Args;

use crate::nc::nc;

use super::{ConnectionArgs, Executor};

#[derive(Args, Debug)]
pub struct Reboot {
    #[command(flatten)]
    connection: ConnectionArgs,
}

impl Executor for Reboot {
    fn execute(&self) -> anyhow::Result<()> {
        let ip = &self.connection.vita_ip;
        let port = self.connection.cmd_port;
        nc(ip, port, "reboot")?;

        Ok(())
    }
}
