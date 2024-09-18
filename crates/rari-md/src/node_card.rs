use comrak::nodes::{AstNode, NodeValue};
use rari_types::locale::Locale;

pub enum NoteCard {
    Callout,
    Warning,
    Note,
}

impl NoteCard {
    pub fn prefix_for_locale(&self, locale: Locale) -> &str {
        match (self, locale) {
            (Self::Callout, Locale::De) => "Hinweis",
            (Self::Warning, Locale::De) => "Warnung",
            (Self::Note, Locale::De) => "Notiz",
            (Self::Callout, Locale::EnUs) => "Callout:",
            (Self::Warning, Locale::EnUs) => "Warning:",
            (Self::Note, Locale::EnUs) => "Note:",
            (Self::Callout, Locale::Es) => "Observación:",
            (Self::Warning, Locale::Es) => "Advertencia:",
            (Self::Note, Locale::Es) => "Nota:",
            (Self::Callout, Locale::Fr) => "Remarque :",
            (Self::Warning, Locale::Fr) => "Attention :",
            (Self::Note, Locale::Fr) => "Note :",
            (Self::Callout, Locale::Ja) => "注目:",
            (Self::Warning, Locale::Ja) => "警告:",
            (Self::Note, Locale::Ja) => "メモ:",
            (Self::Callout, Locale::Ko) => "알림 :",
            (Self::Warning, Locale::Ko) => "경고 :",
            (Self::Note, Locale::Ko) => "참고 :",
            (Self::Callout, Locale::PtBr) => "Observação:",
            (Self::Warning, Locale::PtBr) => "Aviso:",
            (Self::Note, Locale::PtBr) => "Nota:",
            (Self::Callout, Locale::Ru) => "Сноска:",
            (Self::Warning, Locale::Ru) => "Предупреждение:",
            (Self::Note, Locale::Ru) => "Примечание:",
            (Self::Callout, Locale::ZhCn) => "标注：",
            (Self::Warning, Locale::ZhCn) => "警告：",
            (Self::Note, Locale::ZhCn) => "备注：",
            (Self::Callout, Locale::ZhTw) => "标注：",
            (Self::Warning, Locale::ZhTw) => "警告：",
            (Self::Note, Locale::ZhTw) => "备注：",
        }
    }
    pub fn new_prefix(&self) -> &str {
        match self {
            Self::Callout => "[!CALLOUT]",
            Self::Warning => "[!WARNING]",
            Self::Note => "[!NOTE]",
        }
    }
}

pub(crate) fn is_callout<'a>(block_quote: &'a AstNode<'a>, locale: Locale) -> Option<NoteCard> {
    if let Some(grand_child) = block_quote.first_child().and_then(|c| c.first_child()) {
        if matches!(grand_child.data.borrow().value, NodeValue::Strong) {
            if let Some(marker) = grand_child.first_child() {
                if let NodeValue::Text(ref text) = marker.data.borrow().value {
                    let callout = NoteCard::Callout.prefix_for_locale(locale);
                    if text.starts_with(callout) {
                        grand_child.detach();
                        return Some(NoteCard::Callout);
                    }

                    if text.starts_with(NoteCard::Warning.prefix_for_locale(locale)) {
                        grand_child.detach();
                        return Some(NoteCard::Warning);
                    }
                    if text.starts_with(NoteCard::Note.prefix_for_locale(locale)) {
                        grand_child.detach();
                        return Some(NoteCard::Note);
                    }
                }
            }
        }
    }
    if let Some(child) = block_quote.first_child() {
        if let Some(marker) = child.first_child() {
            if let NodeValue::Text(ref text) = marker.data.borrow().value {
                if text.starts_with(NoteCard::Callout.new_prefix()) {
                    marker.detach();
                    return Some(NoteCard::Callout);
                }
                if text.starts_with(NoteCard::Warning.new_prefix()) {
                    marker.detach();
                    return Some(NoteCard::Warning);
                }
                if text.starts_with(NoteCard::Note.new_prefix()) {
                    marker.detach();
                    return Some(NoteCard::Note);
                }
            }
        }
    }
    None
}
