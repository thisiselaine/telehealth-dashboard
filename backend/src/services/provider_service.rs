use crate::db::models::Provider;
use crate::db::queries::{create_provider, get_providers_by_specialty};

pub async fn add_provider(provider: Provider) -> Result<Provider, String> {
    create_provider(&pool, provider).await.map_err(|_| "Error adding provider".into())
}

pub async fn get_providers(specialty: String) -> Result<Vec<Provider>, String> {
    get_providers_by_specialty(&pool, &specialty).await.map_err(|_| "Error retrieving providers".into())
}
