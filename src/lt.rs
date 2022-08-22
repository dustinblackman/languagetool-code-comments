use anyhow::Result;
use async_std::channel;
use futures::future;
use languagetool_rust::{
    check::CheckRequest, check::CheckResponseWithContext, check::Match, server::ServerClient,
};

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

/// Submits a QueryRequest to the LanguageTool API and returns an array of Match issues, excluding
/// issues related to whitespacing (code comments are full of them naturally).
pub async fn query(request: QueryRequest) -> Result<QueryResult> {
    // TODO Move this client, change to depepdency injection.
    // TODO change url
    let client = ServerClient::new("https://languagetool.buffs.cc", "");
    let mut req = CheckRequest::default()
        .with_language("en".to_string())
        .with_text(request.text.clone());
    req.more_context = true;

    let mut check_res = client.check(&req).await?;
    check_res = CheckResponseWithContext::new(request.text, check_res).into();

    let mut check_res_matches = check_res.matches;
    let matches = check_res_matches
        .iter_mut()
        .filter(|e| {
            return e.rule.id != "WHITESPACE_RULE";
        })
        .map(|e| {
            // TODO remove clone.
            return e.to_owned();
        })
        .collect::<Vec<Match>>();

    let query_result = QueryResult {
        text_checksum: request.text_checksum,
        matches,
    };

    return Ok(query_result);
}

/// Submits many QueryRequest to the LanguageTool API in parallel, and returns an array of Match issues, excluding
/// issues related to whitespacing (code comments are full of them naturally).
pub async fn query_many(
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
        let thread = tokio::spawn(async move {
            while let Ok(query_request) = nodes_receive.recv().await {
                let res = query(query_request).await.unwrap();
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

    let mut query_results: Vec<QueryResult> = vec![];
    while let Ok(m) = matches_receive_master.recv().await {
        query_results.push(m);
    }

    return Ok(query_results);
}
