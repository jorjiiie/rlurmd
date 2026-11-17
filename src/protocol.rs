use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
/// Options for rlumd job submission
pub struct SubmitJobOptions {
    /// CPU threads to use
    pub cpus: Option<usize>,

    /// Memory to use in GB
    pub memory_limit: Option<usize>,

    /// Reference to a staged job: ID or name.
    pub id: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CancelJobOptions {
    /// Reference to a queued or running job: ID or name.
    pub id: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Request {
    Stage,
    Submit(SubmitJobOptions),
    Cancel(CancelJobOptions),
    Ls,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Response {
    Stage(StageResponse),
    Submit(SubmitResponse),
    Cancel,
    Ls(LsResponse),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitResponse {}

#[derive(Debug, Serialize, Deserialize)]
pub struct LsResponse {
    pub jobs: Vec<JobInfo>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct JobInfo {
    pub id: u64,
    pub name: String,
    pub priority: i32,
    pub status: JobStatus,
    pub cpus: Option<usize>,
    /// Memory limit in GB; unset while staging unless explicitly supplied.
    pub memory_limit: Option<usize>,
    pub gpu: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum JobStatus {
    Staging,
    Queued,
    Running,
    Done,
    Failed,
    Cancelled,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct StageResponse {
    pub dir: std::path::PathBuf,
    pub id: u64,
    pub name: String,
}
