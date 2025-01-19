#[derive(Debug, Clone, Copy)]
pub struct Location {
    pub line: usize,
    pub col: usize,
    pub char_offset: usize,
}

impl Location {
    pub fn new(line: usize, col: usize, char_offset: usize) -> Self {
        Location {
            line: line,
            col: col,
            char_offset: char_offset,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: Location,
    pub end: Location,
}

impl Span {
    pub fn new(
        start_line: usize,
        start_col: usize,
        start_offset: usize,
        end_line: usize,
        end_col: usize,
        end_offset: usize,
    ) -> Self {
        let start = Location::new(start_line, start_col, start_offset);
        let end = Location::new(end_line, end_col, end_offset);

        Span {
            start: start,
            end: end,
        }
    }

    pub fn merge(start: Span, end: Span) -> Self {
        Span {
            start: start.start,
            end: end.end,
        }
    }

    pub fn from_loc(start: Location, end: Location) -> Self {
        Span {
            start: start,
            end: end,
        }
    }
}
