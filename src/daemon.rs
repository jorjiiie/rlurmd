use crate::{protocol, runner};
use anyhow::{Context, Result, bail, ensure};
use serde::Deserialize;
use std::io::{ErrorKind, Read};
use std::path::{Path, PathBuf};

#[derive(Clone, Debug)]
/// Options for rlurmd daemon
pub struct Config {}

impl Config {
    pub fn parser() -> impl bpaf::Parser<Self> {
        bpaf::pure(Self {})
    }
}

pub fn run(_config: Config) {
    println!("lol i don't do anything yet");
}

#[derive(Deserialize)]
struct JobConfig {
    resources: Resources,
    job: JobSettings,
}

#[derive(Deserialize)]
struct Resources {
    cpus: usize,
    mem_gb: usize,
    gpu: bool,
}

#[derive(Deserialize)]
struct JobSettings {
    name: String,
    #[serde(default)]
    priority: i32,
}

#[derive(Clone)]
struct PreparedJob {
    name: String,
    priority: i32,
    cpus: usize,
    mem_gb: usize,
    gpu: bool,
}

struct Job {
    id: u64,
    dir: PathBuf,
    state: JobState,
}

enum JobState {
    Staging,
    Queued(PreparedJob),
    Cancelled(PreparedJob),
    Running {
        spec: PreparedJob,
        task: runner::Task,
    },
    Finished {
        spec: PreparedJob,
        exit_code: Option<i32>,
    },
}

impl Job {
    fn spec(&self) -> Option<&PreparedJob> {
        match &self.state {
            JobState::Staging => None,
            JobState::Queued(spec)
            | JobState::Cancelled(spec)
            | JobState::Finished { spec, .. } => Some(spec),
            JobState::Running { spec, .. } => Some(spec),
        }
    }

    fn to_info(&self) -> protocol::JobInfo {
        let status = match &self.state {
            JobState::Staging => protocol::JobStatus::Staging,
            JobState::Queued(_) => protocol::JobStatus::Queued,
            JobState::Cancelled(_) => protocol::JobStatus::Cancelled,
            JobState::Running { .. } => protocol::JobStatus::Running,
            JobState::Finished {
                exit_code: Some(0), ..
            } => protocol::JobStatus::Done,
            JobState::Finished { .. } => protocol::JobStatus::Failed,
        };
        let spec = self.spec();
        protocol::JobInfo {
            id: self.id,
            name: spec.map_or_else(|| self.id.to_string(), |spec| spec.name.clone()),
            priority: spec.map_or(0, |spec| spec.priority),
            status,
            cpus: spec.map(|spec| spec.cpus),
            memory_limit: spec.map(|spec| spec.mem_gb),
            gpu: spec.is_some_and(|spec| spec.gpu),
        }
    }
}

struct State {
    root_dir: PathBuf,
    jobs: Vec<Job>,
}

impl State {
    fn get_job(&mut self, reference: &str) -> Result<&mut Job> {
        let numeric_id = reference.parse::<u64>().ok();
        self.jobs
            .iter_mut()
            .find(|job| {
                Some(job.id) == numeric_id || job.spec().is_some_and(|spec| spec.name == reference)
            })
            .with_context(|| format!("job {reference:?} not found"))
    }

    pub fn dispatch(&mut self, request: protocol::Request) -> Result<protocol::Response> {
        match request {
            protocol::Request::Stage => self.stage(),
            protocol::Request::Submit(opt) => self.submit(opt),
            protocol::Request::Cancel(opt) => self.cancel(opt),
            protocol::Request::Ls => Ok(self.ls()),
        }
    }

    pub fn ls(&self) -> protocol::Response {
        protocol::Response::Ls(protocol::LsResponse {
            jobs: self.jobs.iter().map(Job::to_info).collect(),
        })
    }

    pub fn stage(&mut self) -> Result<protocol::Response> {
        let parent = self.root_dir.join("jobs");
        std::fs::create_dir_all(&parent).context("creating jobs dir")?;
        let id = fastrand::u64(0..);
        // Placeholder until readable names are implemented.
        let name = id.to_string();
        let dir = parent.join(&name);
        std::fs::create_dir(&dir)
            .with_context(|| format!("creating job directory {}", dir.display()))?;
        self.jobs.push(Job {
            id,
            dir: dir.clone(),
            state: JobState::Staging,
        });
        Ok(protocol::Response::Stage(protocol::StageResponse {
            id,
            name,
            dir,
        }))
    }

    pub fn submit(&mut self, req: protocol::SubmitJobOptions) -> Result<protocol::Response> {
        let job = self.get_job(&req.id)?;
        ensure!(
            matches!(job.state, JobState::Staging),
            "job {} is not staged",
            job.id
        );
        let prepared = prepare_job(&job.dir, &req)?;
        job.state = JobState::Queued(prepared);
        Ok(protocol::Response::Submit(protocol::SubmitResponse {}))
    }

