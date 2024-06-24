use rari_types::globals::json_l10n_files;
use rari_types::locale::Locale;

#[allow(clippy::wildcard_in_or_patterns)]
pub fn l10n(key: &str, locale: &Locale) -> &'static str {
    match key {
        "experimental_badge_title" => match locale {
            Locale::EnUs => "Experimental. Expect behavior to change in the future.",
            Locale::Es => "Experimental. Espere que el comportamiento cambie en el futuro.",
            Locale::Fr => "Expérimental. Le comportement attendu pourrait évoluer à l'avenir.",
            Locale::Ja => "Experimental. Expect behavior to change in the future.",
            Locale::Ko => "Experimental. 예상되는 동작은 향후 변경될 수 있습니다.",
            Locale::PtBr => "Experimental. Expect behavior to change in the future.",
            Locale::Ru => "Экспериментальная возможность. Её поведение в будущем может измениться",
            Locale::ZhCn => "实验性。预期行为可能会在未来发生变更。",
            Locale::ZhTw => "實驗性質。行為可能會在未來發生變動。",
        },

        "experimental_badge_abbreviation" => match locale {
            Locale::EnUs => "Experimental",
            Locale::Es => "Experimental",
            Locale::Fr => "Expérimental",
            Locale::Ja => "Experimental",
            Locale::Ko => "Experimental",
            Locale::PtBr => "Experimental",
            Locale::Ru => "Экспериментальная возможность",
            Locale::ZhCn => "实验性",
            Locale::ZhTw => "實驗性質",
        },

        "deprecated_badge_title" => match locale {
            Locale::Ko => "지원이 중단되었습니다. 새로운 웹사이트에서 사용하지 마세요.",
            Locale::ZhCn => "已弃用。请不要在新的网站中使用。",
            Locale::ZhTw => "已棄用。請不要在新的網站中使用。",
            Locale::EnUs | _ => "Deprecated. Not for use in new websites.",
        },

        "deprecated_badge_abbreviation" => match locale {
            Locale::Es => "Obsoleto",
            Locale::Fr => "Obsolète",
            Locale::Ja => "非推奨;",
            Locale::Ko => "지원이 중단되었습니다",
            Locale::Ru => "Устарело",
            Locale::ZhCn => "已弃用",
            Locale::ZhTw => "已棄用",
            Locale::EnUs | _ => "Deprecated",
        },

        "non_standard_badge_title" => match locale {
            Locale::Ko => "비표준. 사용하기전에 다른 브라우저에서도 사용 가능한지 확인 해주세요.",
            Locale::ZhCn => "非标准。请在使用前检查跨浏览器支持。",
            Locale::ZhTw => "非標準。請在使用前檢查跨瀏覽器支援。",
            Locale::EnUs | _ => "Non-standard. Check cross-browser support before using.",
        },

        "non_standard_badge_abbreviation" => match locale {
            Locale::Ko => "비표준",
            Locale::ZhCn => "非标准",
            Locale::ZhTw => "非標準",
            Locale::EnUs | _ => "Non-standard",
        },

        "interactive_example_cta" => match locale {
            Locale::EnUs => "Try it",
            Locale::Fr => "Exemple interactif",
            Locale::Ja => "試してみましょう",
            Locale::Ko => "시도해보기",
            Locale::Ru => "Интерактивный пример",
            Locale::PtBr => "Experimente",
            Locale::Es => "Pruébalo",
            Locale::ZhCn => "尝试一下",
            Locale::ZhTw => "嘗試一下",
        },

        _ => "l10n missing",
    }
}

pub fn l10n_json_data(typ: &str, key: &str, locale: &Locale) -> Option<&'static str> {
    json_l10n_files()
        .get(typ)
        .and_then(|file| file.get(key))
        .and_then(|part| part.get(locale.as_url_str()).map(|s| s.as_str()))
}
