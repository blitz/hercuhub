use octocrab::{models::IssueState, Octocrab};
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Get environment variables
    let token = env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN environment variable must be set");

    let owner = env::var("GITHUB_OWNER").unwrap_or_else(|_| "octocat".to_string()); // Default owner

    let repo = env::var("GITHUB_REPO").unwrap_or_else(|_| "Hello-World".to_string()); // Default repo

    println!("Fetching pull requests for {}/{}", owner, repo);

    // Create octocrab instance with authentication
    let octocrab = Octocrab::builder().personal_token(token).build()?;

    // Get all pull requests (open by default)
    let pulls = octocrab
        .pulls(&owner, &repo)
        .list()
        .state(octocrab::params::State::All) // Get both open and closed PRs
        .per_page(100) // Adjust as needed
        .send()
        .await?;

    let prs_with_state: Vec<_> = pulls.items.iter().filter(|pr| pr.state.is_some()).collect();
    println!(
        "\nFound {} pull requests with state:\n",
        prs_with_state.len()
    );

    // List all pull requests that have a state
    for pr in pulls.items {
        if let Some(state) = pr.state {
            let state_emoji = match state {
                IssueState::Open => "🟢",

                IssueState::Closed => "🔴",
                _ => "?",
            };

            println!(
                "{} #{} - {} ({})",
                state_emoji,
                pr.number,
                pr.title.unwrap_or_else(|| "No title".to_string()),
                pr.user.unwrap().login
            );

            if let Some(body) = &pr.body {
                let preview = if body.len() > 100 {
                    format!("{}...", &body[..100])
                } else {
                    body.clone()
                };
                println!("   Description: {}", preview);
            }

            println!("   URL: {:?}", pr.html_url);
            println!();
        }
    }

    Ok(())
}
