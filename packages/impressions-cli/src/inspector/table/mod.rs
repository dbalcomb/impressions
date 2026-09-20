mod history;
mod record;

use std::io;

use impressions::memory::address::{Address, AddressSpace};
use impressions::memory::inspect::{Inspector, Record, RecordBuilder, Status};
use unicode_truncate::UnicodeTruncateStr;

use self::history::History;
use self::record::StoredRecord;

const LABEL_WIDTH: usize = 40;
const DATA_TYPE_WIDTH: usize = 40;

const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";

/// The memory region inspector for the `inspect` command with table formatting.
pub struct TableInspector<W> {
    writer: W,
    buffer: StoredRecord,
    history: History,
    error: Option<io::Error>,
}

impl<W> TableInspector<W>
where
    W: io::Write,
{
    /// Constructs a new table inspector that writes to the given writer.
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            buffer: StoredRecord::default(),
            history: History::new(),
            error: None,
        }
    }

    /// Takes the error that occurred while writing to the underlying writer.
    pub fn take_error(&mut self) -> Option<io::Error> {
        self.error.take()
    }
}

impl<W> TableInspector<W>
where
    W: io::Write,
{
    /// Writes the record at the given position in the history.
    fn write_record(&mut self, position: usize) -> io::Result<()> {
        if self.error.is_some() {
            return Ok(());
        }

        let Some(record) = self.history.get(position) else {
            return Ok(());
        };

        let marker = match record.status {
            Some(Status::Identified) => ' ',
            Some(Status::Unidentified) => '?',
            Some(Status::Vacant) if record.address_space.size().get() == 1 => '‧',
            Some(Status::Vacant) => ':',
            None => '>',
        };

        let style = match record.status {
            Some(Status::Identified) => GREEN,
            Some(Status::Unidentified) => RED,
            Some(Status::Vacant) => DIM,
            None => BOLD,
        };

        write!(
            self.writer,
            "{style}{}  {marker}  ",
            record.address_space.first()
        )?;

        let indentation = position.saturating_mul(2).min(LABEL_WIDTH);

        write!(self.writer, "{:indentation$}", "")?;

        let max_width = LABEL_WIDTH - indentation;
        let (label, width) = record.label.unicode_truncate(max_width);

        write!(
            self.writer,
            "{label}{:padding$}",
            "",
            padding = max_width - width
        )?;

        writeln!(
            self.writer,
            "{RESET}  {:>DATA_TYPE_WIDTH$}  {}",
            record.data_type, record.value
        )?;

        Ok(())
    }

    fn write_gap(&mut self, address: Address) -> io::Result<()> {
        if self.error.is_some() {
            return Ok(());
        }

        writeln!(self.writer, "{DIM}{address}  :{RESET}")?;

        Ok(())
    }
}

impl<W> Inspector for TableInspector<W>
where
    W: io::Write,
{
    fn emit(&mut self, record: Record<'_>) {
        self.buffer.store(record);

        let Some(position) = self.history.position_for(&self.buffer) else {
            return;
        };

        let mut gap = None;

        if position == self.history.len()
            && let Some(prev) = self.history.last()
            && prev.address_space.first() < self.buffer.address_space.first()
        {
            gap = Some(prev.address_space.first());
        }

        if position < self.history.len()
            && let Some(prev) = self.history.last()
            && prev.address_space.last() < self.buffer.address_space.first()
            && !prev
                .address_space
                .is_adjacent_before(self.buffer.address_space)
        {
            gap = prev.address_space.last().next();
        }

        self.history.store(position, &mut self.buffer);

        if let Some(address) = gap
            && let Err(err) = self.write_gap(address)
        {
            self.error = Some(err);

            return;
        }

        if let Err(err) = self.write_record(position) {
            self.error = Some(err);
        }
    }

    fn record(&mut self, address_space: AddressSpace) -> RecordBuilder<'_> {
        RecordBuilder::new(self, address_space)
    }
}

#[cfg(test)]
mod tests {
    use std::fmt::{self, Display};

    use impressions::memory::address::Address;
    use impressions::memory::extent::Size;
    use impressions::memory::inspect::{InspectionValue, Inspector as _};

