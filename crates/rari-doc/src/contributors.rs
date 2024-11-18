use std::collections::HashMap;

use rari_types::locale::Locale;
use serde::Deserialize;

#[derive(Clone, Debug, Deserialize)]
pub(crate) struct WikiHistoryEntry {
    pub contributors: Vec<String>,
}

pub(crate) type WikiHistory = HashMap<String, WikiHistoryEntry>;
pub(crate) type WikiHistories = HashMap<Locale, WikiHistory>;

/// Generates a contributors text report summarizing commit history and original Wiki contributors.
///
/// This function creates a formatted string containing a list of contributors for a given file.
/// It includes:
/// - A section linking to the GitHub commit history for the file.
/// - An optional section listing original Wiki contributors if historical data is available.
///
/// # Parameters
///
/// - `wiki_history`: An optional reference to a `WikiHistoryEntry`, which contains historical
///   contributor data from the Wiki. If `None`, the Wiki contributors section is omitted.
/// - `github_file_url`: A string containing the URL of the file on GitHub. The URL is modified
///   to point to the file's commit history.
///
/// # Returns
///
/// A `String` formatted as a contributors report. The report consists of:
/// 1. A header: `# Contributors by commit history`
/// 2. A link to the GitHub commit history, derived from `github_file_url`.
/// 3. If `wiki_history` is provided, an additional section:
///    - A header: `# Original Wiki contributors`
///    - A newline-separated list of contributors from `wiki_history`.
///
/// # Example
///
/// ```rust,ignore
/// let github_file_url = "https://github.com/user/repo/blob/main/file.txt";
/// let wiki_history = Some(WikiHistoryEntry {
///     contributors: vec!["Alice".to_string(), "Bob".to_string()],
/// });
/// let result = contributors_txt(wiki_history.as_ref(), github_file_url);
/// println!("{}", result);
/// // Output:
/// // # Contributors by commit history
/// // https://github.com/user/repo/commits/main/file.txt
/// //
/// // # Original Wiki contributors
/// // Alice
/// // Bob
/// ```
///
/// If no `wiki_history` is provided:
///
/// ```rust,ignore
/// let result = contributors_txt(None, github_file_url);
/// println!("{}", result);
/// // Output:
/// // # Contributors by commit history
/// // https://github.com/user/repo/commits/main/file.txt
/// ```
pub(crate) fn contributors_txt(
    wiki_history: Option<&WikiHistoryEntry>,
    github_file_url: &str,
) -> String {
    let mut out = String::new();
    out.extend([
        "# Contributors by commit history\n",
        &github_file_url.replace("blob", "commits"),
        "\n\n",
    ]);
    if let Some(wh) = wiki_history {
        out.extend([
            "# Original Wiki contributors\n",
            &wh.contributors.join("\n"),
            "\n",
        ]);
    }
    out
}
