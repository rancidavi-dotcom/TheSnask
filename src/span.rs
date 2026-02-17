use std::fmt;

/// Representa uma posição no código fonte
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Position {
    pub line: usize,
    pub column: usize,
    pub offset: usize, // Offset absoluto no arquivo
}

impl Position {
    pub fn new(line: usize, column: usize, offset: usize) -> Self {
        Position { line, column, offset }
    }

    pub fn start() -> Self {
        Position { line: 1, column: 1, offset: 0 }
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}:{}", self.line, self.column)
    }
}

/// Representa um intervalo no código fonte (início e fim)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

impl Span {
    pub fn new(start: Position, end: Position) -> Self {
        Span { start, end }
    }

    pub fn single(pos: Position) -> Self {
        Span { start: pos, end: pos }
    }

    pub fn dummy() -> Self {
        let pos = Position::start();
        Span { start: pos, end: pos }
    }

    /// Combina dois spans em um único span que cobre ambos
    pub fn merge(&self, other: &Span) -> Span {
        let start = if self.start.offset < other.start.offset {
            self.start
        } else {
            other.start
        };

        let end = if self.end.offset > other.end.offset {
            self.end
        } else {
            other.end
        };

        Span { start, end }
    }

    /// Retorna o comprimento do span em caracteres
    pub fn len(&self) -> usize {
        self.end.offset.saturating_sub(self.start.offset)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

impl fmt::Display for Span {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if self.start.line == self.end.line {
            write!(f, "{}:{}-{}", self.start.line, self.start.column, self.end.column)
        } else {
            write!(f, "{}-{}", self.start, self.end)
        }
    }
}
