use chrono::Local;
use itertools::{Itertools};
use octocrab::{models, Octocrab, Page, Result};
use octocrab::models::events::EventType;
use octocrab::models::events::payload::{EventPayload, PullRequestEventAction};

#[async_trait::async_trait]
trait UserEventExt {
    async fn list_user_events(&self, user: &String) -> Result<Vec<models::events::Event>>;
}

#[async_trait::async_trait]
impl UserEventExt for Octocrab {
    async fn list_user_events(&self, user: &String) -> Result<Vec<models::events::Event>> {
        let result: Result<Page<models::events::Event>> = self.get(format!("/users/{user}/events?per_page=100"), None::<&()>).await;
        match result {
            Ok(page) => self.all_pages(page).await,
            Err(err) => Err(err)
        }
    }
}

fn process_event(event: models::events::Event)  {
    match event.payload.unwrap().specific.unwrap() {
        EventPayload::PushEvent(payload) => {
            let git_ref = payload.r#ref;
            if git_ref != "refs/heads/master" {
                println!("\tPush to {}:", git_ref);
                payload.commits.iter().map(|commit| {
                    let title = commit.message.splitn(2, "\n").next().expect("Empty commit message");
                    format!("{} by {} - {}", commit.sha, commit.author.name, title)
                }).for_each(|m| {
                    println!("\t\t* {}", m)
                })
            }
        }
        EventPayload::PullRequestEvent(payload) => {
            let action = match payload.action {
                PullRequestEventAction::Closed => {
                    match payload.pull_request.merged_at.is_some() {
                        true => Some("Merged".to_string()),
                        false => Some("Closed".to_string())
                    }
                }
                PullRequestEventAction::Opened | PullRequestEventAction::Reopened => Some(format!("{:?}", payload.action)),
                PullRequestEventAction::Edited | PullRequestEventAction::Assigned | PullRequestEventAction::Unassigned | PullRequestEventAction::ReviewRequested | PullRequestEventAction::ReviewRequestRemoved | PullRequestEventAction::Labeled | PullRequestEventAction::Unlabeled | PullRequestEventAction::Synchronize => None,
                unsupported => unimplemented!("Unsupported PullRequestEventAction: {:?}", unsupported)
            };
            match action {
                Some(action_string) => {
                    println!("\tPull request #{} by {} - {} - {}", payload.number, payload.pull_request.user.unwrap().login, action_string, payload.pull_request.title.unwrap())
                }
                None => {}
            }
        }
        EventPayload::IssueCommentEvent(payload) => {
            let issue_type = if payload.issue.pull_request.is_some() { "pull request" } else { "issue" };
            println!("\tCommented on {} {} - {}", issue_type, payload.issue.number, payload.issue.title)
        }
        EventPayload::PullRequestReviewCommentEvent(payload) => {
            println!("\tReviewed PR {} - {}", payload.pull_request.number, payload.pull_request.title.unwrap())
        }
        EventPayload::PullRequestReviewEvent(payload) => {
            println!("\tPull request #{} by {} - {:?} - {}", payload.pull_request.number, payload.pull_request.user.unwrap().login, payload.review.state.unwrap(), payload.pull_request.title.unwrap())
        }
        EventPayload::ReleaseEvent(payload) => {
            println!("\t{:?} release {} - {}", payload.action, payload.release.name.unwrap(), payload.release.html_url)
        }
        EventPayload::CreateEvent(payload) => match payload.ref_type.as_str() {
            "tag" => println!("\tCreated tag {}", payload.r#ref.unwrap()),
            "repository" | "branch" => {}
            unsupported => unimplemented!("Unsupported ref_type for CreateEvent: {}", unsupported)
        }
        _ => unimplemented!("Unsupported Event type {:?}", event.r#type)
    }
}


#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let username = std::env::args().nth(1).expect("no username given");

    let octocrab: Octocrab = Octocrab::builder().personal_token(token).build()?;
    let events = octocrab.list_user_events(&username).await?;

    let grouped_events = events.into_iter().filter(|event| {
        event.r#type != EventType::DeleteEvent && event.actor.login == username
    }).group_by(|event| {
        let repo_name = event.repo.name.clone();
        let day = event.created_at.with_timezone(&Local).date_naive().to_string();
        (day, repo_name)
    });

    grouped_events.into_iter().for_each(|((day, repo_name), group)| {
        println!("{} - {}", day, repo_name);
        group.for_each(process_event);
    });

    Ok(())
}