    pub fn cancel(&mut self, req: protocol::CancelJobOptions) -> Result<protocol::Response> {
        let job = self.get_job(&req.id)?;
        let prepared = match &mut job.state {
            JobState::Queued(prepared) => prepared.clone(),
            JobState::Running { spec, task } => {
                // maybe async this
                task.cancel()?;
                spec.clone()
            }
            JobState::Cancelled(_) => return Ok(protocol::Response::Cancel),
            JobState::Staging => bail!("job {} is staged, not queued or running", job.id),
            JobState::Finished { .. } => bail!("job {} has already finished", job.id),
        };
        job.state = JobState::Cancelled(prepared);
        Ok(protocol::Response::Cancel)
    }

    pub fn new() -> Result<Self> {
        // TODO: cleanup this path dir thing
        let root_dir = PathBuf::from(
            std::env::var_os("STATE_DIRECTORY")
                .context("STATE_DIRECTORY environment variable not set")?,
        );
        std::fs::create_dir_all(&root_dir).context("creating scheduler state directory")?;
        Ok(Self {
            root_dir,
            jobs: Vec::new(),
        })
    }
}

fn prepare_job(dir: &Path, options: &protocol::SubmitJobOptions) -> Result<PreparedJob> {
    ensure!(
        options.cpus.is_none_or(|n| n > 0),
        "CPU count must be positive"
    );
    ensure!(
        options.memory_limit.is_none_or(|n| n > 0),
        "memory limit must be positive"
    );
    let mut contents = String::new();
    required_job_file(dir, "job.toml")?
        .read_to_string(&mut contents)
        .with_context(|| format!("could not read job.toml in {} as UTF-8 text", dir.display()))?;
    let config: JobConfig = toml::from_str(&contents)
        .map_err(|error| anyhow::anyhow!("invalid job.toml in {}:\n{error}", dir.display()))?;
    required_job_file(dir, "main.py")?;

    ensure!(
        config.resources.cpus > 0,
        "invalid job.toml: resources.cpus must be greater than zero"
    );
    ensure!(
        config.resources.mem_gb > 0,
        "invalid job.toml: resources.mem_gb must be greater than zero"
    );
    ensure!(
        !config.job.name.trim().is_empty(),
        "invalid job.toml: job.name must not be empty"
    );

    Ok(PreparedJob {
        name: config.job.name,
        priority: config.job.priority,
        cpus: options.cpus.unwrap_or(config.resources.cpus),
        mem_gb: options.memory_limit.unwrap_or(config.resources.mem_gb),
        gpu: config.resources.gpu,
    })
}

fn required_job_file(dir: &Path, name: &str) -> Result<std::fs::File> {
    let path = dir.join(name);
    let file = match std::fs::File::open(&path) {
        Ok(file) => file,
        Err(error) if error.kind() == ErrorKind::NotFound => {
            bail!(
                "{name} not found in {}; add it before submitting the job",
                dir.display()
            );
        }
        Err(error) => {
            return Err(error)
                .with_context(|| format!("could not open {name} in {}", dir.display()));
        }
    };
    ensure!(
        file.metadata()
            .with_context(|| format!("could not inspect {}", path.display()))?
            .is_file(),
        "{name} in {} must be a regular file",
        dir.display()
    );
    Ok(file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn prepared() -> PreparedJob {
        PreparedJob {
            name: "hold-gpu".into(),
            priority: 0,
            cpus: 1,
            mem_gb: 1,
            gpu: true,
        }
    }

    #[test]
    fn cancels_queued_jobs_by_name_or_id_and_keeps_them_visible() -> Result<()> {
        let mut state = State {
            root_dir: PathBuf::new(),
            jobs: vec![Job {
                id: 42,
                dir: PathBuf::from("jobs/42"),
                state: JobState::Queued(prepared()),
            }],
        };
        state.dispatch(protocol::Request::Cancel(protocol::CancelJobOptions {
            id: "hold-gpu".into(),
        }))?;
        assert!(matches!(state.jobs[0].state, JobState::Cancelled(_)));
        let protocol::Response::Ls(listed) = state.ls() else {
            unreachable!()
        };
        assert_eq!(listed.jobs[0].status, protocol::JobStatus::Cancelled);
        assert_eq!(listed.jobs[0].name, "hold-gpu");
        assert!(listed.jobs[0].gpu);
        // Repeated cancellation is harmless, including through a different reference.
        state.cancel(protocol::CancelJobOptions { id: "42".into() })?;
        assert!(
            state
                .cancel(protocol::CancelJobOptions {
                    id: "missing".into()
                })
                .is_err()
        );
        Ok(())
    }

    #[test]
    fn runner_stub_cannot_report_success_or_change_a_running_job_to_cancelled() {
        let mut state = State {
            root_dir: PathBuf::new(),
            jobs: vec![Job {
                id: 42,
                dir: PathBuf::from("jobs/42"),
                state: JobState::Running {
                    spec: prepared(),
                    task: runner::Task,
                },
            }],
        };
        let error = state
            .cancel(protocol::CancelJobOptions { id: "42".into() })
            .unwrap_err();
        assert!(error.to_string().contains("not implemented"));
        assert_eq!(state.jobs[0].to_info().status, protocol::JobStatus::Running);
        assert!(runner::Task::start(Path::new("jobs/42")).is_err());
        let JobState::Running { task, .. } = &mut state.jobs[0].state else {
            unreachable!()
        };
        assert!(task.check_exit().is_err());
    }
}
