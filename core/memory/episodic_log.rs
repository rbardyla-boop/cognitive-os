//! Append-only episodic memory log model.

#[derive(Clone, Debug, PartialEq)]
pub struct Episode {
    pub episode_id: String,
    pub timestamp: String,
    pub source: String,
    pub raw_payload: String,
    pub parsed_claims: Vec<String>,
    pub confidence: f32,
    pub trace_id: String,
    pub linked_actions: Vec<String>,
    pub linked_rules: Vec<String>,
}

#[derive(Default)]
pub struct EpisodicLog {
    episodes: Vec<Episode>,
}

impl EpisodicLog {
    pub fn append(&mut self, episode: Episode) {
        self.episodes.push(episode);
    }

    pub fn all(&self) -> &[Episode] {
        &self.episodes
    }
}
