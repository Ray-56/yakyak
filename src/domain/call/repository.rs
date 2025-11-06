//! Call repository interface

use crate::domain::call::aggregate::Call;
use crate::domain::shared::result::Result;
use crate::domain::shared::value_objects::CallId;
use async_trait::async_trait;

/// Repository interface for Call aggregate
///
/// This is defined in the domain layer as a trait (port),
/// and implemented in the infrastructure layer (adapter).
#[async_trait]
pub trait CallRepository: Send + Sync {
    /// Find a call by its ID
    async fn find_by_id(&self, id: &CallId) -> Result<Option<Call>>;

    /// Save a call (insert or update)
    async fn save(&self, call: &Call) -> Result<()>;

    /// Delete a call
    async fn delete(&self, id: &CallId) -> Result<()>;

    /// Find all active calls
    async fn find_active_calls(&self) -> Result<Vec<Call>>;

    /// Find calls by endpoint ID
    async fn find_by_endpoint(&self, endpoint_id: &crate::domain::shared::value_objects::EndpointId) -> Result<Vec<Call>>;
}
