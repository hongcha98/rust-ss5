use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(name = "rust-ss5")]
pub struct Opt {
    #[structopt(short = "c", parse(from_str))]
    conf: Option<String>,
}

// const CONF: String = "~/conf/ss5-server.conf".parse().unwrap();

impl Opt {
    pub fn conf(&self) -> String {
        match self.conf.clone() {
            None => "1".to_string(),
            Some(c) => c
        }
    }
}