/// The maximum size of the attribute stack. This represents the maximum entries that a single
/// escape sequence can have.
const ATTR_STACK_SIZE: usize = 4;

/// Enum that represents the current state of the `AnsiParseIterator.`
enum AnsiParserState {
    /// No characters of significance have been found yet.
    None,

    /// Currently looking for a bracket to start the escape.
    Bracket,

    /// ANSI escape prefix has been found and the parser is reading the escape codes.
    Attr
}

/// Enum that represents the state of an entry in the attribute stack of the `AnsiParseIterator`.
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
enum AttrStackEntry {
    /// No entry
    Missing,

    /// Escape prefix found, but escape code not yet determined
    Initialized,

    /// In the process or finished reading escape code, ready to be sent to the receiver.
    Value(u8)
}

/// An ANSI escape sequence parser that does not use heap allocation. This is used as an iterator
/// which returns the codes found as they are found.
pub struct AnsiParseIterator<'a> {
    data: &'a str,
    state: AnsiParserState,
    attr_stack: [AttrStackEntry; ATTR_STACK_SIZE],
    index: usize,
}

impl<'a> AnsiParseIterator<'a> {
    /// Creates a new instance of `AnsiParseIterator`
    pub fn new(data: &str) -> AnsiParseIterator {
        AnsiParseIterator {
            data,
            state: AnsiParserState::None,
            attr_stack: [AttrStackEntry::Missing; ATTR_STACK_SIZE],
            index: 0,
        }
    }

    /// Returns the last entry in the stack that is not unused.
    pub fn current_stack_index(&self) -> usize {
        for (i, entry) in self.attr_stack.iter().enumerate().rev() {
            match entry {
                AttrStackEntry::Missing => (),
                _ => return i,
            }
        }

        0
    }

    /// Returns true if the stack is fully unused.
    pub fn stack_empty(&self) -> bool {
        for entry in self.attr_stack.iter() {
            match entry {
                AttrStackEntry::Missing => (),
                _ => return false,
            }
        }

        true
    }
}

impl <'a> Iterator for AnsiParseIterator<'a> {
    type Item = AnsiSequencePart<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data == "" {
            return None;
        }

        loop {
            match self.state {
                AnsiParserState::None => {
                    if !self.stack_empty() {
                        let current_index = self.current_stack_index();
                        let entry = &self.attr_stack[current_index];

                        let value = match entry {
                            AttrStackEntry::Value(value) => Some(*value),
                            _ => None,
                        };

                        self.attr_stack[current_index] = AttrStackEntry::Missing;

                        if value.is_some() {
                            return Some(AnsiSequencePart::SGR(value.unwrap()));
                        } else {
                            continue;
                        }
                    }

                    self.index = 0;

                    let next_escape = self.data.find('\x1b');
                    match next_escape {
                        Some(0) => {
                            self.data = &self.data[1..];
                            self.state = AnsiParserState::Bracket;
                        },
                        Some(next_escape_index) => {
                            let ret = &self.data[..next_escape_index];
                            self.data = &self.data[next_escape_index..];
                            return Some(AnsiSequencePart::Text(ret));
                        },
                        None => {
                            let ret = &self.data[..];
                            self.data = "";
                            return Some(AnsiSequencePart::Text(ret));
                        }
                    }
                },
                AnsiParserState::Bracket => {
                    if let Some(c) = self.data.chars().nth(self.index) {
                        match c {
                            '[' => {
                                self.state = AnsiParserState::Attr;
                                self.index += 1;
                                self.attr_stack[0] = AttrStackEntry::Initialized;
                            },
                            _ => self.state = AnsiParserState::None,
                        }
                    } else {
                        return None;
                    }
                },
                AnsiParserState::Attr => {
                    if let Some(c) = self.data.chars().nth(self.index) {
                        match c {
                            '0'..='9' => {
                                let current_index = self.current_stack_index();
                                match &self.attr_stack[current_index] {
                                    AttrStackEntry::Missing => {
                                        self.state = AnsiParserState::None;
                                        self.attr_stack = [AttrStackEntry::Missing; ATTR_STACK_SIZE];
                                    },
                                    entry => {
                                        let current_value = match entry {
                                            AttrStackEntry::Initialized => 0,
                                            AttrStackEntry::Value(value) => *value,
                                            _ => unreachable!(),
                                        };

                                        let addition = c.to_digit(10).unwrap() as u8;
                                        self.attr_stack[current_index] = AttrStackEntry::Value(current_value * 10 + addition);
                                    }
                                }

                                self.index += 1;
                            },
                            ';' => {
                                self.attr_stack[self.current_stack_index() + 1] = AttrStackEntry::Initialized;
                                self.index += 1;
                            },
                            'm' => {
                                self.state = AnsiParserState::None;
                                if self.attr_stack.contains(&AttrStackEntry::Initialized) |
                                    self.stack_empty() {
                                    self.attr_stack = [AttrStackEntry::Missing; ATTR_STACK_SIZE];
                                } else {
                                    self.index += 1;
                                    self.data = &self.data[self.index..];
                                }
                            },
                            _ => {
                                self.state = AnsiParserState::None;
                                self.attr_stack = [AttrStackEntry::Missing; ATTR_STACK_SIZE];
                            },
                        }
                    } else {
                        self.state = AnsiParserState::None;
                        self.attr_stack = [AttrStackEntry::Missing; ATTR_STACK_SIZE];
                    }
                },
            }
        }
    }
}

/// A parsed ANSI escape code that gets returned by `AnsiEscapeParser`
#[derive(Debug, Copy, Clone)]
pub enum AnsiSequencePart<'a> {
    /// No escape code, just text in between escape codes
    Text(&'a str),

    /// SGR stands for Select Graphics Rendition, this escape code type modifies the appearance of
    /// text, mainly used for setting colors.
    /// See https://en.wikipedia.org/wiki/ANSI_escape_code#SGR_parameters for more specific
    /// information and examples
    SGR(u8),
}
