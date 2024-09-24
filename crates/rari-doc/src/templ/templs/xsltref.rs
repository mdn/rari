use rari_templ_func::rari_f;
use rari_types::locale::Locale;
use rari_utils::concat_strs;

use crate::error::DocError;

#[rari_f]
pub fn xsltref() -> Result<String, DocError> {
    Ok(concat_strs!(
        r#"<div style="background:#f5f5f5; margin: 5px 0;">"#,
        match env.locale {
            Locale::Es =>
                r#"<b><a href="/es/docs/Web/XSLT/Transformando_XML_con_XSLT#Referencia_de_XSLT.2FXPath">Referencia de XSLT y XPath</a></b>: <a href="/es/docs/Web/XSLT/Element">Elementos XSLT</a>, <a href="/es/docs/Web/EXSLT">Funciones EXSLT</a>, <ahref="/es/docs/Web/XPath/Funciones">XPath:Funciones</a>, <a href="/es/docs/Web/XPath/Ejes">XPath:Ejes</a>"#,
            Locale::Fr =>
                r#"<b><a href="/fr/docs/Web/XSLT/Transformations_XML_avec_XSLT/La_référence_XSLT_XPath_de_Netscape">Référence XSLT/XPath</a></b> : <a href="/fr/docs/Web/XSLT/Element">Éléments XSLT</a>, <a href="/fr/docs/Web/EXSLT">Fonctions EXSLT</a>, <a href="/fr/docs/Web/XPath/Fonctions">Fonctions XPath</a>, <a href="/fr/docs/Web/XPath/Axes">Axes XPath</a>"#,
            Locale::Ja =>
                r#"<b><a href="/ja/docs/Web/XSLT/Transforming_XML_with_XSLT/The_Netscape_XSLT_XPath_Reference">XSLT/XPath リファレンス</a></b>: <a href="/ja/docs/Web/XSLT/Element">XSLT 要素</a>, <a href="/ja/docs/Web/EXSLT">EXSLT 関数</a>, <a href="/ja/docs/Web/XPath/Functions">XPath 関数</a>, <a href="/ja/docs/Web/XPath/Axes">XPath 軸</a>"#,
            Locale::Ko =>
                r#"<b><a href="/ko/docs/Web/XSLT/Transforming_XML_with_XSLT/The_Netscape_XSLT_XPath_Reference">XSLT/XPath 참고 문서</a></b>: <a href="/ko/docs/Web/XSLT/Element">XSLT 요소</a>, <a href="/ko/docs/Web/XPath/Functions">XPath 함수</a>, <a href="/ko/docs/Web/XPath/Axes">XPath 축</a>"#,
            Locale::ZhCn =>
                r#"<b><a href="/zh-CN/docs/Web/XSLT/Transforming_XML_with_XSLT/The_Netscape_XSLT_XPath_Reference">XSLT/XPath 参考</a></b>：<a href="/zh-CN/docs/Web/XSLT/Element">XSLT 元素</a>、<a href="/zh-CN/docs/Web/EXSLT">EXSLT 函数</a>、<a href="/zh-CN/docs/Web/XPath/Functions">XPath 函数</a>、<a href="/zh-CN/docs/Web/XPath/Axes">XPath 轴</a>"#,
            _ =>
                r#"<b><a href="/en-US/docs/Web/XSLT/Transforming_XML_with_XSLT/The_Netscape_XSLT_XPath_Reference">XSLT/XPath Reference</a></b>: <a href="/en-US/docs/Web/XSLT/Element">XSLT elements</a>, <a href="/en-US/docs/Web/EXSLT">EXSLT functions</a>, <a href="/en-US/docs/Web/XPath/Functions">XPath functions</a>, <a href="/en-US/docs/Web/XPath/Axes">XPath axes</a>"#,
        },
        r#"</div>"#
    ))
}
