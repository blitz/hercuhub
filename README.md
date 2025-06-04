# Hercuhub - GitHub PR Branch Sync Bot

**⚠️ This is a proof of concept - not production ready!**

A simple bot that creates local branches for GitHub pull requests to
work around [Hercules CI](https://hercules-ci.com/) limitations.

## Problem

Hercules CI doesn't run build jobs on pull requests from forked
repositories. This bot solves that by creating local branches
(`pr-123`, `pr-456`, etc.) that point to the same commits as the PR
heads, triggering CI to run on these local branches instead. The
annotated commit status is then also visible from the PR and
everything works like people would expect.

## How it works

1. Fetches all pull requests from a GitHub repository
2. For **open** PRs: Creates/updates a local branch `pr-{number}` pointing to the PR's head commit
3. For **closed** PRs: Deletes the corresponding local branch to clean up

## Setup

1. Create a GitHub personal access token with `repo` permissions
2. Set environment variables:
   ```bash
   export GITHUB_TOKEN="your_token_here"
   export GITHUB_OWNER="owner_name" 
   export GITHUB_REPO="repo_name"
   ```
3. Run with `cargo run`
