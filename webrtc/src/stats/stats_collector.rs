use std::collections::HashMap;

use util::sync::Mutex;

use super::StatsReportType;

#[derive(Debug, Default)]
pub struct StatsCollector {
    pub(crate) reports: Mutex<HashMap<String, StatsReportType>>,
}

impl StatsCollector {
    #[tracing::instrument(level = "debug", skip())]
    pub(crate) fn new() -> Self {
        StatsCollector {
            ..Default::default()
        }
    }

    #[tracing::instrument(level = "debug", skip(self, id, stats))]
    pub(crate) fn insert(&self, id: String, stats: StatsReportType) {
        let mut reports = self.reports.lock();
        reports.insert(id, stats);
    }

    #[tracing::instrument(level = "debug", skip(self, stats))]
    pub(crate) fn merge(&self, stats: HashMap<String, StatsReportType>) {
        let mut reports = self.reports.lock();
        reports.extend(stats)
    }

    #[tracing::instrument(level = "debug", skip(self))]
    pub(crate) fn into_reports(self) -> HashMap<String, StatsReportType> {
        self.reports.into_inner()
    }
}
