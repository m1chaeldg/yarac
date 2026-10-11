use serde::{Deserialize, Serialize};
use super::request_state::RequestState;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CollectionItem {
    pub id: String,
    pub name: String,
    pub requests: Vec<RequestState>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceState {
    pub name: String,
    pub collections: Vec<CollectionItem>,
    pub active_request_id: Option<String>,
}

impl WorkspaceState {
    pub fn new(name: &str) -> Self {
        Self {
            name: name.to_string(),
            collections: Vec::new(),
            active_request_id: None,
        }
    }

    pub fn add_collection(&mut self, name: &str) -> String {
        let id = uuid::Uuid::new_v4().to_string();
        self.collections.push(CollectionItem {
            id: id.clone(),
            name: name.to_string(),
            requests: Vec::new(),
        });
        id
    }

    pub fn create_request(&mut self, collection_id: &str, name: &str, method: &str, url: &str) -> String {
        let req = RequestState::new(name, method, url);
        let req_id = req.id.clone();
        if let Some(col) = self.collections.iter_mut().find(|c| c.id == collection_id) {
            col.requests.push(req);
            self.active_request_id = Some(req_id.clone());
        }
        req_id
    }

    pub fn get_active_request(&self) -> Option<&RequestState> {
        let active_id = self.active_request_id.as_ref()?;
        for col in &self.collections {
            for req in &col.requests {
                if &req.id == active_id {
                    return Some(req);
                }
            }
        }
        None
    }

    pub fn get_active_request_mut(&mut self) -> Option<&mut RequestState> {
        let active_id = self.active_request_id.as_ref()?.clone();
        for col in &mut self.collections {
            for req in &mut col.requests {
                if req.id == active_id {
                    return Some(req);
                }
            }
        }
        None
    }
}
