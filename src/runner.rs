use anyhow::{Result, bail};
use std::path::Path;
use std::process::ExitStatus;

pub struct Task;

impl Task {
    pub fn start(_dir: &Path) -> Result<Self> {
        bail!("starting tasks is not implemented yet")
    }

    pub fn cancel(&mut self) -> Result<()> {
        bail!("cancelling running tasks is not implemented yet")
    }

    pub fn check_exit(&mut self) -> Result<Option<ExitStatus>> {
        bail!("checking task exit is not implemented yet")
    }
}
