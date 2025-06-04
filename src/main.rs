use anyhow::{anyhow, Result};
use octocrab::{
    models::{pulls::PullRequest, IssueState},
    params::repos::Reference,
    repos::RepoHandler,
    Octocrab,
};
use std::env;

/// Print a nice log message about a PR.
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

/// Create a local branch that references the PR. This will trigger
/// Hercules CI to execute tests..
async fn sync_open_pr(repo: &RepoHandler<'_>, pr: &PullRequest) -> Result<()> {
    assert_eq!(pr.state, Some(IssueState::Open));

    let branch_name = format!("pr-{}", pr.number);
    let pr_sha = &pr.head.sha;
    println!("Updating {branch_name} to point to {pr_sha}...");

    let branch_ref = Reference::Branch(branch_name);

    // TODO We need to skip the rest of this function when the branch already exists
    // and is up-to-date to avoid spamming the repo with branch delete/create requests.

    // TODO How do we update the ref instead of deleting and
    // recreating it?
    if repo.get_ref(&branch_ref).await.is_ok() {
        repo.delete_ref(&branch_ref).await?;
    }

    repo.create_ref(&branch_ref, pr_sha).await?;

    Ok(())
}

/// Clean up old branches from closed pull requests.
async fn sync_closed_pr(repo: &RepoHandler<'_>, pr: &PullRequest) -> Result<()> {
    assert_eq!(pr.state, Some(IssueState::Closed));

    let branch_name = format!("pr-{}", pr.number);
    println!("Deleting {branch_name}...");

    let branch_ref = Reference::Branch(branch_name);
    if repo.get_ref(&branch_ref).await.is_ok() {
        repo.delete_ref(&branch_ref).await?;
    }

    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let Ok(token) = env::var("GITHUB_TOKEN") else {
        return Err(anyhow!("GITHUB_TOKEN not set"));
    };

    let Ok(owner) = env::var("GITHUB_OWNER") else {
        return Err(anyhow!("GITHUB_OWNER not set"));
    };

    let Ok(repo) = env::var("GITHUB_REPO") else {
        return Err(anyhow!("GITHUB_REPO not set"));
    };

    let octocrab = Octocrab::builder().personal_token(token).build()?;

    let pulls = octocrab
        .pulls(&owner, &repo)
        .list()
        .state(octocrab::params::State::All)
        .per_page(100) // Adjust as needed
        .send()
        .await?;

    let repo = octocrab.repos(&owner, &repo);

    for pr in pulls.items {
        log_pr(&pr);

        match pr.state {
            Some(IssueState::Open) => sync_open_pr(&repo, &pr).await?,
            Some(IssueState::Closed) => sync_closed_pr(&repo, &pr).await?,

            Some(unknown) => todo!("unknown state: {:?}", unknown),
            None => todo!("No state?"),
        }
    }

    Ok(())
}
