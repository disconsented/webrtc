use std::sync::Arc;

use crate::error::Result;
use crate::{Interceptor, InterceptorBuilder};

pub type MockBuilderResult = Result<Arc<dyn Interceptor + Send + Sync>>;

/// MockBuilder is a mock Builder for testing.
pub struct MockBuilder {
    pub build: Box<dyn (Fn(&str) -> MockBuilderResult) + Send + Sync + 'static>,
}

impl MockBuilder {
    #[tracing::instrument(level = "debug", skip(f))]
    pub fn new<F: (Fn(&str) -> MockBuilderResult) + Send + Sync + 'static>(f: F) -> Self {
        MockBuilder { build: Box::new(f) }
    }
}

impl InterceptorBuilder for MockBuilder {
    #[tracing::instrument(level = "debug", skip(self, id))]
    fn build(&self, id: &str) -> MockBuilderResult {
        (self.build)(id)
    }
}
