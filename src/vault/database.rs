use rocket::tokio::task::spawn_blocking;
use std::sync::Arc;

use crate::sync_db::Result;

impl super::Vault {
    pub async fn new_tag_category(&self, name: String, description: String) -> Result<i64> {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().new_tag_category(&name, &description))
            .await
            .unwrap()
    }

    pub async fn new_tag(&self, name: String, category: i64, description: String) -> Result<i64> {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().new_tag(&name, category, &description))
            .await
            .unwrap()
    }

    pub async fn tag_id(&self, name: String) -> Option<i64> {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().tag_id(&name))
            .await
            .unwrap()
    }

    pub async fn tag_category_id(&self, name: String) -> Option<i64> {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().tag_category_id(&name))
            .await
            .unwrap()
    }

    pub async fn tag_entry(&self, entry_id: i64, tag_id: i64) {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().tag_entry(entry_id, tag_id))
            .await
            .unwrap()
    }

    pub async fn tag_entry_many(&self, entry_id: i64, tag_ids: Vec<i64>) {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().tag_entry_many(entry_id, &tag_ids))
            .await
            .unwrap()
    }

    pub async fn tag_category_search(&self, name: String) -> Vec<String> {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().tag_category_search(&name))
            .await
            .unwrap()
    }

    pub async fn tag_search(&self, name: String, category: Option<i64>) -> Vec<String> {
        let t_db = Arc::clone(&self.database);
        spawn_blocking(move || t_db.lock().unwrap().tag_search(&name, category))
            .await
            .unwrap()
    }
}
