use super::{element::*, error::ParseError};
use crate::v2::lex::{
    Keyword,
    Number,
    Symbol,
    Token,
    TokenPosition,
    Whitespace,
};

#[derive(Clone, Debug, PartialEq)]
pub enum State {
    Root,
    SchemaName(String),
    Schema(String),

    /*
    CreatedSchema,
    CreatingTable(Table),
    CreatingTableExpectingAlias(Table),
    CreatedTable,
    CreatedRecord,
    CreatedAttribute,
    ExpectingTable,
    ExpectingRecord,
    ExpectingColumn,
    ExpectingValue(String),
    IdentifierExpectingReferenceValue {
        column: String,
        identifier: String,
    },
    SchemaQualifiedReferenceValueExpectingTable {
        column: String,
        schema: String,
    },
    SchemaQualifiedReferenceValueExpectingAtSign {
        column: String,
        schema: String,
        table: String,
    },
    SchemaQualifiedReferenceValueExpectingRecord {
        column: String,
        schema: String,
        table: String,
    },
    SchemaQualifiedReferenceValueExpectingRecordPeriod {
        column: String,
        schema: String,
        table: String,
        record: String,
    },
    SchemaQualifiedReferenceValueExpectingColumn {
        column: String,
        schema: String,
        table: String,
        record: String,
    },
    */
}

#[derive(Debug)]
pub(super) struct Parser {
    indent_unit: Option<String>,
    pub elements: Vec<Element>,
}

impl Parser {
    pub fn new() -> Self {
        Self {
            indent_unit: None,
            elements: Vec::new(),
        }
    }

    pub fn parse(mut self, tokens: Vec<TokenPosition>) -> Result<Self, ParseError> {
        let mut state = State::LineStart;

        for tp in tokens {
            state = self.receive(state, tp)?;
        }

        Ok(self)
    }

