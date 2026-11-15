use bpaf::{Parser, construct};

mod commands;
mod daemon;

#[derive(Clone, Debug)]
/// Options for rlurmd
enum RlurmdOptions {
    Submit(commands::SubmitJobOptions),
    Daemon(daemon::Config),
    Ls(commands::LsOptions),
}

impl RlurmdOptions {
    fn parser() -> impl Parser<Self> {
        let daemon = daemon::Config::parser()
            .map(RlurmdOptions::Daemon)
            .to_options()
            .descr("start the rlurmd daemon")
            .command("daemon")
            .help("start the rlurmd daemon");

        let submit = commands::SubmitJobOptions::parser()
            .map(RlurmdOptions::Submit)
            .to_options()
            .descr("submit a job and override resource limits if necessary")
            .command("submit")
            .help("submit a job to the rlurmd daemon");

        let ls = commands::LsOptions::parser()
            .map(RlurmdOptions::Ls)
            .to_options()
            .descr("list all jobs in the rlurmd daemon")
            .command("ls")
            .help("list all jobs in the rlurmd daemon");

        construct!([daemon, submit, ls])
    }
}
fn main() {
    let rlurmd_options = RlurmdOptions::parser().run();
    println!("rlurmd options {:?}", rlurmd_options);

    match rlurmd_options {
        RlurmdOptions::Daemon(config) => daemon::run(config),
        RlurmdOptions::Submit(options) => commands::submit_job(options),
        RlurmdOptions::Ls(options) => commands::ls(options),
    }
}
