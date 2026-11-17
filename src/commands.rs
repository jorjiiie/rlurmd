use crate::protocol;
use bpaf::{Parser, construct};

impl protocol::SubmitJobOptions {
    pub fn parser() -> impl Parser<Self> {
        let cpus = bpaf::long("cpus")
            .help("CPU threads to use (override)")
            .argument::<usize>("CPUS")
            .optional();
        let memory_limit = bpaf::long("memory-limit")
            .help("Memory to use in GB (override)")
            .argument::<usize>("MEMORY_LIMIT")
            .optional();
        let id = bpaf::positional::<String>("JOB").help("Staged job to submit (ID or name)");

        construct!(Self {
            cpus,
            memory_limit,
            id
        })
    }
}

pub fn submit_job(options: protocol::SubmitJobOptions) {
    println!("Submitting job with options: {:?}", options);
}

#[derive(Clone, Debug)]
/// Options for ls options
pub struct LsOptions {
    // probbaly a for agents/for humans wrt readable output/token efficiency
}

impl LsOptions {
    pub fn parser() -> impl Parser<Self> {
        construct!(Self {})
    }
}

pub fn ls(options: LsOptions) {
    println!("ls or something like thag");
}

impl protocol::CancelJobOptions {
    pub fn parser() -> impl Parser<Self> {
        let id = bpaf::positional::<String>("JOB").help("Job to cancel (ID or name)");
        construct!(Self { id })
    }
}

pub fn cancel_job(options: protocol::CancelJobOptions) {
    // TODO: send the request over the daemon connection.
    let request = protocol::Request::Cancel(options);
    println!("Cancel request: {request:?}");
}
