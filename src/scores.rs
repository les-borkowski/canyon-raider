pub struct Entry {
    pub name: String,
    pub score: u32,
}

pub struct Scores {
    entries: Vec<Entry>,
}

impl Scores {
    pub fn new() -> Self {
        Self { entries: Vec::new() }
    }

    fn serialize(&self) -> String {
        self.entries
            .iter()
            .map(|e| format!("{}|{}", e.name, e.score))
            .collect::<Vec<_>>()
            .join("\n")
    }

    fn parse(s: &str) -> Self {
        let entries = s
            .lines()
            .filter_map(|line| {
                let mut parts = line.splitn(2, '|');
                let name = parts.next()?.to_string();
                let score = parts.next()?.parse().ok()?;
                Some(Entry { name, score })
            })
            .collect();
        Self { entries }
    }

    pub fn is_high_score(&self, score: u32) -> bool {
        self.entries.len() < 5
            || score > self.entries.last().map_or(0, |e| e.score)
    }

    pub fn insert(&mut self, name: String, score: u32) {
        self.entries.push(Entry { name, score });
        self.entries.sort_by(|a, b| b.score.cmp(&a.score));
        self.entries.truncate(5);
    }

    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
}

impl Default for Scores {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn serialize_empty_is_empty_string() {
        let s = Scores::new();
        assert_eq!(s.serialize(), "");
    }

    #[test]
    fn serialize_single_entry() {
        let mut s = Scores::new();
        s.entries.push(Entry { name: "LES".into(), score: 4200 });
        assert_eq!(s.serialize(), "LES|4200");
    }

    #[test]
    fn serialize_multiple_entries() {
        let mut s = Scores::new();
        s.entries.push(Entry { name: "LES".into(), score: 4200 });
        s.entries.push(Entry { name: "ACE".into(), score: 3100 });
        assert_eq!(s.serialize(), "LES|4200\nACE|3100");
    }

    #[test]
    fn parse_empty_string_gives_empty_board() {
        let s = Scores::parse("");
        assert_eq!(s.entries.len(), 0);
    }

    #[test]
    fn parse_valid_lines() {
        let s = Scores::parse("LES|4200\nACE|3100");
        assert_eq!(s.entries.len(), 2);
        assert_eq!(s.entries[0].name, "LES");
        assert_eq!(s.entries[0].score, 4200);
        assert_eq!(s.entries[1].name, "ACE");
        assert_eq!(s.entries[1].score, 3100);
    }

    #[test]
    fn parse_skips_malformed_lines() {
        let s = Scores::parse("LES|4200\nbad_line\nACE|3100");
        assert_eq!(s.entries.len(), 2);
    }

    #[test]
    fn parse_skips_non_numeric_score() {
        let s = Scores::parse("LES|abc");
        assert_eq!(s.entries.len(), 0);
    }

    #[test]
    fn round_trip() {
        let raw = "LES|4200\nACE|3100";
        assert_eq!(Scores::parse(raw).serialize(), raw);
    }

    #[test]
    fn is_high_score_when_board_empty() {
        assert!(Scores::new().is_high_score(0));
    }

    #[test]
    fn is_high_score_when_fewer_than_five_entries() {
        let mut s = Scores::new();
        s.entries.push(Entry { name: "A".into(), score: 100 });
        assert!(s.is_high_score(1));
    }

    #[test]
    fn is_high_score_beats_last_on_full_board() {
        let mut s = Scores::new();
        for i in (1u32..=5).rev() {
            s.entries.push(Entry { name: "X".into(), score: i * 100 });
        }
        assert!(s.is_high_score(101));
        assert!(!s.is_high_score(100));
        assert!(!s.is_high_score(50));
    }

    #[test]
    fn insert_keeps_sorted_descending() {
        let mut s = Scores::new();
        s.insert("B".into(), 200);
        s.insert("A".into(), 300);
        s.insert("C".into(), 100);
        assert_eq!(s.entries()[0].score, 300);
        assert_eq!(s.entries()[1].score, 200);
        assert_eq!(s.entries()[2].score, 100);
    }

    #[test]
    fn insert_trims_to_five() {
        let mut s = Scores::new();
        for i in 0..7u32 {
            s.insert(format!("P{i}"), i * 100);
        }
        assert_eq!(s.entries().len(), 5);
        assert_eq!(s.entries()[0].score, 600);
    }

    #[test]
    fn entries_returns_slice() {
        let mut s = Scores::new();
        s.insert("X".into(), 42);
        assert_eq!(s.entries().len(), 1);
        assert_eq!(s.entries()[0].score, 42);
    }
}
