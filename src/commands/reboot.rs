use clap::Args;

use crate::nc;

use super::ConnectionArgs;

#[derive(Args, Debug)]
pub struct Reboot {
    #[command(flatten)]
    connection: ConnectionArgs,
}

impl Reboot {
    pub fn execute(&self, verbose: u8) {
        let ip = self.connection.get_vita_ip();
        nc::nc(verbose, &ip, self.connection.cmd_port, "reboot");
    }
}
