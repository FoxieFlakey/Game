use std::fmt;

pub struct IdentedWriter<'ident_string, 'writer, Writer: fmt::Write + ?Sized> {
    indent: &'ident_string str,
    depth: u32,
    parent_writer: &'writer mut Writer,
    needs_indent: bool,
}

impl<Writer: fmt::Write + ?Sized> fmt::Write for IdentedWriter<'_, '_, Writer> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        // AI generated portion by Gemini
        // Split the input string into lines, preserving trailing empty sections
        let mut parts = s.split('\n').peekable();

        while let Some(part) = parts.next() {
            // 1. If we are at the start of a fresh line and the text segment
            // isn't just an empty trailing piece after a final newline, write indent.
            if self.needs_indent && !(part.is_empty() && parts.peek().is_none()) {
                for _ in 0..self.depth {
                    self.parent_writer.write_str(self.indent)?;
                }
                self.needs_indent = false;
            }

            // 2. Write the actual text chunk for this line
            if !part.is_empty() {
                self.parent_writer.write_str(part)?;
            }

            // 3. If there are more parts remaining, it means we hit a '\n' boundary
            if parts.peek().is_none() {
                // If s ended with a newline, parts.next() will yield an empty string
                // as the last element. We do not write a newline here because
                // it was already handled by the previous iteration's writeln logic.
            } else {
                self.parent_writer.write_str("\n")?;
                self.needs_indent = true;
            }
        }

        Ok(())
    }
}

impl<'ident_string, 'writer, Writer: fmt::Write + ?Sized>
    IdentedWriter<'ident_string, 'writer, Writer>
{
    pub fn new(
        depth: u32,
        indent: &'ident_string str,
        writer: &'writer mut Writer,
        dont_indent_first_line: bool,
    ) -> Self {
        Self {
            indent,
            depth,
            parent_writer: writer,
            needs_indent: !dont_indent_first_line,
        }
    }
}
