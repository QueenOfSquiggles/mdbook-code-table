use std::collections::VecDeque;

use mdbook::{
    book::{Book, Chapter},
    preprocess::{Preprocessor, PreprocessorContext},
    BookItem,
};

#[derive(PartialEq, Eq)]
enum RowType {
    Headings,
    Alignments,
    TextEntry,
    CodeEntry,
    Empty,
}
#[derive(Default)]
struct TableRow {
    contents: Vec<String>,
    row_types: Vec<RowType>,
}

#[derive(Default)]
struct CodeTable {
    rows: Vec<TableRow>,
}

pub struct CodeTables;

impl CodeTables {
    const CODE_ANNOTATION: &'static str = "@code";
    const MAX_LOOP_STEPS: u32 = 2048;

    fn get_table_row(string: &str, is_first: bool) -> TableRow {
        let entries: Vec<String> = string
            .split('|')
            .map(|dirty| dirty.trim().to_string())
            .collect();
        let mut types: Vec<RowType> = Vec::new();
        if is_first {
            types.resize_with(entries.len(), || RowType::Headings);
        } else {
            for e in entries.clone() {
                if e.contains('-') && !e.contains(' ') {
                    types.push(RowType::Alignments);
                    continue;
                }
                if e.contains('`') && e.split('`').count() > 1 {
                    types.push(RowType::CodeEntry);
                    continue;
                }
                if e.is_empty() {
                    types.push(RowType::Empty);
                    continue;
                }
                types.push(RowType::TextEntry);
            }
        }

        TableRow {
            contents: entries,
            row_types: types,
        }
    }

    fn get_table_metadata(string: &str) -> Option<(CodeTable, usize)> {
        let mut section_size: usize = 0;
        let mut table_lines: VecDeque<&str> = VecDeque::new();
        for line in string.lines() {
            if !line.contains('|') {
                // first line without | signifies end of table
                break;
            }
            table_lines.push_back(line);
        }
        if table_lines.is_empty() {
            return None;
        }
        let mut table_buffer: CodeTable = Default::default();
        let first = table_lines.pop_front().unwrap();
        section_size += first.len();
        table_buffer.rows.push(Self::get_table_row(first, true));

        for line in table_lines {
            section_size += line.len();
            table_buffer.rows.push(Self::get_table_row(line, false));
        }
        section_size += string.split_at(section_size).1.find('\n').unwrap_or(0); // append to next line break if one exists
        Some((table_buffer, section_size))
    }

    fn parse_chapter_contents(chapter: &Chapter) -> Chapter {
        let mut content = String::with_capacity(chapter.content.len());
        let mut buffer = chapter.content.clone();

        // safer while loop. Guaranteed exit point
        for _ in 0..Self::MAX_LOOP_STEPS {
            if buffer.is_empty() {
                // while cond
                break;
            }
            let target = buffer.find(Self::CODE_ANNOTATION);
            let Some(index) = target else {
                content.push_str(buffer.as_str());
                break;
            };
            let (clear, parse) = buffer.split_at(index);
            content.push_str(clear);
            let Some(valid_str) = parse.strip_prefix(Self::CODE_ANNOTATION) else {
                buffer = parse.split_at(Self::CODE_ANNOTATION.len()).1.to_string();
                continue;
            };
            let Some((meta, offset)) = Self::get_table_metadata(valid_str) else {
                buffer = valid_str.to_string();
                continue;
            };
            buffer = valid_str.split_at(offset).1.to_string();
            content.push_str(&meta.get_as_html());
        }

        // returns parsed chapter content?
        Chapter::new(
            &chapter.name,
            content,
            chapter.path.clone().unwrap_or_default(),
            chapter.parent_names.clone(),
        )
    }
}

impl Preprocessor for CodeTables {
    fn name(&self) -> &str {
        "code-tables"
    }

    fn run(&self, _ctx: &PreprocessorContext, book: Book) -> mdbook::errors::Result<Book> {
        let mut parsed_book = Book::new();
        for item in book.iter() {
            let parsed = match item {
                BookItem::Chapter(contents) => {
                    BookItem::Chapter(Self::parse_chapter_contents(contents))
                }
                BookItem::Separator => BookItem::Separator,
                BookItem::PartTitle(title) => BookItem::PartTitle(title.clone()),
            };
            parsed_book.push_item(parsed);
        }
        Ok(parsed_book)
    }
}

impl CodeTable {
    fn get_as_html(&self) -> String {
        let filter_data = |val: &&TableRow| -> bool { !val.row_types.contains(&RowType::Headings) };
        let filter_headings =
            |val: &&TableRow| -> bool { val.row_types.contains(&RowType::Headings) };
        let data_rows: String = self
            .rows
            .iter()
            .filter(filter_data)
            .map(|row| row.get_as_html())
            .collect();
        let heading: String = self
            .rows
            .iter()
            .filter(filter_headings)
            .map(|row| row.get_as_html())
            .collect();
        format!(
            r"<table>
				<thead>
					{}
				</thead>
				{}
			  </table>",
            heading, data_rows
        )
    }
}

impl TableRow {
    fn get_as_html(&self) -> String {
        let mut entries_data = String::new();
        for (index, entry) in self.contents.iter().enumerate() {
            let data: String = match self.row_types[index] {
                RowType::Alignments => "".to_string(),
                RowType::Empty => "".to_string(),
                RowType::Headings => format!("<th>{}</th>", entry),
                RowType::CodeEntry => format!("<td><pre>{}</pre><td>", entry),
                RowType::TextEntry => format!("<td>{}</td>", entry),
            };
            entries_data += data.as_str();
        }
        format!(r"<tr>{}</tr>", entries_data)
    }
}