    use super::TableInspector;

    struct NamedValue;

    impl Display for NamedValue {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            f.write_str("value")
        }
    }

    impl InspectionValue for NamedValue {
        fn data_type(&self) -> &dyn Display {
            &"custom"
        }
    }

    #[test]
    fn renders_type_supplied_by_inspection_value() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);
        let address_space = Address::new(2).to_space(Size::new(1).unwrap()).unwrap();

        inspector
            .record(address_space)
            .identified()
            .label(&"Field")
            .value(&NamedValue)
            .finish();

        let output = String::from_utf8(output).unwrap();

        assert!(output.contains("custom  value"));
    }

    #[test]
    fn keeps_equal_hierarchy_labels_at_distinct_address_spaces() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);

        let first = Address::new(2).to_space(Size::new(2).unwrap()).unwrap();
        let second = Address::new(4).to_space(Size::new(2).unwrap()).unwrap();

        for (directory, value) in [(first, 1u16), (second, 2u16)] {
            inspector
                .record(directory)
                .label(&"Data Directory")
                .finish();
            inspector.record(directory).field(&"Size", &value);
        }

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<_> = output.lines().collect();

        assert_eq!(lines.len(), 4);
        assert!(lines[2].starts_with("\x1b[1m0x00000004  >  Data Directory"));
    }

    #[test]
    fn truncates_labels_at_unicode_display_width() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);
        let address_space = Address::new(2).to_space(Size::new(1).unwrap()).unwrap();
        let label = "你".repeat(21);

        inspector.record(address_space).label(&label).finish();

        let output = String::from_utf8(output).unwrap();

        assert!(output.contains(&"你".repeat(20)));
        assert!(!output.contains(&label));
    }

    #[test]
    fn derives_depth_from_address_space_containment() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);
        let parent = Address::new(2).to_space(Size::new(4).unwrap()).unwrap();
        let child = Address::new(2).to_space(Size::new(2).unwrap()).unwrap();

        inspector
            .record(parent)
            .identified()
            .label(&"Parent")
            .finish();
        inspector.record(child).vacant().label(&"Child").finish();

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<_> = output.lines().collect();

        assert_eq!(lines.len(), 2);
        assert!(lines[0].starts_with("\x1b[32m0x00000002     Parent"));
        assert!(lines[1].starts_with("\x1b[2m0x00000002  :    Child"));
    }

    #[test]
    fn renders_gap_before_later_root() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);
        let first = Address::new(2).to_space(Size::new(2).unwrap()).unwrap();
        let second = Address::new(6).to_space(Size::new(2).unwrap()).unwrap();

        inspector.record(first).field(&"First", &1u16);
        inspector.record(second).field(&"Second", &2u16);

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<_> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("\x1b[2m0x00000004  :"));
    }

    #[test]
    fn renders_gap_before_later_child() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);
        let parent = Address::new(2).to_space(Size::new(6).unwrap()).unwrap();
        let child = Address::new(4).to_space(Size::new(2).unwrap()).unwrap();

        inspector.record(parent).label(&"Parent").finish();
        inspector.record(child).field(&"Child", &1u16);

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<_> = output.lines().collect();

        assert_eq!(lines.len(), 3);
        assert!(lines[1].starts_with("\x1b[2m0x00000002  :"));
    }

    #[test]
    fn renders_gap_between_non_adjacent_siblings() {
        let mut output = Vec::new();
        let mut inspector = TableInspector::new(&mut output);
        let parent = Address::new(2).to_space(Size::new(6).unwrap()).unwrap();
        let first = Address::new(2).to_space(Size::new(2).unwrap()).unwrap();
        let second = Address::new(6).to_space(Size::new(2).unwrap()).unwrap();

        inspector.record(parent).label(&"Parent").finish();
        inspector.record(first).field(&"First", &1u16);
        inspector.record(parent).label(&"Parent").finish();
        inspector.record(second).field(&"Second", &2u16);

        let output = String::from_utf8(output).unwrap();
        let lines: Vec<_> = output.lines().collect();

        assert_eq!(lines.len(), 4);
        assert!(lines[2].starts_with("\x1b[2m0x00000004  :"));
    }
}
