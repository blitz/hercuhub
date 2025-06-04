use octocrab::{
    models::{pulls::PullRequest, IssueState},
    params::repos::Reference,
    repos::RepoHandler,
    Octocrab,
};
use std::env;

fn log_pr(pr: &PullRequest) {
    let state_emoji = match pr.state {
        Some(IssueState::Open) => "🟢",

        Some(IssueState::Closed) => "🔴",
        _ => "⚪",
    };

    println!(
        "{} #{} - {} ({})",
        state_emoji,
        pr.number,
        pr.title
            .as_ref()
            .map(|t| t.as_str())
            .unwrap_or_else(|| "No title"),
        pr.user
            .as_ref()
            .map(|u| u.login.as_str())
            .unwrap_or_else(|| "???")
    );
}

async fn sync_open_pr(
    repo: &RepoHandler<'_>,
    pr: &PullRequest,
) -> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(pr.state, Some(IssueState::Open));

    let branch_name = format!("pr-{}", pr.number);
    let pr_sha = &pr.head.sha;
    println!("Updating {branch_name} to point to {pr_sha}...");

    let branch_ref = Reference::Branch(branch_name);

    // TODO How do we update the ref instead of deleting and recreating it?
    if repo.get_ref(&branch_ref).await.is_ok() {
        repo.delete_ref(&branch_ref).await?;
    }

    repo.create_ref(&branch_ref, pr_sha).await?;

    Ok(())
}

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

    let repo = octocrab.repos(&owner, &repo);

    // List all pull requests that have a state
    for pr in pulls.items {
        log_pr(&pr);

        match pr.state {
            Some(IssueState::Open) => sync_open_pr(&repo, &pr).await?,

            Some(unknown) => todo!("unknown state: {:?}", unknown),
            None => todo!("No state?"),
        }
    }

    Ok(())
}