    fn receive(&mut self, state: State, tp: TokenPosition) -> Result<State, ParseError> {
        Ok(match state {
            State::Root => match tp.token {
                Token::Comment(_) => {
                    State::Root
                }
                Token::Whitespace(Whitespace::Newline) => {
                    State::Root
                }
                /*
                Token::Whitespace(Whitespace::Inline(i)) => {
                    unimplemented!()
                }
                */
                Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                    State::SchemaName(i)
                }
                _ => return Err(unexpected(tp))
            }

            State::SchemaName(name) => match tp.token {
                Token::Whitespace(Whitespace::Newline) => {
                    State::Schema(name)
                }
                Token::Whitespace(Whitespace::Inline(_)) => {
                    State::SchemaName(name)
                }
                Token::Comment(_) => {
                    State::SchemaName(name)
                }
                _ => return Err(unexpected(tp))
            }

            State::Schema(name) => match tp.token {
                Token::Whitespace(Whitespace::Newline) => {
                    State::Schema(name)
                }
                Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                    State::SchemaName(i)
                }
                _ => return Err(unexpected(tp))
            }

            _ => panic!("state")
        })
            
            /*
            Root => match token {
                Token::Whitespace(Whitespace::Newline) => {
                    line += 1;
                    LineStart
                }
                Token::Whitespace(Whitespace::Indent(indent)) => {
                    if indent.is_empty() {
                        return Err(ParseError::empty_indent(line));
                    }

                    if !indent.trim().is_empty() {
                        return Err(ParseError::invalid_indent(line, indent));
                    }

                    let unit = match &self.indent_unit {
                        Some(u) => u,
                        None => {
                            self.indent_unit = Some(indent.clone());
                            self.indent_unit.as_ref().unwrap()
                        }
                    };

                    match indent_level(unit, &indent) {
                        Some(level) => match level {
                            1 => ExpectingTable,
                            2 => ExpectingRecord,
                            3 => ExpectingColumn,
                            n => return Err(ParseError::unexpected_indent_level(line, n)),
                        },
                        None => {
                            return Err(ParseError::inconsistent_indent(
                                line,
                                unit.clone(),
                                indent,
                            ))
                        }
                    }
                }
                Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                    self.schemas.push(Schema::new(ident));
                    CreatedSchema
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            },
            CreatingTable(table) => match token {
                Token::Whitespace(Whitespace::Newline) => {
                    self.schemas
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_schema(line))?
                        .tables
                        .push(table);

                    line += 1;
                    LineStart
                }
                Token::Keyword(Keyword::As) => {
                    CreatingTableExpectingAlias(table)
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            }
            CreatingTableExpectingAlias(mut table) => match token {
                Token::Identifier(ident) => {
                    table.alias = Some(ident);
                    self.schemas
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_schema(line))?
                        .tables
                        .push(table);

                    CreatedTable
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            }
            CreatedSchema | CreatedTable | CreatedRecord | CreatedAttribute => match token {
                Token::Whitespace(Whitespace::Newline) => {
                    line += 1;
                    LineStart
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            },
            ExpectingTable => match token {
                Token::Whitespace(Whitespace::Newline) => {
                    line += 1;
                    LineStart
                }
                Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                    CreatingTable(Table::new(ident))
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            ExpectingRecord => match token {
                Token::Whitespace(Whitespace::Newline) => {
                    line += 1;
                    LineStart
                }
                Token::Identifier(_) | Token::Symbol(Symbol::Underscore) => {
                    let name = match token {
                        Token::Identifier(ident) => Some(ident),
                        Token::Symbol(Symbol::Underscore) => None,
                        _ => unreachable!(),
                    };

                    self.schemas
                        .last_mut()
                        // It shouldn't actually be possible to return this error here,
                        // since `ExpectingRecord` will only be reached if the indentation
                        // unit has already been set, meaning a line containing a table has
                        // to have already been successfully parsed, meaning a schema should
                        // have to be present.
                        .ok_or_else(|| ParseError::missing_schema(line))?
                        .tables
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_table(line))?
                        .records
                        .push(Record::new(name));

                    CreatedRecord
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            ExpectingColumn => match token {
                Token::Whitespace(Whitespace::Newline) => {
                    line += 1;
                    LineStart
                }
                Token::Identifier(ident) | Token::QuotedIdentifier(ident) => {
                    ExpectingValue(ident)
                }
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            ExpectingValue(column) => match token {
                Token::Boolean(_) | Token::Number(_) | Token::Text(_) => {
                    let value = match token {
                        Token::Boolean(b) => Value::Boolean(b),
                        Token::Number(b) => Value::Number(b),
                        Token::Text(b) => Value::Text(b),
                        _ => unreachable!(),
                    };

                    self.schemas
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_schema(line))? // Should never return error
                        .tables
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_table(line))? // Should never return error
                        .records
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_record(line))?
                        .attributes
                        .push(Attribute {
                            name: column,
                            value,
                        });

                    CreatedAttribute
                }
                Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                    IdentifierExpectingReferenceValue {
                        column,
                        identifier: i,
                    }
                }
                Token::AtSign => {
                    let schema = self.schemas
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_schema(line))?; // Should never return error

                    let table = schema.tables
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_table(line))?; // Should never return error

                    SchemaQualifiedReferenceValueExpectingRecord {
                        column,
                        schema: schema.name.clone(),
                        table: table.name.clone(),

                    }
                }
                _ => return Err(ParseError::missing_column_value(line)),
            },

            IdentifierExpectingReferenceValue { column, identifier } => match token {
                Token::AtSign => {
                    let schema = self.schemas
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_schema(line))? // Should never return error
                        .name.clone();

                    SchemaQualifiedReferenceValueExpectingRecord {
                        column,
                        schema,
                        table: identifier,

                    }
                },
                Token::Symbol(Symbol::Period) => SchemaQualifiedReferenceValueExpectingTable {
                    column,
                    schema: identifier,
                },
                // Saying this identifier itself was unexpected (not obviously being intended
                // as a reference) seems clearer than saying the newline is the problem
                Token::Whitespace(Whitespace::Newline) => return Err(ParseError::unexpected_token(line, Token::Identifier(identifier))),
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            SchemaQualifiedReferenceValueExpectingTable { column, schema } => match token {
                Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                    SchemaQualifiedReferenceValueExpectingAtSign {
                        column,
                        schema,
                        table: i,
                    }
                },
                Token::Whitespace(Whitespace::Newline) => return Err(ParseError::incomplete_reference(
                    line,
                    format!("{}.", schema),
                )),
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            SchemaQualifiedReferenceValueExpectingAtSign { column, schema, table } => match token {
                Token::AtSign => {
                    SchemaQualifiedReferenceValueExpectingRecord {
                        column,
                        schema,
                        table,
                    }
                },
                Token::Whitespace(Whitespace::Newline) => return Err(ParseError::incomplete_reference(
                    line,
                    format!("{}.{}", schema, table),
                )),
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            SchemaQualifiedReferenceValueExpectingRecord { column, schema, table } => match token {
                Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                    SchemaQualifiedReferenceValueExpectingRecordPeriod {
                        column,
                        schema,
                        table,
                        record: i,
                    }
                },
                Token::Whitespace(Whitespace::Newline) => return Err(ParseError::incomplete_reference(
                    line,
                    format!("{}.{}@", schema, table),
                )),
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            SchemaQualifiedReferenceValueExpectingRecordPeriod { column, schema, table, record } => match token {
                Token::Symbol(Symbol::Period) => {
                    SchemaQualifiedReferenceValueExpectingColumn {
                        column,
                        schema,
                        table,
                        record,
                    }
                },
                Token::Whitespace(Whitespace::Newline) => return Err(ParseError::incomplete_reference(
                    line,
                    format!("{}.{}@{}", schema, table, record),
                )),
                _ => return Err(ParseError::unexpected_token(line, token)),
            },

            SchemaQualifiedReferenceValueExpectingColumn { column, schema, table, record } => match token {
                Token::Identifier(i) | Token::QuotedIdentifier(i) => {
                    let value = Value::Reference(Box::new(ReferenceValue {
                        schema,
                        table,
                        record,
                        column: i,
                    }));

                    self.schemas
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_schema(line))? // Should never return error
                        .tables
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_table(line))? // Should never return error
                        .records
                        .last_mut()
                        .ok_or_else(|| ParseError::missing_record(line))?
                        .attributes
                        .push(Attribute {
                            name: column,
                            value,
                        });

                    CreatedAttribute
                },
                Token::Whitespace(Whitespace::Newline) => return Err(ParseError::incomplete_reference(
                    line,
                    format!("{}.{}@{}.", schema, table, record),
                )),
                _ => return Err(ParseError::unexpected_token(line, token)),
            },
            */
    }
}

