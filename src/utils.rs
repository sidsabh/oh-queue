use serde::{Serialize, Deserialize};
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

