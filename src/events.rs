use octocrab::models::events::payload::{
    IssueCommentEventPayload, PullRequestEventAction, PullRequestEventPayload,
    PullRequestReviewCommentEventPayload, PullRequestReviewEventPayload, PushEventPayload,
    ReleaseEventPayload,
};

pub fn handle_release_event(payload: &Box<ReleaseEventPayload>) {
    let release_name = payload.release.name.as_ref().map(|s| s.as_str()).unwrap_or("");
    println!("\t{:?} release {} - {}", payload.action, release_name, payload.release.html_url);
}

pub fn handle_push_event(payload: &Box<PushEventPayload>) {
    let git_ref = &payload.r#ref;
    if git_ref != "refs/heads/master" {
        println!("\tPush to {}:", git_ref);
        payload
            .commits
            .iter()
            .map(|commit| {
                let title = commit.message.splitn(2, "\n").next().expect("Empty commit message");
                format!("{} by {} - {}", commit.sha, commit.author.name, title)
            })
            .for_each(|m| println!("\t\t* {}", m))
    }
}

pub fn handle_pull_request_review_event(payload: &Box<PullRequestReviewEventPayload>) {
    match (
        payload.pull_request.user.as_ref(),
        payload.review.state.as_ref(),
        payload.pull_request.title.as_ref(),
    ) {
        (Some(user), Some(state), Some(title)) => println!(
            "\tPull request #{} by {} - {:?} - {}",
            payload.pull_request.number, user.login, state, title
        ),
        _ => println!("\tPull request #{} - details unavailable", payload.pull_request.number),
    }
}

pub fn handle_pull_request_review_comment_event(
    payload: &Box<PullRequestReviewCommentEventPayload>,
) {
    match payload.pull_request.title.as_ref() {
        Some(title) => println!("\tReviewed PR {} - {}", payload.pull_request.number, title),
        _ => println!("\tReviewed PR {} - title unavailable", payload.pull_request.number),
    }
}

pub fn handle_issue_comment_event(payload: &Box<IssueCommentEventPayload>) {
    let issue_type = match payload.issue.pull_request {
        Some(_) => "pull request",
        None => "issue",
    };
    println!("\tCommented on {} {} - {}", issue_type, payload.issue.number, payload.issue.title);
}

pub fn handle_pull_request_event(payload: &Box<PullRequestEventPayload>) {
    let action = match &payload.action {
        PullRequestEventAction::Closed => payload
            .pull_request
            .merged_at
            .map(|_| "Merged")
            .or(Some("Closed"))
            .map(|s| s.to_string()),
        PullRequestEventAction::Opened | PullRequestEventAction::Reopened => {
            Some(format!("{:?}", payload.action))
        },
        PullRequestEventAction::Edited
        | PullRequestEventAction::Assigned
        | PullRequestEventAction::Unassigned
        | PullRequestEventAction::ReviewRequested
        | PullRequestEventAction::ReviewRequestRemoved
        | PullRequestEventAction::Labeled
        | PullRequestEventAction::Unlabeled
        | PullRequestEventAction::Synchronize => None,
        unsupported => {
            unimplemented!("Unsupported PullRequestEventAction: {:?}", unsupported)
        },
    };
    match action {
        Some(action_string) => {
            match payload.pull_request.user.as_ref().zip(payload.pull_request.title.as_ref()) {
                Some((user, title)) => println!(
                    "\tPull request #{} by {} - {} - {}",
                    payload.number, user.login, action_string, title
                ),
                _ => println!("\tPull request #{} - details unavailable", payload.number),
            }
        },
        None => {},
    }
}
