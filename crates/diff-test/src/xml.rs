use std::io::Cursor;

use quick_xml::events::Event;
use quick_xml::reader::Reader;
use quick_xml::writer::Writer;

pub fn fmt_html(html: &str) -> String {
    let mut reader = Reader::from_str(html);
    reader.config_mut().trim_text(true);
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 0);
    loop {
        match reader.read_event() {
            Ok(Event::Eof) => break,
            // we can either move or borrow the event to write, depending on your use-case
            Ok(e) => assert!(writer.write_event(e).is_ok()),
            _ => {}
        }
    }

    let result = writer.into_inner().into_inner();
    String::from_utf8(result).unwrap()
}
