#[derive(Clone, Debug)]
/// Options for rlurmd daemon
pub struct Config {}

impl Config {
    pub fn parser() -> impl bpaf::Parser<Self> {
        bpaf::pure(Self {})
    }
}

pub fn run(config: Config) {
    println!("lol i don't do anything yet");
}
