#![feature(result_flattening)]

use std::cmp::Ordering;

use chrono::Local;
use itertools::Itertools;
use octocrab::models::events::payload::EventPayload;
use octocrab::models::events::EventType;
use octocrab::{models, Octocrab, Result};

use events::handle_issue_comment_event;

use crate::events::{
    handle_pull_request_event, handle_pull_request_review_comment_event,
    handle_pull_request_review_event, handle_push_event, handle_release_event,
};

mod events;

#[async_trait::async_trait]
trait UserEventExt {
    async fn list_user_events(&self, user: &String) -> Result<Vec<models::events::Event>>;
}

#[async_trait::async_trait]
impl UserEventExt for Octocrab {
    async fn list_user_events(&self, user: &String) -> Result<Vec<models::events::Event>> {
        let result = self
            .get(format!("/users/{user}/events?per_page=100"), None::<&()>)
            .await
            .map(|page| self.all_pages(page));
        match result {
            Ok(pages) => pages.await,
            Err(err) => Err(err),
        }
    }
}

fn process_event(event: models::events::Event) -> () {
    let specific_result = &event
        .payload
        .ok_or(format!("\tNo payload for event {:?}", event.r#type))
        .map(|payload| {
            payload
                .specific
                .ok_or(format!("\tNo specific payload for event {:?}", event.r#type))
        })
        .flatten();
    match specific_result {
        Ok(specific) => match specific {
            EventPayload::PushEvent(payload) => handle_push_event(payload),
            EventPayload::PullRequestEvent(payload) => handle_pull_request_event(payload),
            EventPayload::IssueCommentEvent(payload) => handle_issue_comment_event(payload),
            EventPayload::PullRequestReviewCommentEvent(payload) => {
                handle_pull_request_review_comment_event(payload)
            },
            EventPayload::PullRequestReviewEvent(payload) => {
                handle_pull_request_review_event(payload)
            },
            EventPayload::ReleaseEvent(payload) => handle_release_event(payload),
            EventPayload::CreateEvent(payload) => match payload.ref_type.as_str() {
                "tag" => match &payload.r#ref {
                    Some(ref_value) => println!("\tCreated tag {}", ref_value),
                    None => println!("\tCreated tag with unknown value"),
                },
                "repository" | "branch" => {},
                unsupported => {
                    unimplemented!("Unsupported ref_type for CreateEvent: {}", unsupported)
                },
            },
            _ => unimplemented!("Unsupported Event type {:?}", event.r#type),
        },
        Err(err) => println!("{}", err),
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN env variable is required");
    let username = std::env::args().nth(1).expect("no username given");

    let octocrab: Octocrab = Octocrab::builder().personal_token(token).build()?;
    let events = octocrab.list_user_events(&username).await?;

    events
        .into_iter()
        .filter(|event| event.r#type != EventType::DeleteEvent && event.actor.login == username)
        .sorted_by(|a, b| {
            let date_ordering = Ord::cmp(
                &a.created_at.with_timezone(&Local).date_naive(),
                &b.created_at.with_timezone(&Local).date_naive(),
            );
            match date_ordering {
                Ordering::Equal => Ord::cmp(&a.repo.name, &b.repo.name),
                _ => date_ordering,
            }
        })
        .group_by(|event| {
            let repo_name = event.repo.name.clone();
            let day = event.created_at.with_timezone(&Local).date_naive();
            (day, repo_name)
        })
        .into_iter()
        .for_each(|((day, repo_name), group)| {
            println!("{} - {}", day.format("%Y-%m-%d (%a)"), repo_name);
            group.for_each(process_event);
        });

    Ok(())
}
