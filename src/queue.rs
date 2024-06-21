use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::{read_to_string, File};
use std::io::{self, Write};
use structopt::StructOpt;
use crate::utils::StudentRequest;
use std::path::{PathBuf, Path};

#[derive(Serialize, Deserialize, Debug)]
pub struct Queue {
    students: Vec<StudentRequest>,
    #[serde(skip)]
    path: PathBuf,
}

#[derive(StructOpt, Debug)]
#[structopt(name = "queue")]
pub struct Opt {
    /// Path to the queue file
    #[structopt(parse(from_os_str))]
    pub path: Option<PathBuf>,
}


impl Queue {
    pub fn load(path: PathBuf) -> io::Result<Self> {
        let data = read_to_string(&path).unwrap_or_else(|_| "{}".to_string());
        let mut queue: Queue = serde_json::from_str(&data).unwrap_or_else(|_| Queue { students: vec![], path: path.clone() });
        queue.path = path;
        Ok(queue)
    }

    pub fn save(&self) -> io::Result<()> {
        let data = serde_json::to_string_pretty(&self)?;
        let mut file = File::create(&self.path)?;
        file.write_all(data.as_bytes())
    }

    pub fn add(&mut self, request: StudentRequest) {
        self.students.push(request);
        self.save().expect("Failed to save queue.");
    }

    pub fn remove(&mut self, id: String) -> Result<(), ()> {
        if let Some(index) = self.students.iter().position(|x| x.id == id) {
            self.students.remove(index);
            self.save().expect("Failed to save queue.");
            Ok(())
        } else {
            Err(())
        }
    }

    pub fn position(&self, id: String) -> Result<usize, ()> {
        self.students.iter().position(|x| x.id == id).map(|pos| pos + 1).ok_or(())
    }
}

impl Queue {
    pub fn new(path: PathBuf) -> Self {
        Queue { students: vec![], path }
    }

    pub fn init(path: Option<PathBuf>) -> io::Result<Self> {
        if let Some(path) = path {
            Self::load(path)
        } else {
            let mut path = dirs::home_dir().expect("Could not find home directory");
            path.push("queue.json");
            let queue = Queue::new(path);
            queue.save()?;
            Ok(queue)
        }
    }
}

