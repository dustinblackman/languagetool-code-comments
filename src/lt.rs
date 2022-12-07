#![allow(clippy::implicit_return)]
use anyhow::Result;
use async_std::channel;
use async_std::sync::Arc;
use futures::future;
use languagetool_rust::check::CheckRequest;
use languagetool_rust::check::CheckResponseWithContext;
use languagetool_rust::check::Match;
use languagetool_rust::server::ServerClient;
use url::Url;

#[derive(Clone, Debug)]
pub struct QueryRequest {
    pub language: String,
    pub text: String,
    pub text_checksum: u64,
}
unsafe impl Send for QueryRequest {}
unsafe impl Sync for QueryRequest {}

#[derive(Clone, Debug)]
pub struct QueryResult {
    pub text_checksum: u64,
    pub matches: Vec<Match>,
}
unsafe impl Send for QueryResult {}
unsafe impl Sync for QueryResult {}

#[derive(Default, Clone)]
pub struct Client {
    client: ServerClient,
}

impl Client {
    pub fn new(url: &str) -> Result<Arc<Self>> {
        let parsed = Url::parse(url)?;
        let proto_host = format!("{}://{}", parsed.scheme(), parsed.host_str().unwrap());
        let mut port = parsed.port().unwrap_or_else(|| return 0).to_string();
        if port == "0" {
            port = "".to_string();
        }

        return Ok(Arc::new(Self {
            client: ServerClient::new(&proto_host, &port),
        }));
    }

    /// Submits a QueryRequest to the LanguageTool API and returns an array of
    /// Match issues, excluding issues related to whitespacing (code
    /// comments are full of them naturally).
    pub async fn query(&self, request: QueryRequest) -> Result<QueryResult> {
        let mut req = CheckRequest::default()
            .with_language(request.language)
            .with_text(request.text.clone());
        req.more_context = true;

        let mut check_res = self.client.check(&req).await?;
        check_res = CheckResponseWithContext::new(request.text, check_res).into();

        let mut check_res_matches = check_res.matches;
        let matches = check_res_matches
            .iter_mut()
            .filter(|e| {
                return e.rule.id != "WHITESPACE_RULE";
            })
            .map(|e| {
                return e.to_owned();
            })
            .collect::<Vec<Match>>();

        let query_result = QueryResult {
            text_checksum: request.text_checksum,
            matches,
        };

        return Ok(query_result);
    }

    /// Submits many QueryRequest to the LanguageTool API in parallel, and
    /// returns an array of Match issues, excluding issues related to
    /// whitespacing (code comments are full of them naturally).
    pub async fn query_many(
        self: Arc<Self>,
        requests: Vec<QueryRequest>,
        mut concurrency: usize,
    ) -> Result<Vec<QueryResult>> {
        let (nodes_send_master, nodes_receive_master) = channel::unbounded::<QueryRequest>();
        let (matches_send_master, matches_receive_master) = channel::unbounded::<QueryResult>();

        // Creates a queue to process multiple code comment leaves concurrently.
        let mut threads = Vec::new();
        concurrency = *(vec![concurrency, requests.len()].iter().min().unwrap());

        for _ in 0..concurrency {
            let matches_send = matches_send_master.clone();
            let nodes_receive = nodes_receive_master.clone();
            let this = Arc::clone(&self);
            let thread = tokio::spawn(async move {
                while let Ok(query_request) = nodes_receive.recv().await {
                    let res = this.query(query_request).await.unwrap();
                    matches_send.send(res).await.unwrap();
                }
            });
            threads.push(thread);
        }

        for query_request in requests.iter() {
            // TODO this should be able to pass a pointer.
            nodes_send_master.send(query_request.to_owned()).await?;
        }

        // Waits for queue to empty.
        nodes_send_master.close();
        future::join_all(threads).await;
        matches_send_master.close();

        // Formats and process' results, extracting results for the deduping hashmap,
        // and mapping them back to code comment blocks.
        let mut query_results: Vec<QueryResult> = vec![];
        while let Ok(m) = matches_receive_master.recv().await {
            query_results.push(m);
        }

        return Ok(query_results);
    }
}
