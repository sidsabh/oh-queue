use crossterm::queue;
use serde::{Serialize, Deserialize};
use serde_json;
use std::fs::{read_to_string, File};
use std::io::{self, Write};
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum Purpose {
    ConceptualMaterial,
    ConceptualLab,
    Debugging,
    Other,
}

impl Purpose {
    pub fn is_empty(&self) -> bool {
        match self {
            Purpose::ConceptualMaterial => false,
            Purpose::ConceptualLab => false,
            Purpose::Debugging => false,
            Purpose::Other => false,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StudentInfo {
    pub name: String,
    pub csid: String,
    pub purpose: Purpose,
    pub details: String,
    pub steps: String,
}

impl StudentInfo {
    pub fn new(name: String, csid: String, purpose: Purpose, details: String, steps: String) -> StudentInfo {
        StudentInfo {
            name,
            csid,
            purpose,
            details,
            steps,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct StudentRequest {
    pub info : StudentInfo,
    pub id: String,
}

impl StudentRequest {
    pub fn new (info: StudentInfo) -> StudentRequest {
        StudentRequest {
            info,
            id: Uuid::new_v4().to_string(),
        }
    }
}

// Supporting struct for query parameters
#[derive(Deserialize)]
pub struct IdQuery {
    pub id: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Queue {
    pub students: Vec<StudentRequest>,
    #[serde(skip)]
    path: PathBuf,
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
            
            let queue = if !path.exists() {
                Queue::new(path.clone())
            } else {
                Queue::load(path.clone()).expect("Failed to load queue")
            };
            queue.save().expect("Failed to save queue");
            Ok(queue)
        }
    }

    pub fn size(&self) -> usize {
        self.students.len()
    }
}

