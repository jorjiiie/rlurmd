use bpaf::{Parser, construct};
use std::path::PathBuf;

#[derive(Clone, Debug)]
/// Options for rlumd job submission
pub struct SubmitJobOptions {
    /// CPU threads to use
    cpus: Option<usize>,

    /// Memory to use in GB
    memory_limit: Option<usize>,

    /// job to run
    job_path: PathBuf,
}

impl SubmitJobOptions {
    pub fn parser() -> impl Parser<Self> {
        let cpus = bpaf::long("cpus")
            .help("CPU threads to use (override)")
            .argument::<usize>("CPUS")
            .optional();
        let memory_limit = bpaf::long("memory-limit")
            .help("Memory to use in GB (override)")
            .argument::<usize>("MEMORY_LIMIT")
            .optional();
        let job_path = bpaf::positional::<PathBuf>("JOB").help("job to run");

        construct!(Self {
            cpus,
            memory_limit,
            job_path
        })
    }
}

pub fn submit_job(options: SubmitJobOptions) {
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
