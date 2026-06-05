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
}
