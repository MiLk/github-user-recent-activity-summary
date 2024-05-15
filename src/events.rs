use octocrab::models::events::payload::{
	IssueCommentEventPayload, PullRequestEventAction, PullRequestEventPayload,
	PullRequestReviewCommentEventPayload, PullRequestReviewEventPayload, PushEventPayload,
	ReleaseEventPayload,
};

pub fn handle_release_event(payload: &Box<ReleaseEventPayload>) {
	if let Some(name) = &payload.release.name {
		print_info(format_args!(
			"\t{:?} release {} - {}",
			payload.action, name, payload.release.html_url
		));
	} else {
		print_info(format_args!("\t{:?} release - {}", payload.action, payload.release.html_url));
	}
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
	if let (Some(user), Some(state), Some(title)) = (
		payload.pull_request.user.as_ref(),
		payload.review.state.as_ref(),
		payload.pull_request.title.as_ref(),
	) {
		print_info(format_args!(
			"\tPull request #{} by {} - {:?} - {}",
			payload.pull_request.number, user.login, state, title
		));
	} else {
		print_info(format_args!(
			"\tPull request #{} - details unavailable",
			payload.pull_request.number
		));
	}
}

pub fn handle_pull_request_review_comment_event(
	payload: &Box<PullRequestReviewCommentEventPayload>,
) {
	if let Some(title) = payload.pull_request.title.as_ref() {
		print_info(format_args!("\tReviewed PR {} - {}", payload.pull_request.number, title));
	} else {
		print_info(format_args!(
			"\tReviewed PR {} - title unavailable",
			payload.pull_request.number
		));
	}
}

pub fn handle_issue_comment_event(payload: &Box<IssueCommentEventPayload>, issue_type: &str) {
	print_info(format_args!(
		"\tCommented on {} {} - {}",
		issue_type, payload.issue.number, payload.issue.title
	));
}

fn print_info(args: std::fmt::Arguments) {
	println!("{}", args);
}

pub fn handle_pull_request_event(payload: &Box<PullRequestEventPayload>) {
	let action = match &payload.action {
		PullRequestEventAction::Closed => match payload.pull_request.merged_at.is_some() {
			true => Some("Merged".to_string()),
			false => Some("Closed".to_string()),
		},
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
			if let (Some(user), Some(title)) =
				(payload.pull_request.user.as_ref(), payload.pull_request.title.as_ref())
			{
				println!(
					"\tPull request #{} by {} - {} - {}",
					payload.number, user.login, action_string, title
				)
			} else {
				println!("\tPull request #{} - details unavailable", payload.number)
			}
		},
		None => {},
	}
}
