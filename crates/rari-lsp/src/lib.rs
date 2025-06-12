use tower_lsp_server::{LspService, Server};

mod keywords;
mod lsp;
mod parser;
mod position;

pub fn run() -> Result<(), anyhow::Error> {
    tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap()
        .block_on(async {
            let (stdin, stdout) = (tokio::io::stdin(), tokio::io::stdout());

            let (service, socket) = LspService::build(lsp::Backend::new).finish();
            Server::new(stdin, stdout, socket).serve(service).await;
        });
    Ok(())
}