fn unexpected(tp: TokenPosition) -> ParseError {
    ParseError::unexpected_token(tp.token, tp.start_position)
}

fn indent_level(unit: &str, indent: &str) -> Option<usize> {
    // A valid indent, when split by the indent unit, will yield
    // a collection of empty strings, with the collection length
    // being one longer than the "indent level".
    //
    // Eg: "    ".split("  ") => ["", "", ""]
    let parts: Vec<&str> = indent.split(unit).collect();

    for p in &parts {
        if !p.is_empty() {
            return None;
        }
    }

    Some(parts.len() - 1)
}


#[cfg(test)]
mod tests {
    use super::{Element, Parser, ParseError};

    fn parse(input: &str) -> Result<Vec<Element>, ParseError> {
        let tokens = crate::v2::lex::lex(input).unwrap();

        Ok(Parser::new().parse(tokens)?.elements)
    }

    #[test]
    fn blank() {
        assert_eq!(parse("").unwrap(), Vec::new());
    }

    #[test]
    fn comments() {
        let input =
"--this is a comment

-- and another comment";

        assert_eq!(parse(input).unwrap(), Vec::new());
    }

    #[test]
    fn empty_schemas() {
        let input =
"-- comment
schema1

schema2 -- comment";

        assert_eq!(parse(input).unwrap(), Vec::new());
    }

    #[test]
    fn schemas_with_tables() {
        let input =
"-- comment
schema1
  table1

schema2 -- comment
  table2 -- comment";

        assert_eq!(parse(input).unwrap(), Vec::new());
    }

    /*
    mod indent_level {
        use super::super::indent_level;

        fn spaces(count: usize) -> String {
            " ".repeat(count)
        }

        fn tabs(count: usize) -> String {
            "\t".repeat(count)
        }

        #[test]
        fn valid_indents() {
            let assert_valid = |unit: &str, indent: &str, expected_level: usize| {
                assert_eq!(
                    indent_level(unit, indent),
                    Some(expected_level),
                    "unit: {:?}, indent: {:?}, level: {}",
                    unit,
                    indent,
                    expected_level,
                );
            };

            // Only explicitly care about expected indentation levels...
            //
            //   - schema name: 0
            //   - table name:  1
            //   - record name: 2
            //   - attribute:   3
            //
            // ... and indentation levels defined by...
            //
            //   - single space
            //   - double space
            //   - quadruple space
            //   - tab

            assert_valid(&spaces(1), &spaces(0), 0);
            assert_valid(&spaces(1), &spaces(1), 1);
            assert_valid(&spaces(1), &spaces(2), 2);
            assert_valid(&spaces(1), &spaces(3), 3);

            assert_valid(&spaces(2), &spaces(0), 0);
            assert_valid(&spaces(2), &spaces(2), 1);
            assert_valid(&spaces(2), &spaces(4), 2);
            assert_valid(&spaces(2), &spaces(6), 3);

            assert_valid(&spaces(4), &spaces(0), 0);
            assert_valid(&spaces(4), &spaces(4), 1);
            assert_valid(&spaces(4), &spaces(8), 2);
            assert_valid(&spaces(4), &spaces(12), 3);

            assert_valid(&tabs(1), &tabs(0), 0);
            assert_valid(&tabs(1), &tabs(1), 1);
            assert_valid(&tabs(1), &tabs(2), 2);
            assert_valid(&tabs(1), &tabs(3), 3);
        }

        #[test]
        fn invalid_indents() {
            let assert_invalid = |unit: &str, indent: &str| {
                assert_eq!(
                    indent_level(unit, indent),
                    None,
                    "unit: {:?}, indent: {:?}",
                    unit,
                    indent,
                );
            };
            assert_invalid(&spaces(2), &spaces(1));
            assert_invalid(&spaces(2), &spaces(3));
            assert_invalid(&spaces(2), &spaces(5));

            assert_invalid(&spaces(4), &spaces(1));
            assert_invalid(&spaces(4), &spaces(2));
            assert_invalid(&spaces(4), &spaces(3));
            assert_invalid(&spaces(4), &spaces(5));
            assert_invalid(&spaces(4), &spaces(6));
            assert_invalid(&spaces(4), &spaces(7));
            assert_invalid(&spaces(4), &spaces(9));

            assert_invalid(&spaces(1), &tabs(1));
            assert_invalid(&tabs(1), &spaces(1));
        }
    }
    */
}